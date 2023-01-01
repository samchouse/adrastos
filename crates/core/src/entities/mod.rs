use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use sea_query::{
    enum_def, Alias, ColumnDef, Keyword, PostgresQueryBuilder, Query, SelectStatement, Table,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::{self, oauth2::OAuth2Provider};

pub mod migrations;

pub trait Queries {
    fn query_select(query_builder: &mut SelectStatement) -> String;
    fn query_insert(&self) -> Result<String, ()>;
}

#[enum_def]
pub struct Connection {
    pub id: String,
    pub provider: OAuth2Provider,
    pub user_id: String,
    pub provider_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[enum_def]
#[derive(Debug, Validate, Serialize, Deserialize)]
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
    pub created_at: DateTime<Utc>,
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
            .table(Alias::new(UserIden::Table.to_string().as_str())) // TODO(@Xenfo): use `UserIden::Table` once https://github.com/SeaQL/sea-query/issues/533 is fixed
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
            .to_string(PostgresQueryBuilder)
    }

    fn query_insert(&self) -> Result<String, ()> {
        self.validate().map_err(|_| ())?;

        let Ok(hashed_password) = auth::hash_password(self.password.as_str()) else {
            return Err(());
        };

        Ok(Query::insert()
            .into_table(Alias::new(UserIden::Table.to_string().as_str())) // TODO(@Xenfo): use `UserIden::Table` once https://github.com/SeaQL/sea-query/issues/533 is fixed
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
