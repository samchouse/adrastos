use std::ops::Deref;

use actix_web::{
    get,
    http::header::{CacheControl, CacheDirective},
    web, HttpRequest, HttpResponse, Responder,
};
use adrastos_core::{config::Config, error::Error};
use tokio::sync::Mutex;

use crate::{assets::handle_embedded_file, middleware::user::RequiredUser};

pub mod auth;
pub mod config;
pub mod tables;

pub async fn default(req: HttpRequest) -> actix_web::Result<impl Responder, Error> {
    handle_embedded_file(req.path())
}

#[get("/")]
pub async fn index() -> actix_web::Result<impl Responder, Error> {
    handle_embedded_file("index.html")
}

#[get("/api")]
pub async fn api_index(
    config: web::Data<Mutex<Config>>,
) -> actix_web::Result<impl Responder, Error> {
    Ok(HttpResponse::PermanentRedirect()
        .append_header(("Location", config.lock().await.client_url.clone()))
        .append_header(CacheControl(vec![CacheDirective::NoCache]))
        .finish())
}

#[get("/me")]
pub async fn me(user: RequiredUser) -> actix_web::Result<impl Responder, Error> {
    Ok(HttpResponse::Ok().json(user.deref()))
}
