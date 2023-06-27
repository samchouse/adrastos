use std::ops::Deref;

use chrono::Utc;
use serde_json::json;
use tokio::sync::Mutex;

use actix_web::{get, web, HttpResponse, Responder};
use adrastos_core::{
    config::Config,
    entities::{UpdateUser, User},
    error::Error,
};

use crate::middleware::user::RequiredUser;

pub mod auth;
pub mod config;
pub mod tables;

pub async fn not_found() -> actix_web::Result<String, Error> {
    Err(Error::NotFound)
}

#[get("/")]
pub async fn index(config: web::Data<Mutex<Config>>) -> actix_web::Result<impl Responder, Error> {
    Ok(HttpResponse::PermanentRedirect()
        .append_header(("Location", config.lock().await.client_url.clone()))
        .finish())
}

#[get("/me")]
pub async fn me(user: RequiredUser) -> actix_web::Result<impl Responder, Error> {
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "user": user.deref(),
    })))
}

#[get("/health")]
pub async fn health(
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let user = User {
        id: "zxvczcxvzxcv".to_string(),
        first_name: "asdfasdfasdfasdf".to_string(),
        last_name: "asdfasdfasdfasd".to_string(),
        username: "asdfasdfasdfasdf".to_string(),
        email: "asdfasf@asdfa.asdfasdf".to_string(),
        password: "asdfsdfasdfasdfasdf".to_string(),
        verified: false,
        banned: false,
        mfa_secret: None,
        mfa_backup_codes: None,
        created_at: Utc::now(),
        updated_at: None,

        connections: None,
        refresh_token_trees: None,
    };

    user.update_new(
        &db_pool,
        UpdateUser {
            first_name: Some("test".to_string()),
            mfa_backup_codes: Some(None),
            ..Default::default()
        },
    )
    .await;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
    })))
}
