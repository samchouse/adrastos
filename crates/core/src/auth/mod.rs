use std::fmt;

use argon2::{
    password_hash::{
        self, rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    config::{self, Config},
    entities::{AnyUser, RefreshTokenTree},
    error::Error,
    id::Id,
};

pub mod mfa;
pub mod oauth2;
pub mod passkeys;

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
    db: &deadpool_postgres::Pool,
    config: &Config,
    user: &AnyUser,
    jar: CookieJar,
) -> Result<(TokenInfo, CookieJar), Error> {
    let access_token = TokenType::Access.sign(config, user).map_err(|_| {
        Error::InternalServerError("An error occurred while signing the access token".into())
    })?;
    let refresh_token = TokenType::Refresh.sign(config, user).map_err(|_| {
        Error::InternalServerError("An error occurred while signing the refresh token".into())
    })?;

    RefreshTokenTree {
        id: Id::new().to_string(),
        user_id: user.id.clone(),
        inactive_at: Utc::now() + chrono::Duration::try_days(15).unwrap(),
        expires_at: Utc::now() + chrono::Duration::try_days(90).unwrap(),
        tokens: vec![refresh_token.clone().claims.jti],
        created_at: Utc::now(),
        updated_at: None,
    }
    .create(db)
    .await?;

    Ok((access_token, create_auth_cookies(refresh_token, jar)?))
}

pub fn create_auth_cookies(refresh_token: TokenInfo, jar: CookieJar) -> Result<CookieJar, Error> {
    let cookie_expiration = OffsetDateTime::from_unix_timestamp(
        refresh_token.expires_at.timestamp(),
    )
    .map_err(|_| {
        Error::InternalServerError("An error occurred while parsing the cookie expiration".into())
    })?;

    Ok(jar
        .add(
            Cookie::build(("refreshToken", refresh_token.token.clone()))
                .secure(true)
                .http_only(true)
                .same_site(SameSite::None)
                .path("/api/auth")
                .expires(cookie_expiration),
        )
        .add(
            Cookie::build(("isLoggedIn", true.to_string()))
                .secure(true)
                .http_only(true)
                .same_site(SameSite::None)
                .path("/")
                .expires(cookie_expiration),
        ))
}

impl TokenType {
    pub fn sign(&self, config: &config::Config, user: &AnyUser) -> Result<TokenInfo, Error> {
        let expires_at = match self {
            TokenType::Access => Utc::now() + Duration::try_minutes(15).unwrap(),
            TokenType::Refresh => Utc::now() + Duration::try_days(15).unwrap(),
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
            &EncodingKey::from_secret(config.secret_key.expose_secret().as_bytes()),
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
        let claims = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(config.secret_key.expose_secret().as_bytes()),
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
