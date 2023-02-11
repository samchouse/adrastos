use std::{collections::HashMap, fmt};

use chrono::{DateTime, Utc};
use sea_query::{
    enum_def, Alias, ColumnDef, ColumnType, Expr, Keyword, PostgresQueryBuilder, SelectStatement,
    Table,
};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

use crate::handlers::Error;

use super::{Identity, Migrate, Query};

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomCollection {
    pub id: String,
    pub string_fields: Vec<StringField>,
    pub number_fields: Vec<NumberField>,
    pub boolean_fields: Vec<BooleanField>,
    pub email_fields: Vec<EmailField>,
    pub url_fields: Vec<UrlField>,
    pub select_fields: Vec<SelectField>,
    pub relation_fields: Vec<RelationField>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StringField {
    pub name: String,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub pattern: Option<String>,
    pub non_empty: Option<bool>,
    pub unique: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NumberField {
    pub name: String,
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub non_empty: Option<bool>,
    pub unique: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BooleanField {
    pub name: String,
    pub non_falsey: Option<bool>,
    pub unique: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailField {
    pub name: String,
    pub except: Option<Vec<String>>,
    pub only: Option<Vec<String>>,
    pub non_empty: Option<bool>,
    pub unique: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UrlField {
    pub name: String,
    pub except: Option<Vec<String>>,
    pub only: Option<Vec<String>>,
    pub non_empty: Option<bool>,
    pub unique: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectField {
    pub name: String,
    pub options: String,
    pub max_selected: i32,
    pub non_empty: Option<bool>,
    pub unique: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelationField {
    pub name: String,
    pub table_id: String,
    pub max_selected: i32,
    pub cascade_delete: bool,
    pub non_empty: Option<bool>,
    pub unique: Option<bool>,
}

impl Identity for CustomCollection {
    type Iden = CustomCollectionIden;

    fn table() -> Alias {
        Alias::new(<Self as Identity>::Iden::Table.to_string().as_str())
    }

    fn error_identifier() -> String {
        "custom collection".to_string()
    }
}

impl Migrate for CustomCollection {
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
                ColumnDef::new(<Self as Identity>::Iden::StringFields)
                    .array(ColumnType::Json)
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::NumberFields)
                    .array(ColumnType::Json)
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::BooleanFields)
                    .array(ColumnType::Json)
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::EmailFields)
                    .array(ColumnType::Json)
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::UrlFields)
                    .array(ColumnType::Json)
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::SelectFields)
                    .array(ColumnType::Json)
                    .not_null()
                    .default(vec![] as Vec<String>),
            )
            .col(
                ColumnDef::new(<Self as Identity>::Iden::RelationFields)
                    .array(ColumnType::Json)
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

impl Query for CustomCollection {
    fn query_select(expressions: Vec<sea_query::SimpleExpr>) -> SelectStatement {
        let mut query = sea_query::Query::select();

        for expression in expressions {
            query.and_where(expression);
        }

        query
            .from(Self::table())
            .columns([
                <Self as Identity>::Iden::Id,
                <Self as Identity>::Iden::StringFields,
                <Self as Identity>::Iden::NumberFields,
                <Self as Identity>::Iden::BooleanFields,
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
                <Self as Identity>::Iden::StringFields,
                <Self as Identity>::Iden::NumberFields,
                <Self as Identity>::Iden::BooleanFields,
                <Self as Identity>::Iden::EmailFields,
                <Self as Identity>::Iden::UrlFields,
                <Self as Identity>::Iden::SelectFields,
                <Self as Identity>::Iden::RelationFields,
                <Self as Identity>::Iden::CreatedAt,
                <Self as Identity>::Iden::UpdatedAt,
            ])
            .values_panic([
                self.id.clone().into(),
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

impl From<Row> for CustomCollection {
    fn from(row: Row) -> Self {
        Self {
            id: row.get(<Self as Identity>::Iden::Id.to_string().as_str()),
            string_fields: serde_json::from_value(
                row.get(<Self as Identity>::Iden::StringFields.to_string().as_str()),
            )
            .unwrap_or(vec![]),
            number_fields: serde_json::from_value(
                row.get(<Self as Identity>::Iden::StringFields.to_string().as_str()),
            )
            .unwrap_or(vec![]),
            boolean_fields: serde_json::from_value(
                row.get(<Self as Identity>::Iden::StringFields.to_string().as_str()),
            )
            .unwrap_or(vec![]),
            email_fields: serde_json::from_value(
                row.get(<Self as Identity>::Iden::StringFields.to_string().as_str()),
            )
            .unwrap_or(vec![]),
            url_fields: serde_json::from_value(
                row.get(<Self as Identity>::Iden::StringFields.to_string().as_str()),
            )
            .unwrap_or(vec![]),
            select_fields: serde_json::from_value(
                row.get(<Self as Identity>::Iden::StringFields.to_string().as_str()),
            )
            .unwrap_or(vec![]),
            relation_fields: serde_json::from_value(
                row.get(<Self as Identity>::Iden::StringFields.to_string().as_str()),
            )
            .unwrap_or(vec![]),
            created_at: row.get(<Self as Identity>::Iden::CreatedAt.to_string().as_str()),
            updated_at: row.get(<Self as Identity>::Iden::UpdatedAt.to_string().as_str()),
        }
    }
}

impl fmt::Display for CustomCollectionIden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Table => "custom_collections",
            Self::Id => "id",
            Self::StringFields => "string_fields",
            Self::NumberFields => "number_fields",
            Self::BooleanFields => "boolean_fields",
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
