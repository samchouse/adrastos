use actix_web::{
    get,
    http::header::{CacheControl, CacheDirective},
    HttpRequest, HttpResponse, Responder,
};
use adrastos_core::{
    db::postgres::{Database, DatabaseType},
    entities::{SystemUser, User},
    error::Error,
};

use crate::{
    assets::handle_embedded_file,
    middleware::{config::Config, user::RequiredAnyUser},
};

pub mod auth;
pub mod config;
pub mod storage;
pub mod tables;
pub mod teams;

pub async fn default(req: HttpRequest) -> actix_web::Result<impl Responder, Error> {
    handle_embedded_file(req.path())
}

#[get("/")]
pub async fn index() -> actix_web::Result<impl Responder, Error> {
    handle_embedded_file("index.html")
}

#[get("/api")]
pub async fn api_index(config: Config) -> actix_web::Result<impl Responder, Error> {
    Ok(HttpResponse::PermanentRedirect()
        .append_header(("Location", config.client_url.clone()))
        .append_header(CacheControl(vec![CacheDirective::NoCache]))
        .finish())
}

#[get("/me")]
pub async fn me(db: Database, user: RequiredAnyUser) -> actix_web::Result<impl Responder, Error> {
    match db.1 {
        DatabaseType::System => {
            let system_user = SystemUser::find_by_id(&user.id).one(&db).await?;
            Ok(HttpResponse::Ok().json(system_user))
        }
        DatabaseType::Project(_) => {
            let user = User::find_by_id(&user.id).one(&db).await?;
            Ok(HttpResponse::Ok().json(user))
        }
    }
}
