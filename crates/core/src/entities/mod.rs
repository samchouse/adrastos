// TODO(@Xenfo): use `*Iden::Table` instead of Alias::new() once https://github.com/SeaQL/sea-query/issues/533 is fixed

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use deadpool_postgres::tokio_postgres::Row;
use sea_query::{
    enum_def, Alias, ColumnDef, ColumnType, Expr, ForeignKey, ForeignKeyAction, Keyword,
    PostgresQueryBuilder, Query, SimpleExpr, Table,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::{auth, util};

pub mod migrations;

pub trait Queries {
    type Iden;

    fn table() -> Alias;

    fn query_select(expressions: Vec<SimpleExpr>) -> String;
    fn query_insert(&self) -> Result<String, Option<ValidationErrors>>;
    fn query_update(
        &self,
        updated: HashMap<String, Value>,
    ) -> Result<String, Option<ValidationErrors>>;
    fn query_delete(expression: SimpleExpr) -> String;
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
            Self::Table => "refresh_token_trees".to_string(),
            Self::Id => "id".to_string(),
            Self::UserId => "user_id".to_string(),
            Self::InactiveAt => "inactive_at".to_string(),
            Self::ExpiresAt => "expires_at".to_string(),
            Self::Tokens => "tokens".to_string(),
            Self::CreatedAt => "created_at".to_string(),
            Self::UpdatedAt => "updated_at".to_string(),
        }
    }
}

