use adrastos_macros::{DbCommon, DbQuery, DbSelect};
use chrono::{DateTime, Utc};
use sea_query::{enum_def, Expr, PostgresQueryBuilder};
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing_unwrap::ResultExt;

use super::{Update, User};
use crate::error::Error;

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, DbSelect, DbCommon, DbQuery)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Passkey {
    pub id: String,
    #[adrastos(unique)]
    pub name: String,
    #[serde(skip_serializing)]
    #[adrastos(relation = User)]
    pub user_id: String,
    #[serde(skip_serializing)]
    #[adrastos(find, unique)]
    pub cred_id: String,
    pub last_used: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    #[adrastos(json)]
    pub data: webauthn_rs::prelude::Passkey,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdatePasskey {
    pub name: Option<String>,
    pub last_used: Option<Option<DateTime<Utc>>>,
    pub data: Option<webauthn_rs::prelude::Passkey>,
}

impl Passkey {
    pub async fn update(
        &self,
        db: &deadpool_postgres::Pool,
        update: UpdatePasskey,
    ) -> Result<(), Error> {
        let query = sea_query::Query::update()
            .table(Self::table())
            .values(Update::create([
                (PasskeyIden::Name, update.name.clone().into()),
                (
                    PasskeyIden::Data,
                    update
                        .data
                        .clone()
                        .and_then(|pk| serde_json::to_string(&pk).ok())
                        .into(),
                ),
                (PasskeyIden::LastUsed, update.last_used.into()),
                (PasskeyIden::UpdatedAt, Some(Utc::now()).into()),
            ]))
            .and_where(Expr::col(PasskeyIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder);

        db.get()
            .await
            .unwrap_or_log()
            .execute(&query, &[])
            .await
            .map_err(|e| {
                error!(error = ?e);
                Error::InternalServerError("Failed to update passkey".into())
            })?;

        Ok(())
    }
}
