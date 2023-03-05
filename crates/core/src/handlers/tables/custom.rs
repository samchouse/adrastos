use std::collections::HashMap;

use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::json;

use crate::{
    entities::custom_table::{CustomSelectBuilder, CustomTable},
    handlers::Error,
};

#[get("/row/{row_id}")]
pub async fn rows(
    path: web::Path<(String, String)>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let (table_name, row_id) = path.into_inner();

    let custom_table = CustomTable::select()
        .by_name(&table_name)
        .finish(&db_pool)
        .await?;

    let row = CustomSelectBuilder::from(&custom_table)
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
    .unwrap();

    let custom_table = CustomTable::select()
        .by_name(&path.into_inner())
        .finish(&db_pool)
        .await?;

    let mut table_values = HashMap::new();

    custom_table.string_fields.iter().for_each(|field| {
        let value = body.get(&field.name).unwrap().as_str().unwrap();
        table_values.insert(field.name.clone(), serde_json::to_value(value).unwrap());
    });

    println!("{:#?}", table_values);

    Ok(HttpResponse::Ok().json(json!({})))
}
