use std::{borrow::Cow, collections::HashMap};

use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use core::{
    db::postgres,
    entities::custom_table::{schema::CustomTableSchema, CustomTableSelectBuilder},
    error::Error,
    id::Id,
    url::Url,
    util,
};
use heck::{AsLowerCamelCase, AsSnakeCase};
use regex::Regex;
use sea_query::{Alias, Expr, PostgresQueryBuilder, SimpleExpr, Value};
use serde_json::json;
use validator::{ValidationError, ValidationErrors};

// #[get("/rows")]
// pub async fn rows(
//     path: web::Path<String>,
//     web::Query(query): web::Query<HashMap<String, String>>,
//     db_pool: web::Data<deadpool_postgres::Pool>,
// ) -> actix_web::Result<impl Responder, Error> {
//     todo!();

//     let custom_table = CustomTableSchema::select()
//         .by_name(&path.into_inner())
//         .finish(&db_pool)
//         .await?;

//     let rows = CustomTableSelectBuilder::from(&custom_table)
//         .and_where(
//             query
//                 .iter()
//                 .map(|(field, equals)| Expr::col(Alias::new(field)).eq(equals))
//                 .collect(),
//         )
//         .limit(None) // TODO(@Xenfo): properly convert rows to JSON array
//         .finish(&db_pool)
//         .await?;

//     Ok(HttpResponse::Ok().json(json!({
//         "success": true,
//         "data": rows
//     })))
// }

#[get("/row")]
pub async fn row(
    path: web::Path<String>,
    web::Query(query): web::Query<HashMap<String, String>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let custom_table = CustomTableSchema::select()
        .by_name(&path.into_inner())
        .finish(&db_pool)
        .await?;

    let row = CustomTableSelectBuilder::from(&custom_table)
        .and_where(
            query
                .iter()
                .map(|(field, equals)| {
                    Expr::col(Alias::new(AsSnakeCase(field).to_string())).eq(equals)
                })
                .collect(),
        )
        .finish(&db_pool)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": row
    })))
}

