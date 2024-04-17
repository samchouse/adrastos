use std::vec;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use totp_rs::{Algorithm, Secret, TOTP};

use crate::{
    db::postgres::Database,
    entities::{AnyUser, Project, UpdateAnyUser, UserType},
    error::Error,
};

pub enum VerificationMethod<'a> {
    Code,
    All {
        db: &'a Database,
        user: Box<AnyUser>,
    },
}

pub struct Mfa(TOTP);

pub struct BackupCodes {
    pub codes: Vec<String>,
    pub hashed_codes: Vec<String>,
}

impl Mfa {
    pub fn new(secret: Secret, account_name: String, project: &Option<Project>) -> Self {
        Self(
            TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret.to_bytes().unwrap(),
                Some(
                    project
                        .as_ref()
                        .map(|p| p.name.clone())
                        .unwrap_or("Adrastos".to_string()),
                ),
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
        self.0.get_qr_base64()
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

        let hashed_backup_codes = backup_codes
            .iter()
            .filter_map(|s| {
                Argon2::default()
                    .hash_password(s.as_bytes(), &SaltString::generate(&mut OsRng))
                    .ok()
                    .map(|hash| hash.to_string())
            })
            .collect::<Vec<_>>();
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

    pub async fn verify(&self, code: &str, method: VerificationMethod<'_>) -> Result<bool, Error> {
        if let VerificationMethod::All { user, db } = method {
            if code.len() == 10 && code.chars().all(char::is_alphanumeric) {
                let backup_code_index =
                    user.mfa_backup_codes
                        .clone()
                        .unwrap()
                        .iter()
                        .position(|b_code| {
                            Argon2::default()
                                .verify_password(
                                    code.as_bytes(),
                                    &PasswordHash::new(b_code.as_str()).unwrap(),
                                )
                                .is_ok()
                        });

                if let Some(backup_code_index) = backup_code_index {
                    let mut backup_codes = user.mfa_backup_codes.clone().unwrap();
                    backup_codes.remove(backup_code_index);

                    UserType::from(db)
                        .update(
                            *user,
                            UpdateAnyUser {
                                mfa_backup_codes: Some(Some(backup_codes)),
                                ..Default::default()
                            },
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
