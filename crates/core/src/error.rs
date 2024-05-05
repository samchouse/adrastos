use std::fmt;

use axum::{http::StatusCode, response::IntoResponse};
use serde_json::json;
use validator::ValidationErrors;

#[derive(Debug)]
pub enum Error {
    NotFound,
    Unauthorized,
    Forbidden(String),
    BadRequest(String),
    Custom(StatusCode, String),
    InternalServerError(String),
    ValidationErrors {
        message: String,
        errors: ValidationErrors,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::NotFound => "Not Found",
            Self::Custom(..) => "Custom Error",
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden { .. } => "Forbidden",
            Self::BadRequest { .. } => "Bad Request",
            Self::ValidationErrors { .. } => "Validation Errors",
            Self::InternalServerError { .. } => "Internal Server Error",
        };

        write!(f, "{name}")
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let mut response = json!({
            "success": false,
            "status": self.to_string(),
        });

        let (status_code, patch) = match self {
            Self::Forbidden(message) => (StatusCode::FORBIDDEN, json!({ "message": message })),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, json!({ "message": message })),
            Self::Custom(code, message) => (code, json!({ "message": message })),
            Self::InternalServerError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": error }))
            }
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                json!({ "message": "Resource not found" }),
            ),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                json!({ "message": "User not authenticated" }),
            ),
            Self::ValidationErrors { message, errors } => (
                StatusCode::BAD_REQUEST,
                json!({ "message": message, "errors": errors }),
            ),
        };

        json_patch::merge(&mut response, &patch);

        (status_code, axum::Json(patch)).into_response()
    }
}
