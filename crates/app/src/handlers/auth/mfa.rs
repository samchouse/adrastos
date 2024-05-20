use adrastos_core::{
    auth::{
        self,
        mfa::{Mfa, VerificationMethod},
    },
    db::redis,
    entities,
    error::Error,
};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use chrono::Duration;
use fred::{interfaces::KeysInterface, types::Expiration};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_sessions::Session;

use crate::{
    middleware::extractors::{AnyUser, Config, Database, Project},
    session::SessionKey,
    state::AppState,
};

#[derive(Deserialize)]
pub struct CVDRBody {
    code: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/enable", get(enable))
        .route("/confirm", post(confirm))
        .route("/verify", post(verify))
        .route("/disable", post(disable))
        .route("/codes/regenerate", post(regenerate))
}

pub async fn enable(
    project: Option<Project>,
    AnyUser(user, _): AnyUser,
    State(AppState {
        config, redis_pool, ..
    }): State<AppState>,
) -> Result<impl IntoResponse, Error> {
    if user.mfa_secret.is_some() {
        return Err(Error::BadRequest("MFA is already enabled".into()));
    }

    let mfa = Mfa::new(
        Mfa::generate_secret(),
        user.email.clone(),
        &project.map(|Project(p)| p),
    );

    redis_pool
        .set(
            redis::build_key(&config, format!("mfa:secret:{}", user.id)),
            mfa.get_secret(),
            Some(Expiration::EX(Duration::minutes(10).num_seconds())),
            None,
            false,
        )
        .await
        .map_err(|_| {
            Error::InternalServerError("Something went wrong saving secret to Redis".into())
        })?;

    Ok(Json(json!({
        "secret": mfa.get_secret(),
        "qr_code": mfa.get_qr().unwrap(),
    })))
}

pub async fn confirm(
    project: Option<Project>,
    AnyUser(user, _): AnyUser,
    Database(db): Database,
    State(AppState {
        config, redis_pool, ..
    }): State<AppState>,
    Json(body): Json<CVDRBody>,
) -> Result<impl IntoResponse, Error> {
    if user.mfa_secret.is_some() {
        return Err(Error::BadRequest("MFA is already enabled".into()));
    }

    let mfa_secret = redis_pool
        .get(redis::build_key(&config, format!("mfa:secret:{}", user.id)))
        .await
        .map_err(|_| Error::InternalServerError("Error getting MFA details from Redis".into()))?;

    let mfa = Mfa::new(
        Mfa::secret_from_string(mfa_secret),
        user.email.clone(),
        &project.map(|Project(p)| p),
    );
    if !mfa.verify(&body.code, VerificationMethod::Code).await? {
        return Err(Error::BadRequest("Invalid MFA code".into()));
    }

    let backup_codes = Mfa::generate_codes()
        .await
        .map_err(|_| Error::InternalServerError("Error generating backup codes".into()))?;

    redis_pool
        .del(redis::build_key(&config, format!("mfa:secret:{}", user.id)))
        .await
        .map_err(|_| Error::InternalServerError("Error deleting MFA details from Redis".into()))?;

    entities::UserType::from(&db)
        .update(
            user.clone(),
            entities::UpdateAnyUser {
                mfa_secret: Some(Some(mfa.get_secret())),
                mfa_backup_codes: Some(Some(backup_codes.hashed_codes)),
                ..Default::default()
            },
        )
        .await
        .map_err(|_| Error::InternalServerError("Error updating user".into()))?;

    Ok(Json(backup_codes.codes))
}

