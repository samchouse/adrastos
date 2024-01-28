use std::{ops::Deref, path::PathBuf};

use actix_files::NamedFile;
use actix_web::{
    get,
    http::header::{CacheControl, CacheDirective},
    web, HttpRequest, HttpResponse, Responder,
};
use adrastos_core::{config::Config, error::Error};
use tokio::sync::Mutex;

use crate::middleware::user::RequiredUser;

pub mod auth;
pub mod config;
pub mod tables;

pub async fn default(req: HttpRequest) -> actix_web::Result<impl Responder, Error> {
    if let Some(path) = req.path().split('/').last() {
        if PathBuf::from(path).extension().is_some() {
            return Err(Error::NotFound);
        }
    }

    Ok(NamedFile::open("./packages/dashboard/dist/index.html"))
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
