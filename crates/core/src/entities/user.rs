use std::{collections::HashMap, fmt};

use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use sea_query::{
    enum_def, Alias, ColumnDef, Expr, Keyword, PostgresQueryBuilder, Query as SeaQLQuery,
    SimpleExpr, Table,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use validator::Validate;

use crate::{auth, handlers::Error};

use super::{Identity, Migrate, Query};

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
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Identity for User {
    type Iden = UserIden;

    fn table() -> Alias {
        Alias::new(<Self as Identity>::Iden::Table.to_string().as_str())
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

impl Query for User {
    fn query_select(expressions: Vec<SimpleExpr>) -> String {
        let mut query = SeaQLQuery::select();

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
                <Self as Identity>::Iden::CreatedAt,
                <Self as Identity>::Iden::UpdatedAt,
            ])
            .limit(1)
            .to_string(PostgresQueryBuilder)
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
            Error::InternalServerError {
                error: format!("An error occurred while hashing the password for the {err}"),
            }
        })?;

        Ok(SeaQLQuery::insert()
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

    fn query_update(&self, updated: HashMap<String, Value>) -> Result<String, Error> {
        let mut updated_for_validation = self.clone();
        let mut query = SeaQLQuery::update();

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

        updated_for_validation
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
        SeaQLQuery::delete()
            .from_table(Self::table())
            .and_where(Expr::col(<Self as Identity>::Iden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder)
    }
}

impl From<Row> for User {
    fn from(row: Row) -> Self {
        Self {
            id: row.get(<Self as Identity>::Iden::Id.to_string().as_str()),
            first_name: row.get(<Self as Identity>::Iden::FirstName.to_string().as_str()),
            last_name: row.get(<Self as Identity>::Iden::LastName.to_string().as_str()),
            email: row.get(<Self as Identity>::Iden::Email.to_string().as_str()),
            username: row.get(<Self as Identity>::Iden::Username.to_string().as_str()),
            password: row.get(<Self as Identity>::Iden::Password.to_string().as_str()),
            verified: row.get(<Self as Identity>::Iden::Verified.to_string().as_str()),
            banned: row.get(<Self as Identity>::Iden::Banned.to_string().as_str()),
            created_at: row.get(<Self as Identity>::Iden::CreatedAt.to_string().as_str()),
            updated_at: row.get(<Self as Identity>::Iden::UpdatedAt.to_string().as_str()),
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
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        };

        write!(f, "{name}")
    }
}