#[post("/create")]
pub async fn create(
    path: web::Path<String>,
    bytes: web::Bytes,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let body = serde_json::from_str::<HashMap<String, serde_json::Value>>(
        &String::from_utf8(bytes.to_vec()).unwrap(),
    )
    .map_err(|err| Error::BadRequest(err.to_string()))?;

    let custom_table = CustomTableSchema::select()
        .by_name(&AsSnakeCase(path.into_inner()).to_string())
        .finish(&db_pool)
        .await?;

    let mut errors = ValidationErrors::new();
    let mut table_values: Vec<(_, SimpleExpr)> = vec![
        ("id", Id::new().to_string().into()),
        ("created_at", Utc::now().into()),
        ("updated_at", None::<DateTime<Utc>>.into()),
    ];

    custom_table.string_fields.iter().for_each(|field| {
        let value = body.get(&AsLowerCamelCase(field.name.clone()).to_string());

        match value {
            Some(value) => {
                let value = value.as_str().unwrap();

                let mut length_error = ValidationError::new("length");

                if let Some(min_length) = field.min_length {
                    if value.len() < min_length.try_into().unwrap() {
                        length_error.add_param(Cow::from("min"), &min_length);
                    }
                }
                if let Some(max_length) = field.max_length {
                    if value.len() > max_length.try_into().unwrap() {
                        length_error.add_param(Cow::from("max"), &max_length);
                    }
                }
                if let Some(pattern) = &field.pattern {
                    if let Ok(regex) = Regex::new(pattern) {
                        if !regex.is_match(value) {
                            errors.add(
                                util::string_to_static_str(
                                    AsLowerCamelCase(field.name.clone()).to_string(),
                                ),
                                util::create_validation_error(
                                    "pattern",
                                    Some(format!("Doesn't match '{pattern}'")),
                                ),
                            );
                        }
                    }
                }

                if !length_error.params.is_empty() {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        length_error,
                    )
                }

                table_values.push((&field.name, value.into()));
            }
            None => {
                if field.is_required {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        ValidationError::new("required"),
                    );
                }
            }
        }
    });
    custom_table.number_fields.iter().for_each(|field| {
        let value = body.get(&AsLowerCamelCase(field.name.clone()).to_string());

        match value {
            Some(value) => {
                let value = value.as_i64().unwrap();

                let mut value_error = ValidationError::new("value");

                if let Some(min) = field.min {
                    if value < min.try_into().unwrap() {
                        value_error.add_param(Cow::from("min"), &min);
                    }
                }
                if let Some(max) = field.max {
                    if value > max.try_into().unwrap() {
                        value_error.add_param(Cow::from("max"), &max);
                    }
                }

                if !value_error.params.is_empty() {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        value_error,
                    )
                }

                table_values.push((&field.name, value.into()));
            }
            None => {
                if field.is_required {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        ValidationError::new("required"),
                    );
                }
            }
        }
    });
    custom_table.boolean_fields.iter().for_each(|field| {
        let value = match body.get(&AsLowerCamelCase(field.name.clone()).to_string()) {
            Some(value) => value.as_bool().unwrap(),
            None => false,
        };

        table_values.push((&field.name, value.into()));
    });
    custom_table.date_fields.iter().for_each(|field| {
        let value = body.get(&AsLowerCamelCase(field.name.clone()).to_string());

        match value {
            Some(value) => {
                table_values.push((
                    &field.name,
                    serde_json::from_value::<DateTime<Utc>>(value.to_owned())
                        .unwrap()
                        .into(),
                ));
            }
            None => {
                if field.is_required {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        ValidationError::new("required"),
                    );
                }
            }
        }
    });
    custom_table.email_fields.iter().for_each(|field| {
        let value = body.get(&AsLowerCamelCase(field.name.clone()).to_string());

        match value {
            Some(value) => {
                let value = value.as_str().unwrap();
                let mut value_url = Url::from(value.to_owned());

                let mut pattern_error = ValidationError::new("pattern");

                if !field.only.is_empty() {
                    value_url
                        .validate_with_patterns(field.only.clone())
                        .iter()
                        .for_each(|(c, pattern)| {
                            if !c {
                                pattern_error.add_param(Cow::from("only"), &pattern);
                            }
                        });
                } else if !field.except.is_empty() {
                    value_url
                        .validate_with_patterns(field.except.clone())
                        .iter()
                        .for_each(|(c, pattern)| {
                            if *c {
                                pattern_error.add_param(Cow::from("except"), &pattern);
                            }
                        });
                }

                if !pattern_error.params.is_empty() {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        pattern_error,
                    )
                }

                table_values.push((&field.name, value.into()));
            }
            None => {
                if field.is_required {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        ValidationError::new("required"),
                    );
                }
            }
        }
    });
    custom_table.url_fields.iter().for_each(|field| {
        let value = body.get(&AsLowerCamelCase(field.name.clone()).to_string());

        match value {
            Some(value) => {
                let value = value.as_str().unwrap();
                let mut value_url = Url::from(value.to_owned());

                let mut pattern_error = ValidationError::new("pattern");

                if !field.only.is_empty() {
                    value_url
                        .validate_with_patterns(field.only.clone())
                        .iter()
                        .for_each(|(c, pattern)| {
                            if !c {
                                pattern_error.add_param(Cow::from("only"), &pattern);
                            }
                        });
                } else if !field.except.is_empty() {
                    value_url
                        .validate_with_patterns(field.except.clone())
                        .iter()
                        .for_each(|(c, pattern)| {
                            if *c {
                                pattern_error.add_param(Cow::from("except"), &pattern);
                            }
                        });
                }

                if !pattern_error.params.is_empty() {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        pattern_error,
                    )
                }

                table_values.push((&field.name, value.into()));
            }
            None => {
                if field.is_required {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        ValidationError::new("required"),
                    );
                }
            }
        }
    });
    custom_table.select_fields.iter().for_each(|field| {
        let value = body.get(&AsLowerCamelCase(field.name.clone()).to_string());

        match value {
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
                    .map(|v| (v.to_owned(), field.options.contains(v)))
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

                if let Some(min_selected) = field.min_selected {
                    if value.len() < min_selected.try_into().unwrap() {
                        selections_error.add_param(Cow::from("min"), &min_selected);
                    }
                }
                if let Some(max_selected) = field.max_selected {
                    if value.len() > max_selected.try_into().unwrap() {
                        selections_error.add_param(Cow::from("max"), &max_selected);
                    }
                }

                if !selections_error.params.is_empty() {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        selections_error,
                    )
                }

                table_values.push((&field.name, value.into()));
            }
            None => {
                if field.is_required {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        ValidationError::new("required"),
                    );
                }
            }
        }
    });
    // custom_table.relation_fields.iter().for_each(|field| {
    //     let value = body.get(&field.name).unwrap().as_str().unwrap();
    //     table_values.insert(field.name.clone(), serde_json::to_value(value).unwrap());
    // });

    if !errors.is_empty() {
        return Err(Error::ValidationErrors {
            message: "Validation failed".to_string(),
            errors,
        });
    }

    db_pool
        .get()
        .await
        .unwrap()
        .execute(
            sea_query::Query::insert()
                .into_table(Alias::new(&custom_table.name))
                .columns(table_values.iter().map(|v| Alias::new(v.0)))
                .values_panic(table_values.iter().map(|v| v.1.clone()))
                .to_string(PostgresQueryBuilder)
                .as_str(),
            &[],
        )
        .await
        .map_err(|error| {
            let Some(db_error) = error.as_db_error() else {
                return Error::InternalServerError("Unable to convert error".to_string())
            };
            let Some(routine) = db_error.routine() else {
                return Error::InternalServerError("Unable to get error info".to_string())
            };
            let Some(error) = postgres::Error::try_from(routine).ok() else {
                return Error::InternalServerError("Unsupported database error code".to_string())
            };

            match error {
                postgres::Error::UniqueKeyViolation => {
                    let pre = Regex::new(r"\(.+\)=\('.+'\)").unwrap();

                    let Some(detail) = db_error.detail() else {
                        return Error::InternalServerError("Unable to get error info".to_string())
                    };
                    let Some(matched) = pre.find(detail) else {
                        return Error::InternalServerError("Invalid error details".to_string())
                    };

                    let mut details = matched.as_str().split('=').collect::<Vec<_>>().into_iter();

                    let Some(key) = details.next() else {
                        return Error::InternalServerError("Invalid error details".to_string())
                    };
                    let Some(value) = details.next() else {
                        return Error::InternalServerError("Invalid error details".to_string())
                    };

                    Error::BadRequest(format!(
                        "Key '{}' already exists with value '{}'",
                        key.replace(['(', ')'], ""),
                        value.replace("('", "").replace("')", "")
                    ))
                }
            }
        })?;

    let mut data = json!({});

    table_values
        .into_iter()
        .filter_map(|(name, value)| {
            let camel_case_name = heck::AsLowerCamelCase(name).to_string();
            let camel_case_name = camel_case_name.as_str();

            match value {
                SimpleExpr::Value(value) => match value {
                    Value::String(value) => Some(json!({ camel_case_name: value })),
                    Value::Int(value) => Some(json!({ camel_case_name: value })),
                    Value::Bool(value) => Some(json!({ camel_case_name: value })),
                    Value::ChronoDateTimeUtc(value) => Some(json!({ camel_case_name: value })),
                    Value::Array(_, Some(value)) => {
                        let value = value
                            .iter()
                            .filter_map(|value| match value {
                                Value::String(value) => Some(value),
                                _ => None,
                            })
                            .collect::<Vec<_>>();

                        Some(json!({ camel_case_name: value }))
                    }
                    _ => None,
                },
                _ => None,
            }
        })
        .for_each(|patch| json_patch::merge(&mut data, &patch));

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": data
    })))
}

