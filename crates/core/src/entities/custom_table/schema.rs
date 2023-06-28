use std::{collections::HashMap, fmt};

use adrastos_macros::DbDeserialize;
use chrono::{DateTime, Utc};
use sea_query::{
    enum_def, Alias, ColumnDef, ColumnType, Expr, Keyword, PostgresQueryBuilder, SimpleExpr, Table,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    entities::{Identity, Init, Query},
    error::Error,
};

use super::fields::{
    BooleanField, DateField, EmailField, NumberField, RelationField, SelectField, StringField,
    UrlField,
};

pub struct CustomTableSchemaSelectBuilder {
    query_builder: sea_query::SelectStatement,
}

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, DbDeserialize)]
pub struct CustomTableSchema {
    pub id: String,
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

impl CustomTableSchemaSelectBuilder {
    pub fn by_id(&mut self, id: &str) -> &mut Self {
        self.query_builder
            .and_where(Expr::col(CustomTableSchemaIden::Id).eq(id));

        self
    }

    pub fn by_name(&mut self, name: &str) -> &mut Self {
        self.query_builder
            .and_where(Expr::col(CustomTableSchemaIden::Name).eq(name));

        self
    }

    pub fn and_where(&mut self, expressions: Vec<SimpleExpr>) -> &mut Self {
        for expression in expressions {
            self.query_builder.and_where(expression);
        }

        self
    }

    pub async fn finish(
        &mut self,
        db_pool: &deadpool_postgres::Pool,
    ) -> Result<CustomTableSchema, Error> {
        let row = db_pool
            .get()
            .await
            .unwrap()
            .query(
                self.query_builder.to_string(PostgresQueryBuilder).as_str(),
                &[],
            )
            .await
            .map_err(|e| {
                let error = format!(
                    "An error occurred while fetching the {}: {e}",
                    CustomTableSchema::error_identifier(),
                );
                Error::InternalServerError(error)
            })?
            .into_iter()
            .next()
            .ok_or_else(|| {
                let message = format!("No {} was found", CustomTableSchema::error_identifier());
                Error::BadRequest(message)
            })?;

        Ok(row.into())
    }
}

impl Identity for CustomTableSchema {
    fn table() -> Alias {
        Alias::new(CustomTableSchemaIden::Table.to_string())
    }

    fn error_identifier() -> String {
        "custom table".to_string()
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

impl CustomTableSchema {
    pub fn select() -> CustomTableSchemaSelectBuilder {
        CustomTableSchemaSelectBuilder {
            query_builder: sea_query::Query::select()
                .from(Self::table())
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
                .limit(1)
                .to_owned(),
        }
    }
}

impl Query for CustomTableSchema {
    fn query_select(expressions: Vec<sea_query::SimpleExpr>) -> sea_query::SelectStatement {
        let mut query = sea_query::Query::select();

        for expression in expressions {
            query.and_where(expression);
        }

        query
            .from(Self::table())
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
            .to_owned()
    }

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

    fn query_update(&self, updated: &HashMap<String, serde_json::Value>) -> Result<String, Error> {
        let mut query = sea_query::Query::update();

        if let Some(name) = updated.get(CustomTableSchemaIden::Name.to_string().as_str()) {
            if let Some(name) = name.as_str() {
                query.values([(CustomTableSchemaIden::Name, name.into())]);
            }
        }
        if let Some(string_fields) =
            updated.get(CustomTableSchemaIden::StringFields.to_string().as_str())
        {
            let string_fields = string_fields
                .as_array()
                .unwrap()
                .iter()
                .map(|f| serde_json::from_str::<StringField>(f.as_str().unwrap()).unwrap())
                .collect::<Vec<_>>();

            query.values([(
                CustomTableSchemaIden::StringFields,
                string_fields
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
                    .into(),
            )]);
        }

        Ok(query
            .table(Self::table())
            .and_where(Expr::col(CustomTableSchemaIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder))
    }

    fn query_delete(&self) -> String {
        sea_query::Query::delete()
            .from_table(Self::table())
            .and_where(Expr::col(CustomTableSchemaIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder)
    }
}

impl fmt::Display for CustomTableSchemaIden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Table => "custom_tables",
            Self::Id => "id",
            Self::Name => "name",
            Self::StringFields => "string_fields",
            Self::NumberFields => "number_fields",
            Self::BooleanFields => "boolean_fields",
            Self::DateFields => "date_fields",
            Self::EmailFields => "email_fields",
            Self::UrlFields => "url_fields",
            Self::SelectFields => "select_fields",
            Self::RelationFields => "relation_fields",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        };

        write!(f, "{name}")
    }
}
