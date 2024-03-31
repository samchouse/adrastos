use actix_session::Session;
use actix_web::{
    cookie::{Cookie, SameSite},
    delete, get,
    http::header,
    post, web, HttpRequest, HttpResponse, Responder,
};
use adrastos_core::{
    auth::{self, passkeys},
    config::Config,
    db::postgres::Database,
    entities::{AnyUserJoin, Passkey, UpdatePasskey, User, UserJoin, UserType},
    error::Error,
    id::Id,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::RwLock;
use webauthn_rs::prelude::{
    Base64UrlSafeData, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
    RegisterPublicKeyCredential,
};

use crate::{
    middleware::{project::Project, user::RequiredAnyUser},
    session::SessionKey,
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

#[get("/list")]
pub async fn list(user: RequiredAnyUser) -> actix_web::Result<impl Responder, Error> {
    Ok(HttpResponse::Ok().json(user.passkeys.clone().unwrap_or_default()))
}

#[post("/update/{id}")]
pub async fn update(
    db: Database,
    user: RequiredAnyUser,
    id: web::Path<String>,
    body: web::Json<UpdateBody>,
) -> actix_web::Result<impl Responder, Error> {
    let passkey = user
        .passkeys
        .clone()
        .unwrap()
        .into_iter()
        .find(|pk| pk.id == id.to_string())
        .unwrap();

    passkey
        .update(
            &db,
            UpdatePasskey {
                name: Some(body.name.clone()),
                ..Default::default()
            },
        )
        .await?;

    let passkey = Passkey::find_by_id(&id).one(&db).await?;
    Ok(HttpResponse::Ok().json(passkey))
}

#[delete("/delete/{id}")]
pub async fn delete(
    db: Database,
    user: RequiredAnyUser,
    id: web::Path<String>,
) -> actix_web::Result<impl Responder, Error> {
    let passkey = user
        .passkeys
        .clone()
        .unwrap()
        .into_iter()
        .find(|pk| pk.id == id.to_string())
        .unwrap();

    passkey.delete(&db).await?;

    Ok(HttpResponse::Ok().json(Value::Null))
}

#[post("/register/start")]
pub async fn register_start(
    req: HttpRequest,
    session: Session,
    project: Project,
    user: RequiredAnyUser,
    config: web::Data<RwLock<Config>>,
) -> actix_web::Result<impl Responder, Error> {
    let webauthn =
        passkeys::build_webauthn(req.headers().get(header::ORIGIN), &project, &config).await;

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
    db: Database,
    req: HttpRequest,
    session: Session,
    project: Project,
    config: web::Data<RwLock<Config>>,
    body: web::Json<RegisterFinishBody>,
) -> actix_web::Result<impl Responder, Error> {
    let user = User::find_by_id(
        &session
            .get::<String>(&SessionKey::UserId.to_string())
            .unwrap()
            .unwrap(),
    )
    .one(&db)
    .await?;

    let registration = session
        .get::<PasskeyRegistration>(&SessionKey::PasskeyRegistration.to_string())
        .unwrap()
        .unwrap();

    session.remove(&SessionKey::UserId.to_string());
    session.remove(&SessionKey::PasskeyRegistration.to_string());

    let webauthn =
        passkeys::build_webauthn(req.headers().get(header::ORIGIN), &project, &config).await;

    let passkey = webauthn
        .finish_passkey_registration(&body.passkey, &registration)
        .unwrap();

    let passkey = Passkey {
        id: Id::new().to_string(),
        name: body.name.clone(),
        user_id: user.id.clone(),
        cred_id: passkey.cred_id().to_string(),
        last_used: None,
        data: passkey,
        created_at: Utc::now(),
        updated_at: None,
    };

    passkey.create(&db).await?;

    Ok(HttpResponse::Ok().json(Value::Null))
}

#[post("/login/start")]
pub async fn login_start(
    db: Database,
    req: HttpRequest,
    session: Session,
    project: Project,
    config: web::Data<RwLock<Config>>,
    body: web::Json<Option<LoginBody>>,
) -> actix_web::Result<impl Responder, Error> {
    let webauthn =
        passkeys::build_webauthn(req.headers().get(header::ORIGIN), &project, &config).await;

    let allowed = {
        if let Some(body) = &body.0 {
            let user = User::find_by_id(&body.id)
                .join(UserJoin::Passkeys)
                .one(&db)
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
    db: Database,
    req: HttpRequest,
    session: Session,
    project: Project,
    config: web::Data<RwLock<Config>>,
    body: web::Json<PublicKeyCredential>,
) -> actix_web::Result<impl Responder, Error> {
    let (user, passkey) = {
        let user_id = session
            .get::<String>(&SessionKey::UserId.to_string())
            .unwrap();
        if let Some(user_id) = user_id {
            let user = UserType::from(&db)
                .find_by_id(&user_id)
                .join(AnyUserJoin::Passkeys)
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
            let passkey = Passkey::find().by_cred_id(body.id.clone()).one(&db).await?;

            let user = UserType::from(&db)
                .find_by_id(&passkey.user_id)
                .join(AnyUserJoin::Passkeys)
                .one()
                .await?;

            (user.clone(), passkey)
        }
    };

    let mut authentication = session
        .get::<PasskeyAuthentication>(&SessionKey::PasskeyAuthentication.to_string())
        .unwrap()
        .unwrap();

    let webauthn =
        passkeys::build_webauthn(req.headers().get(header::ORIGIN), &project, &config).await;

    let result = webauthn
        .finish_passkey_authentication(&body, &mut authentication, Some(vec![passkey.data.clone()]))
        .unwrap();

    session.remove(&SessionKey::UserId.to_string());
    session.remove(&SessionKey::PasskeyAuthentication.to_string());

    let mut passkey_update = UpdatePasskey {
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

    let auth = auth::authenticate(&db, &config.read().await.clone(), &user).await?;
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
