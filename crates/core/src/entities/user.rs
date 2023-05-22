#![allow(dead_code)]

use std::{collections::HashMap, fmt};

use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use sea_query::{
    enum_def, Alias, ColumnDef, ColumnType, Expr, Keyword, PostgresQueryBuilder, SelectStatement,
    SimpleExpr, Table,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use validator::Validate;

use crate::{auth, error::Error};

use super::{Connection, Identity, JoinKeys, Migrate, Query, RefreshTokenTree};

pub struct UserSelectBuilder {
    query_builder: sea_query::SelectStatement,
}

#[enum_def]
#[derive(Debug, Validate, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    #[validate(length(max = 50))]
    pub first_name: String,
    #[validate(length(max = 50))]
    pub last_name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 5, max = 64))]
    pub username: String,
    #[serde(skip_serializing)]
    #[validate(length(min = 8, max = 64))]
    pub password: String,
    pub verified: bool,
    pub banned: bool,
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,
    // #[serde(skip_serializing)]
    pub mfa_backup_codes: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,

    pub connections: Option<Vec<Connection>>,
    pub refresh_token_trees: Option<Vec<RefreshTokenTree>>,
}

impl UserSelectBuilder {
    pub fn by_id(&mut self, id: &str) -> &mut Self {
        self.query_builder
            .and_where(Expr::col(<User as Identity>::Iden::Id).eq(id));

        self
    }

    pub fn by_email(&mut self, email: &str) -> &mut Self {
        self.query_builder
            .and_where(Expr::col(<User as Identity>::Iden::Email).eq(email));

        self
    }

    pub fn by_username(&mut self, username: &str) -> &mut Self {
        self.query_builder
            .and_where(Expr::col(<User as Identity>::Iden::Username).eq(username));

        self
    }

    pub fn and_where(&mut self, expressions: Vec<SimpleExpr>) -> &mut Self {
        for expression in expressions {
            self.query_builder.and_where(expression);
        }

        self
    }

    pub fn join<T: Query + Identity>(&mut self, alias: Alias) -> &mut Self {
        self.query_builder.expr(Expr::cust(
            format!(
                "(SELECT json_agg({}) FROM ({}) {}) as {}",
                JoinKeys::from_identity::<T>(),
                T::query_select(vec![Expr::col(alias).eq(format!(
                    "{}.{}",
                    UserIden::Table,
                    UserIden::Id
                ))])
                .to_string(PostgresQueryBuilder),
                JoinKeys::from_identity::<T>(),
                JoinKeys::from_identity::<T>().plural()
            )
            .as_str(),
        ));

        self
    }

    pub async fn finish(&mut self, db_pool: &deadpool_postgres::Pool) -> Result<User, Error> {
        let row = db_pool
            .get()
            .await
            .unwrap()
            .query(
                self.query_builder.to_string(PostgresQueryBuilder).as_str(),
                &[],
            )
            .await
            .map_err(|e| {
                let error = format!(
                    "An error occurred while fetching the {}: {e}",
                    User::error_identifier(),
                );
                Error::InternalServerError(error)
            })?
            .into_iter()
            .next()
            .ok_or_else(|| {
                let message = format!("No {} was found", User::error_identifier());
                Error::BadRequest(message)
            })?;

        Ok(row.into())
    }
}

impl Identity for User {
    type Iden = UserIden;

    fn table() -> Alias {
        Alias::new(<Self as Identity>::Iden::Table.to_string())
    }

    fn error_identifier() -> String {
        "user".into()
    }
}

impl Migrate for User {
    fn migrate() -> String {
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
                ColumnDef::new(<Self as Identity>::Iden::FirstName)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::LastName)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::Email)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::Username)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::Password)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::Verified)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::Banned)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(ColumnDef::new(<Self as Identity>::Iden::MfaSecret).string())
            .col(
                ColumnDef::new(<Self as Identity>::Iden::MfaBackupCodes)
                    .array(ColumnType::String(None)),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(ColumnDef::new(<Self as Identity>::Iden::UpdatedAt).timestamp_with_time_zone())
            .to_string(PostgresQueryBuilder)
    }
}

impl User {
    pub fn select() -> UserSelectBuilder {
        UserSelectBuilder {
            query_builder: sea_query::Query::select()
                .from(Self::table())
                .columns([
                    <Self as Identity>::Iden::Id,
                    <Self as Identity>::Iden::FirstName,
                    <Self as Identity>::Iden::LastName,
                    <Self as Identity>::Iden::Email,
                    <Self as Identity>::Iden::Username,
                    <Self as Identity>::Iden::Password,
                    <Self as Identity>::Iden::Verified,
                    <Self as Identity>::Iden::Banned,
                    <Self as Identity>::Iden::MfaSecret,
                    <Self as Identity>::Iden::MfaBackupCodes,
                    <Self as Identity>::Iden::CreatedAt,
                    <Self as Identity>::Iden::UpdatedAt,
                ])
                .limit(1)
                .to_owned(),
        }
    }
}

