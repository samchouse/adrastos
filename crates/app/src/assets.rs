use std::path::PathBuf;

use actix_web::{HttpResponse, Responder};
use adrastos_core::error::Error;
use mime_guess::from_path;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../packages/dashboard/dist"]
pub struct Asset;

pub fn handle_embedded_file(path: &str) -> Result<impl Responder, Error> {
    match Asset::get(path.strip_prefix('/').unwrap_or(path)) {
        Some(content) => Ok(HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned())),
        None => {
            if let Some(path) = path.split('/').last() {
                if PathBuf::from(path).extension().is_some() {
                    return Err(Error::NotFound);
                }
            }

            handle_embedded_file("index.html")
        }
    }
}
