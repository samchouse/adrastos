use std::future::{ready, Ready};

use actix_web::{FromRequest, HttpMessage};
use adrastos_core::entities;

#[derive(Debug)]
pub struct Project(Option<entities::Project>);

impl std::ops::Deref for Project {
    type Target = Option<entities::Project>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Project {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let value = req.extensions().get::<entities::Project>().cloned();
        ready(Ok(Project(value)))
    }
}

#[derive(Debug)]
pub struct RequiredProject(entities::Project);

impl std::ops::Deref for RequiredProject {
    type Target = entities::Project;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for RequiredProject {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let value = req.extensions().get::<entities::Project>().cloned();
        let result = match value {
            Some(v) => Ok(RequiredProject(v)),
            None => {
                Err(adrastos_core::error::Error::BadRequest("Missing project ID".into()).into())
            }
        };

        ready(result)
    }
}
