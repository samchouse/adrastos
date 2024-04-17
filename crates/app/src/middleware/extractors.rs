use std::sync::Arc;

use adrastos_core::auth::oauth2;
use adrastos_core::{config, entities};
use adrastos_core::{
    db::postgres::{self, DatabaseType},
    error::Error,
};
use axum::RequestPartsExt;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use secrecy::ExposeSecret;
use tracing_unwrap::{OptionExt, ResultExt};

pub struct Config(pub config::Config);

#[async_trait]
impl<S> FromRequestParts<S> for Config
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Config(
            parts.extensions.get::<config::Config>().unwrap().clone(),
        ))
    }
}

pub struct Project(pub entities::Project);

#[async_trait]
impl<S> FromRequestParts<S> for Project
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        match parts.extensions.get::<entities::Project>().cloned() {
            Some(v) => Ok(Project(v)),
            None => Err(Error::BadRequest("Missing project ID".into())),
        }
    }
}

pub struct Database(pub postgres::Database);

#[async_trait]
impl<S> FromRequestParts<S> for Database
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        match parts
            .extensions
            .get::<(Arc<deadpool_postgres::Pool>, DatabaseType)>()
            .cloned()
        {
            Some(v) => Ok(Database(postgres::Database(v.0, v.1))),
            None => Err(Error::Unauthorized),
        }
    }
}

pub struct SystemDatabase(pub postgres::Database);

#[async_trait]
impl<S> FromRequestParts<S> for SystemDatabase
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        match parts.extensions.get::<postgres::Database>().cloned() {
            Some(v) => Ok(SystemDatabase(v)),
            None => Err(Error::Unauthorized),
        }
    }
}

pub struct ProjectDatabase(pub postgres::Database);

#[async_trait]
impl<S> FromRequestParts<S> for ProjectDatabase
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        match parts
            .extensions
            .get::<(Arc<deadpool_postgres::Pool>, DatabaseType)>()
            .cloned()
        {
            Some(v) => match v.1 {
                DatabaseType::Project(_) => Ok(ProjectDatabase(postgres::Database(v.0, v.1))),
                _ => Err(Error::BadRequest("Missing project ID".into())),
            },
            None => Err(Error::Unauthorized),
        }
    }
}

pub struct User(pub entities::User);

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        match parts
            .extensions
            .get::<(Arc<deadpool_postgres::Pool>, DatabaseType)>()
            .cloned()
        {
            Some((_, db_type)) => match db_type {
                DatabaseType::Project(_) => match parts.extensions.get::<entities::User>().cloned()
                {
                    Some(v) => Ok(User(v)),
                    None => Err(Error::Unauthorized),
                },
                _ => Err(Error::BadRequest("Missing project ID".into())),
            },
            None => Err(Error::BadRequest("Invalid project ID".into())),
        }
    }
}

pub struct SystemUser(pub entities::SystemUser);

#[async_trait]
impl<S> FromRequestParts<S> for SystemUser
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        match parts.extensions.get::<entities::SystemUser>().cloned() {
            Some(v) => Ok(SystemUser(v)),
            None => Err(Error::Unauthorized),
        }
    }
}

pub struct AnyUser(pub entities::AnyUser);

#[async_trait]
impl<S> FromRequestParts<S> for AnyUser
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        match parts.extensions.get::<entities::AnyUser>().cloned() {
            Some(v) => Ok(AnyUser(v)),
            None => Err(Error::Unauthorized),
        }
    }
}

pub struct Mailer(pub AsyncSmtpTransport<Tokio1Executor>);

#[async_trait]
impl<S> FromRequestParts<S> for Mailer
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let config = parts
            .extract::<Config>()
            .await
            .map(|Config(config)| config)
            .map_err(|_| Error::InternalServerError("Couldn't get config".into()))?;

        if config.smtp_host.is_none() {
            return Err(Error::BadRequest("SMTP isn't configured".into()));
        }

        Ok(Mailer(
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host.unwrap())
                .unwrap_or_log()
                .port(config.smtp_port.unwrap_or_log())
                .credentials(Credentials::new(
                    config.smtp_username.clone().unwrap_or_log(),
                    config
                        .smtp_password
                        .clone()
                        .unwrap_or_log()
                        .expose_secret()
                        .to_string(),
                ))
                .build::<Tokio1Executor>(),
        ))
    }
}

pub struct OAuth2(pub oauth2::OAuth2);

#[async_trait]
impl<S> FromRequestParts<S> for OAuth2
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let config = parts
            .extract::<Config>()
            .await
            .map(|Config(config)| config)
            .map_err(|_| Error::InternalServerError("Couldn't get config".into()))?;

        Ok(OAuth2(oauth2::OAuth2::new(&config)))
    }
}
