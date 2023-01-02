#![feature(let_chains)]

use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::HttpResponse;
use actix_web::{cookie::Key, error, web, App, HttpServer};
use dotenvy::dotenv;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde_json::json;
use std::process::exit;

use crate::auth::oauth2::OAuth2;
use crate::config::Config;
use crate::config::ConfigKey;
use crate::db::{postgres, redis};

mod auth;
mod config;
mod db;
mod entities;
mod handlers;
mod id;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = Config::new();
    match config {
        Ok(config) => {
            let cfg = config.clone();
            let db_pool = postgres::create_pool(config.clone());

            entities::migrations::migrate(db_pool.clone()).await;

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
                        error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest()
                                .json(json!({
                                    "message": "Validation failed",
                                    "error": err_string
                                }))
                                .into(),
                        )
                        .into()
                    }))
                    .wrap(SessionMiddleware::new(
                        RedisActorSessionStore::new(
                            cfg.get(ConfigKey::DragonflyUrl)
                                .as_ref()
                                .unwrap()
                                .replace("redis://", ""),
                        ),
                        Key::from(cfg.get(ConfigKey::SecretKey).as_ref().unwrap().as_bytes()),
                    ))
                    .service(handlers::auth::signup)
                    .service(handlers::auth::login)
                    .service(handlers::auth::logout)
                    .service(handlers::auth::token::refresh)
                    .service(handlers::auth::oauth2::login)
                    .service(handlers::auth::oauth2::callback)
            })
            .bind_openssl(
                config
                    .get(ConfigKey::ServerUrl)
                    .as_ref()
                    .unwrap()
                    .to_string(),
                builder,
            )?
            .run();

            let (server, _) = tokio::join!(
                server,
                server_started(config.get(ConfigKey::ServerUrl).unwrap())
            );

            server
        }
        Err(errors) => {
            println!("Missing required environment variables: {:#?}", errors);
            exit(1)
        }
    }
}

async fn server_started(url: String) {
    println!("Server started at https://{}", url);
}
