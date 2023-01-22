use serde::Serialize;
pub mod auth;

#[derive(Serialize)]
struct Error {
    message: String,
}
