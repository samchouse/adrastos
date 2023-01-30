use std::fmt::{self, Debug};

use actix_web::{body, error, HttpResponse};
use reqwest::StatusCode;
use serde::Serialize;
use serde_json::json;
use validator::ValidationErrors;

pub mod auth;

#[derive(Debug, Serialize)]
pub enum Error {
    Unauthorized,
    Forbidden {
        message: String,
    },
    BadRequest {
        message: String,
    },
    InternalServerError {
        error: String,
    },
    ValidationErrors {
        message: String,
        errors: ValidationErrors,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden { .. } => "Forbidden",
            Self::BadRequest { .. } => "Bad Request",
            Self::ValidationErrors { .. } => "Validation Errors",
            Self::InternalServerError { .. } => "Internal Server Error",
        };

        write!(f, "{name}")
    }
}

impl error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse<body::BoxBody> {
        let mut response = json!({
            "success": false,
            "status": self.to_string(),
        });

        let patch = match self {
            Self::Unauthorized => json!({ "message": "User not authenticated" }),
            Self::Forbidden { message } => json!({ "message": message }),
            Self::BadRequest { message } => json!({ "message": message }),
            Self::InternalServerError { error } => json!({ "error": error }),
            Self::ValidationErrors { message, errors } => {
                json!({ "message": message, "errors": errors })
            }
        };

        json_patch::merge(&mut response, &patch);

        HttpResponse::build(self.status_code()).json(response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden { .. } => StatusCode::FORBIDDEN,
            Self::BadRequest { .. } => StatusCode::BAD_REQUEST,
            Self::ValidationErrors { .. } => StatusCode::BAD_REQUEST,
            Self::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
