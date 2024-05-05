use std::path::PathBuf;

use adrastos_core::error::Error;
use axum::{http::header, response::IntoResponse};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../packages/dashboard/dist"]
pub struct Asset;

pub fn handle_embedded_file(path: &str) -> Result<impl IntoResponse, Error> {
    match Asset::get(path.strip_prefix('/').unwrap_or(path)) {
        Some(content) => Ok((
            [(header::CONTENT_TYPE, content.metadata.mimetype())],
            content.data.clone().into_owned(),
        )
            .into_response()),
        None => {
            if path.starts_with("/api") {
                return Err(Error::NotFound);
            }

            if let Some(path) = path.split('/').last() {
                if PathBuf::from(path).extension().is_some() {
                    return Err(Error::NotFound);
                }
            }

            handle_embedded_file("index.html")
        }
    }
}
