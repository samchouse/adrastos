use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use sea_query::{PostgresQueryBuilder, TableCreateStatement};
use serde::Deserialize;
use serde_json::json;

use crate::{
    entities::{
        custom_table::schema::{
            BooleanField, CustomTableSchema, DateField, EmailField, NumberField, RelationField,
            SelectField, StringField, UrlField,
        },
        Mutate,
    },
    handlers::Error,
    id::Id,
};

pub mod custom;

#[derive(Deserialize)]
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

#[post("/create")]
pub async fn create(
    body: web::Json<CreateBody>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let body = body.into_inner();

    let custom_table = CustomTableSchema {
        id: Id::new().to_string(),
        name: body.name,
        string_fields: body.string_fields.unwrap_or(vec![]),
        number_fields: body.number_fields.unwrap_or(vec![]),
        boolean_fields: body.boolean_fields.unwrap_or(vec![]),
        date_fields: body.date_fields.unwrap_or(vec![]),
        email_fields: body.email_fields.unwrap_or(vec![]),
        url_fields: body.url_fields.unwrap_or(vec![]),
        select_fields: body.select_fields.unwrap_or(vec![]),
        relation_fields: body.relation_fields.unwrap_or(vec![]),
        created_at: Utc::now(),
        updated_at: None,
    };

    let found_table = CustomTableSchema::select()
        .by_name(&custom_table.name)
        .finish(&db_pool)
        .await;
    if found_table.is_ok() {
        return Err(Error::BadRequest {
            message: "A table with this name already exists".into(),
        });
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
