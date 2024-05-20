use adrastos_macros::{DbCommon, DbQuery, DbSelect};
use chrono::{DateTime, Utc};
use sea_query::{enum_def, Expr, PostgresQueryBuilder};
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing_unwrap::ResultExt;

use crate::{entities::Update, error::Error};

use super::{fields::Field, permissions::Permissions};

#[enum_def]
#[derive(Debug, Default, Serialize, Deserialize, Clone, DbSelect, DbCommon, DbQuery)]
#[adrastos(rename = "custom_tables")]
pub struct CustomTableSchema {
    pub id: String,
    #[adrastos(find, unique)]
    pub name: String,
    pub fields: Vec<Field>,
    #[adrastos(json)]
    pub permissions: Permissions,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateCustomTableSchema {
    pub name: Option<String>,
    pub fields: Option<Vec<Field>>,
    pub permissions: Option<Permissions>,
}

impl CustomTableSchema {
    pub async fn update(
        &self,
        db: &deadpool_postgres::Pool,
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
                (
                    CustomTableSchemaIden::Permissions,
                    update
                        .permissions
                        .map(|p| serde_json::to_string(&p).unwrap())
                        .into(),
                ),
                (CustomTableSchemaIden::UpdatedAt, Some(Utc::now()).into()),
            ]))
            .and_where(Expr::col(CustomTableSchemaIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder);

        db.get()
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
