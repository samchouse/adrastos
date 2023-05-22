use std::{collections::HashMap, vec};

use actix_web::web;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde_json::Value;
use totp_rs::{Algorithm, Secret, TOTP};

use crate::{
    entities::{User, UserIden, Mutate},
    error::Error,
};

pub enum VerificationMethod {
    Code,
    All {
        user: Box<User>,
        db_pool: web::Data<deadpool_postgres::Pool>,
    },
}

pub struct Mfa(TOTP);

pub struct BackupCodes {
    pub codes: Vec<String>,
    pub hashed_codes: Vec<String>,
}

impl Mfa {
    pub fn new(secret: Secret, account_name: String) -> Self {
        Self(
            TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret.to_bytes().unwrap(),
                Some("Adrastos".to_string()), // TODO(@Xenfo): Change to project name depending on config
                account_name,
            )
            .unwrap(),
        )
    }

    pub fn generate_secret() -> Secret {
        Secret::generate_secret()
    }

    pub fn secret_from_string(secret: String) -> Secret {
        Secret::Encoded(secret)
    }

    pub fn get_secret(&self) -> String {
        self.0.get_secret_base32()
    }

    pub fn get_qr(&self) -> Result<String, String> {
        self.0.get_qr()
    }

    pub async fn generate_codes() -> Result<BackupCodes, Error> {
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

    pub async fn verify(&self, code: &str, method: VerificationMethod) -> Result<bool, Error> {
        if let VerificationMethod::All { user, db_pool } = method {
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
                        &db_pool,
                        &HashMap::from([(
                            UserIden::MfaBackupCodes.to_string(),
                            Value::from(Some(backup_codes)),
                        )]),
                    )
                    .await
                    .map_err(|_| {
                        Error::InternalServerError(
                            "An error occurred while updating the user".into(),
                        )
                    })?;

                    return Ok(true);
                }
            }
        }

        if code.len() != 6 || !code.chars().all(char::is_numeric) {
            return Err(Error::BadRequest("Invalid MFA code format".into()));
        }

        Ok(self.0.check_current(code).unwrap())
    }
}
