use serde::Serialize;
use utoipa::ToSchema;
pub mod auth;

#[derive(Serialize)]
struct Error {
    message: String,
}
