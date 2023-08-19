use std::borrow::Cow;

use regex::Regex;
use sea_query::{Alias, ColumnDef, ColumnType, SimpleExpr};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use validator::ValidationError;

use crate::util;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RelationTarget {
    Single,
    Many,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FieldInfo {
    String {
        min_length: Option<i32>,
        max_length: Option<i32>,
        pattern: Option<String>,
        is_required: bool,
        is_unique: bool,
    },
    Number {
        min: Option<i32>,
        max: Option<i32>,
        is_required: bool,
        is_unique: bool,
    },
    Boolean,
    Date {
        is_required: bool,
        is_unique: bool,
    },
    Email {
        except: Vec<String>,
        only: Vec<String>,
        is_required: bool,
        is_unique: bool,
    },
    Url {
        except: Vec<String>,
        only: Vec<String>,
        is_required: bool,
        is_unique: bool,
    },
    Select {
        options: Vec<String>,
        min_selected: Option<i32>,
        max_selected: Option<i32>,
        is_required: bool,
        is_unique: bool,
    },
    Relation {
        table: String,
        target: RelationTarget,
        min_selected: Option<i32>,
        max_selected: Option<i32>,
        cascade_delete: bool,
        is_required: bool,
        is_unique: bool,
    },
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    #[serde(flatten)]
    pub info: FieldInfo,
}

impl Field {
    pub fn column(&self) -> ColumnDef {
        let mut column = ColumnDef::new(Alias::new(&self.name));

        match &self.info {
            FieldInfo::String {
                is_required,
                is_unique,
                ..
            } => {
                if *is_required {
                    column.not_null();
                }
                if *is_unique {
                    column.unique_key();
                }

                column.string();
            }
            FieldInfo::Number {
                is_required,
                is_unique,
                ..
            } => {
                if *is_required {
                    column.not_null();
                }
                if *is_unique {
                    column.unique_key();
                }

                column.string();
            }
            FieldInfo::Boolean => {
                column.boolean();
            }
            FieldInfo::Date {
                is_required,
                is_unique,
            } => {
                if *is_required {
                    column.not_null();
                }
                if *is_unique {
                    column.unique_key();
                }

                column.timestamp_with_time_zone();
            }
            FieldInfo::Email {
                is_required,
                is_unique,
                ..
            } => {
                if *is_required {
                    column.not_null();
                }
                if *is_unique {
                    column.unique_key();
                }

                column.string();
            }
            FieldInfo::Url {
                is_required,
                is_unique,
                ..
            } => {
                if *is_required {
                    column.not_null();
                }
                if *is_unique {
                    column.unique_key();
                }

                column.string();
            }
            FieldInfo::Select {
                is_required,
                is_unique,
                ..
            } => {
                if *is_required {
                    column.not_null();
                }
                if *is_unique {
                    column.unique_key();
                }

                column.array(ColumnType::String(None));
            }
            FieldInfo::Relation {
                target,
                is_required,
                is_unique,
                ..
            } => {
                if target == &RelationTarget::Single {
                    if *is_required {
                        column.not_null();
                    }
                    if *is_unique {
                        column.unique_key();
                    }

                    column.string();
                };
            }
        }

        column
    }

    pub fn validate(&self, value: Option<&Value>) -> Result<SimpleExpr, Vec<ValidationError>> {
        let mut errors = vec![];

        match &self.info {
            FieldInfo::String {
                min_length,
                max_length,
                pattern,
                is_required,
                ..
            } => {
                match value {
                    Some(value) => {
                        let value = value.as_str().unwrap();

                        let mut length_error = ValidationError::new("length");

                        if let Some(min_length) = min_length {
                            if value.len() < (*min_length).try_into().unwrap() {
                                length_error.add_param(Cow::from("min"), &min_length);
                            }
                        }
                        if let Some(max_length) = max_length {
                            if value.len() > (*max_length).try_into().unwrap() {
                                length_error.add_param(Cow::from("max"), &max_length);
                            }
                        }
                        if let Some(pattern) = &pattern {
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
                        if *is_required {
                            errors.push(ValidationError::new("required"));
                        }
                    }
                };
            }
            _ => todo!(),
        }

        Err(errors)
    }
}
