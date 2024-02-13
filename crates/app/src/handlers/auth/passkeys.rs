use actix_session::Session;
use actix_web::{
    cookie::{Cookie, SameSite},
    post, web, HttpResponse, Responder,
};
use adrastos_core::{
    auth, config,
    entities::{UpdateUser, User},
    error::Error,
};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::Mutex;
use webauthn_rs::prelude::*;

use crate::{middleware::user, session::SessionKey};

#[derive(Deserialize)]
pub struct LoginBody {
    id: String,
}

#[post("/register/start")]
pub async fn register_start(
    user: user::RequiredUser,
    session: Session,
) -> actix_web::Result<impl Responder, Error> {
    let webauthn = WebauthnBuilder::new("arch", &Url::parse("https://arch:5173").unwrap())
        .unwrap()
        .rp_name("Adrastos")
        .build()
        .unwrap();

    let (ccr, registration) = webauthn
        .start_passkey_registration(
            &user.id,
            &user.username,
            &format!("{} {}", user.first_name, user.last_name),
            Some(
                user.passkeys
                    .iter()
                    .map(|pk| pk.cred_id().clone())
                    .collect(),
            ),
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

    let webauthn = WebauthnBuilder::new("arch", &Url::parse("https://arch:5173").unwrap())
        .unwrap()
        .rp_name("Adrastos")
        .build()
        .unwrap();

    let passkey = webauthn
        .finish_passkey_registration(&body, &registration)
        .unwrap();

    user.update(
        &db_pool,
        UpdateUser {
            passkeys: Some([user.passkeys.clone(), vec![passkey]].concat()),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| Error::InternalServerError("Unable to update user".to_string()))?;

    Ok(HttpResponse::Ok().json(Value::Null))
}

#[post("/login/start")]
pub async fn login_start(
    session: Session,
    body: web::Json<Option<LoginBody>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let webauthn = WebauthnBuilder::new("arch", &Url::parse("https://arch:5173").unwrap())
        .unwrap()
        .rp_name("Adrastos")
        .build()
        .unwrap();

    let allowed = {
        if let Some(body) = &body.0 {
            let user = User::find_by_id(&body.id).one(&db_pool).await?;

            session
                .insert(SessionKey::UserId.to_string(), user.id.clone())
                .unwrap();

            user.passkeys
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
    session: Session,
    config: web::Data<Mutex<config::Config>>,
    req: web::Json<PublicKeyCredential>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let (user, allowed) = {
        let user_id = session
            .get::<String>(&SessionKey::UserId.to_string())
            .unwrap();
        if let Some(user_id) = user_id {
            (User::find_by_id(&user_id).one(&db_pool).await?, None)
        } else {
            let user = User::find()
                .all(&db_pool)
                .await?
                .into_iter()
                .find(|user| {
                    user.passkeys
                        .clone()
                        .iter()
                        .any(|pk| pk.cred_id().to_string() == req.id)
                })
                .unwrap();
            (user.clone(), Some(user.passkeys.clone()))
        }
    };

    let mut authentication = session
        .get::<PasskeyAuthentication>(&SessionKey::PasskeyAuthentication.to_string())
        .unwrap()
        .unwrap();

    let webauthn = WebauthnBuilder::new("arch", &Url::parse("https://arch:5173").unwrap())
        .unwrap()
        .rp_name("Adrastos")
        .build()
        .unwrap();

    let result = webauthn
        .finish_passkey_authentication(&req, &mut authentication, allowed)
        .unwrap();

    session.remove(&SessionKey::UserId.to_string());
    session.remove(&SessionKey::PasskeyAuthentication.to_string());

    user.update(
        &db_pool,
        UpdateUser {
            passkeys: Some(
                user.passkeys
                    .clone()
                    .iter_mut()
                    .map(|pk| {
                        pk.update_credential(&result);
                        pk.clone()
                    })
                    .collect::<Vec<_>>(),
            ),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| Error::InternalServerError("Unable to update user".to_string()))?;

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
