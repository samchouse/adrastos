use adrastos_macros::{DbDeserialize, DbSelect, DbIdentity};
use chrono::{DateTime, Utc};
use sea_query::{
    enum_def, Alias, ColumnDef, ColumnType, Expr, Keyword, PostgresQueryBuilder, Table,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing_unwrap::ResultExt;
use utoipa::ToSchema;

use crate::{
    entities::{Identity, Init, Join, Query, Update},
    error::Error,
};

use super::fields::{
    BooleanField, DateField, EmailField, NumberField, RelationField, SelectField, StringField,
    UrlField,
};

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, DbDeserialize, DbSelect, DbIdentity)]
#[identity(rename = "custom_table")]
pub struct CustomTableSchema {
    pub id: String,
    #[select(find)]
    pub name: String,
    pub string_fields: Vec<StringField>,
    pub number_fields: Vec<NumberField>,
    pub boolean_fields: Vec<BooleanField>,
    pub date_fields: Vec<DateField>,
    pub email_fields: Vec<EmailField>,
    pub url_fields: Vec<UrlField>,
    pub select_fields: Vec<SelectField>,
    pub relation_fields: Vec<RelationField>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateCustomTableSchema {
    pub name: Option<String>,
    pub string_fields: Option<Vec<StringField>>,
    pub number_fields: Option<Vec<NumberField>>,
    pub boolean_fields: Option<Vec<BooleanField>>,
    pub date_fields: Option<Vec<DateField>>,
    pub email_fields: Option<Vec<EmailField>>,
    pub url_fields: Option<Vec<UrlField>>,
    pub select_fields: Option<Vec<SelectField>>,
    pub relation_fields: Option<Vec<RelationField>>,
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
                    CustomTableSchemaIden::StringFields,
                    update
                        .string_fields
                        .clone()
                        .map(|v| {
                            v.into_iter()
                                .map(|v| serde_json::to_string(&v).unwrap_or_log())
                                .collect::<Vec<_>>()
                        })
                        .into(),
                ),
                (
                    CustomTableSchemaIden::NumberFields,
                    update
                        .number_fields
                        .clone()
                        .map(|v| {
                            v.into_iter()
                                .map(|v| serde_json::to_string(&v).unwrap_or_log())
                                .collect::<Vec<_>>()
                        })
                        .into(),
                ),
                (
                    CustomTableSchemaIden::BooleanFields,
                    update
                        .boolean_fields
                        .clone()
                        .map(|v| {
                            v.into_iter()
                                .map(|v| serde_json::to_string(&v).unwrap_or_log())
                                .collect::<Vec<_>>()
                        })
                        .into(),
                ),
                (
                    CustomTableSchemaIden::DateFields,
                    update
                        .date_fields
                        .clone()
                        .map(|v| {
                            v.into_iter()
                                .map(|v| serde_json::to_string(&v).unwrap_or_log())
                                .collect::<Vec<_>>()
                        })
                        .into(),
                ),
                (
                    CustomTableSchemaIden::EmailFields,
                    update
                        .email_fields
                        .clone()
                        .map(|v| {
                            v.into_iter()
                                .map(|v| serde_json::to_string(&v).unwrap_or_log())
                                .collect::<Vec<_>>()
                        })
                        .into(),
                ),
                (
                    CustomTableSchemaIden::UrlFields,
                    update
                        .url_fields
                        .clone()
                        .map(|v| {
                            v.into_iter()
                                .map(|v| serde_json::to_string(&v).unwrap_or_log())
                                .collect::<Vec<_>>()
                        })
                        .into(),
                ),
                (
                    CustomTableSchemaIden::SelectFields,
                    update
                        .select_fields
                        .clone()
                        .map(|v| {
                            v.into_iter()
                                .map(|v| serde_json::to_string(&v).unwrap_or_log())
                                .collect::<Vec<_>>()
                        })
                        .into(),
                ),
                (
                    CustomTableSchemaIden::RelationFields,
                    update
                        .relation_fields
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

impl Init for CustomTableSchema {
    fn init() -> String {
        Table::create()
            .table(Self::table())
            .if_not_exists()
            .col(
                ColumnDef::new(CustomTableSchemaIden::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(CustomTableSchemaIden::Name)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(CustomTableSchemaIden::StringFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(CustomTableSchemaIden::NumberFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(CustomTableSchemaIden::BooleanFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(CustomTableSchemaIden::DateFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(CustomTableSchemaIden::EmailFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(CustomTableSchemaIden::UrlFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(CustomTableSchemaIden::SelectFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(CustomTableSchemaIden::RelationFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(CustomTableSchemaIden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(ColumnDef::new(CustomTableSchemaIden::UpdatedAt).timestamp_with_time_zone())
            .to_string(PostgresQueryBuilder)
    }
}

impl Join for CustomTableSchema {
    fn join(expr: sea_query::SimpleExpr) -> sea_query::SelectStatement {
        Self::find().and_where(vec![expr]).query_builder.clone()
    }
}

impl Query for CustomTableSchema {
    fn query_insert(&self) -> Result<String, Error> {
        Ok(sea_query::Query::insert()
            .into_table(Self::table())
            .columns([
                CustomTableSchemaIden::Id,
                CustomTableSchemaIden::Name,
                CustomTableSchemaIden::StringFields,
                CustomTableSchemaIden::NumberFields,
                CustomTableSchemaIden::BooleanFields,
                CustomTableSchemaIden::DateFields,
                CustomTableSchemaIden::EmailFields,
                CustomTableSchemaIden::UrlFields,
                CustomTableSchemaIden::SelectFields,
                CustomTableSchemaIden::RelationFields,
                CustomTableSchemaIden::CreatedAt,
                CustomTableSchemaIden::UpdatedAt,
            ])
            .values_panic([
                self.id.clone().into(),
                self.name.clone().into(),
                self.string_fields
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
                    .into(),
                self.number_fields
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
                    .into(),
                self.boolean_fields
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
                    .into(),
                self.date_fields
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
                    .into(),
                self.email_fields
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
                    .into(),
                self.url_fields
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
                    .into(),
                self.select_fields
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
                    .into(),
                self.relation_fields
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
                    .into(),
                self.created_at.into(),
                self.updated_at.into(),
            ])
            .to_string(PostgresQueryBuilder))
    }

    fn query_delete(&self) -> String {
        sea_query::Query::delete()
            .from_table(Self::table())
            .and_where(Expr::col(CustomTableSchemaIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder)
    }
}
