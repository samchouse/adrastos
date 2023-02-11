#![feature(let_chains)]

use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, error::InternalError, web, App, HttpResponse, HttpServer};
use dotenvy::dotenv;
use openapi::ApiDoc;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde_json::json;
use std::process;
use tracing::{error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    auth::oauth2::OAuth2,
    config::{Config, ConfigKey},
    db::{postgres, redis},
};

mod auth;
mod config;
mod db;
mod entities;
mod handlers;
mod id;
mod openapi;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = Config::new().unwrap_or_else(|err| {
        error!("missing required environment variables: {:#?}", err);
        process::exit(1)
    });
    let cfg = config.clone();
    let db_pool = postgres::create_pool(config.clone());

    entities::migrations::migrate(&db_pool).await;

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("../../certs/key.pem", SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_chain_file("../../certs/cert.pem")
        .unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cfg.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis::create_pool(cfg.clone())))
            .app_data(web::Data::new(OAuth2::new(cfg.clone())))
            .app_data(web::JsonConfig::default().error_handler(|err, _| {
                let err_string = err.to_string();
                InternalError::from_response(
                    err,
                    HttpResponse::BadRequest().json(json!({
                        "message": "Validation failed",
                        "error": err_string
                    })),
                )
                .into()
            }))
            .wrap(SessionMiddleware::new(
                RedisActorSessionStore::new(
                    cfg.get(ConfigKey::DragonflyUrl)
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .replace("redis://", ""),
                ),
                Key::from(
                    cfg.get(ConfigKey::SecretKey)
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .as_bytes(),
                ),
            ))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
            .service(
                web::scope("/auth")
                    .service((
                        handlers::auth::signup,
                        handlers::auth::login,
                        handlers::auth::logout,
                        handlers::auth::token::refresh,
                    ))
                    .service(web::scope("/oauth2").service((
                        handlers::auth::oauth2::login,
                        handlers::auth::oauth2::callback,
                    ))),
            )
            .service(
                web::scope("/tables")
                    .service(web::scope("/{table_name}").service(handlers::tables::custom::rows)),
            )
            .default_service(web::route().to(handlers::not_found))
    })
    .bind_openssl(
        config
            .get(ConfigKey::ServerUrl)
            .unwrap()
            .as_ref()
            .unwrap()
            .to_string(),
        builder,
    )?
    .run();

    let (server, _) = tokio::join!(
        server,
        server_started(config.get(ConfigKey::ServerUrl).unwrap().unwrap())
    );

    server
}

async fn server_started(url: String) {
    info!("Server started at https://{url}");
}