impl Query for User {
    fn query_select(expressions: Vec<SimpleExpr>) -> SelectStatement {
        let mut query = sea_query::Query::select();

        for expression in expressions {
            query.and_where(expression);
        }

        query
            .from(Self::table())
            .columns([
                <Self as Identity>::Iden::Id,
                <Self as Identity>::Iden::FirstName,
                <Self as Identity>::Iden::LastName,
                <Self as Identity>::Iden::Email,
                <Self as Identity>::Iden::Username,
                <Self as Identity>::Iden::Password,
                <Self as Identity>::Iden::Verified,
                <Self as Identity>::Iden::Banned,
                <Self as Identity>::Iden::MfaSecret,
                <Self as Identity>::Iden::MfaBackupCodes,
                <Self as Identity>::Iden::CreatedAt,
                <Self as Identity>::Iden::UpdatedAt,
            ])
            .to_owned()
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
                <Self as Identity>::Iden::Id,
                <Self as Identity>::Iden::FirstName,
                <Self as Identity>::Iden::LastName,
                <Self as Identity>::Iden::Email,
                <Self as Identity>::Iden::Username,
                <Self as Identity>::Iden::Password,
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

    fn query_update(&self, updated: &HashMap<String, Value>) -> Result<String, Error> {
        let mut updated_for_validation = self.clone();
        let mut query = sea_query::Query::update();

        if let Some(first_name) =
            updated.get(<Self as Identity>::Iden::FirstName.to_string().as_str())
        {
            if let Some(first_name) = first_name.as_str() {
                updated_for_validation.first_name = first_name.into();
                query.values([(<Self as Identity>::Iden::FirstName, first_name.into())]);
            }
        }
        if let Some(last_name) =
            updated.get(<Self as Identity>::Iden::LastName.to_string().as_str())
        {
            if let Some(last_name) = last_name.as_str() {
                updated_for_validation.last_name = last_name.into();
                query.values([(<Self as Identity>::Iden::LastName, last_name.into())]);
            }
        }
        if let Some(email) = updated.get(<Self as Identity>::Iden::Email.to_string().as_str()) {
            if let Some(email) = email.as_str() {
                updated_for_validation.email = email.into();
                query.values([(<Self as Identity>::Iden::Email, email.into())]);
            }
        }
        if let Some(username) = updated.get(<Self as Identity>::Iden::Username.to_string().as_str())
        {
            if let Some(username) = username.as_str() {
                updated_for_validation.username = username.into();
                query.values([(<Self as Identity>::Iden::Username, username.into())]);
            }
        }
        if let Some(password) = updated.get(<Self as Identity>::Iden::Password.to_string().as_str())
        {
            if let Some(password) = password.as_str() {
                updated_for_validation.password = password.into();
                query.values([(<Self as Identity>::Iden::Password, password.into())]);
            }
        }
        if let Some(mfa_secret) =
            updated.get(<Self as Identity>::Iden::MfaSecret.to_string().as_str())
        {
            if let Ok(mfa_secret) = serde_json::from_value::<Option<String>>(mfa_secret.clone()) {
                query.values([(<Self as Identity>::Iden::MfaSecret, mfa_secret.into())]);
            }
        }
        if let Some(mfa_backup_codes) = updated.get(
            <Self as Identity>::Iden::MfaBackupCodes
                .to_string()
                .as_str(),
        ) {
            if let Ok(mfa_backup_codes) =
                serde_json::from_value::<Option<Vec<String>>>(mfa_backup_codes.clone())
            {
                query.values([(
                    <Self as Identity>::Iden::MfaBackupCodes,
                    mfa_backup_codes.into(),
                )]);
            }
        }

        Self {
            password: "password".into(),
            ..updated_for_validation
        }
        .validate()
        .map_err(|err| Error::ValidationErrors {
            message: format!(
                "An error occurred while validating the {}",
                Self::error_identifier(),
            ),
            errors: err,
        })?;

        Ok(query
            .table(Self::table())
            .and_where(Expr::col(<Self as Identity>::Iden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder))
    }

    fn query_delete(&self) -> String {
        sea_query::Query::delete()
            .from_table(Self::table())
            .and_where(Expr::col(<Self as Identity>::Iden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder)
    }
}

impl From<Row> for User {
    fn from(row: Row) -> Self {
        let connections = row
            .try_get::<_, serde_json::Value>(JoinKeys::Connection.plural().as_str())
            .ok();
        let refresh_token_trees = row
            .try_get::<_, serde_json::Value>(JoinKeys::RefreshTokenTree.plural().as_str())
            .ok();

        Self {
            id: row.get(<Self as Identity>::Iden::Id.to_string().as_str()),
            first_name: row.get(<Self as Identity>::Iden::FirstName.to_string().as_str()),
            last_name: row.get(<Self as Identity>::Iden::LastName.to_string().as_str()),
            email: row.get(<Self as Identity>::Iden::Email.to_string().as_str()),
            username: row.get(<Self as Identity>::Iden::Username.to_string().as_str()),
            password: row.get(<Self as Identity>::Iden::Password.to_string().as_str()),
            verified: row.get(<Self as Identity>::Iden::Verified.to_string().as_str()),
            banned: row.get(<Self as Identity>::Iden::Banned.to_string().as_str()),
            mfa_secret: row.get(<Self as Identity>::Iden::MfaSecret.to_string().as_str()),
            mfa_backup_codes: row.get(
                <Self as Identity>::Iden::MfaBackupCodes
                    .to_string()
                    .as_str(),
            ),
            created_at: row.get(<Self as Identity>::Iden::CreatedAt.to_string().as_str()),
            updated_at: row.get(<Self as Identity>::Iden::UpdatedAt.to_string().as_str()),

            connections: match connections {
                Some(connections) => serde_json::from_value(connections).ok(),
                None => None,
            },
            refresh_token_trees: match refresh_token_trees {
                Some(refresh_token_trees) => serde_json::from_value(refresh_token_trees).ok(),
                None => None,
            },
        }
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
