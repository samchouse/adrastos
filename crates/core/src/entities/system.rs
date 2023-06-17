use std::fmt;

use sea_query::{enum_def, Alias, ColumnDef, Expr, PostgresQueryBuilder, Query, Table};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

use super::{Identity, Init};

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
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
}

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct System {
    pub id: String,
    pub current_version: String,
    pub previous_version: Option<String>,

    pub smtp_config: Option<SmtpConfig>,

    pub google_config: Option<OAuth2Config>,
    pub facebook_config: Option<OAuth2Config>,
    pub github_config: Option<OAuth2Config>,
    pub twitter_config: Option<OAuth2Config>,
    pub discord_config: Option<OAuth2Config>,
}

impl System {
    pub fn get() -> String {
        Query::select()
            .from(Self::table())
            .columns([
                <Self as Identity>::Iden::Id,
                <Self as Identity>::Iden::CurrentVersion,
                <Self as Identity>::Iden::PreviousVersion,
                <Self as Identity>::Iden::SmtpConfig,
                <Self as Identity>::Iden::GoogleConfig,
                <Self as Identity>::Iden::FacebookConfig,
                <Self as Identity>::Iden::GithubConfig,
                <Self as Identity>::Iden::TwitterConfig,
                <Self as Identity>::Iden::DiscordConfig,
            ])
            .and_where(Expr::col(<Self as Identity>::Iden::Id).eq("system"))
            .to_string(PostgresQueryBuilder)
    }

    pub fn set(&self) -> String {
        Query::update()
            .table(System::table())
            .values([
                (
                    <System as Identity>::Iden::PreviousVersion,
                    self.previous_version.clone().into(),
                ),
                (
                    <System as Identity>::Iden::SmtpConfig,
                    self.smtp_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (
                    <System as Identity>::Iden::GoogleConfig,
                    self.google_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (
                    <System as Identity>::Iden::FacebookConfig,
                    self.facebook_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (
                    <System as Identity>::Iden::GithubConfig,
                    self.github_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (
                    <System as Identity>::Iden::TwitterConfig,
                    self.twitter_config
                        .as_ref()
                        .and_then(|v| serde_json::to_string(v).ok())
                        .into(),
                ),
                (
                    <System as Identity>::Iden::DiscordConfig,
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
    type Iden = SystemIden;

    fn table() -> Alias {
        Alias::new(<Self as Identity>::Iden::Table.to_string())
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
                ColumnDef::new(<Self as Identity>::Iden::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::CurrentVersion)
                    .string()
                    .not_null(),
            )
            .col(ColumnDef::new(<Self as Identity>::Iden::PreviousVersion).string())
            .col(ColumnDef::new(<Self as Identity>::Iden::SmtpConfig).string())
            .col(ColumnDef::new(<Self as Identity>::Iden::GoogleConfig).string())
            .col(ColumnDef::new(<Self as Identity>::Iden::FacebookConfig).string())
            .col(ColumnDef::new(<Self as Identity>::Iden::GithubConfig).string())
            .col(ColumnDef::new(<Self as Identity>::Iden::TwitterConfig).string())
            .col(ColumnDef::new(<Self as Identity>::Iden::DiscordConfig).string())
            .to_string(PostgresQueryBuilder)
    }
}

impl From<Row> for System {
    // TODO(@Xenfo): automate this trait
    fn from(row: Row) -> Self {
        Self {
            id: row.get(<Self as Identity>::Iden::Id.to_string().as_str()),
            current_version: row.get(
                <Self as Identity>::Iden::CurrentVersion
                    .to_string()
                    .as_str(),
            ),
            previous_version: row.get(
                <Self as Identity>::Iden::PreviousVersion
                    .to_string()
                    .as_str(),
            ),
            smtp_config: row
                .get::<_, Option<String>>(<Self as Identity>::Iden::SmtpConfig.to_string().as_str())
                .map(|v| serde_json::from_str(&v).unwrap()),
            google_config: row
                .get::<_, Option<String>>(
                    <Self as Identity>::Iden::GoogleConfig.to_string().as_str(),
                )
                .map(|v| serde_json::from_str(&v).unwrap()),
            facebook_config: row
                .get::<_, Option<String>>(
                    <Self as Identity>::Iden::FacebookConfig
                        .to_string()
                        .as_str(),
                )
                .map(|v| serde_json::from_str(&v).unwrap()),
            github_config: row
                .get::<_, Option<String>>(
                    <Self as Identity>::Iden::GithubConfig.to_string().as_str(),
                )
                .map(|v| serde_json::from_str(&v).unwrap()),
            twitter_config: row
                .get::<_, Option<String>>(
                    <Self as Identity>::Iden::TwitterConfig.to_string().as_str(),
                )
                .map(|v| serde_json::from_str(&v).unwrap()),
            discord_config: row
                .get::<_, Option<String>>(
                    <Self as Identity>::Iden::DiscordConfig.to_string().as_str(),
                )
                .map(|v| serde_json::from_str(&v).unwrap()),
        }
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
