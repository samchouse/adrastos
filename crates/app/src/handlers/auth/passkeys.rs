use adrastos_core::{
    auth::{self, passkeys},
    entities,
    error::Error,
    id::Id,
};
use axum::{
    extract::Path,
    http::{header, HeaderMap},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use tower_sessions::Session;
use webauthn_rs::prelude::{
    CredentialID, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
    RegisterPublicKeyCredential,
};

use crate::{
    middleware::extractors::{AnyUser, Config, Database, Project},
    session::SessionKey,
    state::AppState,
};

#[derive(Deserialize)]
pub struct UpdateBody {
    name: String,
}

#[derive(Deserialize)]
pub struct RegisterFinishBody {
    name: String,
    passkey: RegisterPublicKeyCredential,
}

#[derive(Deserialize)]
pub struct LoginBody {
    id: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/update/:id", post(update))
        .route("/delete/:id", axum::routing::delete(delete))
        .route("/register/start", post(register_start))
        .route("/register/finish", post(register_finish))
        .route("/login/start", post(login_start))
        .route("/login/finish", post(login_finish))
}

pub async fn list(AnyUser(user, _): AnyUser) -> Result<impl IntoResponse, Error> {
    Ok(Json(user.passkeys.clone().unwrap_or_default()))
}

pub async fn update(
    Path(id): Path<String>,
    AnyUser(user, _): AnyUser,
    Database(db): Database,
    Json(body): Json<UpdateBody>,
) -> Result<impl IntoResponse, Error> {
    let passkey = user
        .passkeys
        .clone()
        .unwrap()
        .into_iter()
        .find(|pk| pk.id == id)
        .unwrap();

    passkey
        .update(
            &db,
            entities::UpdatePasskey {
                name: Some(body.name.clone()),
                ..Default::default()
            },
        )
        .await?;

    let passkey = entities::Passkey::find_by_id(&id).one(&db).await?;
    Ok(Json(passkey))
}

pub async fn delete(
    Database(db): Database,
    AnyUser(user, _): AnyUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let passkey = user
        .passkeys
        .clone()
        .unwrap()
        .into_iter()
        .find(|pk| pk.id == id)
        .unwrap();

    passkey.delete(&db).await?;

    Ok(Json(Value::Null))
}

pub async fn register_start(
    session: Session,
    headers: HeaderMap,
    project: Option<Project>,
    Config(config): Config,
    AnyUser(user, _): AnyUser,
) -> Result<impl IntoResponse, Error> {
    let webauthn = passkeys::build_webauthn(
        headers.get(header::ORIGIN),
        &project.map(|Project(p)| p),
        &config,
    )
    .await;

    let (ccr, registration) = webauthn
        .start_passkey_registration(
            &user.id,
            &user.username,
            &format!("{} {}", user.first_name, user.last_name),
            user.passkeys.clone().map(|pks| {
                pks.iter()
                    .map(|pk| CredentialID::from(pk.cred_id.clone().into_bytes()))
                    .collect::<Vec<_>>()
            }),
        )
        .unwrap();

    session
        .insert(&SessionKey::UserId.to_string(), user.id.clone())
        .await
        .unwrap();
    session
        .insert(&SessionKey::UserType.to_string(), user.clone())
        .await
        .unwrap();
    session
        .insert(&SessionKey::PasskeyRegistration.to_string(), registration)
        .await
        .unwrap();

    Ok(Json(ccr))
}

pub async fn register_finish(
    session: Session,
    headers: HeaderMap,
    project: Option<Project>,
    Config(config): Config,
    Database(db): Database,
    body: Json<RegisterFinishBody>,
) -> Result<impl IntoResponse, Error> {
    let user = entities::UserType::from(&db)
        .find_by_id(
            &session
                .remove::<String>(&SessionKey::UserId.to_string())
                .await
                .unwrap()
                .unwrap(),
        )
        .one()
        .await?;

    let registration = session
        .remove::<PasskeyRegistration>(&SessionKey::PasskeyRegistration.to_string())
        .await
        .unwrap()
        .unwrap();

    let webauthn = passkeys::build_webauthn(
        headers.get(header::ORIGIN),
        &project.map(|Project(p)| p),
        &config,
    )
    .await;

    let passkey = webauthn
        .finish_passkey_registration(&body.passkey, &registration)
        .unwrap();

    let passkey = entities::Passkey {
        id: Id::new().to_string(),
        name: body.name.clone(),
        user_id: user.id.clone(),
        cred_id: serde_json::to_string(passkey.cred_id()).unwrap(),
        last_used: None,
        data: passkey,
        created_at: Utc::now(),
        updated_at: None,
    };

    passkey.create(&db).await?;

    Ok(Json(Value::Null))
}

pub async fn login_start(
    session: Session,
    headers: HeaderMap,
    project: Option<Project>,
    Config(config): Config,
    Database(db): Database,
    body: Option<Json<LoginBody>>,
) -> Result<impl IntoResponse, Error> {
    let webauthn = passkeys::build_webauthn(
        headers.get(header::ORIGIN),
        &project.map(|Project(p)| p),
        &config,
    )
    .await;

    let allowed = {
        if let Some(Json(body)) = &body {
            let user = entities::UserType::from(&db)
                .find_by_id(&body.id)
                .join(entities::AnyUserJoin::Passkeys)
                .one()
                .await?;

            session
                .insert(&SessionKey::UserId.to_string(), user.id.clone())
                .await
                .unwrap();

            if let Some(passkeys) = user.passkeys {
                passkeys.into_iter().map(|pk| pk.data).collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    };

    let (rcr, authentication) = webauthn.start_passkey_authentication(&allowed).unwrap();

    session
        .insert(
            &SessionKey::PasskeyAuthentication.to_string(),
            authentication,
        )
        .await
        .unwrap();

    Ok(Json(rcr))
}

pub async fn login_finish(
    jar: CookieJar,
    session: Session,
    headers: HeaderMap,
    project: Option<Project>,
    Config(config): Config,
    Database(db): Database,
    body: Json<PublicKeyCredential>,
) -> Result<impl IntoResponse, Error> {
    let (user, passkey) = {
        let user_id = session
            .remove::<String>(&SessionKey::UserId.to_string())
            .await
            .unwrap();
        if let Some(user_id) = user_id {
            let user = entities::UserType::from(&db)
                .find_by_id(&user_id)
                .join(entities::AnyUserJoin::Passkeys)
                .one()
                .await?;

            (
                user.clone(),
                user.passkeys
                    .unwrap()
                    .into_iter()
                    .find(|pk| pk.cred_id == body.id)
                    .unwrap(),
            )
        } else {
            let passkey = entities::Passkey::find()
                .by_cred_id(body.id.clone())
                .one(&db)
                .await?;

            let user = entities::UserType::from(&db)
                .find_by_id(&passkey.user_id)
                .join(entities::AnyUserJoin::Passkeys)
                .one()
                .await?;

            (user.clone(), passkey)
        }
    };

    let mut authentication = session
        .remove::<PasskeyAuthentication>(&SessionKey::PasskeyAuthentication.to_string())
        .await
        .unwrap()
        .unwrap();

    let webauthn = passkeys::build_webauthn(
        headers.get(header::ORIGIN),
        &project.map(|Project(p)| p),
        &config,
    )
    .await;

    let result = webauthn
        .finish_passkey_authentication(&body, &mut authentication, Some(vec![passkey.data.clone()]))
        .unwrap();

    let mut passkey_update = entities::UpdatePasskey {
        last_used: Some(Some(Utc::now())),
        ..Default::default()
    };

    if result.needs_update() {
        let mut passkey = passkey.clone();
        passkey.data.update_credential(&result);
        passkey_update.data = Some(passkey.data.clone());
    }

    passkey
        .update(&db, passkey_update)
        .await
        .map_err(|_| Error::InternalServerError("Unable to update passkey".to_string()))?;

    let (_, jar) = auth::authenticate(&db, &config.clone(), &user, jar).await?;
    Ok((jar, Json(user)).into_response())
}
