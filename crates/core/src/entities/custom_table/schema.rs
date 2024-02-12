use adrastos_macros::{DbCommon, DbQuery, DbSelect};
use chrono::{DateTime, Utc};
use sea_query::{enum_def, Alias, Expr, PostgresQueryBuilder};
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing_unwrap::ResultExt;

use crate::{entities::Update, error::Error};

use super::fields::Field;

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, DbSelect, DbCommon, DbQuery)]
#[adrastos(rename = "custom_tables")]
pub struct CustomTableSchema {
    pub id: String,
    #[adrastos(find, unique)]
    pub name: String,
    pub fields: Vec<Field>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateCustomTableSchema {
    pub name: Option<String>,
    pub fields: Option<Vec<Field>>,
}

impl CustomTableSchema {
    pub async fn update(
        &self,
        db_pool: &deadpool_postgres::Pool,
        update: UpdateCustomTableSchema,
    ) -> Result<(), Error> {
        let query = sea_query::Query::update()
            .table(Self::table())
            .values(Update::create([
                (CustomTableSchemaIden::Name, update.name.into()),
                (
                    CustomTableSchemaIden::Fields,
                    update
                        .fields
                        .clone()
                        .map(|v| {
                            v.into_iter()
                                .map(|v| serde_json::to_string(&v).unwrap_or_log())
                                .collect::<Vec<_>>()
                        })
                        .into(),
                ),
                (CustomTableSchemaIden::UpdatedAt, Some(Utc::now()).into()),
            ]))
            .and_where(Expr::col(CustomTableSchemaIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder);

        db_pool
            .get()
            .await
            .unwrap_or_log()
            .execute(&query, &[])
            .await
            .map_err(|e| {
                error!(error = ?e);
                Error::InternalServerError("Failed to update custom table schema".into())
            })?;

        Ok(())
    }
}
