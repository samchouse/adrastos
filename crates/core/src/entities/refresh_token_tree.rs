use std::{collections::HashMap, fmt};

use chrono::{DateTime, Duration, Utc};
use deadpool_postgres::tokio_postgres::Row;
use sea_query::{
    enum_def, Alias, ColumnDef, ColumnType, Expr, ForeignKey, ForeignKeyAction, Keyword,
    PostgresQueryBuilder, SelectStatement, SimpleExpr, Table,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::{ValidationError, ValidationErrors};

use crate::{handlers::Error, util};

use super::{Identity, Migrate, Query, User};

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshTokenTree {
    pub id: String,
    pub user_id: String,
    pub inactive_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub tokens: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Identity for RefreshTokenTree {
    type Iden = RefreshTokenTreeIden;

    fn table() -> Alias {
        Alias::new(&Self::Iden::Table.to_string())
    }

    fn error_identifier() -> String {
        "refresh token tree".into()
    }
}

impl Migrate for RefreshTokenTree {
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
                ColumnDef::new(<Self as Identity>::Iden::UserId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::InactiveAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::ExpiresAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::Tokens)
                    .array(ColumnType::String(None))
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(ColumnDef::new(<Self as Identity>::Iden::UpdatedAt).timestamp_with_time_zone())
            .foreign_key(
                ForeignKey::create()
                    .name("FK_refresh_token_tree_user_id")
                    .from(RefreshTokenTree::table(), <Self as Identity>::Iden::UserId)
                    .to(User::table(), <User as Identity>::Iden::Id)
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_string(PostgresQueryBuilder)
    }
}

impl Query for RefreshTokenTree {
    fn query_select(expressions: Vec<SimpleExpr>) -> SelectStatement {
        let mut query = sea_query::Query::select();

        for expression in expressions {
            query.and_where(expression);
        }

        query
            .from(Self::table())
            .columns([
                <Self as Identity>::Iden::Id,
                <Self as Identity>::Iden::UserId,
                <Self as Identity>::Iden::InactiveAt,
                <Self as Identity>::Iden::ExpiresAt,
                <Self as Identity>::Iden::Tokens,
                <Self as Identity>::Iden::CreatedAt,
                <Self as Identity>::Iden::UpdatedAt,
            ])
            .to_owned()
    }

    fn query_insert(&self) -> Result<String, Error> {
        Ok(sea_query::Query::insert()
            .into_table(Self::table())
            .columns([
                <Self as Identity>::Iden::Id,
                <Self as Identity>::Iden::UserId,
                <Self as Identity>::Iden::InactiveAt,
                <Self as Identity>::Iden::ExpiresAt,
                <Self as Identity>::Iden::Tokens,
                <Self as Identity>::Iden::CreatedAt,
            ])
            .values_panic([
                self.id.clone().into(),
                self.user_id.clone().into(),
                self.inactive_at.into(),
                self.expires_at.into(),
                self.tokens.clone().into(),
                self.created_at.into(),
            ])
            .to_string(PostgresQueryBuilder))
    }

    fn query_update(&self, updated: HashMap<String, Value>) -> Result<String, Error> {
        let mut errors = ValidationErrors::new();
        let Some(tokens) = updated.get(<Self as Identity>::Iden::Tokens.to_string().as_str()) else {
            errors.add(util::string_to_static_str(<Self as Identity>::Iden::Tokens.to_string()), ValidationError::new("required"));
            return Err(Error::ValidationErrors { message: format!(
                "An error occurred while validating the {}",
                Self::error_identifier(),
            ), errors });
        };
        let Some(tokens) = tokens.as_array() else {
            errors.add(util::string_to_static_str(<Self as Identity>::Iden::Tokens.to_string()), ValidationError::new("invalid_type"));
            return Err(Error::ValidationErrors { message: format!(
                "An error occurred while validating the {}",
                Self::error_identifier(),
            ), errors });
        };

        let tokens = tokens
            .iter()
            .filter_map(|token| token.as_str().map(|token| token.to_string()))
            .collect::<Vec<String>>();

        Ok(sea_query::Query::update()
            .table(Self::table())
            .values([
                (
                    <Self as Identity>::Iden::InactiveAt,
                    (Utc::now() + Duration::days(15)).into(),
                ),
                (<Self as Identity>::Iden::Tokens, tokens.into()),
                (<Self as Identity>::Iden::UpdatedAt, Utc::now().into()),
            ])
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

impl From<Row> for RefreshTokenTree {
    fn from(row: Row) -> Self {
        Self {
            id: row.get(<Self as Identity>::Iden::Id.to_string().as_str()),
            user_id: row.get(<Self as Identity>::Iden::UserId.to_string().as_str()),
            inactive_at: row.get(<Self as Identity>::Iden::InactiveAt.to_string().as_str()),
            expires_at: row.get(<Self as Identity>::Iden::ExpiresAt.to_string().as_str()),
            tokens: row.get(<Self as Identity>::Iden::Tokens.to_string().as_str()),
            created_at: row.get(<Self as Identity>::Iden::CreatedAt.to_string().as_str()),
            updated_at: row.get(<Self as Identity>::Iden::UpdatedAt.to_string().as_str()),
        }
    }
}

impl fmt::Display for RefreshTokenTreeIden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Table => "refresh_token_trees",
            Self::Id => "id",
            Self::UserId => "user_id",
            Self::InactiveAt => "inactive_at",
            Self::ExpiresAt => "expires_at",
            Self::Tokens => "tokens",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        };

        write!(f, "{name}")
    }
}
