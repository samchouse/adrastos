use adrastos_core::{config, entities};
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderValue, Method, Uri},
    middleware::Next,
    response::IntoResponse,
};

use crate::state::AppState;

pub async fn run(
    State(AppState { config, .. }): State<AppState>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    if req.method() == Method::OPTIONS
        && req
            .headers()
            .get(header::ACCESS_CONTROL_REQUEST_METHOD)
            .and_then(|hdr| {
                hdr.to_str()
                    .ok()
                    .and_then(|meth| Method::try_from(meth).ok())
            })
            .is_some()
    {
        return all_headers(&req, config, true).into_response();
    }

    let headers = all_headers(&req, config, false);

    let mut res = next.run(req).await;
    res.headers_mut().extend(headers);

    res.into_response()
}

fn all_headers(req: &Request, config: config::Config, preflight: bool) -> HeaderMap {
    let default_hostnames = if let Ok(uri) = Uri::try_from(&config.client_url)
        && uri.host().is_some()
    {
        vec![config.client_url]
    } else {
        vec![]
    };

    let hostnames = req
        .extensions()
        .get::<entities::Project>()
        .map(|p| {
            let mut hostnames = p.hostnames.clone();
            hostnames.append(&mut default_hostnames.clone());
            hostnames
        })
        .unwrap_or(default_hostnames);

    if hostnames.is_empty() {
        return HeaderMap::new();
    }

    let default_headers = "Origin, Access-Control-Request-Method, Access-Control-Request-Headers";

    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_str(&hostnames.join(", ")).unwrap(),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );
    headers.insert(
        header::VARY,
        match req.headers().get(header::VARY) {
            Some(hdr) => {
                HeaderValue::from_str(&format!("{}, {}", hdr.to_str().unwrap(), default_headers))
                    .unwrap()
            }
            None => HeaderValue::from_static(default_headers),
        },
    );

    if !preflight {
        return headers;
    }

    let methods = [
        Method::GET,
        Method::POST,
        Method::PATCH,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ];

    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_str(
            &methods
                .iter()
                .map(|m| m.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        )
        .unwrap(),
    );

    if let Some(req_headers) = req.headers().get(header::ACCESS_CONTROL_REQUEST_HEADERS) {
        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, req_headers.clone());
    }

    headers
}
