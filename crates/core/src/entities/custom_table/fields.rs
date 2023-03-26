use std::borrow::Cow;

use regex::Regex;
use sea_query::{Alias, ColumnDef, SimpleExpr};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use validator::ValidationError;

use crate::util;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RelationType {
    Single,
    Many,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StringField {
    pub name: String,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub pattern: Option<String>,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NumberField {
    pub name: String,
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BooleanField {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DateField {
    pub name: String,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EmailField {
    pub name: String,
    pub except: Vec<String>,
    pub only: Vec<String>,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UrlField {
    pub name: String,
    pub except: Vec<String>,
    pub only: Vec<String>,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SelectField {
    pub name: String,
    pub options: Vec<String>,
    pub min_selected: Option<i32>,
    pub max_selected: Option<i32>,
    pub is_required: bool,
    pub is_unique: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RelationField {
    pub name: String,
    pub table: String,
    #[serde(rename = "type")]
    pub relation_type: RelationType,
    pub min_selected: Option<i32>,
    pub max_selected: Option<i32>,
    pub cascade_delete: bool,
    pub is_required: bool,
    pub is_unique: bool,
}

impl StringField {
    pub fn column(&self) -> ColumnDef {
        let mut column = ColumnDef::new(Alias::new(&self.name));

        if self.is_required {
            column.not_null();
        }
        if self.is_unique {
            column.unique_key();
        }

        column.string();

        column
    }

    pub fn validate(&self, value: Option<&Value>) -> Result<SimpleExpr, Vec<ValidationError>> {
        let mut errors = vec![];

        match value {
            Some(value) => {
                let value = value.as_str().unwrap();

                let mut length_error = ValidationError::new("length");

                if let Some(min_length) = self.min_length {
                    if value.len() < min_length.try_into().unwrap() {
                        length_error.add_param(Cow::from("min"), &min_length);
                    }
                }
                if let Some(max_length) = self.max_length {
                    if value.len() > max_length.try_into().unwrap() {
                        length_error.add_param(Cow::from("max"), &max_length);
                    }
                }
                if let Some(pattern) = &self.pattern {
                    if let Ok(regex) = Regex::new(pattern) {
                        if !regex.is_match(value) {
                            errors.push(util::create_validation_error(
                                "pattern",
                                Some(format!("Doesn't match '{pattern}'")),
                            ));
                        }
                    }
                }

                if !length_error.params.is_empty() {
                    errors.push(length_error)
                }

                if errors.is_empty() {
                    return Ok(value.into());
                }
            }
            None => {
                if self.is_required {
                    errors.push(ValidationError::new("required"));
                }
            }
        };

        Err(errors)
    }
}