impl RefreshTokenTree {
    fn migrate() -> String {
        Table::create()
            .table(Self::table())
            .if_not_exists()
            .col(
                ColumnDef::new(<RefreshTokenTree as Queries>::Iden::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(<RefreshTokenTree as Queries>::Iden::UserId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<RefreshTokenTree as Queries>::Iden::InactiveAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<RefreshTokenTree as Queries>::Iden::ExpiresAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<RefreshTokenTree as Queries>::Iden::Tokens)
                    .array(ColumnType::String(None))
                    .not_null(),
            )
            .col(
                ColumnDef::new(<RefreshTokenTree as Queries>::Iden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(
                ColumnDef::new(<RefreshTokenTree as Queries>::Iden::UpdatedAt)
                    .timestamp_with_time_zone(),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("FK_refresh_token_tree_user_id")
                    .from(
                        RefreshTokenTree::table(),
                        <RefreshTokenTree as Queries>::Iden::UserId,
                    )
                    .to(
                        User::table(),
                        <User as Queries>::Iden::Id,
                    )
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_string(PostgresQueryBuilder)
    }
}

impl Queries for RefreshTokenTree {
    type Iden = RefreshTokenTreeIden;

    fn table() -> Alias {
        Alias::new(Self::Iden::Table.to_string().as_str())
    }

    fn query_select(expressions: Vec<SimpleExpr>) -> String {
        let mut query = Query::select();

        for expression in expressions {
            query.and_where(expression);
        }

        query
            .from(Self::table())
            .columns([
                Self::Iden::Id,
                Self::Iden::UserId,
                Self::Iden::InactiveAt,
                Self::Iden::ExpiresAt,
                Self::Iden::Tokens,
                Self::Iden::CreatedAt,
                Self::Iden::UpdatedAt,
            ])
            .limit(1)
            .to_string(PostgresQueryBuilder)
    }

    fn query_insert(&self) -> Result<String, Option<ValidationErrors>> {
        Ok(Query::insert()
            .into_table(Self::table())
            .columns([
                Self::Iden::Id,
                Self::Iden::UserId,
                Self::Iden::InactiveAt,
                Self::Iden::ExpiresAt,
                Self::Iden::Tokens,
                Self::Iden::CreatedAt,
            ])
            .values_panic([
                self.id.clone().into(),
                self.user_id.clone().into(),
                self.inactive_at.clone().into(),
                self.expires_at.clone().into(),
                self.tokens.clone().into(),
                self.created_at.clone().into(),
            ])
            .to_string(PostgresQueryBuilder))
    }

    fn query_update(
        &self,
        updated: HashMap<String, Value>,
    ) -> Result<String, Option<ValidationErrors>> {
        let mut errors = ValidationErrors::new();
        let Some(tokens) = updated.get(Self::Iden::Tokens.to_string().as_str()) else {
            errors.add(util::string_to_static_str(Self::Iden::Tokens.to_string()), ValidationError::new("required"));
            return Err(Some(errors));
        };
        let Some(tokens) = tokens.as_array() else {
            errors.add(util::string_to_static_str(Self::Iden::Tokens.to_string()), ValidationError::new("invalid_type"));
            return Err(Some(errors));
        };

        let tokens = tokens
            .iter()
            .map(|token| {
                if let Some(token) = token.as_str() {
                    Some(token.to_string())
                } else {
                    None
                }
            })
            .filter(|token| token.is_some())
            .map(|token| token.unwrap())
            .collect::<Vec<String>>();

        Ok(Query::update()
            .table(Self::table())
            .values([
                (
                    Self::Iden::InactiveAt,
                    (Utc::now() + Duration::days(15)).into(),
                ),
                (Self::Iden::Tokens, tokens.into()),
                (Self::Iden::UpdatedAt, Utc::now().into()),
            ])
            .and_where(Expr::col(Self::Iden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder))
    }

    fn query_delete(expression: SimpleExpr) -> String {
        Query::delete()
            .from_table(Self::table())
            .and_where(expression)
            .to_string(PostgresQueryBuilder)
    }
}

impl From<&Row> for RefreshTokenTree {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(<RefreshTokenTree as Queries>::Iden::Id.to_string().as_str()),
            user_id: row.get(
                <RefreshTokenTree as Queries>::Iden::UserId
                    .to_string()
                    .as_str(),
            ),
            inactive_at: row.get(
                <RefreshTokenTree as Queries>::Iden::InactiveAt
                    .to_string()
                    .as_str(),
            ),
            expires_at: row.get(
                <RefreshTokenTree as Queries>::Iden::ExpiresAt
                    .to_string()
                    .as_str(),
            ),
            tokens: row.get(
                <RefreshTokenTree as Queries>::Iden::Tokens
                    .to_string()
                    .as_str(),
            ),
            created_at: row.get(
                <RefreshTokenTree as Queries>::Iden::CreatedAt
                    .to_string()
                    .as_str(),
            ),
            updated_at: row.get(
                <RefreshTokenTree as Queries>::Iden::UpdatedAt
                    .to_string()
                    .as_str(),
            ),
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
            Self::Table => "connections".to_string(),
            Self::Id => "id".to_string(),
            Self::Provider => "provider".to_string(),
            Self::UserId => "user_id".to_string(),
            Self::ProviderId => "provider_id".to_string(),
            Self::CreatedAt => "created_at".to_string(),
            Self::UpdatedAt => "updated_at".to_string(),
        }
    }
}

impl Connection {
    fn migrate() -> String {
        Table::create()
            .table(Self::table())
            .if_not_exists()
            .col(
                ColumnDef::new(<Connection as Queries>::Iden::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(<Connection as Queries>::Iden::UserId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Connection as Queries>::Iden::Provider)
                    .text()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Connection as Queries>::Iden::ProviderId)
                    .text()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Connection as Queries>::Iden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(
                ColumnDef::new(<Connection as Queries>::Iden::UpdatedAt).timestamp_with_time_zone(),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("FK_connection_user_id")
                    .from(Connection::table(), <Connection as Queries>::Iden::UserId)
                    .to(User::table(), <User as Queries>::Iden::Id)
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_string(PostgresQueryBuilder)
    }
}

impl Queries for Connection {
    type Iden = ConnectionIden;

    fn table() -> Alias {
        Alias::new(Self::Iden::Table.to_string().as_str())
    }

    fn query_select(expressions: Vec<SimpleExpr>) -> String {
        let mut query = Query::select();

        for expression in expressions {
            query.and_where(expression);
        }

        query
            .from(Self::table())
            .columns(vec![
                Self::Iden::Id,
                Self::Iden::Provider,
                Self::Iden::UserId,
                Self::Iden::ProviderId,
                Self::Iden::CreatedAt,
                Self::Iden::UpdatedAt,
            ])
            .limit(1)
            .to_string(PostgresQueryBuilder)
    }

    fn query_insert(&self) -> Result<String, Option<ValidationErrors>> {
        Ok(Query::insert()
            .into_table(Self::table())
            .columns(vec![
                Self::Iden::Id,
                Self::Iden::Provider,
                Self::Iden::UserId,
                Self::Iden::ProviderId,
                Self::Iden::CreatedAt,
                Self::Iden::UpdatedAt,
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

    fn query_update(&self, _: HashMap<String, Value>) -> Result<String, Option<ValidationErrors>> {
        Err(None)
    }

    fn query_delete(expression: SimpleExpr) -> String {
        Query::delete()
            .from_table(Self::table())
            .and_where(expression)
            .to_string(PostgresQueryBuilder)
    }
}

impl From<&Row> for Connection {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(<Connection as Queries>::Iden::Id.to_string().as_str()),
            provider: row.get(<Connection as Queries>::Iden::Provider.to_string().as_str()),
            user_id: row.get(<Connection as Queries>::Iden::UserId.to_string().as_str()),
            provider_id: row.get(
                <Connection as Queries>::Iden::ProviderId
                    .to_string()
                    .as_str(),
            ),
            created_at: row.get(
                <Connection as Queries>::Iden::CreatedAt
                    .to_string()
                    .as_str(),
            ),
            updated_at: row.get(
                <Connection as Queries>::Iden::UpdatedAt
                    .to_string()
                    .as_str(),
            ),
        }
    }
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
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl ToString for UserIden {
    fn to_string(&self) -> String {
        match self {
            Self::Table => "users".to_string(),
            Self::Id => "id".to_string(),
            Self::FirstName => "first_name".to_string(),
            Self::LastName => "last_name".to_string(),
            Self::Email => "email".to_string(),
            Self::Username => "username".to_string(),
            Self::Password => "password".to_string(),
            Self::Verified => "verified".to_string(),
            Self::Banned => "banned".to_string(),
            Self::CreatedAt => "created_at".to_string(),
            Self::UpdatedAt => "updated_at".to_string(),
        }
    }
}

impl User {
    fn migrate() -> String {
        Table::create()
            .table(Self::table())
            .if_not_exists()
            .col(
                ColumnDef::new(<User as Queries>::Iden::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(<User as Queries>::Iden::FirstName)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<User as Queries>::Iden::LastName)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<User as Queries>::Iden::Email)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(<User as Queries>::Iden::Username)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(<User as Queries>::Iden::Password)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<User as Queries>::Iden::Verified)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(<User as Queries>::Iden::Banned)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(<User as Queries>::Iden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(ColumnDef::new(<User as Queries>::Iden::UpdatedAt).timestamp_with_time_zone())
            .to_string(PostgresQueryBuilder)
    }
}

impl Queries for User {
    type Iden = UserIden;

    fn table() -> Alias {
        Alias::new(Self::Iden::Table.to_string().as_str())
    }

    fn query_select(expressions: Vec<SimpleExpr>) -> String {
        let mut query = Query::select();

        for expression in expressions {
            query.and_where(expression);
        }

        query
            .from(Self::table())
            .columns([
                Self::Iden::Id,
                Self::Iden::FirstName,
                Self::Iden::LastName,
                Self::Iden::Email,
                Self::Iden::Username,
                Self::Iden::Password,
                Self::Iden::Verified,
                Self::Iden::Banned,
                Self::Iden::CreatedAt,
                Self::Iden::UpdatedAt,
            ])
            .limit(1)
            .to_string(PostgresQueryBuilder)
    }

    fn query_insert(&self) -> Result<String, Option<ValidationErrors>> {
        self.validate()?;

        let Ok(hashed_password) = auth::hash_password(self.password.as_str()) else {
            return Err(None);
        };

        Ok(Query::insert()
            .into_table(Self::table())
            .columns([
                Self::Iden::Id,
                Self::Iden::FirstName,
                Self::Iden::LastName,
                Self::Iden::Email,
                Self::Iden::Username,
                Self::Iden::Password,
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

    fn query_update(
        &self,
        updated: HashMap<String, Value>,
    ) -> Result<String, Option<ValidationErrors>> {
        let mut updated_for_validation = self.clone();
        let mut query = Query::update();

        if let Some(first_name) = updated.get(Self::Iden::FirstName.to_string().as_str()) {
            if let Some(first_name) = first_name.as_str() {
                updated_for_validation.first_name = first_name.clone().to_string();
                query.values([(Self::Iden::FirstName, first_name.clone().into())]);
            }
        }
        if let Some(last_name) = updated.get(Self::Iden::LastName.to_string().as_str()) {
            if let Some(last_name) = last_name.as_str() {
                updated_for_validation.last_name = last_name.clone().to_string();
                query.values([(Self::Iden::LastName, last_name.clone().into())]);
            }
        }
        if let Some(email) = updated.get(Self::Iden::Email.to_string().as_str()) {
            if let Some(email) = email.as_str() {
                updated_for_validation.email = email.clone().to_string();
                query.values([(Self::Iden::Email, email.clone().into())]);
            }
        }
        if let Some(username) = updated.get(Self::Iden::Username.to_string().as_str()) {
            if let Some(username) = username.as_str() {
                updated_for_validation.username = username.clone().to_string();
                query.values([(Self::Iden::Username, username.clone().into())]);
            }
        }
        if let Some(password) = updated.get(Self::Iden::Password.to_string().as_str()) {
            if let Some(password) = password.as_str() {
                updated_for_validation.password = password.clone().to_string();
                query.values([(Self::Iden::Password, password.clone().into())]);
            }
        }

        updated_for_validation.validate()?;

        Ok(query
            .table(Self::table())
            .and_where(Expr::col(Self::Iden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder))
    }

    fn query_delete(expression: SimpleExpr) -> String {
        Query::delete()
            .from_table(Self::table())
            .and_where(expression)
            .to_string(PostgresQueryBuilder)
    }
}

impl From<&Row> for User {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(<User as Queries>::Iden::Id.to_string().as_str()),
            first_name: row.get(<User as Queries>::Iden::FirstName.to_string().as_str()),
            last_name: row.get(<User as Queries>::Iden::LastName.to_string().as_str()),
            email: row.get(<User as Queries>::Iden::Email.to_string().as_str()),
            username: row.get(<User as Queries>::Iden::Username.to_string().as_str()),
            password: row.get(<User as Queries>::Iden::Password.to_string().as_str()),
            verified: row.get(<User as Queries>::Iden::Verified.to_string().as_str()),
            banned: row.get(<User as Queries>::Iden::Banned.to_string().as_str()),
            created_at: row.get(<User as Queries>::Iden::CreatedAt.to_string().as_str()),
            updated_at: row.get(<User as Queries>::Iden::UpdatedAt.to_string().as_str()),
        }
    }
}
