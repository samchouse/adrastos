use std::fmt;

use adrastos_macros::DbDeserialize;
use sea_query::{enum_def, Alias, ColumnDef, Expr, PostgresQueryBuilder, Query, Table};
use serde::{Deserialize, Serialize};

use super::{Identity, Init};

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, DbDeserialize)]
pub struct System {
    pub id: String,
    pub current_version: String,
    pub previous_version: String,

    pub smtp_config: Option<SmtpConfig>,

    pub google_config: Option<OAuth2Config>,
    pub facebook_config: Option<OAuth2Config>,
    pub github_config: Option<OAuth2Config>,
    pub twitter_config: Option<OAuth2Config>,
    pub discord_config: Option<OAuth2Config>,
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

impl Identity for System {
    fn table() -> Alias {
        Alias::new(SystemIden::Table.to_string())
    }

    fn error_identifier() -> String {
        "system".into()
    }
}

impl Init for System {
    fn init() -> String {
        Table::create()
            .table(Self::table())
            .if_not_exists()
            .col(
                ColumnDef::new(SystemIden::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(SystemIden::CurrentVersion)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(SystemIden::PreviousVersion)
                    .string()
                    .not_null(),
            )
            .col(ColumnDef::new(SystemIden::SmtpConfig).string())
            .col(ColumnDef::new(SystemIden::GoogleConfig).string())
            .col(ColumnDef::new(SystemIden::FacebookConfig).string())
            .col(ColumnDef::new(SystemIden::GithubConfig).string())
            .col(ColumnDef::new(SystemIden::TwitterConfig).string())
            .col(ColumnDef::new(SystemIden::DiscordConfig).string())
            .to_string(PostgresQueryBuilder)
    }
}

impl fmt::Display for SystemIden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Table => "system",
            Self::Id => "id",
            Self::CurrentVersion => "current_version",
            Self::PreviousVersion => "previous_version",
            Self::SmtpConfig => "smtp_config",
            Self::GoogleConfig => "google_config",
            Self::FacebookConfig => "facebook_config",
            Self::GithubConfig => "github_config",
            Self::TwitterConfig => "twitter_config",
            Self::DiscordConfig => "discord_config",
        };

        write!(f, "{name}")
    }
}
