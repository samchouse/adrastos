use std::{collections::HashMap, fmt};

use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use sea_query::{
    enum_def, Alias, ColumnDef, Expr, ForeignKey, ForeignKeyAction, Keyword, PostgresQueryBuilder,
    Query as SeaQLQuery, SimpleExpr, Table,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::handlers::Error;

use super::{Identity, Migrate, Query, User};

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

impl Identity for Connection {
    type Iden = ConnectionIden;

    fn table() -> Alias {
        Alias::new(<Self as Identity>::Iden::Table.to_string().as_str())
    }

    fn error_identifier() -> String {
        "connection".into()
    }
}

impl Migrate for Connection {
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
                ColumnDef::new(<Self as Identity>::Iden::Provider)
                    .text()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::ProviderId)
                    .text()
                    .not_null(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::UpdatedAt)
                    .timestamp_with_time_zone(),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("FK_connection_user_id")
                    .from(Connection::table(), <Self as Identity>::Iden::UserId)
                    .to(User::table(), <User as Identity>::Iden::Id)
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_string(PostgresQueryBuilder)
    }
}

impl Query for Connection {
    fn query_select(expressions: Vec<SimpleExpr>) -> String {
        let mut query = SeaQLQuery::select();

        for expression in expressions {
            query.and_where(expression);
        }

        query
            .from(Self::table())
            .columns(vec![
                <Self as Identity>::Iden::Id,
                <Self as Identity>::Iden::Provider,
                <Self as Identity>::Iden::UserId,
                <Self as Identity>::Iden::ProviderId,
                <Self as Identity>::Iden::CreatedAt,
                <Self as Identity>::Iden::UpdatedAt,
            ])
            .limit(1)
            .to_string(PostgresQueryBuilder)
    }

    fn query_insert(&self) -> Result<String, Error> {
        Ok(SeaQLQuery::insert()
            .into_table(Self::table())
            .columns(vec![
                <Self as Identity>::Iden::Id,
                <Self as Identity>::Iden::Provider,
                <Self as Identity>::Iden::UserId,
                <Self as Identity>::Iden::ProviderId,
                <Self as Identity>::Iden::CreatedAt,
                <Self as Identity>::Iden::UpdatedAt,
            ])
            .values_panic(vec![
                self.id.clone().into(),
                self.provider.clone().into(),
                self.user_id.clone().into(),
                self.provider_id.clone().into(),
                self.created_at.into(),
                self.updated_at.into(),
            ])
            .to_string(PostgresQueryBuilder))
    }

    fn query_update(&self, _: HashMap<String, Value>) -> Result<String, Error> {
        todo!()
    }

    fn query_delete(&self) -> String {
        SeaQLQuery::delete()
            .from_table(Self::table())
            .and_where(Expr::col(<Self as Identity>::Iden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder)
    }
}

impl From<Row> for Connection {
    fn from(row: Row) -> Self {
        Self {
            id: row.get(<Self as Identity>::Iden::Id.to_string().as_str()),
            provider: row.get(
                <Self as Identity>::Iden::Provider
                    .to_string()
                    .as_str(),
            ),
            user_id: row.get(<Self as Identity>::Iden::UserId.to_string().as_str()),
            provider_id: row.get(
                <Self as Identity>::Iden::ProviderId
                    .to_string()
                    .as_str(),
            ),
            created_at: row.get(
                <Self as Identity>::Iden::CreatedAt
                    .to_string()
                    .as_str(),
            ),
            updated_at: row.get(
                <Self as Identity>::Iden::UpdatedAt
                    .to_string()
                    .as_str(),
            ),
        }
    }
}

impl fmt::Display for ConnectionIden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Table => "connections",
            Self::Id => "id",
            Self::Provider => "provider",
            Self::UserId => "user_id",
            Self::ProviderId => "provider_id",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        };

        write!(f, "{name}")
    }
}
