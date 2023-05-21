use std::fmt;

use actix_web::{cookie::{time::OffsetDateTime, Cookie, Expiration}, web};
use argon2::{
    password_hash::{
        self, rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

use crate::{
    config::{self, Config, ConfigKey},
    entities::{Mutate, RefreshTokenTree, User},
    error::Error,
    id::Id,
};

pub mod oauth2;

#[derive(Clone, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TokenType::Access => "access",
            TokenType::Refresh => "refresh",
        };

        write!(f, "{name}")
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Claims {
    pub jti: String,
    iat: usize,
    exp: usize,
    pub sub: String,
    #[serde(rename = "type")]
    pub token_type: String,
}

#[derive(Clone)]
pub struct TokenInfo {
    pub token: String,
    pub claims: Claims,
    pub token_type: TokenType,
    pub expires_at: DateTime<Utc>,
}

pub struct Authentication {
    pub token: TokenInfo,
    pub cookie: Cookie<'static>,
}

pub fn hash_password(password: &str) -> Result<String, password_hash::Error> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, password_hash::Error> {
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &PasswordHash::new(hash)?)
        .is_ok())
}

pub async fn authenticate(
    db_pool: &web::Data<deadpool_postgres::Pool>,
    config: &Config,
    user: &User,
) -> Result<Authentication, Error> {
    let access_token = TokenType::Access.sign(config, user).map_err(|_| {
        Error::InternalServerError("An error occurred while signing the access token".into())
    })?;
    let refresh_token = TokenType::Refresh.sign(config, user).map_err(|_| {
        Error::InternalServerError("An error occurred while signing the refresh token".into())
    })?;

    let refresh_token_tree = RefreshTokenTree {
        id: Id::new().to_string(),
        user_id: user.id.clone(),
        inactive_at: Utc::now() + chrono::Duration::days(15),
        expires_at: Utc::now() + chrono::Duration::days(90),
        tokens: vec![refresh_token.clone().claims.jti],
        created_at: Utc::now(),
        updated_at: None,
    };

    refresh_token_tree.create(db_pool).await?;

    let cookie_expiration = OffsetDateTime::from_unix_timestamp(
        refresh_token.expires_at.timestamp(),
    )
    .map_err(|_| {
        Error::InternalServerError("An error occurred while parsing the cookie expiration".into())
    })?;

    let cookie = Cookie::build("refreshToken", refresh_token.token.clone())
        .secure(true)
        .http_only(true)
        .expires(Expiration::from(cookie_expiration))
        .finish();

    Ok(Authentication {
        cookie,
        token: access_token,
    })
}

impl TokenType {
    pub fn sign(&self, config: &config::Config, user: &User) -> Result<TokenInfo, Error> {
        let secret_key = config.get(ConfigKey::SecretKey)?;

        let expires_at = match self {
            TokenType::Access => Utc::now() + Duration::minutes(15),
            TokenType::Refresh => Utc::now() + Duration::days(15),
        };
        let claims = Claims {
            jti: Id::new().to_string(),
            iat: Utc::now().timestamp() as usize,
            exp: expires_at.timestamp() as usize,
            sub: user.id.clone(),
            token_type: self.to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret_key.as_bytes()),
        )
        .map_err(|err| {
            Error::InternalServerError(format!("Unable to encode {self} token: {err}"))
        })?;

        Ok(TokenInfo {
            token,
            claims,
            expires_at,
            token_type: self.clone(),
        })
    }

    pub fn verify(config: &config::Config, token: String) -> Result<TokenInfo, Error> {
        let secret_key = config.get(ConfigKey::SecretKey)?;

        let claims = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret_key.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|err| match err.into_kind() {
            ErrorKind::ExpiredSignature => Error::Unauthorized,
            _ => Error::InternalServerError("Unable to decode token".into()),
        })?;

        let token_type = match claims.token_type.as_str() {
            "access" => TokenType::Access,
            "refresh" => TokenType::Refresh,
            _ => {
                return Err(Error::InternalServerError(
                    "Token has an invalid token type".into(),
                ))
            }
        };

        Ok(TokenInfo {
            token,
            claims: claims.clone(),
            expires_at: Utc.timestamp_opt(claims.exp as i64, 0).unwrap(),
            token_type,
        })
    }
}
