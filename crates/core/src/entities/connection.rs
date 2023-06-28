use std::fmt;

use adrastos_macros::{DbDeserialize, DbSelect};
use chrono::{DateTime, Utc};
use sea_query::{
    enum_def, Alias, ColumnDef, Expr, ForeignKey, ForeignKeyAction, Keyword, PostgresQueryBuilder,
    Table,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::Error;

use super::{Identity, Init, Query, User, UserIden, Join};

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, DbDeserialize, DbSelect)]
pub struct Connection {
    pub id: String,
    #[select(find)]
    pub provider: String,
    pub user_id: String,
    #[select(find)]
    pub provider_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Identity for Connection {
    fn table() -> Alias {
        Alias::new(ConnectionIden::Table.to_string())
    }

    fn error_identifier() -> String {
        "connection".into()
    }
}

impl Init for Connection {
    fn init() -> String {
        Table::create()
            .table(Self::table())
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
                    .from(Connection::table(), ConnectionIden::UserId)
                    .to(User::table(), UserIden::Id)
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_string(PostgresQueryBuilder)
    }
}

impl Join for Connection {
    fn join(expr: sea_query::SimpleExpr) -> sea_query::SelectStatement {
        Self::find().and_where(vec![expr]).query_builder.clone()
    }
}

impl Query for Connection {
    fn query_insert(&self) -> Result<String, Error> {
        Ok(sea_query::Query::insert()
            .into_table(Self::table())
            .columns([
                ConnectionIden::Id,
                ConnectionIden::Provider,
                ConnectionIden::UserId,
                ConnectionIden::ProviderId,
                ConnectionIden::CreatedAt,
                ConnectionIden::UpdatedAt,
            ])
            .values_panic([
                self.id.clone().into(),
                self.provider.clone().into(),
                self.user_id.clone().into(),
                self.provider_id.clone().into(),
                self.created_at.into(),
                self.updated_at.into(),
            ])
            .to_string(PostgresQueryBuilder))
    }

    fn query_delete(&self) -> String {
        sea_query::Query::delete()
            .from_table(Self::table())
            .and_where(Expr::col(ConnectionIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder)
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
