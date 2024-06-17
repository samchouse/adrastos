use std::fmt;

use adrastos_macros::DbCommon;
use chrono::{DateTime, Utc};
use sea_query::{enum_def, Expr, PostgresQueryBuilder, Query};
use serde::{Deserialize, Serialize};

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, DbCommon)]
#[adrastos(rename = "system")]
pub struct System {
    pub id: String,
    pub current_version: Option<String>,
    pub previous_version: Option<String>,

    pub webhook_config: Option<WebhookConfig>,

    pub max_files: Option<i64>,
    pub max_file_size: Option<i64>,
    pub size_unit: Option<SizeUnit>,
    pub accepted_file_extensions: Option<Vec<String>>,

    pub smtp_config: Option<SmtpConfig>,

    pub google_config: Option<OAuth2Config>,
    pub facebook_config: Option<OAuth2Config>,
    pub github_config: Option<OAuth2Config>,
    pub twitter_config: Option<OAuth2Config>,
    pub discord_config: Option<OAuth2Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub key: String,
    pub permissions_url: String,
    #[serde(flatten)]
    pub provider: WebhookProvider,
    pub historical: Option<Historical>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Historical {
    pub hash: u64,
    pub build_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "provider")]
pub enum WebhookProvider {
    GitHub {
        branch: String,
        secret: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SizeUnit {
    Mb,
    Gb,
}

impl fmt::Display for SizeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Mb => "MB",
            Self::Gb => "GB",
        };

        write!(f, "{name}")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String, // TODO(@Xenfo): Encrypt this
    pub sender_name: String,
    pub sender_email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
}

impl System {
    pub fn get() -> String {
        Query::select()
            .from(Self::table())
            .columns([
                SystemIden::Id,
                SystemIden::CurrentVersion,
                SystemIden::PreviousVersion,
                SystemIden::WebhookConfig,
                SystemIden::MaxFiles,
                SystemIden::MaxFileSize,
                SystemIden::SizeUnit,
                SystemIden::AcceptedFileExtensions,
                SystemIden::SmtpConfig,
                SystemIden::GoogleConfig,
                SystemIden::FacebookConfig,
                SystemIden::GithubConfig,
                SystemIden::TwitterConfig,
                SystemIden::DiscordConfig,
            ])
            .and_where(Expr::col(SystemIden::Id).eq("system"))
            .to_string(PostgresQueryBuilder)
    }

    pub fn set(&self) -> String {
        Query::update()
            .table(System::table())
            .values([
                (
                    SystemIden::CurrentVersion,
                    self.current_version.clone().into(),
                ),
                (
                    SystemIden::PreviousVersion,
                    self.previous_version.clone().into(),
                ),
                (
                    SystemIden::WebhookConfig,
                    self.webhook_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (SystemIden::MaxFiles, self.max_files.into()),
                (SystemIden::MaxFileSize, self.max_file_size.into()),
                (
                    SystemIden::SizeUnit,
                    self.size_unit
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (
                    SystemIden::AcceptedFileExtensions,
                    self.accepted_file_extensions.clone().into(),
                ),
                (
                    SystemIden::SmtpConfig,
                    self.smtp_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (
                    SystemIden::GoogleConfig,
                    self.google_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (
                    SystemIden::FacebookConfig,
                    self.facebook_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (
                    SystemIden::GithubConfig,
                    self.github_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (
                    SystemIden::TwitterConfig,
                    self.twitter_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (
                    SystemIden::DiscordConfig,
                    self.discord_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
            ])
            .to_string(PostgresQueryBuilder)
    }
}
