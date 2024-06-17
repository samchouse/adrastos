#![feature(let_chains)]

use std::{net::TcpListener, path::PathBuf, process, sync::Arc};

use adrastos_core::{
    config::Config,
    db::{
        postgres::{DatabaseType, Databases},
        redis,
    },
    entities::System,
    migrations::Migrations,
    s3::S3, task_queue::TaskQueue,
};
use axum::{routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use cli::{Cli, Command};
use dotenvy::dotenv;
use rustls::crypto;
use sea_query::PostgresQueryBuilder;
use secrecy::ExposeSecret;
use sentry_tower::NewSentryLayer;
use state::{AppState, Flag};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{
    normalize_path::NormalizePath, request_id::MakeRequestUuid, trace::TraceLayer,
    ServiceBuilderExt,
};
use tower_sessions::{cookie::Key, SessionManagerLayer};
use tower_sessions_redis_store::RedisStore;
use tracing::{error, info};
use tracing_unwrap::{OptionExt, ResultExt};

mod assets;
mod cli;
mod handlers;
mod middleware;
mod session;
mod state;
mod telemetry;
mod util;

#[tokio::main]
async fn main() {
    dotenv().ok();
    telemetry::register_subscriber();
    let _ = crypto::ring::default_provider().install_default();

    let mut config = Config::new();

    let _sentry_guard = telemetry::init_sentry(&config);

    let databases = Arc::new(Databases::new());
    #[allow(clippy::let_underscore_future)]
    let _ = Databases::start_expiry_worker(databases.clone());
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

        return;
    }

    let (redis_pool, subscriber) = redis::create_pool_and_subscriber(&config).await;
    #[allow(clippy::let_underscore_future)]
    let _rs_task = subscriber.manage_subscriptions();

    let state = AppState {
        databases,
        subscriber,
        config: config.clone(),
        redis_pool: redis_pool.clone(),
        s3: Arc::new(S3::new(&config).await),
        task_queue: Arc::new(RwLock::new(TaskQueue::new())),
        flags: vec![("/api/storage/get".into(), vec![Flag::AllowProjectIdParam])],
    };

    let app = tower::make::Shared::new(NormalizePath::trim_trailing_slash(
        Router::new()
            .route("/api", get(handlers::api))
            .route("/api/me", get(handlers::me))
            .nest("/api/auth", handlers::auth::routes())
            .nest("/api/teams", handlers::teams::routes())
            .nest("/api/config", handlers::config::routes())
            .nest("/api/tables", handlers::tables::routes())
            .nest("/api/storage", handlers::storage::routes())
            .fallback(handlers::root)
            .with_state(state.clone())
            .layer(
                ServiceBuilder::new()
                    .set_x_request_id(MakeRequestUuid)
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(middleware::trace::MakeSpan)
                            .on_response(middleware::trace::OnResponse),
                    )
                    .propagate_x_request_id()
                    .layer(NewSentryLayer::new_from_top())
                    .layer(axum::middleware::from_fn_with_state(
                        state.clone(),
                        middleware::cors::run,
                    ))
                    .layer(
                        SessionManagerLayer::new(RedisStore::new(redis_pool))
                            .with_signed(Key::from(config.secret_key.expose_secret().as_bytes())),
                    )
                    .layer(axum::middleware::from_fn_with_state(
                        state.clone(),
                        middleware::run,
                    )),
            ),
    ));

    let listener = TcpListener::bind(&config.server_url).unwrap();

    {
        let mut url = format!("https://{}", config.server_url);
        if !config.use_tls {
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

    if config.use_tls {
        let Some(certs_path) = config.certs_path else {
            error!("TLS is enabled but no certs path is provided");
            process::exit(1);
        };

        axum_server::from_tcp_rustls(
            listener,
            RustlsConfig::from_pem_chain_file(
                PathBuf::from(&certs_path).join("cert.pem"),
                PathBuf::from(&certs_path).join("key.pem"),
            )
            .await
            .unwrap(),
        )
        .serve(app)
        .await
    } else {
        axum_server::from_tcp(listener).serve(app).await
    }
    .unwrap();
}
