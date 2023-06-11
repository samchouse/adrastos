use actix_web::{get, web, HttpResponse, Responder};
use adrastos_core::{
    config::{Config, ConfigKey},
    error::Error,
};

pub mod auth;
pub mod tables;

#[get("/")]
pub async fn index(config: web::Data<Config>) -> actix_web::Result<impl Responder, Error> {
    Ok(HttpResponse::PermanentRedirect()
        .append_header(("Location", config.get(ConfigKey::ClientUrl).unwrap()))
        .finish())
}

pub async fn not_found() -> actix_web::Result<String, Error> {
    Err(Error::NotFound)
}
