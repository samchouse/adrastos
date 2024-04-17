use std::collections::HashMap;

use adrastos_core::{
    auth::oauth2::providers::OAuth2Provider,
    entities::{OAuth2Config, SmtpConfig},
    error::Error,
};
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    middleware::extractors::{Config, ProjectDatabase, SystemUser},
    state::AppState,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Oauth2Body {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SmtpBody {
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    sender_name: String,
    sender_email: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/details", get(details))
        .route("/smtp", post(smtp))
        .route("/oauth2", post(oauth2))
}

pub async fn details(_: SystemUser, Config(config): Config) -> Result<impl IntoResponse, Error> {
    let system = config.system();
    let Some(system) = system else {
        return Err(Error::InternalServerError(
            "Something went wrong getting the system.".into(),
        ));
    };

    Ok(Json(json!({
        "smtpConfig": system.smtp_config.as_ref().map(|c| json!({
            "host": c.host,
            "port": c.port,
            "username": c.username,
            "senderName": c.sender_name,
            "senderEmail": c.sender_email,
        })),
        "oauth2Config": {
            "google": system.google_config,
            "facebook": system.facebook_config,
            "github": system.github_config,
            "twitter": system.twitter_config,
            "discord": system.discord_config,
        }
    })))
}

pub async fn smtp(
    _: SystemUser,
    Config(config): Config,
    ProjectDatabase(db): ProjectDatabase,
    body: Option<Json<SmtpBody>>,
) -> Result<impl IntoResponse, Error> {
    let system = config.system();
    let Some(mut system) = system.clone() else {
        return Err(Error::InternalServerError(
            "Something went wrong getting the system.".into(),
        ));
    };

    system.smtp_config = match body {
        Some(Json(body)) => {
            let password = body
                .password
                .clone()
                .or(system.smtp_config.as_ref().map(|c| c.password.clone()));
            let Some(password) = password else {
                return Err(Error::BadRequest(
                    "A password is required to enable SMTP".to_string(),
                ));
            };
            if password.is_empty() {
                return Err(Error::BadRequest(
                    "A password is required to enable SMTP".to_string(),
                ));
            };

            Ok(Some(SmtpConfig {
                host: body.host.clone(),
                port: body.port,
                username: body.username.clone(),
                password,
                sender_name: body.sender_name.clone(),
                sender_email: body.sender_email,
            }))
        }
        None => Ok(None),
    }?;

    db.get()
        .await
        .unwrap()
        .execute(&system.set(), &[])
        .await
        .unwrap();

    Ok(Json(system.smtp_config.as_ref().map(|c| {
        json!({
            "host": c.host,
            "port": c.port,
            "username": c.username,
            "senderName": c.sender_name,
            "senderEmail": c.sender_email,
        })
    })))
}

pub async fn oauth2(
    _: SystemUser,
    Config(config): Config,
    ProjectDatabase(db): ProjectDatabase,
    body: Json<HashMap<OAuth2Provider, Option<Oauth2Body>>>,
) -> Result<impl IntoResponse, Error> {
    let system = config.system();
    let Some(mut system) = system.clone() else {
        return Err(Error::InternalServerError(
            "Something went wrong getting the system.".into(),
        ));
    };

    body.iter().for_each(|(k, v)| {
        let v = v.as_ref().map(|v| OAuth2Config {
            client_id: v.client_id.clone(),
            client_secret: v.client_secret.clone(),
        });

        match k {
            OAuth2Provider::Google => system.google_config = v,
            OAuth2Provider::Facebook => system.facebook_config = v,
            OAuth2Provider::GitHub => system.github_config = v,
            OAuth2Provider::Twitter => system.twitter_config = v,
            OAuth2Provider::Discord => system.discord_config = v,
        }
    });

    db.get()
        .await
        .unwrap()
        .execute(&system.set(), &[])
        .await
        .unwrap();

    Ok(Json(json!({
        "google": system.google_config,
        "facebook": system.facebook_config,
        "github": system.github_config,
        "twitter": system.twitter_config,
        "discord": system.discord_config,
    })))
}
