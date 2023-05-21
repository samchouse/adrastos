use std::{collections::HashMap, vec};

use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use adrastos_core::{
    auth,
    config::Config,
    entities::{Mutate, User, UserIden},
    error::Error,
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use deadpool_redis::redis;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use serde_json::{json, Value};
use totp_rs::{Algorithm, Secret, TOTP};

use crate::{middleware::user, session::SessionKey};

#[derive(Deserialize)]
pub struct CVDBody {
    code: String,
}

struct BackupCodes {
    codes: Vec<String>,
    hashed_codes: Vec<String>,
}

async fn generate_codes() -> Result<BackupCodes, Error> {
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

    let block_backup_codes = backup_codes.clone();
    let hashed_backup_codes = web::block(move || {
        block_backup_codes
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

    Ok(BackupCodes {
        codes: backup_codes,
        hashed_codes: hashed_backup_codes,
    })
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

async fn verify_mfa(
    db_pool: &web::Data<deadpool_postgres::Pool>,
    user: &User,
    code: &str,
) -> Result<bool, Error> {
    let totp = create_totp(
        Secret::Encoded(user.mfa_secret.as_ref().unwrap().clone()),
        user.email.clone(),
    );

    if code.len() == 10 && code.chars().all(char::is_alphanumeric) {
        let block_user = user.clone();
        let block_code = code.to_string();
        let backup_code_index = web::block(move || {
            return block_user
                .mfa_backup_codes
                .clone()
                .unwrap()
                .iter()
                .position(|b_code| {
                    Argon2::default()
                        .verify_password(
                            block_code.as_bytes(),
                            &PasswordHash::new(b_code.as_str()).unwrap(),
                        )
                        .is_ok()
                });
        })
        .await
        .map_err(|_| {
            Error::InternalServerError(
                "An error occurred while verifying the backup MFA code".into(),
            )
        })?;

        if let Some(backup_code_index) = backup_code_index {
            let mut backup_codes = user.mfa_backup_codes.clone().unwrap();
            backup_codes.remove(backup_code_index);

            user.update(
                db_pool,
                &HashMap::from([(
                    UserIden::MfaBackupCodes.to_string(),
                    Value::from(Some(backup_codes)),
                )]),
            )
            .await
            .map_err(|_| {
                Error::InternalServerError("An error occurred while updating the user".into())
            })?;

            return Ok(true);
        }
    }

    if code.len() != 6 || !code.chars().all(char::is_numeric) {
        return Err(Error::BadRequest("Invalid MFA code format".into()));
    }

    Ok(totp.check_current(code).unwrap())
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

    let backup_codes = generate_codes()
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
                Value::from(totp.get_secret_base32()),
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
    config: web::Data<Config>,
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
    if user.mfa_secret.is_none() {
        return Err(Error::BadRequest("MFA is disabled".into()));
    };

    verify_mfa(&db_pool, &user, &body.code).await?;

    let auth = auth::authenticate(&db_pool, &config, &user).await?;
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
    let user = user.clone();
    if user.mfa_secret.is_none() {
        return Err(Error::BadRequest("MFA is already disabled".into()));
    };

    verify_mfa(&db_pool, &user, &body.code).await?;

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
    let user = user.clone();
    if user.mfa_secret.is_none() {
        return Err(Error::BadRequest("MFA is disabled".into()));
    };

    let backup_codes = generate_codes()
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
