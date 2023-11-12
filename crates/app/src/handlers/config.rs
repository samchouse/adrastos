use std::collections::HashMap;

use actix_web::{get, post, web, HttpResponse, Responder};
use adrastos_core::{
    auth::oauth2::providers::OAuth2Provider,
    config,
    entities::{OAuth2Config, SmtpConfig, System},
    error::Error,
};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;

use crate::middleware::user::RequiredUser;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Oauth2Body {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SmtpBody {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub sender_name: String,
    pub sender_email: String,
}

#[get("/details")]
pub async fn details(
    _: RequiredUser,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let conn = db_pool.get().await.unwrap();
    let system: System = conn
        .query(&System::get(), &[])
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .into();

    Ok(HttpResponse::Ok().json(json!({
        "smtpConfig": system.smtp_config.map(|c| json!({
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

#[post("/smtp")]
pub async fn smtp(
    _: RequiredUser,
    body: web::Json<Option<SmtpBody>>,
    config: web::Data<Mutex<config::Config>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let conn = db_pool.get().await.unwrap();
    let mut system: System = conn
        .query(&System::get(), &[])
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .into();

    system.smtp_config = match body {
        web::Json(Some(body)) => {
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
        web::Json(None) => Ok(None),
    }?;

    conn.execute(&system.set(), &[]).await.unwrap();
    config.lock().await.attach_system(&system);

    Ok(HttpResponse::Ok().json(system.smtp_config.map(|c| json!({
        "host": c.host,
        "port": c.port,
        "username": c.username,
        "senderName": c.sender_name,
        "senderEmail": c.sender_email,
    }))))
}

#[post("/oauth2")]
pub async fn oauth2(
    _: RequiredUser,
    config: web::Data<Mutex<config::Config>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
    body: web::Json<HashMap<OAuth2Provider, Option<Oauth2Body>>>,
) -> actix_web::Result<impl Responder, Error> {
    let conn = db_pool.get().await.unwrap();
    let mut system: System = conn
        .query(&System::get(), &[])
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .into();

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

    conn.execute(&system.set(), &[]).await.unwrap();
    config.lock().await.attach_system(&system);

    Ok(HttpResponse::Ok().json(json!({
        "google": system.google_config,
        "facebook": system.facebook_config,
        "github": system.github_config,
        "twitter": system.twitter_config,
        "discord": system.discord_config,
    })))
}