pub async fn verify(
    jar: CookieJar,
    session: Session,
    project: Option<Project>,
    Config(config): Config,
    Database(db): Database,
    Json(body): Json<CVDRBody>,
) -> Result<impl IntoResponse, Error> {
    let Ok(Some(retries)) = session.get::<u8>(&SessionKey::MfaRetries.to_string()).await else {
        return Err(Error::BadRequest(
            "You have not started the login process".into(),
        ));
    };
    if retries == 0 {
        session
            .remove::<String>(&SessionKey::LoginUserId.to_string())
            .await
            .map_err(|_| Error::BadRequest("You have not started the login process".into()))?;
        session
            .remove::<u8>(&SessionKey::MfaRetries.to_string())
            .await
            .map_err(|_| Error::BadRequest("You have not started the login process".into()))?;

        return Err(Error::BadRequest(
            "You have exceeded the maximum number of retries".into(),
        ));
    }

    let Ok(Some(user_id)) = session
        .get::<String>(&SessionKey::LoginUserId.to_string())
        .await
    else {
        return Err(Error::BadRequest(
            "You have not started the login process".into(),
        ));
    };

    let user = entities::UserType::from(&db)
        .find_by_id(&user_id)
        .one()
        .await?;
    let Some(mfa_secret) = user.mfa_secret.clone() else {
        return Err(Error::BadRequest("MFA is disabled".into()));
    };

    let mfa = Mfa::new(
        Mfa::secret_from_string(mfa_secret),
        user.email.clone(),
        &project.map(|Project(p)| p),
    );
    if !mfa
        .verify(
            &body.code,
            VerificationMethod::All {
                db: &db,
                user: Box::new(user.clone()),
            },
        )
        .await?
    {
        return Err(Error::BadRequest("Invalid MFA code".into()));
    }

    let (_, jar) = auth::authenticate(&db, &config.clone(), &user, jar).await?;
    Ok((jar, Json(user)))
}

pub async fn disable(
    project: Option<Project>,
    AnyUser(user, _): AnyUser,
    Database(db): Database,
    Json(body): Json<CVDRBody>,
) -> Result<impl IntoResponse, Error> {
    let Some(mfa_secret) = user.mfa_secret.clone() else {
        return Err(Error::BadRequest("MFA is already disabled".into()));
    };

    let mfa = Mfa::new(
        Mfa::secret_from_string(mfa_secret),
        user.email.clone(),
        &project.map(|Project(p)| p),
    );
    if !mfa
        .verify(
            &body.code,
            VerificationMethod::All {
                db: &db,
                user: Box::new(user.clone()),
            },
        )
        .await?
    {
        return Err(Error::BadRequest("Invalid MFA code".into()));
    }

    entities::UserType::from(&db)
        .update(
            user.clone(),
            entities::UpdateAnyUser {
                mfa_secret: Some(None),
                mfa_backup_codes: Some(None),
                ..Default::default()
            },
        )
        .await
        .map_err(|_| Error::InternalServerError("Error updating user".into()))?;

    Ok(Json(Value::Null))
}

pub async fn regenerate(
    project: Option<Project>,
    AnyUser(user, _): AnyUser,
    Database(db): Database,
    Json(body): Json<CVDRBody>,
) -> Result<impl IntoResponse, Error> {
    let Some(mfa_secret) = user.mfa_secret.clone() else {
        return Err(Error::BadRequest("MFA is disabled".into()));
    };

    let mfa = Mfa::new(
        Mfa::secret_from_string(mfa_secret),
        user.email.clone(),
        &project.map(|Project(p)| p),
    );
    if !mfa.verify(&body.code, VerificationMethod::Code).await? {
        return Err(Error::BadRequest("Invalid MFA code".into()));
    }

    let backup_codes = Mfa::generate_codes()
        .await
        .map_err(|_| Error::InternalServerError("Error generating backup codes".into()))?;

    entities::UserType::from(&db)
        .update(
            user.clone(),
            entities::UpdateAnyUser {
                mfa_backup_codes: Some(Some(backup_codes.hashed_codes)),
                ..Default::default()
            },
        )
        .await
        .map_err(|_| Error::InternalServerError("Error updating user".into()))?;

    Ok(Json(backup_codes.codes))
}
