use std::borrow::Cow;

use chrono::{DateTime, Utc};
use regex::Regex;
use sea_query::{Alias, ColumnDef, ColumnType, SimpleExpr};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::ValidationError;

use crate::{url::Url, util};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RelationTarget {
    Single,
    Many,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FieldInfo {
    #[serde(rename_all = "camelCase")]
    String {
        min_length: Option<i32>,
        max_length: Option<i32>,
        pattern: Option<String>,
        is_required: bool,
        is_unique: bool,
    },
    #[serde(rename_all = "camelCase")]
    Number {
        min: Option<i32>,
        max: Option<i32>,
        is_required: bool,
        is_unique: bool,
    },
    Boolean,
    #[serde(rename_all = "camelCase")]
    Date {
        is_required: bool,
        is_unique: bool,
    },
    #[serde(rename_all = "camelCase")]
    Email {
        except: Vec<String>,
        only: Vec<String>,
        is_required: bool,
        is_unique: bool,
    },
    #[serde(rename_all = "camelCase")]
    Url {
        except: Vec<String>,
        only: Vec<String>,
        is_required: bool,
        is_unique: bool,
    },
    #[serde(rename_all = "camelCase")]
    Select {
        options: Vec<String>,
        min_selected: Option<i32>,
        max_selected: Option<i32>,
        is_required: bool,
        is_unique: bool,
    },
    #[serde(rename_all = "camelCase")]
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

                column.big_integer();
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
                            if value.len() < *min_length as usize {
                                length_error.add_param(Cow::from("min"), &min_length);
                            }
                        }
                        if let Some(max_length) = max_length {
                            if value.len() > *max_length as usize {
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
            FieldInfo::Number {
                min,
                max,
                is_required,
                ..
            } => match value {
                Some(value) => {
                    let value = value.as_i64().unwrap();

                    let mut value_error = ValidationError::new("value");

                    if let Some(min) = min {
                        if value < (*min).into() {
                            value_error.add_param(Cow::from("min"), &min);
                        }
                    }
                    if let Some(max) = max {
                        if value > (*max).into() {
                            value_error.add_param(Cow::from("max"), &max);
                        }
                    }

                    if !value_error.params.is_empty() {
                        errors.push(value_error)
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
            },
            FieldInfo::Boolean => {
                let value = match value {
                    Some(value) => value.as_bool().unwrap(),
                    None => false,
                };

                return Ok(value.into());
            }
            FieldInfo::Date { is_required, .. } => match value {
                Some(value) => {
                    return Ok(serde_json::from_value::<DateTime<Utc>>(value.to_owned())
                        .unwrap()
                        .into());
                }
                None => {
                    if *is_required {
                        errors.push(ValidationError::new("required"));
                    }
                }
            },
            FieldInfo::Email {
                only,
                except,
                is_required,
                ..
            } => match value {
                Some(value) => {
                    let value = value.as_str().unwrap();
                    let mut value_url = Url::from(value.to_owned());

                    let mut pattern_error = ValidationError::new("pattern");

                    if !only.is_empty() {
                        value_url
                            .validate_with_patterns(only.clone())
                            .iter()
                            .for_each(|(c, pattern)| {
                                if !c {
                                    pattern_error.add_param(Cow::from("only"), &pattern);
                                }
                            });
                    } else if !except.is_empty() {
                        value_url
                            .validate_with_patterns(except.clone())
                            .iter()
                            .for_each(|(c, pattern)| {
                                if *c {
                                    pattern_error.add_param(Cow::from("except"), &pattern);
                                }
                            });
                    }

                    if !pattern_error.params.is_empty() {
                        errors.push(pattern_error)
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
            },
            FieldInfo::Url {
                only,
                except,
                is_required,
                ..
            } => match value {
                Some(value) => {
                    let value = value.as_str().unwrap();
                    let mut value_url = Url::from(value.to_owned());

                    let mut pattern_error = ValidationError::new("pattern");

                    if !only.is_empty() {
                        value_url
                            .validate_with_patterns(only.clone())
                            .iter()
                            .for_each(|(c, pattern)| {
                                if !c {
                                    pattern_error.add_param(Cow::from("only"), &pattern);
                                }
                            });
                    } else if !except.is_empty() {
                        value_url
                            .validate_with_patterns(except.clone())
                            .iter()
                            .for_each(|(c, pattern)| {
                                if *c {
                                    pattern_error.add_param(Cow::from("except"), &pattern);
                                }
                            });
                    }

                    if !pattern_error.params.is_empty() {
                        errors.push(pattern_error)
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
            },
            FieldInfo::Select {
                options,
                max_selected,
                min_selected,
                is_required,
                ..
            } => match value {
                Some(value) => {
                    let value = value
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().to_owned())
                        .collect::<Vec<_>>();

                    let mut selections_error = ValidationError::new("selections");

                    let invalid_selections = value
                        .iter()
                        .map(|v| (v.to_owned(), options.contains(v)))
                        .collect::<Vec<_>>();

                    if !invalid_selections.iter().all(|(_, contains)| *contains) {
                        selections_error.add_param(
                            Cow::from("invalid"),
                            &invalid_selections
                                .iter()
                                .filter_map(|(v, contains)| {
                                    if !contains {
                                        return Some(v);
                                    }

                                    None
                                })
                                .collect::<Vec<_>>(),
                        );
                    }

                    if let Some(min_selected) = min_selected {
                        if value.len() < *min_selected as usize {
                            selections_error.add_param(Cow::from("min"), &min_selected);
                        }
                    }
                    if let Some(max_selected) = max_selected {
                        if value.len() > *max_selected as usize {
                            selections_error.add_param(Cow::from("max"), &max_selected);
                        }
                    }

                    if !selections_error.params.is_empty() {
                        errors.push(selections_error)
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
            },
            _ => todo!(),
        }

        Err(errors)
    }
}
