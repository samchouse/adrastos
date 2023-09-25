use std::collections::HashMap;

use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use adrastos_core::{
    db::postgres,
    entities::custom_table::{
        fields::{FieldInfo, RelationTarget},
        mm_relation::ManyToManyRelationTable,
        schema::CustomTableSchema,
        CustomTableSelectBuilder,
    },
    error::Error,
    id::Id,
    util,
};
use chrono::{DateTime, Utc};
use heck::{AsLowerCamelCase, AsSnakeCase};
use regex::Regex;
use sea_query::{Alias, Expr, PostgresQueryBuilder, SimpleExpr, Value};
use serde_json::json;
use validator::ValidationErrors;

use crate::middleware::user::RequiredUser;

#[get("/rows")]
pub async fn rows(
    _: RequiredUser,
    path: web::Path<String>,
    web::Query(mut query): web::Query<HashMap<String, String>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db_pool)
        .await?;

    let page = query.get("page").map(|s| s.parse::<u64>().unwrap());
    let limit = query.get("limit").map(|s| s.parse::<u64>().unwrap());
    query.remove("page");
    query.remove("limit");

    let mut builder = CustomTableSelectBuilder::from(&custom_table);
    builder
        .and_where(
            query
                .iter()
                .map(|(field, equals)| Expr::col(Alias::new(field)).eq(equals))
                .collect(),
        )
        .paginate(page, limit)
        .join();

    let rows = builder.finish(&db_pool).await?;
    let mut response = json!({
        "success": true,
        "data": rows
    });

    if let Some(page) = page && let Some(limit) = limit {
        let count = builder.count().finish(&db_pool).await?.as_i64().unwrap() as u64;

        crate::util::attach_pagination_details(
            &mut response,
            crate::util::PaginationInfo { page, limit, count },
        );
    }

    Ok(HttpResponse::Ok().json(response))
}

#[get("/row")]
pub async fn row(
    _: RequiredUser,
    path: web::Path<String>,
    web::Query(query): web::Query<HashMap<String, String>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db_pool)
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
        .join()
        .finish(&db_pool)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": row.as_array().unwrap().first()
    })))
}

