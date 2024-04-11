use std::future::{ready, Ready};

use actix_web::{FromRequest, HttpMessage};
use adrastos_core::config;

#[derive(Debug)]
pub struct Config(config::Config);

impl std::ops::Deref for Config {
    type Target = config::Config;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Config {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let value = req.extensions().get::<config::Config>().cloned().unwrap();
        ready(Ok(Config(value)))
    }
}
