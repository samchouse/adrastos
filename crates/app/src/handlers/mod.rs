use std::ops::Deref;

use serde_json::json;
use tokio::sync::Mutex;

use actix_web::{get, web, HttpResponse, Responder};
use adrastos_core::{config::Config, error::Error};

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
