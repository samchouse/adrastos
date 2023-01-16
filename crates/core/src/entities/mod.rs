// TODO(@Xenfo): use `*Iden::Table` instead of Alias::new() once https://github.com/SeaQL/sea-query/issues/533 is fixed

use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use sea_query::{
    enum_def, Alias, ColumnDef, ColumnType, ForeignKey, ForeignKeyAction, Keyword,
    PostgresQueryBuilder, Query, SelectStatement, Table,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationErrors};

use crate::auth;

pub mod migrations;

pub trait Queries {
    fn query_select(query_builder: &mut SelectStatement) -> String;
    fn query_insert(&self) -> Result<String, Option<ValidationErrors>>;
}

#[enum_def]
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenTree {
    pub id: String,
    pub user_id: String,
    pub inactive_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub tokens: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl ToString for RefreshTokenTreeIden {
    fn to_string(&self) -> String {
        match self {
            RefreshTokenTreeIden::Table => "refresh_token_trees".to_string(),
            RefreshTokenTreeIden::Id => "id".to_string(),
            RefreshTokenTreeIden::UserId => "user_id".to_string(),
            RefreshTokenTreeIden::InactiveAt => "inactive_at".to_string(),
            RefreshTokenTreeIden::ExpiresAt => "expires_at".to_string(),
            RefreshTokenTreeIden::Tokens => "tokens".to_string(),
            RefreshTokenTreeIden::CreatedAt => "created_at".to_string(),
            RefreshTokenTreeIden::UpdatedAt => "updated_at".to_string(),
        }
    }
}

impl RefreshTokenTree {
    fn migrate() -> String {
        Table::create()
            .table(Alias::new(RefreshTokenTreeIden::Table.to_string().as_str()))
            .if_not_exists()
            .col(
                ColumnDef::new(RefreshTokenTreeIden::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(RefreshTokenTreeIden::UserId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(RefreshTokenTreeIden::InactiveAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(RefreshTokenTreeIden::ExpiresAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(RefreshTokenTreeIden::Tokens)
                    .array(ColumnType::String(None))
                    .not_null(),
            )
            .col(
                ColumnDef::new(RefreshTokenTreeIden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(ColumnDef::new(RefreshTokenTreeIden::UpdatedAt).timestamp_with_time_zone())
            .foreign_key(
                ForeignKey::create()
                    .name("FK_refresh_token_tree_user_id")
                    .from(
                        Alias::new(RefreshTokenTreeIden::Table.to_string().as_str()),
                        RefreshTokenTreeIden::UserId,
                    )
                    .to(
                        Alias::new(UserIden::Table.to_string().as_str()),
                        UserIden::Id,
                    )
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_string(PostgresQueryBuilder)
    }
}

impl Queries for RefreshTokenTree {
    fn query_select(query_builder: &mut SelectStatement) -> String {
        query_builder
            .from(Alias::new(RefreshTokenTreeIden::Table.to_string().as_str()))
            .columns(vec![
                RefreshTokenTreeIden::Id,
                RefreshTokenTreeIden::UserId,
                RefreshTokenTreeIden::InactiveAt,
                RefreshTokenTreeIden::ExpiresAt,
                RefreshTokenTreeIden::Tokens,
                RefreshTokenTreeIden::CreatedAt,
                RefreshTokenTreeIden::UpdatedAt,
            ])
            .to_string(PostgresQueryBuilder)
    }

    fn query_insert(&self) -> Result<String, Option<ValidationErrors>> {
        Ok(Query::insert()
            .into_table(Alias::new(RefreshTokenTreeIden::Table.to_string().as_str()))
            .columns(vec![
                RefreshTokenTreeIden::Id,
                RefreshTokenTreeIden::UserId,
                RefreshTokenTreeIden::InactiveAt,
                RefreshTokenTreeIden::ExpiresAt,
                RefreshTokenTreeIden::Tokens,
                RefreshTokenTreeIden::CreatedAt,
            ])
            .values_panic(vec![
                self.id.clone().into(),
                self.user_id.clone().into(),
                self.inactive_at.clone().into(),
                self.expires_at.clone().into(),
                self.tokens.clone().into(),
                self.created_at.clone().into(),
            ])
            .to_string(PostgresQueryBuilder)
            .replace("'{'", "'{")
            .replace("'}'", "}'"))
    }
}

impl From<&Row> for RefreshTokenTree {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(RefreshTokenTreeIden::Id.to_string().as_str()),
            user_id: row.get(RefreshTokenTreeIden::UserId.to_string().as_str()),
            inactive_at: row.get(RefreshTokenTreeIden::InactiveAt.to_string().as_str()),
            expires_at: row.get(RefreshTokenTreeIden::ExpiresAt.to_string().as_str()),
            tokens: row.get(RefreshTokenTreeIden::Tokens.to_string().as_str()),
            created_at: row.get(RefreshTokenTreeIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(RefreshTokenTreeIden::UpdatedAt.to_string().as_str()),
        }
    }
}

#[enum_def]
#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub provider: String,
    pub user_id: String,
    pub provider_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl ToString for ConnectionIden {
    fn to_string(&self) -> String {
        match self {
            ConnectionIden::Table => "connections".to_string(),
            ConnectionIden::Id => "id".to_string(),
            ConnectionIden::Provider => "provider".to_string(),
            ConnectionIden::UserId => "user_id".to_string(),
            ConnectionIden::ProviderId => "provider_id".to_string(),
            ConnectionIden::CreatedAt => "created_at".to_string(),
            ConnectionIden::UpdatedAt => "updated_at".to_string(),
        }
    }
}

impl Connection {
    fn migrate() -> String {
        Table::create()
            .table(Alias::new(ConnectionIden::Table.to_string().as_str()))
            .if_not_exists()
            .col(
                ColumnDef::new(ConnectionIden::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(ConnectionIden::UserId).string().not_null())
            .col(ColumnDef::new(ConnectionIden::Provider).text().not_null())
            .col(ColumnDef::new(ConnectionIden::ProviderId).text().not_null())
            .col(
                ColumnDef::new(ConnectionIden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(ColumnDef::new(ConnectionIden::UpdatedAt).timestamp_with_time_zone())
            .foreign_key(
                ForeignKey::create()
                    .name("FK_connection_user_id")
                    .from(
                        Alias::new(ConnectionIden::Table.to_string().as_str()),
                        ConnectionIden::UserId,
                    )
                    .to(
                        Alias::new(UserIden::Table.to_string().as_str()),
                        UserIden::Id,
                    )
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_string(PostgresQueryBuilder)
    }
}

impl Queries for Connection {
    fn query_select(query_builder: &mut SelectStatement) -> String {
        query_builder
            .from(Alias::new(ConnectionIden::Table.to_string().as_str()))
            .columns(vec![
                ConnectionIden::Id,
                ConnectionIden::Provider,
                ConnectionIden::UserId,
                ConnectionIden::ProviderId,
                ConnectionIden::CreatedAt,
                ConnectionIden::UpdatedAt,
            ])
            .to_string(PostgresQueryBuilder)
    }

    fn query_insert(&self) -> Result<String, Option<ValidationErrors>> {
        Ok(Query::insert()
            .into_table(Alias::new(UserIden::Table.to_string().as_str()))
            .columns(vec![
                ConnectionIden::Id,
                ConnectionIden::Provider,
                ConnectionIden::UserId,
                ConnectionIden::ProviderId,
                ConnectionIden::CreatedAt,
                ConnectionIden::UpdatedAt,
            ])
            .values_panic(vec![
                self.id.clone().into(),
                self.provider.clone().into(),
                self.user_id.clone().into(),
                self.provider_id.clone().into(),
                self.created_at.clone().into(),
                self.updated_at.clone().into(),
            ])
            .to_string(PostgresQueryBuilder))
    }
}

impl From<&Row> for Connection {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(ConnectionIden::Id.to_string().as_str()),
            provider: row.get(ConnectionIden::Provider.to_string().as_str()),
            user_id: row.get(ConnectionIden::UserId.to_string().as_str()),
            provider_id: row.get(ConnectionIden::ProviderId.to_string().as_str()),
            created_at: row.get(ConnectionIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(ConnectionIden::UpdatedAt.to_string().as_str()),
        }
    }
}

#[enum_def]
#[derive(Debug, Validate, Serialize, Deserialize, Clone, ToSchema)]
pub struct User {
    pub id: String,
    #[serde(rename = "firstName")]
    #[validate(length(max = 50))]
    pub first_name: String,
    #[serde(rename = "lastName")]
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
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl ToString for UserIden {
    fn to_string(&self) -> String {
        match self {
            UserIden::Table => "users".to_string(),
            UserIden::Id => "id".to_string(),
            UserIden::FirstName => "first_name".to_string(),
            UserIden::LastName => "last_name".to_string(),
            UserIden::Email => "email".to_string(),
            UserIden::Username => "username".to_string(),
            UserIden::Password => "password".to_string(),
            UserIden::Verified => "verified".to_string(),
            UserIden::Banned => "banned".to_string(),
            UserIden::CreatedAt => "created_at".to_string(),
            UserIden::UpdatedAt => "updated_at".to_string(),
        }
    }
}

impl User {
    fn migrate() -> String {
        Table::create()
            .table(Alias::new(UserIden::Table.to_string().as_str()))
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

impl Queries for User {
    fn query_select(query_builder: &mut SelectStatement) -> String {
        query_builder
            .from(Alias::new(UserIden::Table.to_string().as_str()))
            .columns(vec![
                UserIden::Id,
                UserIden::FirstName,
                UserIden::LastName,
                UserIden::Email,
                UserIden::Username,
                UserIden::Password,
                UserIden::Verified,
                UserIden::Banned,
                UserIden::CreatedAt,
                UserIden::UpdatedAt,
            ])
            .to_string(PostgresQueryBuilder)
    }

    fn query_insert(&self) -> Result<String, Option<ValidationErrors>> {
        self.validate()?;

        let Ok(hashed_password) = auth::hash_password(self.password.as_str()) else {
            return Err(None);
        };

        Ok(Query::insert()
            .into_table(Alias::new(UserIden::Table.to_string().as_str()))
            .columns(vec![
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
                hashed_password.clone().into(),
            ])
            .to_string(PostgresQueryBuilder))
    }
}

impl From<&Row> for User {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(UserIden::Id.to_string().as_str()),
            first_name: row.get(UserIden::FirstName.to_string().as_str()),
            last_name: row.get(UserIden::LastName.to_string().as_str()),
            email: row.get(UserIden::Email.to_string().as_str()),
            username: row.get(UserIden::Username.to_string().as_str()),
            password: row.get(UserIden::Password.to_string().as_str()),
            verified: row.get(UserIden::Verified.to_string().as_str()),
            banned: row.get(UserIden::Banned.to_string().as_str()),
            created_at: row.get(UserIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(UserIden::UpdatedAt.to_string().as_str()),
        }
    }
}
