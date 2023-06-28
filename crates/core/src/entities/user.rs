#![allow(dead_code)]

use std::fmt;

use adrastos_macros::{DbDeserialize, DbSelect};
use chrono::{DateTime, Utc};
use sea_query::{
    enum_def, Alias, ColumnDef, ColumnType, Expr, Keyword, PostgresQueryBuilder, SelectStatement,
    SimpleExpr, Table,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing_unwrap::ResultExt;
use utoipa::ToSchema;
use validator::Validate;

use crate::{auth, error::Error};

use super::{Connection, Identity, Init, JoinKeys, Query, RefreshTokenTree, Update};

#[enum_def]
#[derive(Debug, Validate, Serialize, Deserialize, Clone, ToSchema, DbDeserialize, DbSelect)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    #[select(find)]
    #[validate(length(max = 50))]
    pub first_name: String,
    #[select(find)]
    #[validate(length(max = 50))]
    pub last_name: String,
    #[select(find)]
    #[validate(email)]
    pub email: String,
    #[select(find)]
    #[validate(length(min = 5, max = 64))]
    pub username: String,
    #[serde(skip_serializing)]
    #[validate(length(min = 8, max = 64))]
    pub password: String,
    pub verified: bool,
    pub banned: bool,
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,
    #[serde(skip_serializing)]
    pub mfa_backup_codes: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,

    #[select(skip)]
    #[serde(skip_serializing)]
    pub connections: Option<Vec<Connection>>,
    #[select(skip)]
    #[serde(skip_serializing)]
    pub refresh_token_trees: Option<Vec<RefreshTokenTree>>,
}

#[derive(Debug, Validate, Clone, Default)]
pub struct UpdateUser {
    #[validate(length(max = 50))]
    pub first_name: Option<String>,
    #[validate(length(max = 50))]
    pub last_name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 5, max = 64))]
    pub username: Option<String>,
    #[validate(length(min = 8, max = 64))]
    pub password: Option<String>,
    pub verified: Option<bool>,
    pub banned: Option<bool>,
    pub mfa_secret: Option<Option<String>>,
    pub mfa_backup_codes: Option<Option<Vec<String>>>,
}

impl UserSelectBuilder {
    pub fn join<T: Query + Identity>(&mut self, alias: Alias) -> &mut Self {
        self.query_builder.expr(Expr::cust(
            format!(
                "(SELECT json_agg({}) FROM ({}) {}) as {}",
                JoinKeys::from_identity::<T>(),
                T::query_select(vec![Expr::col(alias).equals((User::table(), UserIden::Id))])
                    .to_string(PostgresQueryBuilder),
                JoinKeys::from_identity::<T>(),
                JoinKeys::from_identity::<T>().plural()
            )
            .as_str(),
        ));

        self
    }
}

impl User {
    pub async fn update(
        &self,
        db_pool: &deadpool_postgres::Pool,
        update: UpdateUser,
    ) -> Result<(), Error> {
        update.validate().map_err(|e| Error::ValidationErrors {
            errors: e,
            message: "Invalid user update".into(),
        })?;

        let query = sea_query::Query::update()
            .table(Self::table())
            .values(Update::create([
                (UserIden::FirstName, update.first_name.into()),
                (UserIden::LastName, update.last_name.into()),
                (UserIden::Email, update.email.into()),
                (UserIden::Username, update.username.into()),
                (
                    UserIden::Password,
                    update
                        .password
                        .map(|v| auth::hash_password(v.as_str()).unwrap_or_log())
                        .into(),
                ),
                (UserIden::Verified, update.verified.into()),
                (UserIden::Banned, update.banned.into()),
                (UserIden::MfaSecret, update.mfa_secret.into()),
                (UserIden::MfaBackupCodes, update.mfa_backup_codes.into()),
                (UserIden::UpdatedAt, Some(Utc::now()).into()),
            ]))
            .and_where(Expr::col(UserIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder);

        db_pool
            .get()
            .await
            .unwrap_or_log()
            .execute(&query, &[])
            .await
            .map_err(|e| {
                error!(error = ?e);
                Error::InternalServerError("Failed to update user".into())
            })?;

        Ok(())
    }
}

impl Identity for User {
    fn table() -> Alias {
        Alias::new(UserIden::Table.to_string())
    }

    fn error_identifier() -> String {
        "user".into()
    }
}

impl Init for User {
    fn init() -> String {
        Table::create()
            .table(Self::table())
            .if_not_exists()
            .col(
                ColumnDef::new(UserIden::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(UserIden::FirstName).string().not_null())
            .col(ColumnDef::new(UserIden::LastName).string().not_null())
            .col(
                ColumnDef::new(UserIden::Email)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(UserIden::Username)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(ColumnDef::new(UserIden::Password).string().not_null())
            .col(
                ColumnDef::new(UserIden::Verified)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(UserIden::Banned)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(ColumnDef::new(UserIden::MfaSecret).string())
            .col(ColumnDef::new(UserIden::MfaBackupCodes).array(ColumnType::String(None)))
            .col(
                ColumnDef::new(UserIden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(ColumnDef::new(UserIden::UpdatedAt).timestamp_with_time_zone())
            .to_string(PostgresQueryBuilder)
    }
}

impl Query for User {
    fn query_select(_: Vec<SimpleExpr>) -> SelectStatement {
        unimplemented!("User does not implement Query::query_select")
    }

    fn query_insert(&self) -> Result<String, Error> {
        self.validate().map_err(|err| Error::ValidationErrors {
            message: format!(
                "An error occurred while validating the {}",
                Self::error_identifier(),
            ),
            errors: err,
        })?;

        let hashed_password = auth::hash_password(self.password.as_str()).map_err(|err| {
            Error::InternalServerError(format!(
                "An error occurred while hashing the password for the {err}"
            ))
        })?;

        Ok(sea_query::Query::insert()
            .into_table(Self::table())
            .columns([
                UserIden::Id,
                UserIden::FirstName,
                UserIden::LastName,
                UserIden::Email,
                UserIden::Username,
                UserIden::Password,
            ])
            .values_panic(vec![
                self.id.clone().into(),
                self.first_name.clone().into(),
                self.last_name.clone().into(),
                self.email.clone().into(),
                self.username.clone().into(),
                hashed_password.into(),
            ])
            .to_string(PostgresQueryBuilder))
    }

    fn query_delete(&self) -> String {
        sea_query::Query::delete()
            .from_table(Self::table())
            .and_where(Expr::col(UserIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder)
    }
}

impl fmt::Display for UserIden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Table => "users",
            Self::Id => "id",
            Self::FirstName => "first_name",
            Self::LastName => "last_name",
            Self::Email => "email",
            Self::Username => "username",
            Self::Password => "password",
            Self::Verified => "verified",
            Self::Banned => "banned",
            Self::MfaSecret => "mfa_secret",
            Self::MfaBackupCodes => "mfa_backup_codes",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",

            Self::Connections => "connections",
            Self::RefreshTokenTrees => "refresh_token_trees",
        };

        write!(f, "{name}")
    }
}
