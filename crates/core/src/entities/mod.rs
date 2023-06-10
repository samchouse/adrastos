// TODO(@Xenfo): use `*Iden::Table` instead of Alias::new() once https://github.com/SeaQL/sea-query/issues/533 is fixed

use std::{collections::HashMap, fmt};

use actix_web::web;
use async_trait::async_trait;
use deadpool_postgres::tokio_postgres::Row;
use sea_query::{Alias, Iden, PostgresQueryBuilder, SelectStatement, SimpleExpr};
use serde_json::Value;

use crate::error::Error;

pub use connection::*;
pub use refresh_token_tree::*;
pub use user::*;

use self::custom_table::schema::CustomTableSchema;

pub mod connection;
pub mod custom_table;
pub mod refresh_token_tree;
pub mod user;

trait Init {
    fn init() -> String;
}

pub trait Identity {
    type Iden;

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

#[async_trait]
pub trait Mutate: Sized {
    async fn find(
        db_pool: &web::Data<deadpool_postgres::Pool>,
        expressions: Vec<SimpleExpr>,
    ) -> Result<Self, Error>;
    async fn create(&self, db_pool: &web::Data<deadpool_postgres::Pool>) -> Result<(), Error>;
    async fn update(
        &self,
        db_pool: &web::Data<deadpool_postgres::Pool>,
        updated: &HashMap<String, Value>,
    ) -> Result<(), Error>;
    async fn delete(&self, db_pool: &web::Data<deadpool_postgres::Pool>) -> Result<(), Error>;
}

#[async_trait]
impl<T: Identity + Query + Init + From<Row> + Sync> Mutate for T {
    async fn find(
        db_pool: &web::Data<deadpool_postgres::Pool>,
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

    async fn create(&self, db_pool: &web::Data<deadpool_postgres::Pool>) -> Result<(), Error> {
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

    async fn update(
        &self,
        db_pool: &web::Data<deadpool_postgres::Pool>,
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

    async fn delete(&self, db_pool: &web::Data<deadpool_postgres::Pool>) -> Result<(), Error> {
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

pub async fn init(db_pool: &deadpool_postgres::Pool) {
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
        User::init(),
        Connection::init(),
        RefreshTokenTree::init(),
        CustomTableSchema::init(),
    ];

    for init in inits {
        conn.execute(init.as_str(), &[]).await.unwrap();
    }
}
