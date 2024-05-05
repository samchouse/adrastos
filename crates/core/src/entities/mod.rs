// TODO(@Xenfo): use `*Iden::Table` instead of Alias::new() once https://github.com/SeaQL/sea-query/issues/533 is fixed

use chrono::Utc;
use sea_query::{IntoIden, PostgresQueryBuilder, SimpleExpr};
use secrecy::ExposeSecret;

use crate::{config::Config, db::postgres::DatabaseType, id::Id};

use self::custom_table::schema::CustomTableSchema;

pub use any_user::*;
pub use connection::*;
pub use passkey::*;
pub use project::*;
pub use refresh_token_tree::*;
pub use system::*;
pub use system_user::*;
pub use team::*;
pub use upload::*;
pub use user::*;

pub mod any_user;
pub mod connection;
pub mod custom_table;
pub mod passkey;
pub mod project;
pub mod refresh_token_tree;
pub mod system;
pub mod system_user;
pub mod team;
pub mod upload;
pub mod user;

#[derive(Debug, Clone)]
enum Update {
    Skip,
    Set(SimpleExpr),
}

impl<T> From<Option<T>> for Update
where
    T: Into<SimpleExpr>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Update::Set(value.into()),
            None => Update::Skip,
        }
    }
}

impl Update {
    fn create<T, I>(values: I) -> Vec<(T, SimpleExpr)>
    where
        T: IntoIden,
        I: IntoIterator<Item = (T, Update)>,
    {
        values
            .into_iter()
            .filter_map(|(key, value)| match value {
                Update::Skip => None,
                Update::Set(value) => Some((key, value)),
            })
            .collect()
    }
}

pub async fn init(db_type: &DatabaseType, db: &deadpool_postgres::Pool, config: &Config) {
    let conn = db.get().await.unwrap();

    let query = conn
        .query(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';",
            &[],
        )
        .await
        .unwrap();
    let count = query.first().unwrap().get::<_, i64>(0);
    if count > 0 {
        return;
    }

    let inits = match db_type {
        DatabaseType::System => {
            vec![
                System::init(),
                SystemUser::init(),
                Team::init(),
                Project::init(),
                Connection::init(),
                Passkey::init(),
                RefreshTokenTree::init(),
            ]
        }
        DatabaseType::Project(_) => {
            vec![
                System::init(),
                User::init(),
                Connection::init(),
                RefreshTokenTree::init(),
                CustomTableSchema::init(),
                Passkey::init(),
                Upload::init(),
            ]
        }
    };
    for init in inits {
        conn.execute(&init, &[]).await.unwrap();
    }

    let mut smtp_config = None;
    let mut google_config = None;
    let mut facebook_config = None;
    let mut github_config = None;
    let mut twitter_config = None;
    let mut discord_config = None;

    if db_type == &DatabaseType::System {
        Team {
            id: Id::new().to_string(),
            name: "Personal Projects".into(),
            created_at: Utc::now(),
            ..Default::default()
        }
        .create(db)
        .await
        .unwrap();

        if let Some(host) = config.smtp_host.clone() {
            if let Some(port) = config.smtp_port {
                if let Some(username) = config.smtp_username.clone() {
                    if let Some(password) = config.smtp_password.clone() {
                        smtp_config = Some(SmtpConfig {
                            host,
                            port,
                            username,
                            password: password.expose_secret().to_string(),
                            sender_name: "Adrastos".into(),
                            sender_email: "no-reply@adrastos.example.com".into(),
                        });
                    }
                }
            }
        }
        if let Some(client_id) = config.google_client_id.clone() {
            if let Some(client_secret) = config.google_client_secret.clone() {
                google_config = Some(OAuth2Config {
                    client_id,
                    client_secret: client_secret.expose_secret().to_string(),
                });
            }
        }
        if let Some(client_id) = config.facebook_client_id.clone() {
            if let Some(client_secret) = config.facebook_client_secret.clone() {
                facebook_config = Some(OAuth2Config {
                    client_id,
                    client_secret: client_secret.expose_secret().to_string(),
                });
            }
        }
        if let Some(client_id) = config.github_client_id.clone() {
            if let Some(client_secret) = config.github_client_secret.clone() {
                github_config = Some(OAuth2Config {
                    client_id,
                    client_secret: client_secret.expose_secret().to_string(),
                });
            }
        }
        if let Some(client_id) = config.twitter_client_id.clone() {
            if let Some(client_secret) = config.twitter_client_secret.clone() {
                twitter_config = Some(OAuth2Config {
                    client_id,
                    client_secret: client_secret.expose_secret().to_string(),
                });
            }
        }
        if let Some(client_id) = config.discord_client_id.clone() {
            if let Some(client_secret) = config.discord_client_secret.clone() {
                discord_config = Some(OAuth2Config {
                    client_id,
                    client_secret: client_secret.expose_secret().to_string(),
                });
            }
        }
    }

    let query = sea_query::Query::insert()
        .into_table(System::table())
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
        .values_panic([
            "system".into(),
            env!("CARGO_PKG_VERSION").into(),
            env!("CARGO_PKG_VERSION").into(),
            smtp_config
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok())
                .into(),
            google_config
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok())
                .into(),
            facebook_config
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok())
                .into(),
            github_config
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok())
                .into(),
            twitter_config
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok())
                .into(),
            discord_config
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok())
                .into(),
        ])
        .to_string(PostgresQueryBuilder);
    conn.execute(&query, &[]).await.unwrap();
}
