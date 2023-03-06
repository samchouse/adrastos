use std::collections::HashMap;

use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use regex::Regex;
use sea_query::{Alias, ArrayType, Expr, PostgresQueryBuilder, SimpleExpr, Value};
use serde_json::json;

use crate::{
    entities::custom_table::{schema::CustomTableSchema, CustomTableSelectBuilder},
    handlers::Error,
    id::Id,
};

#[get("/rows")]
pub async fn rows(
    path: web::Path<String>,
    web::Query(query): web::Query<HashMap<String, String>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    todo!();

    let custom_table = CustomTableSchema::select()
        .by_name(&path.into_inner())
        .finish(&db_pool)
        .await?;

    let rows = CustomTableSelectBuilder::from(&custom_table)
        .and_where(
            query
                .iter()
                .map(|(field, equals)| Expr::col(Alias::new(field)).eq(equals))
                .collect(),
        )
        .limit(None) // TODO(@Xenfo): properly convert rows to JSON array
        .finish(&db_pool)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": rows
    })))
}

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
                .map(|(field, equals)| Expr::col(Alias::new(field)).eq(equals))
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
    .map_err(|err| Error::BadRequest {
        message: err.to_string(),
    })?;

    let custom_table = CustomTableSchema::select()
        .by_name(&path.into_inner())
        .finish(&db_pool)
        .await?;

    let mut errors = vec![];
    let mut table_values: Vec<(_, SimpleExpr)> = vec![
        ("id", Id::new().to_string().into()),
        ("created_at", Utc::now().into()),
        ("updated_at", None::<DateTime<Utc>>.into()),
    ];

    custom_table.string_fields.iter().for_each(|field| {
        let value = body.get(&field.name);

        match value {
            Some(value) => {
                let value = value.as_str().unwrap();

                if let Some(max_length) = field.max_length && value.len() > max_length.try_into().unwrap() {
                    errors.push(format!(
                        "The length of {} is too big. Max length is {}",
                        field.name, max_length
                    ));
                }
                if let Some(min_length) = field.min_length && value.len() < min_length.try_into().unwrap() {
                    errors.push(format!(
                        "The length of {} is too small. Min length is {}",
                        field.name, min_length
                    ));
                }
                if let Some(pattern) = &field.pattern && let Ok(regex) = Regex::new(pattern) && !regex.is_match(value) {
                    errors.push(format!(
                        "The value of {} is not valid. Pattern {} doesn't match {}",
                        field.name, pattern, value
                    ));
                }

                table_values.push((&field.name, value.into()));
            }
            None => {
                if field.is_required {
                    errors.push(format!("{} is required", field.name))
                }
            }
        }
    });
    custom_table.number_fields.iter().for_each(|field| {
        let value = body.get(&field.name);

        match value {
            Some(value) => {
                let value = value.as_i64().unwrap();

                if let Some(max) = field.max && value > max.try_into().unwrap() {
                    return errors.push(format!(
                        "The value of {} is too big. Max value is {}",
                        field.name, max
                    ));
                }
                if let Some(min) = field.min && value < min.try_into().unwrap() {
                    return errors.push(format!(
                        "The value of {} is too small. Min value is {}",
                        field.name, min
                    ));
                }

                table_values.push((&field.name, value.into()));
            }
            None => {
                if field.is_required {
                    errors.push(format!("{} is required", field.name))
                }
            }
        }
    });
    custom_table.boolean_fields.iter().for_each(|field| {
        let value = match body.get(&field.name) {
            Some(value) => value.as_bool().unwrap(),
            None => false,
        };

        table_values.push((&field.name, value.into()));
    });
    custom_table.date_fields.iter().for_each(|field| {
        let value = body.get(&field.name);

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
                    errors.push(format!("{} is required", field.name))
                }
            }
        }
    });
    custom_table.email_fields.iter().for_each(|field| {
        let value = body.get(&field.name);

        match value {
            Some(value) => {
                let value: Vec<_> = value
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect();

                // if let Some(except) = field.except {

                // }

                table_values.push((
                    &field.name,
                    SimpleExpr::Value(Value::Array(
                        ArrayType::String,
                        Some(Box::new(
                            value
                                .into_iter()
                                .map(|v| Value::String(Some(Box::new(v.into()))))
                                .collect(),
                        )),
                    )),
                ));
            }
            None => {
                if field.is_required {
                    errors.push(format!("{} is required", field.name))
                }
            }
        }
    });
    custom_table.url_fields.iter().for_each(|field| {
        let value = body.get(&field.name);

        match value {
            Some(value) => {
                let value: Vec<_> = value
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect();

                table_values.push((
                    &field.name,
                    SimpleExpr::Value(Value::Array(
                        ArrayType::String,
                        Some(Box::new(
                            value
                                .into_iter()
                                .map(|v| Value::String(Some(Box::new(v.into()))))
                                .collect(),
                        )),
                    )),
                ));
            }
            None => {
                if field.is_required {
                    errors.push(format!("{} is required", field.name))
                }
            }
        }
    });
    // custom_table.select_fields.iter().for_each(|field| {
    //     let value = body.get(&field.name).unwrap().as_array().unwrap();
    //     table_values.insert(field.name.clone(), serde_json::to_value(value).unwrap());
    // });
    // custom_table.relation_fields.iter().for_each(|field| {
    //     let value = body.get(&field.name).unwrap().as_str().unwrap();
    //     table_values.insert(field.name.clone(), serde_json::to_value(value).unwrap());
    // });

    if !errors.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "errors": errors
        })));
    }

    println!("{:#?}", table_values);

    let query = sea_query::Query::insert()
        .into_table(Alias::new(&custom_table.name))
        .columns(table_values.iter().map(|v| Alias::new(v.0)))
        .values_panic(table_values.iter().map(|v| v.1.clone()))
        .to_string(PostgresQueryBuilder);

    db_pool
        .get()
        .await
        .unwrap()
        .execute(query.as_str(), &[])
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
                    Value::String(value) => Some(json!({ camel_case_name: value.unwrap() })),
                    Value::Int(value) => Some(json!({ camel_case_name: value.unwrap() })),
                    Value::Bool(value) => Some(json!({ camel_case_name: value.unwrap() })),
                    Value::ChronoDateTimeUtc(value) => {
                        Some(json!({ camel_case_name: value.unwrap() }))
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
