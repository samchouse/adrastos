use std::{collections::HashMap, vec};

use actix_web::{get, post, web, HttpResponse, Responder};
use adrastos_core::{
    entities::{Mutate, UserIden},
    error::Error,
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use deadpool_redis::redis;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use serde_json::{json, Value};
use totp_rs::{Algorithm, Secret, TOTP};

use crate::middleware::user;

#[derive(Deserialize)]
pub struct CVDBody {
    code: String,
}

fn create_totp(secret: Secret, account_name: String) -> TOTP {
    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().unwrap(),
        Some("Adrastos".to_string()), // TODO(@Xenfo): Change to project name depending on config,
        account_name,
    )
    .unwrap()
}

#[get("/enable")]
pub async fn enable(
    user: user::RequiredUser,
    redis_pool: web::Data<deadpool_redis::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let user = user.clone();
    if user.mfa_secret.is_some() {
        return Err(Error::BadRequest("MFA is already enabled".into()));
    }

    let totp = create_totp(Secret::generate_secret(), user.email);

    let mut conn = redis_pool.get().await.unwrap();
    let _: String = redis::cmd("SETEX")
        .arg(format!("mfa:secret:{}", user.id))
        .arg(60 * 10)
        .arg(totp.get_secret_base32())
        .query_async(&mut conn)
        .await
        .map_err(|_| {
            Error::InternalServerError("An error ocurred while saving MFA details to Redis".into())
        })?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "MFA process successfully started",
        "secret": totp.get_secret_base32(),
        "qr_code": totp.get_qr().unwrap(),
    })))
}

#[post("/confirm")]
pub async fn confirm(
    user: user::RequiredUser,
    body: web::Json<CVDBody>,
    db_pool: web::Data<deadpool_postgres::Pool>,
    redis_pool: web::Data<deadpool_redis::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let user = user.clone();
    if user.mfa_secret.is_some() {
        return Err(Error::BadRequest("MFA is already enabled".into()));
    }

    let mut conn = redis_pool.get().await.unwrap();
    let mfa_secret = redis::cmd("GET")
        .arg(format!("mfa:secret:{}", user.id))
        .query_async(&mut conn)
        .await
        .map_err(|_| Error::InternalServerError("Error getting MFA details from Redis".into()))?;

    let totp = create_totp(Secret::Encoded(mfa_secret), user.email.clone());
    if !totp.check_current(&body.code).unwrap() {
        return Err(Error::BadRequest("Invalid MFA code".into()));
    }

    let backup_codes = vec!["".to_string(); 10]
        .iter()
        .map(|_| {
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect::<String>()
        })
        .collect::<Vec<_>>();

    let closure_backup_codes = backup_codes.clone();
    let hashed_backup_codes = web::block(move || {
        closure_backup_codes
            .iter()
            .filter_map(|s| {
                Argon2::default()
                    .hash_password(s.as_bytes(), &SaltString::generate(&mut OsRng))
                    .ok()
                    .map(|hash| hash.to_string())
            })
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|_| Error::InternalServerError("Error hashing backup codes".into()))?;
    if hashed_backup_codes.len() != 10 {
        return Err(Error::InternalServerError(
            "Error hashing backup codes".into(),
        ));
    }

    println!("{:#?}", hashed_backup_codes);

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
                Value::from(totp.get_secret_base32()),
            ),
            (
                UserIden::MfaBackupCodes.to_string(),
                Value::from(hashed_backup_codes),
            ),
        ]),
    )
    .await
    .map_err(|_| Error::InternalServerError("Error updating user".into()))?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "MFA successfully enabled",
        "codes": backup_codes,
    })))
}

#[post("/verify")]
pub async fn verify() -> actix_web::Result<impl Responder, Error> {
    Ok("Hello")
}

#[post("/disable")]
pub async fn disable(
    body: web::Json<CVDBody>,
    user: user::RequiredUser,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let user = user.clone();
    let Some(mfa_secret) = &user.mfa_secret else {
        return Err(Error::BadRequest("MFA is already disabled".into()));
    };

    let totp = create_totp(Secret::Encoded(mfa_secret.to_string()), user.email.clone());
    if !totp.check_current(&body.code).unwrap() {
        return Err(Error::BadRequest("Invalid MFA code".into()));
    }

    user.update(
        &db_pool,
        &HashMap::from([
            (UserIden::MfaSecret.to_string(), Value::Null),
            (UserIden::MfaBackupCodes.to_string(), Value::Null),
        ]),
    )
    .await
    .map_err(|_| Error::InternalServerError("Error updating user".into()))?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "MFA successfully disabled",
    })))
}
