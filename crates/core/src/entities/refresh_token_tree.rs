// TODO(@Xenfo): support many browser tabs being open at the same time, currently it'll invalidate the other tabs

use adrastos_macros::{DbCommon, DbQuery, DbSelect};
use chrono::{DateTime, Duration, Utc};
use sea_query::{enum_def, Expr, PostgresQueryBuilder};
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing_unwrap::ResultExt;

use crate::error::Error;

use super::{Update, User, UserIden};

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, DbCommon, DbSelect, DbQuery)]
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
        db: &deadpool_postgres::Pool,
        tokens: Vec<String>,
    ) -> Result<(), Error> {
        let query = sea_query::Query::update()
            .table(Self::table())
            .values(Update::create([
                (RefreshTokenTreeIden::Tokens, Some(tokens).into()),
                (
                    RefreshTokenTreeIden::InactiveAt,
                    Some(Utc::now() + Duration::try_days(15).unwrap()).into(),
                ),
                (RefreshTokenTreeIden::UpdatedAt, Some(Utc::now()).into()),
            ]))
            .and_where(Expr::col(UserIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder);

        db.get()
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
