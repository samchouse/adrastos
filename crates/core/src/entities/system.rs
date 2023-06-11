use std::fmt;

use sea_query::{enum_def, Alias, ColumnDef, PostgresQueryBuilder, Table};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

use super::{Identity, Init};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub sender_name: String,
    pub sender_email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FacebookConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GithubConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TwitterConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordConfig {
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

    pub google_config: Option<GoogleConfig>,
    pub facebook_config: Option<FacebookConfig>,
    pub github_config: Option<GithubConfig>,
    pub twitter_config: Option<TwitterConfig>,
    pub discord_config: Option<DiscordConfig>,
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
            smtp_config: serde_json::from_value(
                serde_json::to_value(row.get::<_, Option<String>>(
                    <Self as Identity>::Iden::SmtpConfig.to_string().as_str(),
                ))
                .unwrap(),
            )
            .unwrap(),
            google_config: serde_json::from_value(
                serde_json::to_value(row.get::<_, Option<String>>(
                    <Self as Identity>::Iden::GoogleConfig.to_string().as_str(),
                ))
                .unwrap(),
            )
            .unwrap(),
            facebook_config: serde_json::from_value(
                serde_json::to_value(
                    row.get::<_, Option<String>>(
                        <Self as Identity>::Iden::FacebookConfig
                            .to_string()
                            .as_str(),
                    ),
                )
                .unwrap(),
            )
            .unwrap(),
            github_config: serde_json::from_value(
                serde_json::to_value(row.get::<_, Option<String>>(
                    <Self as Identity>::Iden::GithubConfig.to_string().as_str(),
                ))
                .unwrap(),
            )
            .unwrap(),
            twitter_config: serde_json::from_value(
                serde_json::to_value(row.get::<_, Option<String>>(
                    <Self as Identity>::Iden::TwitterConfig.to_string().as_str(),
                ))
                .unwrap(),
            )
            .unwrap(),
            discord_config: serde_json::from_value(
                serde_json::to_value(row.get::<_, Option<String>>(
                    <Self as Identity>::Iden::DiscordConfig.to_string().as_str(),
                ))
                .unwrap(),
            )
            .unwrap(),
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
