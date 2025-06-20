use std::collections::HashMap;

use adrastos_core::{
    db::postgres,
    entities::{
        custom_table::{
            fields::{FieldInfo, RelationTarget},
            mm_relation::ManyToManyRelationTable,
            permissions::Permission,
            schema::CustomTableSchema,
            CustomTableSelectBuilder,
        },
        AlternateUserType,
    },
    error::Error,
    id::Id,
};
use axum::{
    extract::{Path, Query},
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use heck::{ToLowerCamelCase, ToSnakeCase};
use regex::Regex;
use sea_query::{Alias, Expr, PostgresQueryBuilder, SimpleExpr, Value};
use serde_json::json;
use tracing_unwrap::ResultExt;
use validator::ValidationErrors;

use crate::{
    middleware::extractors::{AnyUser, ProjectDatabase},
    state::AppState,
    util,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/rows", get(rows))
        .route("/row", get(row))
        .route("/create", post(create))
        .route("/update", patch(update))
        .route("/delete", delete(remove))
}

pub async fn rows(
    AnyUser(user, user_type): AnyUser,
    Path(path): Path<String>,
    ProjectDatabase(db): ProjectDatabase,
    Query(mut query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, Error> {
    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db)
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

    if matches!(user_type, AlternateUserType::Normal) {
        if let Some(permission) = custom_table.permissions.view.clone() {
            builder.cond_where(Permission::parse(&custom_table, permission)?.to_sql_cond(&user));
        }
    }

    let rows = builder.finish(&db).await?;
    let mut response = json!({ "rows": rows });

    if let Some(page) = page
        && let Some(limit) = limit
    {
        let count = builder.count().finish(&db).await?.as_i64().unwrap() as u64;

        crate::util::attach_pagination_details(
            &mut response,
            crate::util::PaginationInfo { page, limit, count },
        );
    }

    Ok(Json(response))
}

pub async fn row(
    AnyUser(user, user_type): AnyUser,
    Path(path): Path<String>,
    ProjectDatabase(db): ProjectDatabase,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, Error> {
    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db)
        .await?;

    let mut builder = CustomTableSelectBuilder::from(&custom_table);
    builder
        .and_where(
            query
                .iter()
                .map(|(field, equals)| Expr::col(Alias::new(field)).eq(equals))
                .collect(),
        )
        .join();

    if matches!(user_type, AlternateUserType::Normal) {
        if let Some(permission) = custom_table.permissions.view.clone() {
            builder.cond_where(Permission::parse(&custom_table, permission)?.to_sql_cond(&user));
        }
    }

    let row = builder.finish(&db).await?;
    Ok(Json(row.as_array().unwrap().first().cloned()))
}

pub async fn create(
    _: AnyUser,
    Path(path): Path<String>,
    ProjectDatabase(db): ProjectDatabase,
    Json(body): Json<HashMap<String, serde_json::Value>>,
) -> Result<impl IntoResponse, Error> {
    let custom_table = CustomTableSchema::find()
        .by_name(path.to_snake_case())
        .one(&db)
        .await?;

    let id = Id::new().to_string();
    let mut errors = ValidationErrors::new();
    let mut table_values: Vec<(_, SimpleExpr)> = vec![
        ("id", id.clone().into()),
        ("created_at", Utc::now().into()),
        ("updated_at", None::<DateTime<Utc>>.into()),
    ];

    table_values = table_values
        .clone()
        .into_iter()
        .map(|(key, value)| {
            if key != "id" {
                return (key, value);
            }

            let Some(Some(id)) = body.get("id").map(|f| f.as_str()) else {
                return (key, value);
            };

            (key, id.into())
        })
        .collect::<Vec<_>>();

    custom_table.fields.iter().for_each(|field| {
        let validation_results =
            field.validate(body.get(&field.name.clone().to_lower_camel_case()));

        match validation_results {
            Ok(value) => {
                if let FieldInfo::Relation { target, .. } = &field.info {
                    if matches!(target, RelationTarget::Many) {
                        return;
                    }
                }

                table_values.push((&field.name, value));
            }
            Err(validation_errors) => {
                validation_errors.iter().for_each(|error| {
                    errors.add(
                        util::string_to_static_str(field.name.clone().to_lower_camel_case()),
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
                RelationTarget::Many => {
                    let values = body
                        .get(&field.name.clone().to_lower_camel_case())
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
                _ => None,
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

    db.get()
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
        db.get()
            .await
            .unwrap()
            .execute(query.to_string(PostgresQueryBuilder).as_str(), &[])
            .await
            .unwrap_or_log();
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

    Ok(Json(data))
}

pub async fn update(
    _: AnyUser,
    Path(path): Path<String>,
    ProjectDatabase(db): ProjectDatabase,
    Query(query): Query<HashMap<String, String>>,
    Json(body): Json<HashMap<String, serde_json::Value>>,
) -> Result<impl IntoResponse, Error> {
    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db)
        .await?;

    let mut db_query = sea_query::Query::update();
    // TODO(@Xenfo): Add support for multiple rows
    db_query.limit(1);

    query.iter().for_each(|(field, equals)| {
        db_query.and_where(Expr::col(Alias::new(field.to_snake_case())).eq(equals));
    });

    let mut errors = ValidationErrors::new();
    let mut table_values: Vec<(_, SimpleExpr)> = vec![("updated_at", Utc::now().into())];

    custom_table.fields.iter().for_each(|field| {
        let validation_results = field.validate(body.get(&field.name.clone().to_snake_case()));

        match validation_results {
            Ok(value) => {
                table_values.push((&field.name, value));
            }
            Err(validation_errors) => {
                validation_errors.iter().for_each(|error| {
                    errors.add(
                        util::string_to_static_str(field.name.clone().to_snake_case()),
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

    db.get()
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

    Ok(Json(data))
}

pub async fn remove(
    _: AnyUser,
    Path(path): Path<String>,
    ProjectDatabase(db): ProjectDatabase,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, Error> {
    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db)
        .await?;

    let mut db_query = sea_query::Query::delete();
    // TODO(@Xenfo): Add support for multiple rows
    db_query.limit(1);

    query.iter().for_each(|(field, equals)| {
        db_query.and_where(Expr::col(Alias::new(field.to_snake_case())).eq(equals));
    });

    db.get()
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

    Ok(Json(serde_json::Value::Null))
}
