#![feature(let_chains)]

use actix_cors::Cors;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key, error::InternalError, middleware::Logger, web, App, HttpResponse, HttpServer,
};
use adrastos_core::{
    auth::oauth2::OAuth2,
    config::{Config, ConfigKey},
    db::{postgres, redis},
    entities::{self, Identity, System},
    migrations::Migrations,
};
use clap::Parser;
use cli::{Cli, Command};
use dotenvy::dotenv;
use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use openapi::ApiDoc;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use serde_json::json;
use std::{fs::File, io::BufReader, process};
use tracing::{error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod cli;
mod handlers;
mod middleware;
mod openapi;
mod session;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let mut config = Config::new().unwrap_or_else(|err| {
        error!("missing required environment variables: {:#?}", err);
        process::exit(1)
    });

    let db_pool = postgres::create_pool(&config);
    entities::init(&db_pool).await;

    {
        let conn = db_pool.get().await.unwrap();
        config.attach_system(
            conn.query(
                &Query::select()
                    .from(System::table())
                    .columns([
                        <System as Identity>::Iden::Id,
                        <System as Identity>::Iden::CurrentVersion,
                        <System as Identity>::Iden::PreviousVersion,
                        <System as Identity>::Iden::SmtpConfig,
                        <System as Identity>::Iden::GoogleConfig,
                        <System as Identity>::Iden::FacebookConfig,
                        <System as Identity>::Iden::GithubConfig,
                        <System as Identity>::Iden::TwitterConfig,
                        <System as Identity>::Iden::DiscordConfig,
                    ])
                    .and_where(Expr::col(<System as Identity>::Iden::Id).eq("system"))
                    .to_string(PostgresQueryBuilder),
                &[],
            )
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .into(),
        );
    }

    let cli = Cli::parse();
    if cli.command == Some(Command::Migrate) {
        let conn = db_pool.get().await.unwrap();

        let migrations = Migrations::all_from("0.1.1");
        for migration in &migrations {
            info!("Migration: {}", migration.version);
            for query in &migration.queries {
                info!("Query: {}", query.to_string(PostgresQueryBuilder));

                conn.execute(query.to_string(PostgresQueryBuilder).as_str(), &[])
                    .await
                    .unwrap();
            }
        }

        return Ok(());
    }

    let use_tls = config.get(ConfigKey::UseTls).ok();
    let certs_path = config.get(ConfigKey::CertsPath).unwrap();
    let server_url = config.get(ConfigKey::ServerUrl).unwrap();

    let store = RedisSessionStore::new(config.get(ConfigKey::DragonflyUrl).unwrap())
        .await
        .unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(middleware::user::GetUser {
                config: config.clone(),
                db_pool: db_pool.clone(),
            })
            .wrap(SessionMiddleware::new(
                store.clone(),
                Key::from(config.get(ConfigKey::SecretKey).unwrap().as_bytes()),
            ))
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(OAuth2::new(&config)))
            .app_data(web::Data::new(redis::create_pool(&config)))
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
            .app_data(web::Data::new(
                if let Ok(smtp_host) = config.get(ConfigKey::SmtpHost) {
                    Some(
                        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
                            .unwrap()
                            .port(config.get(ConfigKey::SmtpPort).unwrap().parse().unwrap())
                            .credentials(Credentials::new(
                                config.get(ConfigKey::SmtpUsername).unwrap(),
                                config.get(ConfigKey::SmtpPassword).unwrap(),
                            ))
                            .build::<Tokio1Executor>(),
                    )
                } else {
                    None
                },
            ))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
            .service(handlers::index)
            .service(
                web::scope("/auth")
                    .service((
                        handlers::auth::signup,
                        handlers::auth::login,
                        handlers::auth::logout,
                        handlers::auth::verify,
                        handlers::auth::token::refresh,
                    ))
                    .service(web::scope("/oauth2").service((
                        handlers::auth::oauth2::login,
                        handlers::auth::oauth2::callback,
                    )))
                    .service(web::scope("/mfa").service((
                        handlers::auth::mfa::enable,
                        handlers::auth::mfa::disable,
                        handlers::auth::mfa::verify,
                        handlers::auth::mfa::confirm,
                        handlers::auth::mfa::regenerate,
                    ))),
            )
            .service(
                web::scope("/tables")
                    .service((
                        handlers::tables::create,
                        handlers::tables::update,
                        handlers::tables::delete,
                    ))
                    .service(web::scope("/{name}").service((
                        handlers::tables::custom::row,
                        handlers::tables::custom::rows,
                        handlers::tables::custom::create,
                        handlers::tables::custom::delete,
                    ))),
            )
            .default_service(web::route().to(handlers::not_found))
    });

    let server = match use_tls.clone() {
        Some(use_tls) if use_tls == "true" => {
            let rustls_config = ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth();

            let cert_file =
                &mut BufReader::new(File::open(format!("{}/cert.pem", certs_path)).unwrap());
            let key_file =
                &mut BufReader::new(File::open(format!("{}/key.pem", certs_path)).unwrap());

            let cert_chain = certs(cert_file)
                .unwrap()
                .into_iter()
                .map(Certificate)
                .collect();
            let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
                .unwrap()
                .into_iter()
                .map(PrivateKey)
                .collect();

            if keys.is_empty() {
                error!("Couldn't locate private keys");
                process::exit(1);
            }

            let rustls_config = rustls_config
                .with_single_cert(cert_chain, keys.remove(0))
                .unwrap();

            server.bind_rustls(&server_url, rustls_config)
        }
        _ => server.bind(&server_url),
    }?
    .run();

    let use_tls = use_tls.unwrap();
    let (server, _) = tokio::join!(server, server_started(&use_tls, &server_url));

    server
}

async fn server_started(use_tls: &str, url: &str) {
    let mut url = format!("https://{url}");
    if use_tls == "false" {
        url = url.replace("https", "http")
    };

    info!("Server started at {url}");
}
