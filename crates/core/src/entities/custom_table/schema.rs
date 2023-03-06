use std::{collections::HashMap, fmt};

use actix_web::web;
use chrono::{DateTime, Utc};
use sea_query::{
    enum_def, Alias, ColumnDef, ColumnType, Expr, Keyword, PostgresQueryBuilder, SimpleExpr, Table,
};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

use crate::entities::{Identity, Migrate, Query};
use crate::handlers::Error;

pub struct CustomTableSchemaSelectBuilder {
    query_builder: sea_query::SelectStatement,
}

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StringField {
    pub name: String,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub pattern: Option<String>,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberField {
    pub name: String,
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BooleanField {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DateField {
    pub name: String,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmailField {
    pub name: String,
    pub except: Vec<String>,
    pub only: Vec<String>,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UrlField {
    pub name: String,
    pub except: Vec<String>,
    pub only: Vec<String>,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectField {
    pub name: String,
    pub options: String,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RelationField {
    pub name: String,
    pub table_id: String,
    pub min_selected: i32,
    pub max_selected: i32,
    pub cascade_delete: bool,
    pub is_required: bool,
    pub is_unique: bool,
}

impl CustomTableSchemaSelectBuilder {
    pub fn by_id(&mut self, id: &str) -> &mut Self {
        self.query_builder
            .and_where(Expr::col(<CustomTableSchema as Identity>::Iden::Id).eq(id));

        self
    }

    pub fn by_name(&mut self, name: &str) -> &mut Self {
        self.query_builder
            .and_where(Expr::col(<CustomTableSchema as Identity>::Iden::Name).eq(name));

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
        db_pool: &web::Data<deadpool_postgres::Pool>,
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
                Error::InternalServerError { error }
            })?
            .into_iter()
            .next()
            .ok_or_else(|| {
                let message = format!("No {} was found", CustomTableSchema::error_identifier());
                Error::BadRequest { message }
            })?;

        Ok(row.into())
    }
}

impl Identity for CustomTableSchema {
    type Iden = CustomTableSchemaIden;

    fn table() -> Alias {
        Alias::new(<Self as Identity>::Iden::Table.to_string())
    }

    fn error_identifier() -> String {
        "custom table".to_string()
    }
}

impl Migrate for CustomTableSchema {
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
                ColumnDef::new(<Self as Identity>::Iden::Name)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::StringFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::NumberFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::BooleanFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::DateFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::EmailFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::UrlFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::SelectFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::RelationFields)
                    .array(ColumnType::String(None))
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(ColumnDef::new(<Self as Identity>::Iden::UpdatedAt).timestamp_with_time_zone())
            .to_string(PostgresQueryBuilder)
    }
}

impl CustomTableSchema {
    pub fn select() -> CustomTableSchemaSelectBuilder {
        CustomTableSchemaSelectBuilder {
            query_builder: sea_query::Query::select()
                .from(Self::table())
                .columns([
                    <Self as Identity>::Iden::Id,
                    <Self as Identity>::Iden::Name,
                    <Self as Identity>::Iden::StringFields,
                    <Self as Identity>::Iden::NumberFields,
                    <Self as Identity>::Iden::BooleanFields,
                    <Self as Identity>::Iden::DateFields,
                    <Self as Identity>::Iden::EmailFields,
                    <Self as Identity>::Iden::UrlFields,
                    <Self as Identity>::Iden::SelectFields,
                    <Self as Identity>::Iden::RelationFields,
                    <Self as Identity>::Iden::CreatedAt,
                    <Self as Identity>::Iden::UpdatedAt,
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
                <Self as Identity>::Iden::Id,
                <Self as Identity>::Iden::Name,
                <Self as Identity>::Iden::StringFields,
                <Self as Identity>::Iden::NumberFields,
                <Self as Identity>::Iden::BooleanFields,
                <Self as Identity>::Iden::DateFields,
                <Self as Identity>::Iden::EmailFields,
                <Self as Identity>::Iden::UrlFields,
                <Self as Identity>::Iden::SelectFields,
                <Self as Identity>::Iden::RelationFields,
                <Self as Identity>::Iden::CreatedAt,
                <Self as Identity>::Iden::UpdatedAt,
            ])
            .to_owned()
    }

    fn query_insert(&self) -> Result<String, Error> {
        Ok(sea_query::Query::insert()
            .into_table(Self::table())
            .columns([
                <Self as Identity>::Iden::Id,
                <Self as Identity>::Iden::Name,
                <Self as Identity>::Iden::StringFields,
                <Self as Identity>::Iden::NumberFields,
                <Self as Identity>::Iden::BooleanFields,
                <Self as Identity>::Iden::DateFields,
                <Self as Identity>::Iden::EmailFields,
                <Self as Identity>::Iden::UrlFields,
                <Self as Identity>::Iden::SelectFields,
                <Self as Identity>::Iden::RelationFields,
                <Self as Identity>::Iden::CreatedAt,
                <Self as Identity>::Iden::UpdatedAt,
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

    fn query_update(&self, _: HashMap<String, serde_json::Value>) -> Result<String, Error> {
        todo!()
    }

    fn query_delete(&self) -> String {
        sea_query::Query::delete()
            .from_table(Self::table())
            .and_where(Expr::col(<Self as Identity>::Iden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder)
    }
}

impl From<Row> for CustomTableSchema {
    fn from(row: Row) -> Self {
        Self {
            id: row.get(<Self as Identity>::Iden::Id.to_string().as_str()),
            name: row.get(<Self as Identity>::Iden::Name.to_string().as_str()),
            string_fields: row
                .get::<_, Vec<String>>(<Self as Identity>::Iden::StringFields.to_string().as_str())
                .iter()
                .map(|s| serde_json::from_str(s).unwrap())
                .collect::<Vec<StringField>>(),
            number_fields: row
                .get::<_, Vec<String>>(<Self as Identity>::Iden::NumberFields.to_string().as_str())
                .iter()
                .map(|s| serde_json::from_str(s).unwrap())
                .collect::<Vec<NumberField>>(),
            boolean_fields: row
                .get::<_, Vec<String>>(<Self as Identity>::Iden::BooleanFields.to_string().as_str())
                .iter()
                .map(|s| serde_json::from_str(s).unwrap())
                .collect::<Vec<BooleanField>>(),
            date_fields: row
                .get::<_, Vec<String>>(<Self as Identity>::Iden::DateFields.to_string().as_str())
                .iter()
                .map(|s| serde_json::from_str(s).unwrap())
                .collect::<Vec<DateField>>(),
            email_fields: row
                .get::<_, Vec<String>>(<Self as Identity>::Iden::EmailFields.to_string().as_str())
                .iter()
                .map(|s| serde_json::from_str(s).unwrap())
                .collect::<Vec<EmailField>>(),
            url_fields: row
                .get::<_, Vec<String>>(<Self as Identity>::Iden::UrlFields.to_string().as_str())
                .iter()
                .map(|s| serde_json::from_str(s).unwrap())
                .collect::<Vec<UrlField>>(),
            select_fields: row
                .get::<_, Vec<String>>(<Self as Identity>::Iden::SelectFields.to_string().as_str())
                .iter()
                .map(|s| serde_json::from_str(s).unwrap())
                .collect::<Vec<SelectField>>(),
            relation_fields: row
                .get::<_, Vec<String>>(
                    <Self as Identity>::Iden::RelationFields
                        .to_string()
                        .as_str(),
                )
                .iter()
                .map(|s| serde_json::from_str(s).unwrap())
                .collect::<Vec<RelationField>>(),
            created_at: row.get(<Self as Identity>::Iden::CreatedAt.to_string().as_str()),
            updated_at: row.get(<Self as Identity>::Iden::UpdatedAt.to_string().as_str()),
        }
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
