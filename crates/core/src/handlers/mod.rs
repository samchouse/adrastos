use actix_web::{post, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use libinjection::sqli;
use sea_query::{Alias, PostgresQueryBuilder, Table};
use serde::Serialize;

#[derive(Serialize)]
struct Error {
    message: String,
}

#[post("/")]
pub async fn index(req_body: String, pool: web::Data<Pool>) -> impl Responder {
    let (is_sqli, _) = sqli(&req_body).unwrap();
    if is_sqli {
        return HttpResponse::BadRequest().json(Error {
            message: "Bad request".to_string(),
        });
    }

    let client = pool.get().await.unwrap();
    client
        .execute(
            &Table::create()
                .table(Alias::new(&req_body))
                .if_not_exists()
                .to_string(PostgresQueryBuilder),
            &[],
        )
        .await
        .unwrap();

    HttpResponse::Ok().body("Hello world!")
}
