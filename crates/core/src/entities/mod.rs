// TODO(@Xenfo): use `*Iden::Table` instead of Alias::new() once https://github.com/SeaQL/sea-query/issues/533 is fixed

use std::collections::HashMap;

use actix_web::web;
use async_trait::async_trait;
use deadpool_postgres::tokio_postgres::Row;
use sea_query::{Alias, SimpleExpr};
use serde_json::Value;

use crate::handlers::Error;

pub use connection::*;
pub use refresh_token_tree::*;
pub use user::*;

pub mod connection;
pub mod migrations;
pub mod refresh_token_tree;
pub mod user;

trait Migrate {
    fn migrate() -> String;
}

trait Identity {
    type Iden;

    fn table() -> Alias;
    fn error_identifier() -> String;
}

trait Query {
    fn query_select(expressions: Vec<SimpleExpr>) -> String;
    fn query_insert(&self) -> Result<String, Error>;
    fn query_update(&self, updated: HashMap<String, Value>) -> Result<String, Error>;
    fn query_delete(&self) -> String;
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
        updated: HashMap<String, Value>,
    ) -> Result<(), Error>;
    async fn delete(&self, db_pool: &web::Data<deadpool_postgres::Pool>) -> Result<(), Error>;
}

#[async_trait]
impl<T: Identity + Query + Migrate + From<Row> + Sync> Mutate for T {
    async fn find(
        db_pool: &web::Data<deadpool_postgres::Pool>,
        expressions: Vec<SimpleExpr>,
    ) -> Result<Self, Error> {
        let row = db_pool
            .get()
            .await
            .unwrap()
            .query(T::query_select(expressions).as_str(), &[])
            .await
            .map_err(|e| {
                let error = format!(
                    "An error occurred while fetching the {}: {e}",
                    T::error_identifier(),
                );
                Error::InternalServerError { error }
            })?
            .into_iter()
            .next()
            .ok_or_else(|| {
                let message = format!("No {} was found", Self::error_identifier());
                Error::BadRequest { message }
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
                Error::InternalServerError { error }
            })?;

        Ok(())
    }

    async fn update(
        &self,
        db_pool: &web::Data<deadpool_postgres::Pool>,
        updated: HashMap<String, Value>,
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
                Error::InternalServerError { error }
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
                Error::InternalServerError { error }
            })?;

        Ok(())
    }
}
