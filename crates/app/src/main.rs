#![feature(let_chains)]

use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, error::InternalError, web, App, HttpResponse, HttpServer};
use adrastos_core::{
    auth::oauth2::OAuth2,
    config::Config,
    db::{
        postgres::{DatabaseType, Databases},
        redis,
    },
    entities::System,
    migrations::Migrations,
};
use clap::Parser;
use cli::{Cli, Command};
use dotenvy::dotenv;
use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use openapi::ApiDoc;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use sea_query::PostgresQueryBuilder;
use secrecy::ExposeSecret;
use serde_json::json;
use std::{fs::File, io::BufReader, process};
use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;
use tracing_unwrap::{OptionExt, ResultExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod assets;
mod cli;
mod handlers;
mod middleware;
mod openapi;
mod session;
mod telemetry;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let _guard = telemetry::register_subscriber();

    let mut config = Config::new();

    let _sentry_guard = telemetry::init_sentry(&config);

    let databases = std::sync::Arc::new(Databases::new());
    let inner_databases = databases.clone();
    let db = databases.get(&DatabaseType::System, &config).await;

    {
        config.attach_system(
            &db.get()
                .await
                .unwrap_or_log()
                .query(&System::get(), &[])
                .await
                .unwrap_or_log()
                .into_iter()
                .next()
                .unwrap_or_log()
                .into(),
        );
    }

    let cli = Cli::parse();
    if cli.command == Some(Command::Migrate) {
        let conn = db.get().await.unwrap_or_log();

        let migrations = Migrations::all_from(&config.previous_version);
        for migration in &migrations {
            info!("Migration: {}", migration.version);
            for query in &migration.queries {
                info!("Query: {}", query.to_string(PostgresQueryBuilder));

                conn.execute(query.to_string(PostgresQueryBuilder).as_str(), &[])
                    .await
                    .unwrap_or_log();
            }
        }

        return Ok(());
    }

    let use_tls = config.use_tls;
    let server_url = config.server_url.clone();
    let certs_path = config.certs_path.clone();

    let store = RedisSessionStore::new(config.redis_url.clone())
        .await
        .unwrap_or_log();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(sentry_actix::Sentry::new())
            .wrap(actix_web::middleware::NormalizePath::trim())
            .wrap(middleware::Config {
                config: config.clone(),
                databases: inner_databases.clone(),
            })
            .wrap(middleware::Cors {
                config: config.clone(),
            })
            .wrap(SessionMiddleware::new(
                store.clone(),
                Key::from(config.secret_key.expose_secret().as_bytes()),
            ))
            .app_data(web::Data::new(OAuth2::new(&config)))
            .app_data(web::Data::new(RwLock::new(config.clone())))
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
            .app_data(web::Data::new(if let Some(smtp_host) = &config.smtp_host {
                Some(
                    AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_host)
                        .unwrap_or_log()
                        .port(config.smtp_port.unwrap_or_log())
                        .credentials(Credentials::new(
                            config.smtp_username.clone().unwrap_or_log(),
                            config
                                .smtp_password
                                .clone()
                                .unwrap_or_log()
                                .expose_secret()
                                .to_string(),
                        ))
                        .build::<Tokio1Executor>(),
                )
            } else {
                None
            }))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
            .service(handlers::api_index)
            .service(
                web::scope("/api")
                    .service(handlers::me)
                    .service(
                        web::scope("/auth")
                            .service((
                                handlers::auth::register,
                                handlers::auth::login,
                                handlers::auth::logout,
                                handlers::auth::verify,
                                handlers::auth::resend_verification,
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
                            )))
                            .service(web::scope("/passkeys").service((
                                handlers::auth::passkeys::list,
                                handlers::auth::passkeys::update,
                                handlers::auth::passkeys::delete,
                                handlers::auth::passkeys::register_start,
                                handlers::auth::passkeys::register_finish,
                                handlers::auth::passkeys::login_start,
                                handlers::auth::passkeys::login_finish,
                            ))),
                    )
                    .service(web::scope("/config").service((
                        handlers::config::details,
                        handlers::config::oauth2,
                        handlers::config::smtp,
                    )))
                    .service(
                        web::scope("/tables")
                            .service((
                                handlers::tables::list,
                                handlers::tables::create,
                                handlers::tables::update,
                                handlers::tables::delete,
                            ))
                            .service(web::scope("/{name}").service((
                                handlers::tables::custom::row,
                                handlers::tables::custom::rows,
                                handlers::tables::custom::create,
                                handlers::tables::custom::update,
                                handlers::tables::custom::delete,
                            ))),
                    )
                    .service(
                        web::scope("/teams")
                            .service((
                                handlers::teams::list,
                                handlers::teams::create,
                                handlers::teams::delete,
                                handlers::teams::projects::get
                            ))
                            .service(web::scope("/{team_id}/projects").service((
                                handlers::teams::projects::list,
                                handlers::teams::projects::create,
                                handlers::teams::projects::delete,
                            ))),
                    ),
            )
            .service(handlers::index)
            .default_service(web::route().to(handlers::default))
    });

    let server = if use_tls {
        let Some(certs_path) = certs_path else {
            error!("TLS is enabled but no certs path is provided");
            process::exit(1);
        };

        let rustls_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth();

        let cert_file =
            &mut BufReader::new(File::open(format!("{}/cert.pem", certs_path)).unwrap_or_log());
        let key_file =
            &mut BufReader::new(File::open(format!("{}/key.pem", certs_path)).unwrap_or_log());

        let cert_chain = certs(cert_file)
            .unwrap_or_log()
            .into_iter()
            .map(Certificate)
            .collect();
        let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
            .unwrap_or_log()
            .into_iter()
            .map(PrivateKey)
            .collect();

        if keys.is_empty() {
            error!("Couldn't locate private keys");
            process::exit(1);
        }

        let rustls_config = rustls_config
            .with_single_cert(cert_chain, keys.remove(0))
            .unwrap_or_log();

        server.bind_rustls(&server_url, rustls_config)
    } else {
        server.bind(&server_url)
    }?
    .run();

    let (server, _, _) = tokio::join!(
        server,
        server_started(use_tls, &server_url),
        Databases::start_expiry_worker(databases)
    );

    server
}

async fn server_started(use_tls: bool, url: &str) {
    let mut url = format!("https://{url}");
    if !use_tls {
        url = url.replace("https", "http")
    };

    info!("server started at {url}");

    if url.contains("0.0.0.0") {
        info!(
            "you can access the server at {}",
            url.replace("0.0.0.0", "127.0.0.1")
        );
    }
}
