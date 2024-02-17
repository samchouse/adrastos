use actix_session::Session;
use actix_web::{
    cookie::{Cookie, SameSite},
    get,
    http::header,
    post, web, HttpRequest, HttpResponse, Responder,
};
use adrastos_core::{
    auth::{self, passkeys},
    config,
    entities::{Passkey, UpdatePasskey, User, UserJoin},
    error::Error,
    id::Id,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::Mutex;
use webauthn_rs::prelude::{
    Base64UrlSafeData, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
    RegisterPublicKeyCredential,
};

use crate::{
    middleware::user::{self, RequiredUser},
    session::SessionKey,
};

#[derive(Deserialize)]
pub struct LoginBody {
    id: String,
}

#[get("/list")]
pub async fn list(user: RequiredUser) -> actix_web::Result<impl Responder, Error> {
    Ok(HttpResponse::Ok().json(user.passkeys.clone()))
}

#[post("/register/start")]
pub async fn register_start(
    req: HttpRequest,
    user: user::RequiredUser,
    session: Session,
) -> actix_web::Result<impl Responder, Error> {
    let webauthn =
        passkeys::build_webauthn(req.headers().get(header::ORIGIN).unwrap().to_str().unwrap());

    let (ccr, registration) = webauthn
        .start_passkey_registration(
            &user.id,
            &user.username,
            &format!("{} {}", user.first_name, user.last_name),
            user.passkeys.clone().map(|pks| {
                pks.iter()
                    .map(|pk| Base64UrlSafeData(pk.cred_id.clone().into_bytes()))
                    .collect::<Vec<_>>()
            }),
        )
        .unwrap();

    session
        .insert(SessionKey::UserId.to_string(), user.id.clone())
        .unwrap();
    session
        .insert(SessionKey::PasskeyRegistration.to_string(), registration)
        .unwrap();

    Ok(HttpResponse::Ok().json(ccr))
}

#[post("/register/finish")]
pub async fn register_finish(
    req: HttpRequest,
    session: Session,
    body: web::Json<RegisterPublicKeyCredential>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let user = User::find_by_id(
        &session
            .get::<String>(&SessionKey::UserId.to_string())
            .unwrap()
            .unwrap(),
    )
    .one(&db_pool)
    .await?;

    let registration = session
        .get::<PasskeyRegistration>(&SessionKey::PasskeyRegistration.to_string())
        .unwrap()
        .unwrap();

    session.remove(&SessionKey::UserId.to_string());
    session.remove(&SessionKey::PasskeyRegistration.to_string());

    let webauthn =
        passkeys::build_webauthn(req.headers().get(header::ORIGIN).unwrap().to_str().unwrap());

    let passkey = webauthn
        .finish_passkey_registration(&body, &registration)
        .unwrap();

    let passkey = Passkey {
        id: Id::new().to_string(),
        name: "Primary".to_string(),
        user_id: user.id.clone(),
        cred_id: passkey.cred_id().to_string(),
        last_used: None,
        data: passkey,
        created_at: Utc::now(),
        updated_at: None,
    };

    passkey.create(&db_pool).await?;

    Ok(HttpResponse::Ok().json(Value::Null))
}

#[post("/login/start")]
pub async fn login_start(
    req: HttpRequest,
    session: Session,
    body: web::Json<Option<LoginBody>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let webauthn =
        passkeys::build_webauthn(req.headers().get(header::ORIGIN).unwrap().to_str().unwrap());

    req.headers()
        .get(header::ORIGIN)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let allowed = {
        if let Some(body) = &body.0 {
            let user = User::find_by_id(&body.id)
                .join(UserJoin::Passkeys)
                .one(&db_pool)
                .await?;

            session
                .insert(SessionKey::UserId.to_string(), user.id.clone())
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
            SessionKey::PasskeyAuthentication.to_string(),
            authentication,
        )
        .unwrap();

    Ok(HttpResponse::Ok().json(rcr))
}

#[post("/login/finish")]
pub async fn login_finish(
    req: HttpRequest,
    session: Session,
    config: web::Data<Mutex<config::Config>>,
    body: web::Json<PublicKeyCredential>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let (user, passkey) = {
        let user_id = session
            .get::<String>(&SessionKey::UserId.to_string())
            .unwrap();
        if let Some(user_id) = user_id {
            (
                User::find_by_id(&user_id)
                    .join(UserJoin::Passkeys)
                    .one(&db_pool)
                    .await?,
                None,
            )
        } else {
            let passkey = Passkey::find()
                .by_cred_id(body.id.clone())
                .one(&db_pool)
                .await?;

            let user = User::find_by_id(&passkey.user_id)
                .join(UserJoin::Passkeys)
                .one(&db_pool)
                .await?;

            (user.clone(), Some(passkey))
        }
    };

    let mut authentication = session
        .get::<PasskeyAuthentication>(&SessionKey::PasskeyAuthentication.to_string())
        .unwrap()
        .unwrap();

    let webauthn =
        passkeys::build_webauthn(req.headers().get(header::ORIGIN).unwrap().to_str().unwrap());

    let result = webauthn
        .finish_passkey_authentication(&body, &mut authentication, passkey.map(|pk| vec![pk.data]))
        .unwrap();

    session.remove(&SessionKey::UserId.to_string());
    session.remove(&SessionKey::PasskeyAuthentication.to_string());

    if result.needs_update() {
        let mut passkey = user
            .passkeys
            .clone()
            .unwrap()
            .into_iter()
            .find(|pk| pk.cred_id == result.cred_id().to_string())
            .unwrap();
        passkey.data.update_credential(&result);
        passkey
            .update(
                &db_pool,
                UpdatePasskey {
                    passkey: Some(passkey.data.clone()),
                    ..Default::default()
                },
            )
            .await
            .map_err(|_| Error::InternalServerError("Unable to update passkey".to_string()))?;
    }

    let auth = auth::authenticate(&db_pool, &config.lock().await.clone(), &user).await?;
    Ok(HttpResponse::Ok()
        .cookie(auth.cookie.clone())
        .cookie(
            Cookie::build("isLoggedIn", true.to_string())
                .secure(true)
                .http_only(true)
                .same_site(SameSite::None)
                .path("/")
                .expires(auth.cookie.expires().unwrap())
                .finish(),
        )
        .json(user))
}
