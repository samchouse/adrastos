use std::collections::HashMap;

use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use adrastos_core::{
    auth::{
        self,
        mfa::{Mfa, VerificationMethod},
    },
    config::Config,
    entities::{Mutate, User, UserIden},
    error::Error,
};
use chrono::Duration;
use deadpool_redis::redis;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::{middleware::user, session::SessionKey};

#[derive(Deserialize)]
pub struct CVDBody {
    code: String,
}

#[get("/enable")]
pub async fn enable(
    user: user::RequiredUser,
    redis_pool: web::Data<deadpool_redis::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    if user.mfa_secret.is_some() {
        return Err(Error::BadRequest("MFA is already enabled".into()));
    }

    let mfa = Mfa::new(Mfa::generate_secret(), user.email.clone());

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
        "success": true,
        "message": "MFA process successfully started",
        "secret": mfa.get_secret(),
        "qr_code": mfa.get_qr().unwrap(),
    })))
}

#[post("/confirm")]
pub async fn confirm(
    user: user::RequiredUser,
    body: web::Json<CVDBody>,
    db_pool: web::Data<deadpool_postgres::Pool>,
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

    let mfa = Mfa::new(Mfa::secret_from_string(mfa_secret), user.email.clone());
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

    user.update(
        &db_pool,
        &HashMap::from([
            (
                UserIden::MfaSecret.to_string(),
                Value::from(mfa.get_secret()),
            ),
            (
                UserIden::MfaBackupCodes.to_string(),
                Value::from(Some(backup_codes.hashed_codes)),
            ),
        ]),
    )
    .await
    .map_err(|_| Error::InternalServerError("Error updating user".into()))?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "MFA successfully enabled",
        "codes": backup_codes.codes,
    })))
}

#[post("/verify")]
pub async fn verify(
    session: Session,
    body: web::Json<CVDBody>,
    config: web::Data<Mutex<Config>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let Ok(Some(retries)) = session.get::<u8>(&SessionKey::MfaRetries.to_string()) else {
        return Err(Error::BadRequest("You have not started the login process".into()));
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
        return Err(Error::BadRequest("You have not started the login process".into()));
    };

    let user = User::select().by_id(&user_id).finish(&db_pool).await?;
    let Some(mfa_secret) = user.mfa_secret.clone() else {
        return Err(Error::BadRequest("MFA is disabled".into()));
    };

    let mfa = Mfa::new(Mfa::secret_from_string(mfa_secret), user.email.clone());
    mfa.verify(
        &body.code,
        VerificationMethod::All {
            db_pool: db_pool.clone(),
            user: Box::new(user.clone()),
        },
    )
    .await?;

    let auth = auth::authenticate(&db_pool, &config.lock().await.clone(), &user).await?;
    Ok(HttpResponse::Ok().cookie(auth.cookie).json(json!({
        "success": true,
        "user": user,
        "accessToken": auth.token.clone().token,
    })))
}

#[post("/disable")]
pub async fn disable(
    body: web::Json<CVDBody>,
    user: user::RequiredUser,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let Some(mfa_secret) = user.mfa_secret.clone() else {
        return Err(Error::BadRequest("MFA is already disabled".into()));
    };

    let mfa = Mfa::new(Mfa::secret_from_string(mfa_secret), user.email.clone());
    mfa.verify(
        &body.code,
        VerificationMethod::All {
            db_pool: db_pool.clone(),
            user: Box::new(user.clone()),
        },
    )
    .await?;

    user.update(
        &db_pool,
        &HashMap::from([
            (UserIden::MfaSecret.to_string(), Value::from(None::<String>)),
            (
                UserIden::MfaBackupCodes.to_string(),
                Value::from(None::<Vec<String>>),
            ),
        ]),
    )
    .await
    .map_err(|_| Error::InternalServerError("Error updating user".into()))?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "MFA successfully disabled",
    })))
}

#[post("/codes/regenerate")]
pub async fn regenerate(
    user: user::RequiredUser,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    if user.mfa_secret.is_none() {
        return Err(Error::BadRequest("MFA is disabled".into()));
    };

    let backup_codes = Mfa::generate_codes()
        .await
        .map_err(|_| Error::InternalServerError("Error generating backup codes".into()))?;

    user.update(
        &db_pool,
        &HashMap::from([(
            UserIden::MfaBackupCodes.to_string(),
            Value::from(Some(backup_codes.hashed_codes)),
        )]),
    )
    .await
    .map_err(|_| Error::InternalServerError("Error updating user".into()))?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "MFA backup codes successfully regenerated",
        "codes": backup_codes.codes,
    })))
}
