use adrastos_core::{config, entities::SizeUnit, error::Error};
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn run(req: Request, next: Next) -> Result<Response, Error> {
    let config = req.extensions().get::<config::Config>().unwrap().clone();

    let mut max_file_size = config.max_file_size.unwrap() * 1000 * 1000;
    if config.size_unit.clone().unwrap() == SizeUnit::Gb {
        max_file_size *= 1000
    }

    if let Some(content_length) = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
    {
        if content_length > (config.max_files.unwrap() * max_file_size + 10 * 1000 * 1000) {
            // TODO(@samchouse): https://github.com/hyperium/hyper/issues/2384
            return Err(Error::Custom(
                StatusCode::PAYLOAD_TOO_LARGE,
                "Payload is too large".into(),
            ));
        }
    }

    Ok(next.run(req).await)
}
