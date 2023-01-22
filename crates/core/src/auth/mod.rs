use actix_web::web;
use argon2::{
    password_hash::{
        rand_core::OsRng, Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{
    config::{self, ConfigKey},
    entities::User,
    id::Id,
};

pub mod oauth2;

#[derive(Clone)]
pub enum TokenType {
    Access,
    Refresh,
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            TokenType::Access => "access".to_string(),
            TokenType::Refresh => "refresh".to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Claims {
    pub jti: String,
    iat: usize,
    exp: usize,
    pub sub: String,
    #[serde(rename = "type")]
    token_type: String,
}

#[derive(Clone)]
pub struct TokenInfo {
    pub token: String,
    pub claims: Claims,
    pub token_type: TokenType,
    pub expires_at: DateTime<Utc>,
}

pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
    let hash = PasswordHash::new(hash)?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .is_ok())
}

impl TokenType {
    pub fn sign(
        &self,
        config: web::Data<config::Config>,
        user: User,
    ) -> Result<TokenInfo, Option<jsonwebtoken::errors::Error>> {
        let secret_key = config.get(ConfigKey::SecretKey).ok_or(None)?;

        let expires_at = match self {
            TokenType::Access => Utc::now() + Duration::minutes(15),
            TokenType::Refresh => Utc::now() + Duration::days(15),
        };
        let claims = Claims {
            jti: Id::new().to_string(),
            iat: Utc::now().timestamp() as usize,
            exp: expires_at.timestamp() as usize,
            sub: user.id,
            token_type: self.to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret_key.as_bytes()),
        )
        .map_err(|err| Some(err))?;

        Ok(TokenInfo {
            token,
            claims,
            expires_at,
            token_type: self.clone(),
        })
    }

    pub fn verify(
        config: web::Data<config::Config>,
        token: String,
    ) -> Result<TokenInfo, Option<jsonwebtoken::errors::Error>> {
        let secret_key = config.get(ConfigKey::SecretKey).ok_or(None)?;

        let claims = decode::<Claims>(
            token.as_str(),
            &DecodingKey::from_secret(secret_key.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|err| Some(err))?;

        let token_type = match claims.token_type.as_str() {
            "access" => TokenType::Access,
            "refresh" => TokenType::Refresh,
            _ => return Err(None),
        };

        Ok(TokenInfo {
            token,
            claims: claims.clone(),
            expires_at: Utc.timestamp_opt(claims.exp as i64, 0).unwrap(),
            token_type,
        })
    }
}
