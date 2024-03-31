use std::{
    future::{ready, Ready},
    sync::Arc,
};

use actix_web::{FromRequest, HttpMessage};
use adrastos_core::db::postgres::{Database, DatabaseType};

#[derive(Debug)]
pub struct SystemDatabase(Database);

impl std::ops::Deref for SystemDatabase {
    type Target = Arc<deadpool_postgres::Pool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for SystemDatabase {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let value = req.extensions().get::<Database>().cloned();
        let result = match value {
            Some(v) => Ok(SystemDatabase(v)),
            None => Err(adrastos_core::error::Error::Unauthorized.into()),
        };

        ready(result)
    }
}

#[derive(Debug)]
pub struct ProjectDatabase(Database);

impl std::ops::Deref for ProjectDatabase {
    type Target = Arc<deadpool_postgres::Pool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for ProjectDatabase {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let value = req
            .extensions()
            .get::<(Arc<deadpool_postgres::Pool>, DatabaseType)>()
            .cloned();
        let result = match value {
            Some(v) => match v.1 {
                DatabaseType::Project(_) => Ok(ProjectDatabase(Database(v.0, v.1))),
                _ => {
                    Err(adrastos_core::error::Error::BadRequest("Missing project ID".into()).into())
                }
            },
            None => Err(adrastos_core::error::Error::Unauthorized.into()),
        };

        ready(result)
    }
}
