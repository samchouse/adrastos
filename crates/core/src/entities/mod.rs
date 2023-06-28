// TODO(@Xenfo): use `*Iden::Table` instead of Alias::new() once https://github.com/SeaQL/sea-query/issues/533 is fixed

use std::{collections::HashMap, fmt};

use async_trait::async_trait;
use deadpool_postgres::tokio_postgres::Row;
use sea_query::{Alias, Iden, IntoIden, PostgresQueryBuilder, SelectStatement, SimpleExpr};
use secrecy::ExposeSecret;
use serde_json::Value;

use crate::{config::Config, error::Error};

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

trait Init {
    fn init() -> String;
}

pub trait Identity {
    fn table() -> Alias;
    fn error_identifier() -> String;
}

pub trait Query {
    fn query_select(expressions: Vec<SimpleExpr>) -> SelectStatement;
    fn query_insert(&self) -> Result<String, Error>;
    fn query_update(&self, updated: &HashMap<String, Value>) -> Result<String, Error>;
    fn query_delete(&self) -> String;
}

enum JoinKeys {
    Connection,
    RefreshTokenTree,
}

impl JoinKeys {
    fn plural(&self) -> String {
        self.to_string() + "s"
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

#[async_trait]
pub trait Mutate: Sized {
    async fn find(
        db_pool: &deadpool_postgres::Pool,
        expressions: Vec<SimpleExpr>,
    ) -> Result<Self, Error>;
    async fn create(&self, db_pool: &deadpool_postgres::Pool) -> Result<(), Error>;
    async fn update_old(
        &self,
        db_pool: &deadpool_postgres::Pool,
        updated: &HashMap<String, Value>,
    ) -> Result<(), Error>;
    async fn delete(&self, db_pool: &deadpool_postgres::Pool) -> Result<(), Error>;
}

#[async_trait]
impl<T: Identity + Query + From<Row> + Sync> Mutate for T {
    async fn find(
        db_pool: &deadpool_postgres::Pool,
        expressions: Vec<SimpleExpr>,
    ) -> Result<Self, Error> {
        let row = db_pool
            .get()
            .await
            .unwrap()
            .query(
                T::query_select(expressions)
                    .limit(1)
                    .to_string(PostgresQueryBuilder)
                    .as_str(),
                &[],
            )
            .await
            .map_err(|e| {
                let error = format!(
                    "An error occurred while fetching the {}: {e}",
                    T::error_identifier(),
                );
                Error::InternalServerError(error)
            })?
            .into_iter()
            .next()
            .ok_or_else(|| {
                let message = format!("No {} was found", Self::error_identifier());
                Error::BadRequest(message)
            })?;

        Ok(row.into())
    }

    async fn create(&self, db_pool: &deadpool_postgres::Pool) -> Result<(), Error> {
        db_pool
            .get()
            .await
            .unwrap()
            .query(self.query_insert()?.as_str(), &[])
            .await
            .map_err(|e| {
                let error = format!(
                    "An error occurred while creating the {}: {e}",
                    T::error_identifier(),
                );
                Error::InternalServerError(error)
            })?;

        Ok(())
    }

    async fn update_old(
        &self,
        db_pool: &deadpool_postgres::Pool,
        updated: &HashMap<String, Value>,
    ) -> Result<(), Error> {
        db_pool
            .get()
            .await
            .unwrap()
            .query(self.query_update(updated)?.as_str(), &[])
            .await
            .map_err(|e| {
                let error = format!(
                    "An error occurred while updating the {}: {e}",
                    T::error_identifier(),
                );
                Error::InternalServerError(error)
            })?;

        Ok(())
    }

    async fn delete(&self, db_pool: &deadpool_postgres::Pool) -> Result<(), Error> {
        db_pool
            .get()
            .await
            .unwrap()
            .query(self.query_delete().as_str(), &[])
            .await
            .map_err(|e| {
                let error = format!(
                    "An error occurred while deleting the {}: {e}",
                    T::error_identifier(),
                );
                Error::InternalServerError(error)
            })?;

        Ok(())
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