#[post("/create")]
pub async fn create(
    _: RequiredUser,
    bytes: web::Bytes,
    path: web::Path<String>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let body = serde_json::from_str::<HashMap<String, serde_json::Value>>(
        &String::from_utf8(bytes.to_vec()).unwrap(),
    )
    .map_err(|err| Error::BadRequest(err.to_string()))?;

    let custom_table = CustomTableSchema::find()
        .by_name(AsSnakeCase(path.into_inner()).to_string())
        .one(&db_pool)
        .await?;

    let id = Id::new().to_string();
    let mut errors = ValidationErrors::new();
    let mut table_values: Vec<(_, SimpleExpr)> = vec![
        ("id", id.clone().into()),
        ("created_at", Utc::now().into()),
        ("updated_at", None::<DateTime<Utc>>.into()),
    ];

    custom_table.fields.iter().for_each(|field| {
        let validation_results =
            field.validate(body.get(&AsLowerCamelCase(field.name.clone()).to_string()));

        match validation_results {
            Ok(value) => {
                table_values.push((&field.name, value));
            }
            Err(validation_errors) => {
                validation_errors.iter().for_each(|error| {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        error.clone(),
                    );
                });
            }
        }
    });

    let insert_queries = custom_table
        .fields
        .iter()
        .filter_map(|field| {
            let FieldInfo::Relation { target, .. } = &field.info else {
                return None;
            };

            match target {
                RelationTarget::Single => {
                    let value = body
                        .get(&AsLowerCamelCase(field.name.clone()).to_string())
                        .unwrap()
                        .as_str()
                        .unwrap();

                    table_values.push((&field.name, value.into()));

                    None
                }
                RelationTarget::Many => {
                    let values = body
                        .get(&AsLowerCamelCase(field.name.clone()).to_string())
                        .unwrap()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().to_string())
                        .collect::<Vec<_>>();

                    Some(ManyToManyRelationTable::insert_query(
                        &custom_table,
                        field,
                        id.clone(),
                        values,
                    ))
                }
            }
        })
        .flatten()
        .collect::<Vec<_>>();

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
                return Error::InternalServerError("Unable to convert error".to_string());
            };
            let Some(routine) = db_error.routine() else {
                return Error::InternalServerError("Unable to get error info".to_string());
            };
            let Some(error) = postgres::Error::try_from(routine).ok() else {
                return Error::InternalServerError("Unsupported database error code".to_string());
            };

            match error {
                postgres::Error::UniqueKeyViolation => {
                    let pre = Regex::new(r"\(.+\)=\('.+'\)").unwrap();

                    let Some(detail) = db_error.detail() else {
                        return Error::InternalServerError("Unable to get error info".to_string());
                    };
                    let Some(matched) = pre.find(detail) else {
                        return Error::InternalServerError("Invalid error details".to_string());
                    };

                    let mut details = matched.as_str().split('=').collect::<Vec<_>>().into_iter();

                    let Some(key) = details.next() else {
                        return Error::InternalServerError("Invalid error details".to_string());
                    };
                    let Some(value) = details.next() else {
                        return Error::InternalServerError("Invalid error details".to_string());
                    };

                    Error::BadRequest(format!(
                        "Key '{}' already exists with value '{}'",
                        key.replace(['(', ')'], ""),
                        value.replace("('", "").replace("')", "")
                    ))
                }
                _ => todo!(),
            }
        })?;

    for query in insert_queries {
        db_pool
            .get()
            .await
            .unwrap()
            .execute(query.to_string(PostgresQueryBuilder).as_str(), &[])
            .await
            .unwrap();
    }

    let mut data = json!({});

    table_values
        .into_iter()
        .filter_map(|(name, value)| {
            let camel_case_name = heck::AsLowerCamelCase(name).to_string();
            let camel_case_name = camel_case_name.as_str();

            match value {
                SimpleExpr::Value(value) => match value {
                    Value::String(value) => Some(json!({ camel_case_name: value })),
                    Value::BigInt(value) => Some(json!({ camel_case_name: value })),
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
    _: RequiredUser,
    bytes: web::Bytes,
    path: web::Path<String>,
    web::Query(query): web::Query<HashMap<String, String>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let body = serde_json::from_str::<HashMap<String, serde_json::Value>>(
        &String::from_utf8(bytes.to_vec()).unwrap(),
    )
    .map_err(|err| Error::BadRequest(err.to_string()))?;

    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db_pool)
        .await?;

    let mut db_query = sea_query::Query::update();

    query.iter().for_each(|(field, equals)| {
        db_query.and_where(Expr::col(Alias::new(AsSnakeCase(field).to_string())).eq(equals));
    });

    let mut errors = ValidationErrors::new();
    let mut table_values: Vec<(_, SimpleExpr)> = vec![("updated_at", Utc::now().into())];

    custom_table.fields.iter().for_each(|field| {
        let validation_results =
            field.validate(body.get(&AsLowerCamelCase(field.name.clone()).to_string()));

        match validation_results {
            Ok(value) => {
                table_values.push((&field.name, value));
            }
            Err(validation_errors) => {
                validation_errors.iter().for_each(|error| {
                    errors.add(
                        util::string_to_static_str(
                            AsLowerCamelCase(field.name.clone()).to_string(),
                        ),
                        error.clone(),
                    );
                });
            }
        }
    });

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
            db_query
                .table(Alias::new(&custom_table.name))
                .values(
                    table_values
                        .clone()
                        .into_iter()
                        .map(|(f, v)| (Alias::new(f), v.clone())),
                )
                .to_string(PostgresQueryBuilder)
                .as_str(),
            &[],
        )
        .await
        .unwrap();

    let mut data = json!({});

    table_values
        .into_iter()
        .filter_map(|(name, value)| {
            let camel_case_name = heck::AsLowerCamelCase(name).to_string();
            let camel_case_name = camel_case_name.as_str();

            match value {
                SimpleExpr::Value(value) => match value {
                    Value::String(value) => Some(json!({ camel_case_name: value })),
                    Value::BigInt(value) => Some(json!({ camel_case_name: value })),
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
        "message": "Row updated successfully",
        "data": data
    })))
}

#[delete("/delete")]
pub async fn delete(
    _: RequiredUser,
    path: web::Path<String>,
    web::Query(query): web::Query<HashMap<String, String>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db_pool)
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
