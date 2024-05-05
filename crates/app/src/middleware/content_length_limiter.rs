use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{header, StatusCode},
    Error, HttpMessage,
};
use adrastos_core::{config::Config, entities::SizeUnit, error};
use futures_util::{future::LocalBoxFuture, TryStreamExt};

#[derive(Debug)]
pub struct ContentLengthLimiter;

impl<S, B> Transform<S, ServiceRequest> for ContentLengthLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ContentLengthLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ContentLengthLimiterMiddleware {
            service: Rc::new(service),
        }))
    }
}

#[derive(Debug)]
pub struct ContentLengthLimiterMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ContentLengthLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = req.extensions().get::<Config>().cloned().unwrap();

        let mut max_file_size = config.max_file_size.unwrap() * 1000 * 1000;
        if config.size_unit.clone().unwrap() == SizeUnit::Gb {
            max_file_size *= 1000
        }

        Box::pin(async move {
            if let Some(content_length) = req
                .headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<i64>().ok())
            {
                if content_length > (config.max_files.unwrap() * max_file_size + 10 * 1000 * 1000) {
                    // TODO(@Xenfo): change to proper method once https://github.com/actix/actix-web/issues/2695 is fixed
                    let mut payload = req.take_payload();
                    while let Ok(Some(_)) = payload.try_next().await {}

                    return Ok(req.into_response(
                        error::Error::Custom(
                            StatusCode::PAYLOAD_TOO_LARGE,
                            "Payload is too large".into(),
                        )
                        .response()
                        .map_into_right_body(),
                    ));
                }
            }

            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}
