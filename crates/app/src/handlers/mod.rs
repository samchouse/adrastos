use tokio::sync::Mutex;

use actix_web::{get, web, HttpResponse, Responder};
use adrastos_core::{
    config::{Config, ConfigKey},
    error::Error,
};

pub mod auth;
pub mod config;
pub mod tables;

#[get("/")]
pub async fn index(config: web::Data<Mutex<Config>>) -> actix_web::Result<impl Responder, Error> {
    Ok(HttpResponse::PermanentRedirect()
        .append_header((
            "Location",
            config.lock().await.get(ConfigKey::ClientUrl).unwrap(),
        ))
        .finish())
}

pub async fn not_found() -> actix_web::Result<String, Error> {
    Err(Error::NotFound)
}
