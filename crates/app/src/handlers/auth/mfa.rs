use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use adrastos_core::{
    auth::{
        self,
        mfa::{Mfa, VerificationMethod},
    },
    config::Config,
    db::postgres::Database,
    entities::{UpdateAnyUser, UserType},
    error::Error,
};
use chrono::Duration;
use deadpool_redis::redis;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::RwLock;

use crate::{
    middleware::{project::Project, user},
    session::SessionKey,
};

#[derive(Deserialize)]
pub struct CVDRBody {
    code: String,
}

#[get("/enable")]
pub async fn enable(
    project: Project,
    user: user::RequiredAnyUser,
    redis_pool: web::Data<deadpool_redis::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    if user.mfa_secret.is_some() {
        return Err(Error::BadRequest("MFA is already enabled".into()));
    }

    let mfa = Mfa::new(Mfa::generate_secret(), user.email.clone(), &project);

    let mut conn = redis_pool.get().await.unwrap();
    redis::cmd("SETEX")
        .arg(format!("mfa:secret:{}", user.id))
        .arg(Duration::minutes(10).num_seconds())
        .arg(mfa.get_secret())
        .query_async(&mut conn)
        .await
        .map_err(|_| {
            Error::InternalServerError("An error ocurred while saving MFA details to Redis".into())
        })?;

    Ok(HttpResponse::Ok().json(json!({
        "secret": mfa.get_secret(),
        "qr_code": mfa.get_qr().unwrap(),
    })))
}

#[post("/confirm")]
pub async fn confirm(
    db: Database,
    project: Project,
    body: web::Json<CVDRBody>,
    user: user::RequiredAnyUser,
    redis_pool: web::Data<deadpool_redis::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    if user.mfa_secret.is_some() {
        return Err(Error::BadRequest("MFA is already enabled".into()));
    }

    let mut conn = redis_pool.get().await.unwrap();
    let mfa_secret = redis::cmd("GET")
        .arg(format!("mfa:secret:{}", user.id))
        .query_async(&mut conn)
        .await
        .map_err(|_| Error::InternalServerError("Error getting MFA details from Redis".into()))?;

    let mfa = Mfa::new(
        Mfa::secret_from_string(mfa_secret),
        user.email.clone(),
        &project,
    );
    if !mfa.verify(&body.code, VerificationMethod::Code).await? {
        return Err(Error::BadRequest("Invalid MFA code".into()));
    }

    let backup_codes = Mfa::generate_codes()
        .await
        .map_err(|_| Error::InternalServerError("Error generating backup codes".into()))?;

    redis::cmd("DEL")
        .arg(format!("mfa:secret:{}", user.id))
        .query_async(&mut conn)
        .await
        .map_err(|_| Error::InternalServerError("Error deleting MFA details from Redis".into()))?;

    UserType::from(&db)
        .update(
            user.clone(),
            UpdateAnyUser {
                mfa_secret: Some(Some(mfa.get_secret())),
                mfa_backup_codes: Some(Some(backup_codes.hashed_codes)),
                ..Default::default()
            },
        )
        .await
        .map_err(|_| Error::InternalServerError("Error updating user".into()))?;

    Ok(HttpResponse::Ok().json(backup_codes.codes))
}

#[post("/verify")]
pub async fn verify(
    db: Database,
    project: Project,
    session: Session,
    body: web::Json<CVDRBody>,
    config: web::Data<RwLock<Config>>,
) -> actix_web::Result<impl Responder, Error> {
    let Ok(Some(retries)) = session.get::<u8>(&SessionKey::MfaRetries.to_string()) else {
        return Err(Error::BadRequest(
            "You have not started the login process".into(),
        ));
    };
    if retries == 0 {
        session
            .remove(&SessionKey::LoginUserId.to_string())
            .ok_or(Error::BadRequest(
                "You have not started the login process".into(),
            ))?;
        session
            .remove(&SessionKey::MfaRetries.to_string())
            .ok_or(Error::BadRequest(
                "You have not started the login process".into(),
            ))?;

        return Err(Error::BadRequest(
            "You have exceeded the maximum number of retries".into(),
        ));
    }

    let Ok(Some(user_id)) = session.get::<String>(&SessionKey::LoginUserId.to_string()) else {
        return Err(Error::BadRequest(
            "You have not started the login process".into(),
        ));
    };

    let user = UserType::from(&db).find_by_id(&user_id).one().await?;
    let Some(mfa_secret) = user.mfa_secret.clone() else {
        return Err(Error::BadRequest("MFA is disabled".into()));
    };

    let mfa = Mfa::new(
        Mfa::secret_from_string(mfa_secret),
        user.email.clone(),
        &project,
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

    let auth = auth::authenticate(&db, &config.read().await.clone(), &user).await?;
    Ok(HttpResponse::Ok().cookie(auth.cookie).json(user))
}

#[post("/disable")]
pub async fn disable(
    db: Database,
    project: Project,
    body: web::Json<CVDRBody>,
    user: user::RequiredAnyUser,
) -> actix_web::Result<impl Responder, Error> {
    let Some(mfa_secret) = user.mfa_secret.clone() else {
        return Err(Error::BadRequest("MFA is already disabled".into()));
    };

    let mfa = Mfa::new(
        Mfa::secret_from_string(mfa_secret),
        user.email.clone(),
        &project,
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

    UserType::from(&db)
        .update(
            user.clone(),
            UpdateAnyUser {
                mfa_secret: Some(None),
                mfa_backup_codes: Some(None),
                ..Default::default()
            },
        )
        .await
        .map_err(|_| Error::InternalServerError("Error updating user".into()))?;

    Ok(HttpResponse::Ok().json(Value::Null))
}

#[post("/codes/regenerate")]
pub async fn regenerate(
    db: Database,
    project: Project,
    body: web::Json<CVDRBody>,
    user: user::RequiredAnyUser,
) -> actix_web::Result<impl Responder, Error> {
    let Some(mfa_secret) = user.mfa_secret.clone() else {
        return Err(Error::BadRequest("MFA is disabled".into()));
    };

    let mfa = Mfa::new(
        Mfa::secret_from_string(mfa_secret),
        user.email.clone(),
        &project,
    );
    if !mfa.verify(&body.code, VerificationMethod::Code).await? {
        return Err(Error::BadRequest("Invalid MFA code".into()));
    }

    let backup_codes = Mfa::generate_codes()
        .await
        .map_err(|_| Error::InternalServerError("Error generating backup codes".into()))?;

    UserType::from(&db)
        .update(
            user.clone(),
            UpdateAnyUser {
                mfa_backup_codes: Some(Some(backup_codes.hashed_codes)),
                ..Default::default()
            },
        )
        .await
        .map_err(|_| Error::InternalServerError("Error updating user".into()))?;

    Ok(HttpResponse::Ok().json(backup_codes.codes))
}
