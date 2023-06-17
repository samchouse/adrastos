use std::collections::HashMap;

use actix_web::{post, web, HttpResponse, Responder};
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

#[post("/smtp")]
pub async fn smtp(
    _: RequiredUser,
    body: web::Json<Option<SmtpConfig>>,
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

    system.smtp_config = body.into_inner();

    conn.execute(&system.set(), &[]).await.unwrap();
    config.lock().await.attach_system(system);

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "SMTP settings successfully configured",
    })))
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
    config.lock().await.attach_system(system);

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "OAuth2 providers successfully configured",
    })))
}