#[patch("/update")]
pub async fn update(
    path: web::Path<String>,
    web::Query(query): web::Query<HashMap<String, String>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let custom_table = CustomTableSchema::select()
        .by_name(&path.into_inner())
        .finish(&db_pool)
        .await?;

    let mut db_query = sea_query::Query::update();

    query.iter().for_each(|(field, equals)| {
        db_query.and_where(Expr::col(Alias::new(AsSnakeCase(field).to_string())).eq(equals));
    });

    db_pool
        .get()
        .await
        .unwrap()
        .execute(
            db_query
                .table(Alias::new(&custom_table.name))
                .to_string(PostgresQueryBuilder)
                .as_str(),
            &[],
        )
        .await
        .unwrap();

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Row updated successfully"
    })))
}

#[delete("/delete")]
pub async fn delete(
    path: web::Path<String>,
    web::Query(query): web::Query<HashMap<String, String>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let custom_table = CustomTableSchema::select()
        .by_name(&path.into_inner())
        .finish(&db_pool)
        .await?;

    let mut db_query = sea_query::Query::delete();

    query.iter().for_each(|(field, equals)| {
        db_query.and_where(Expr::col(Alias::new(AsSnakeCase(field).to_string())).eq(equals));
    });

    db_pool
        .get()
        .await
        .unwrap()
        .execute(
            db_query
                .from_table(Alias::new(&custom_table.name))
                .to_string(PostgresQueryBuilder)
                .as_str(),
            &[],
        )
        .await
        .unwrap();

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Row deleted successfully"
    })))
}
