use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        header::{self, HeaderName, HeaderValue},
        Method,
    },
    Error, HttpMessage, HttpResponse,
};
use adrastos_core::{config, entities};
use futures_util::future::LocalBoxFuture;
use oauth2::http::Uri;

pub struct Cors {
    pub config: config::Config,
}

impl<S, B> Transform<S, ServiceRequest> for Cors
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = Middleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(Middleware {
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct Middleware<S> {
    service: Rc<S>,
    config: config::Config,
}

impl<S> Middleware<S> {
    fn all_headers(
        req: &ServiceRequest,
        config: config::Config,
        preflight: bool,
    ) -> Vec<(HeaderName, HeaderValue)> {
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
            return vec![];
        }

        let default_headers =
            "Origin, Access-Control-Request-Method, Access-Control-Request-Headers";

        let mut headers = vec![
            (
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_str(&hostnames.join(", ")).unwrap(),
            ),
            (
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                HeaderValue::from_static("true"),
            ),
            (
                header::VARY,
                match req.headers().get(header::VARY) {
                    Some(hdr) => HeaderValue::from_str(&format!(
                        "{}, {}",
                        hdr.to_str().unwrap(),
                        default_headers
                    ))
                    .unwrap(),
                    None => HeaderValue::from_static(default_headers),
                },
            ),
        ];

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

        headers.push((
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_str(
                &methods
                    .iter()
                    .map(|m| m.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
            .unwrap(),
        ));

        if let Some(req_headers) = req.headers().get(header::ACCESS_CONTROL_REQUEST_HEADERS) {
            headers.push((header::ACCESS_CONTROL_ALLOW_HEADERS, req_headers.clone()));
        }

        headers
    }
}

impl<S, B> Service<ServiceRequest> for Middleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();

        Box::pin(async move {
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
                let mut res = HttpResponse::Ok();

                let headers = Self::all_headers(&req, config, true);
                headers.into_iter().for_each(|header| {
                    res.insert_header(header);
                });

                return Ok(req.into_response(res).map_into_right_body());
            }

            let headers = Self::all_headers(&req, config, false);

            let mut res = service.call(req).await?;
            let res_headers = res.headers_mut();

            headers.into_iter().for_each(|(name, value)| {
                res_headers.insert(name, value);
            });

            Ok(res.map_into_left_body())
        })
    }
}
