// TODO(@Xenfo): possibly reimplement reporting all missing keys at once, instead of one by one

use std::{env, fmt};

use secrecy::Secret;
use tracing_unwrap::ResultExt;

use crate::entities::{SizeUnit, System};

#[derive(Clone, Debug)]
enum ConfigKey {
    SentryDsn,
    SecretKey,
    UseTls,
    CertsPath,
    ClientUrl,
    ServerUrl,
    PostgresUrl,
    RedisUrl,
    S3Bucket,
    S3Region,
    S3Endpoint,
    S3AccessKey,
    S3SecretKey,

    GoogleClientId,
    GoogleClientSecret,
    FacebookClientId,
    FacebookClientSecret,
    GitHubClientId,
    GitHubClientSecret,
    TwitterClientId,
    TwitterClientSecret,
    DiscordClientId,
    DiscordClientSecret,
}

impl fmt::Display for ConfigKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::SentryDsn => "SENTRY_DSN",
            Self::SecretKey => "SECRET_KEY",
            Self::UseTls => "USE_TLS",
            Self::CertsPath => "CERTS_PATH",
            Self::ClientUrl => "CLIENT_URL",
            Self::ServerUrl => "SERVER_URL",
            Self::PostgresUrl => "POSTGRES_URL",
            Self::RedisUrl => "REDIS_URL",
            Self::S3Bucket => "S3_BUCKET",
            Self::S3Region => "S3_REGION",
            Self::S3Endpoint => "S3_ENDPOINT",
            Self::S3AccessKey => "S3_ACCESS_KEY",
            Self::S3SecretKey => "S3_SECRET_KEY",

            Self::GoogleClientId => "GOOGLE_CLIENT_ID",
            Self::GoogleClientSecret => "GOOGLE_CLIENT_SECRET",
            Self::FacebookClientId => "FACEBOOK_CLIENT_ID",
            Self::FacebookClientSecret => "FACEBOOK_CLIENT_SECRET",
            Self::GitHubClientId => "GITHUB_CLIENT_ID",
            Self::GitHubClientSecret => "GITHUB_CLIENT_SECRET",
            Self::TwitterClientId => "TWITTER_CLIENT_ID",
            Self::TwitterClientSecret => "TWITTER_CLIENT_SECRET",
            Self::DiscordClientId => "DISCORD_CLIENT_ID",
            Self::DiscordClientSecret => "DISCORD_CLIENT_SECRET",
        };

        write!(f, "{name}")
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    system: Option<System>,

    // Environment
    pub sentry_dsn: Option<String>,

    pub use_tls: bool,
    pub certs_path: Option<String>,

    pub client_url: String,
    pub server_url: String,

    pub postgres_url: String,
    pub redis_url: String,

    pub secret_key: Secret<String>,

    pub s3_bucket: String,
    pub s3_region: String,
    pub s3_endpoint: String,
    pub s3_access_key: String,
    pub s3_secret_key: Secret<String>,

    // System
    pub current_version: String,
    pub previous_version: String,

    pub max_files: Option<i64>,
    pub max_file_size: Option<i64>,
    pub size_unit: Option<SizeUnit>,
    pub accepted_file_extensions: Option<Vec<String>>,

    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<Secret<String>>,
    pub smtp_sender_name: Option<String>,
    pub smtp_sender_email: Option<String>,

    pub google_client_id: Option<String>,
    pub google_client_secret: Option<Secret<String>>,

    pub facebook_client_id: Option<String>,
    pub facebook_client_secret: Option<Secret<String>>,

    pub github_client_id: Option<String>,
    pub github_client_secret: Option<Secret<String>>,

    pub twitter_client_id: Option<String>,
    pub twitter_client_secret: Option<Secret<String>>,

    pub discord_client_id: Option<String>,
    pub discord_client_secret: Option<Secret<String>>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            system: None,

            sentry_dsn: env::var(ConfigKey::SentryDsn.to_string()).ok(),
            secret_key: Secret::new(env::var(ConfigKey::SecretKey.to_string()).unwrap_or(
                "l19OOJaOvpal21ofSlsxYyNVQN2EeTY6gEuq6p_hobH_QmJb3dPELmoGKFstBpCI".into(),
            )),
            use_tls: env::var(ConfigKey::UseTls.to_string())
                .unwrap_or("false".to_string())
                .parse()
                .unwrap_or_log(),
            certs_path: env::var(ConfigKey::CertsPath.to_string()).ok(),
            client_url: env::var(ConfigKey::ClientUrl.to_string()).unwrap_or("/".into()),
            server_url: env::var(ConfigKey::ServerUrl.to_string())
                .unwrap_or("127.0.0.1:8000".into()),
            postgres_url: env::var(ConfigKey::PostgresUrl.to_string()).unwrap_or_log(),
            redis_url: env::var(ConfigKey::RedisUrl.to_string()).unwrap_or_log(),
            s3_bucket: env::var(ConfigKey::S3Bucket.to_string()).unwrap_or_log(),
            s3_region: env::var(ConfigKey::S3Region.to_string()).unwrap_or_log(),
            s3_endpoint: env::var(ConfigKey::S3Endpoint.to_string()).unwrap_or_log(),
            s3_access_key: env::var(ConfigKey::S3AccessKey.to_string()).unwrap_or_log(),
            s3_secret_key: env::var(ConfigKey::S3SecretKey.to_string())
                .map(Secret::new)
                .unwrap_or_log(),

            current_version: env!("CARGO_PKG_VERSION").into(),
            previous_version: env!("CARGO_PKG_VERSION").into(),
            max_files: None,
            max_file_size: None,
            size_unit: None,
            accepted_file_extensions: None,
            smtp_host: None,
            smtp_port: None,
            smtp_username: None,
            smtp_password: None,
            smtp_sender_name: None,
            smtp_sender_email: None,
            google_client_id: env::var(ConfigKey::GoogleClientId.to_string()).ok(),
            google_client_secret: env::var(ConfigKey::GoogleClientSecret.to_string())
                .ok()
                .map(Secret::new),
            facebook_client_id: env::var(ConfigKey::FacebookClientId.to_string()).ok(),
            facebook_client_secret: env::var(ConfigKey::FacebookClientSecret.to_string())
                .ok()
                .map(Secret::new),
            github_client_id: env::var(ConfigKey::GitHubClientId.to_string()).ok(),
            github_client_secret: env::var(ConfigKey::GitHubClientSecret.to_string())
                .ok()
                .map(Secret::new),
            twitter_client_id: env::var(ConfigKey::TwitterClientId.to_string()).ok(),
            twitter_client_secret: env::var(ConfigKey::TwitterClientSecret.to_string())
                .ok()
                .map(Secret::new),
            discord_client_id: env::var(ConfigKey::DiscordClientId.to_string()).ok(),
            discord_client_secret: env::var(ConfigKey::DiscordClientSecret.to_string())
                .ok()
                .map(Secret::new),
        }
    }

    pub fn system(&self) -> &Option<System> {
        &self.system
    }

    pub fn attach_system(&mut self, system: &System) {
        self.system = Some(system.clone());

        if let Some(v) = system.current_version.as_ref() {
            self.current_version.clone_from(v)
        }
        if let Some(v) = system.previous_version.as_ref() {
            self.previous_version.clone_from(v)
        }

        self.max_files = system.max_files;
        self.max_file_size = system.max_file_size;
        self.size_unit.clone_from(&system.size_unit);
        self.accepted_file_extensions
            .clone_from(&system.accepted_file_extensions);

        self.smtp_host = system.smtp_config.clone().map(|c| c.host.clone());
        self.smtp_port = system.smtp_config.clone().map(|c| c.port);
        self.smtp_username = system.smtp_config.clone().map(|c| c.username.clone());
        self.smtp_password = system
            .smtp_config
            .as_ref()
            .map(|c| Secret::new(c.password.clone()));
        self.smtp_sender_name = system.smtp_config.clone().map(|c| c.sender_name.clone());
        self.smtp_sender_email = system.smtp_config.clone().map(|c| c.sender_email.clone());

        self.google_client_id = system.google_config.clone().map(|c| c.client_id.clone());
        self.google_client_secret = system
            .google_config
            .as_ref()
            .map(|c| Secret::new(c.client_secret.clone()));

        self.facebook_client_id = system.facebook_config.clone().map(|c| c.client_id.clone());
        self.facebook_client_secret = system
            .facebook_config
            .as_ref()
            .map(|c| Secret::new(c.client_secret.clone()));

        self.github_client_id = system.github_config.clone().map(|c| c.client_id.clone());
        self.github_client_secret = system
            .github_config
            .as_ref()
            .map(|c| Secret::new(c.client_secret.clone()));

        self.twitter_client_id = system.twitter_config.clone().map(|c| c.client_id.clone());
        self.twitter_client_secret = system
            .twitter_config
            .as_ref()
            .map(|c| Secret::new(c.client_secret.clone()));

        self.discord_client_id = system.discord_config.clone().map(|c| c.client_id.clone());
        self.discord_client_secret = system
            .discord_config
            .as_ref()
            .map(|c| Secret::new(c.client_secret.clone()));
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
