use actix_web::{get, HttpResponse, Responder};

#[get("/auth/token/refresh")]
pub async fn refresh() -> impl Responder {
    HttpResponse::Ok().finish()
}
