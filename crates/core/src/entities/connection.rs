use adrastos_macros::{DbSelect, DbCommon};
use chrono::{DateTime, Utc};
use sea_query::{enum_def, Alias, Expr, PostgresQueryBuilder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::Error;

use super::{Identity, Join, Query, User};

#[enum_def]
#[derive(
    Debug, Serialize, Deserialize, Clone, ToSchema, DbSelect, DbCommon,
)]
pub struct Connection {
    pub id: String,
    #[adrastos(find)]
    pub provider: String,
    #[adrastos(relation = User)]
    pub user_id: String,
    #[adrastos(find)]
    pub provider_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
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
