#![feature(let_chains)]

use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};
use auth::oauth2::OAuth2;
use config::ConfigKey;
use db::{postgres, redis};
use dotenvy::dotenv;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::process::exit;

use crate::config::Config;

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

            let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
            builder
                .set_private_key_file("../../certs/key.pem", SslFiletype::PEM)
                .unwrap();
            builder.set_certificate_chain_file("../../certs/cert.pem").unwrap();

            let server = HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(cfg.clone()))
                    .app_data(web::Data::new(postgres::create_pool(cfg.clone())))
                    .app_data(web::Data::new(redis::create_pool(cfg.clone())))
                    .app_data(web::Data::new(OAuth2::new(cfg.clone())))
                    .wrap(SessionMiddleware::new(
                        RedisActorSessionStore::new(
                            cfg.get(ConfigKey::DragonflyUrl)
                                .as_ref()
                                .unwrap()
                                .replace("redis://", ""),
                        ),
                        Key::from(cfg.get(ConfigKey::SecretKey).as_ref().unwrap().as_bytes()),
                    ))
                    .service(handlers::auth::oauth2::login)
                    .service(handlers::auth::oauth2::callback)
            })
            .bind_openssl(config.get(ConfigKey::Url).as_ref().unwrap().to_string(), builder)?
            .run();

            let (server, _) =
                tokio::join!(server, server_started(config.get(ConfigKey::Url).unwrap()));

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
