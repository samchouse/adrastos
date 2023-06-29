// TODO(@Xenfo): use `*Iden::Table` instead of Alias::new() once https://github.com/SeaQL/sea-query/issues/533 is fixed

use std::fmt;

use sea_query::{Alias, Iden, IntoIden, PostgresQueryBuilder, SimpleExpr};
use secrecy::ExposeSecret;

use crate::config::Config;

pub use connection::*;
pub use refresh_token_tree::*;
pub use system::*;
pub use user::*;

use self::custom_table::schema::CustomTableSchema;

pub mod connection;
pub mod custom_table;
pub mod refresh_token_tree;
pub mod system;
pub mod user;

pub trait Identity {
    fn table() -> Alias;
    fn error_identifier() -> String;
}

pub trait Join {
    fn join(expr: SimpleExpr) -> sea_query::SelectStatement;
}

enum JoinKeys {
    Connection,
    RefreshTokenTree,
}

impl JoinKeys {
    fn plural(&self) -> String {
        format!("{}s", self.to_string())
    }

    fn from_identity<T: Identity>() -> Self {
        match T::table().to_string() {
            con if con == Connection::table().to_string() => JoinKeys::Connection,
            tree if tree == RefreshTokenTree::table().to_string() => JoinKeys::RefreshTokenTree,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for JoinKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            JoinKeys::Connection => "connection",
            JoinKeys::RefreshTokenTree => "refresh_token_tree",
        };

        write!(f, "{name}")
    }
}

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

pub async fn init(db_pool: &deadpool_postgres::Pool, config: &Config) {
    let conn = db_pool.get().await.unwrap();

    let query = conn
        .query(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';",
            &[],
        )
        .await
        .unwrap();
    let count = query.get(0).unwrap().get::<_, i64>(0);
    if count > 0 {
        return;
    }

    let inits = vec![
        System::init(),
        User::init(),
        Connection::init(),
        RefreshTokenTree::init(),
        CustomTableSchema::init(),
    ];
    for init in inits {
        conn.execute(&init, &[]).await.unwrap();
    }

    let mut smtp_config = None;
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
    let mut google_config = None;
    if let Some(client_id) = config.google_client_id.clone() {
        if let Some(client_secret) = config.google_client_secret.clone() {
            google_config = Some(OAuth2Config {
                client_id,
                client_secret: client_secret.expose_secret().to_string(),
            });
        }
    }
    let mut facebook_config = None;
    if let Some(client_id) = config.facebook_client_id.clone() {
        if let Some(client_secret) = config.facebook_client_secret.clone() {
            facebook_config = Some(OAuth2Config {
                client_id,
                client_secret: client_secret.expose_secret().to_string(),
            });
        }
    }
    let mut github_config = None;
    if let Some(client_id) = config.github_client_id.clone() {
        if let Some(client_secret) = config.github_client_secret.clone() {
            github_config = Some(OAuth2Config {
                client_id,
                client_secret: client_secret.expose_secret().to_string(),
            });
        }
    }
    let mut twitter_config = None;
    if let Some(client_id) = config.twitter_client_id.clone() {
        if let Some(client_secret) = config.twitter_client_secret.clone() {
            twitter_config = Some(OAuth2Config {
                client_id,
                client_secret: client_secret.expose_secret().to_string(),
            });
        }
    }
    let mut discord_config = None;
    if let Some(client_id) = config.discord_client_id.clone() {
        if let Some(client_secret) = config.discord_client_secret.clone() {
            discord_config = Some(OAuth2Config {
                client_id,
                client_secret: client_secret.expose_secret().to_string(),
            });
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
