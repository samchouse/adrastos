use actix_web::{get, web, HttpResponse, Responder};

use crate::handlers::Error;

#[get("/rows")]
pub async fn rows(path: web::Path<String>) -> actix_web::Result<impl Responder, Error> {
    let table_name = path.into_inner();

    Ok(HttpResponse::Ok().json(table_name))
}
