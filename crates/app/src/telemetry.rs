use adrastos_core::config::Config;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{filter::LevelFilter, prelude::*, EnvFilter};

pub fn register_subscriber() {
    let axiom_layer = tracing_axiom::default("adrastos");

    let subscriber = tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
                .add_directive("tower_sessions=warn".parse().unwrap()),
        )
        .with(JsonStorageLayer)
        .with(
            BunyanFormattingLayer::new("adrastos".into(), std::io::stdout)
                .skip_fields(vec!["line", "file"].into_iter())
                .unwrap(),
        )
        .with(sentry_tracing::layer());

    if let Ok(layer) = axiom_layer {
        subscriber.with(layer).init();
        return;
    }

    subscriber.init();
}

pub fn init_sentry(config: &Config) -> Option<sentry::ClientInitGuard> {
    config.sentry_dsn.as_ref().map(|dsn| {
        sentry::init((
            dsn.to_owned(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        ))
    })
}
