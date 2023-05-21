use adrastos_core::error::Error;

pub mod auth;
pub mod tables;

pub async fn not_found() -> actix_web::Result<String, Error> {
    Err(Error::NotFound)
}
