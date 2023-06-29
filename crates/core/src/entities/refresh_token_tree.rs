// TODO(@Xenfo): support many browser tabs being open at the same time, currently it'll invalidate the other tabs

use adrastos_macros::{DbCommon, DbSelect};
use chrono::{DateTime, Duration, Utc};
use sea_query::{enum_def, Alias, Expr, PostgresQueryBuilder};
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing_unwrap::ResultExt;
use utoipa::ToSchema;

use crate::error::Error;

use super::{Identity, Join, Query, Update, User, UserIden};

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, DbCommon, DbSelect)]
pub struct RefreshTokenTree {
    pub id: String,
    #[adrastos(relation = User)]
    pub user_id: String,
    pub inactive_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub tokens: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateRefreshTokenTree {
    pub inactive_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub tokens: Option<Vec<String>>,
}

impl RefreshTokenTree {
    pub async fn update(
        &self,
        db_pool: &deadpool_postgres::Pool,
        tokens: Vec<String>,
    ) -> Result<(), Error> {
        let query = sea_query::Query::update()
            .table(Self::table())
            .values(Update::create([
                (RefreshTokenTreeIden::Tokens, Some(tokens).into()),
                (
                    RefreshTokenTreeIden::InactiveAt,
                    Some(Utc::now() + Duration::days(15)).into(),
                ),
                (RefreshTokenTreeIden::UpdatedAt, Some(Utc::now()).into()),
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
                Error::InternalServerError("Failed to update refresh token tree".into())
            })?;

        Ok(())
    }
}

impl Join for RefreshTokenTree {
    fn join(expr: sea_query::SimpleExpr) -> sea_query::SelectStatement {
        Self::find().and_where(vec![expr]).query_builder.clone()
    }
}

impl Query for RefreshTokenTree {
    fn query_insert(&self) -> Result<String, Error> {
        Ok(sea_query::Query::insert()
            .into_table(Self::table())
            .columns([
                RefreshTokenTreeIden::Id,
                RefreshTokenTreeIden::UserId,
                RefreshTokenTreeIden::InactiveAt,
                RefreshTokenTreeIden::ExpiresAt,
                RefreshTokenTreeIden::Tokens,
                RefreshTokenTreeIden::CreatedAt,
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

    fn query_delete(&self) -> String {
        sea_query::Query::delete()
            .from_table(Self::table())
            .and_where(Expr::col(RefreshTokenTreeIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder)
    }
}
