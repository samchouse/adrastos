use std::collections::HashMap;

use actix_web::{delete, patch, post, web, HttpResponse, Responder};
use chrono::Utc;
use heck::AsSnakeCase;
use sea_query::{Alias, PostgresQueryBuilder, Table, TableCreateStatement};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use core::{
    entities::{
        custom_table::schema::{
            BooleanField, CustomTableSchema, CustomTableSchemaIden, DateField, EmailField,
            NumberField, RelationField, SelectField, StringField, UrlField,
        },
        Mutate,
    },
    error::Error,
    id::Id,
};

pub mod custom;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBody {
    name: String,
    string_fields: Option<Vec<StringField>>,
    number_fields: Option<Vec<NumberField>>,
    boolean_fields: Option<Vec<BooleanField>>,
    date_fields: Option<Vec<DateField>>,
    email_fields: Option<Vec<EmailField>>,
    url_fields: Option<Vec<UrlField>>,
    select_fields: Option<Vec<SelectField>>,
    relation_fields: Option<Vec<RelationField>>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBody {
    name: Option<String>,
    // string_fields: Option<Vec<StringField>>,
    // number_fields: Option<Vec<NumberField>>,
    // boolean_fields: Option<Vec<BooleanField>>,
    // date_fields: Option<Vec<DateField>>,
    // email_fields: Option<Vec<EmailField>>,
    // url_fields: Option<Vec<UrlField>>,
    // select_fields: Option<Vec<SelectField>>,
    // relation_fields: Option<Vec<RelationField>>,
}

#[utoipa::path(
    post,
    path = "/tables/create",
    request_body = CreateBody,
    responses(
        (status = 200, description = "", body = Response<CustomTableSchema>),
    )
)]
#[post("/create")]
pub async fn create(
    body: web::Json<CreateBody>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let body = body.into_inner();

    let custom_table = CustomTableSchema {
        id: Id::new().to_string(),
        name: AsSnakeCase(body.name).to_string(),
        string_fields: body
            .string_fields
            .unwrap_or(vec![])
            .into_iter()
            .map(|mut f| {
                f.name = AsSnakeCase(f.name).to_string();
                f
            })
            .collect(),
        number_fields: body
            .number_fields
            .unwrap_or(vec![])
            .into_iter()
            .map(|mut f| {
                f.name = AsSnakeCase(f.name).to_string();
                f
            })
            .collect(),
        boolean_fields: body
            .boolean_fields
            .unwrap_or(vec![])
            .into_iter()
            .map(|mut f| {
                f.name = AsSnakeCase(f.name).to_string();
                f
            })
            .collect(),
        date_fields: body
            .date_fields
            .unwrap_or(vec![])
            .into_iter()
            .map(|mut f| {
                f.name = AsSnakeCase(f.name).to_string();
                f
            })
            .collect(),
        email_fields: body
            .email_fields
            .unwrap_or(vec![])
            .into_iter()
            .map(|mut f| {
                f.name = AsSnakeCase(f.name).to_string();
                f
            })
            .collect(),
        url_fields: body
            .url_fields
            .unwrap_or(vec![])
            .into_iter()
            .map(|mut f| {
                f.name = AsSnakeCase(f.name).to_string();
                f
            })
            .collect(),
        select_fields: body
            .select_fields
            .unwrap_or(vec![])
            .into_iter()
            .map(|mut f| {
                f.name = AsSnakeCase(f.name).to_string();
                f
            })
            .collect(),
        relation_fields: body
            .relation_fields
            .unwrap_or(vec![])
            .into_iter()
            .map(|mut f| {
                f.name = AsSnakeCase(f.name).to_string();
                f
            })
            .collect(),
        created_at: Utc::now(),
        updated_at: None,
    };

    let found_table = CustomTableSchema::select()
        .by_name(&custom_table.name)
        .finish(&db_pool)
        .await;
    if found_table.is_ok() {
        return Err(Error::BadRequest(
            "A table with this name already exists".into(),
        ));
    }

    custom_table.create(&db_pool).await?;

    db_pool
        .get()
        .await
        .unwrap()
        .execute(
            TableCreateStatement::from(&custom_table)
                .to_string(PostgresQueryBuilder)
                .as_str(),
            &[],
        )
        .await
        .unwrap();

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Table created successfully",
        "table": custom_table
    })))
}

#[patch("/update/{name}")]
pub async fn update(
    path: web::Path<String>,
    body: web::Json<UpdateBody>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let body = body.into_inner();

    let custom_table = CustomTableSchema::select()
        .by_name(&path.into_inner())
        .finish(&db_pool)
        .await?;

    let mut updated_table = HashMap::new();

    if let Some(name) = body.name {
        if name != custom_table.name {
            let found_table = CustomTableSchema::select()
                .by_name(&name)
                .finish(&db_pool)
                .await;
            if found_table.is_ok() {
                return Err(Error::BadRequest(
                    "A table with this name already exists".into(),
                ));
            }

            updated_table.insert(CustomTableSchemaIden::Name.to_string(), Value::from(name));
        }
    }

    custom_table.update(&db_pool, updated_table).await?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Table updated successfully",
    })))
}

#[utoipa::path(
    delete,
    path = "/tables/delete/{name}",
    request_body = CreateBody,
    responses(
        (status = 200, description = "", body = CustomTableSchema),
    ),
    params(
        ("name" = String, Path, description = "The name of the table to delete"),
    )
)]
#[delete("/delete/{name}")]
pub async fn delete(
    path: web::Path<String>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let custom_table = CustomTableSchema::select()
        .by_name(&path.into_inner())
        .finish(&db_pool)
        .await?;

    custom_table.delete(&db_pool).await?;

    db_pool
        .get()
        .await
        .unwrap()
        .execute(
            Table::drop()
                .table(Alias::new(custom_table.name))
                .to_string(PostgresQueryBuilder)
                .as_str(),
            &[],
        )
        .await
        .unwrap();

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Table deleted successfully",
    })))
}
