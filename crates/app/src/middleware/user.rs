use std::future::{ready, Ready};

use actix_web::{Error, FromRequest, HttpMessage};
use adrastos_core::{db::postgres::DatabaseType, entities};

pub struct User(Option<entities::User>);

impl FromRequest for User {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let result = match req
            .extensions()
            .get::<(std::sync::Arc<deadpool_postgres::Pool>, DatabaseType)>()
            .cloned()
        {
            Some((_, db_type)) => match db_type {
                DatabaseType::Project(_) => match req.extensions().get::<entities::User>().cloned()
                {
                    Some(v) => Ok(User(Some(v))),
                    None => Ok(User(None)),
                },
                _ => Err(adrastos_core::error::Error::BadRequest(
                    "Missing project ID".into(),
                )),
            },
            None => Err(adrastos_core::error::Error::BadRequest(
                "Invalid project ID".into(),
            )),
        };

        ready(result.map_err(|e| e.into()))
    }
}

impl std::ops::Deref for User {
    type Target = Option<entities::User>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RequiredUser(entities::User);

impl FromRequest for RequiredUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let result = match req
            .extensions()
            .get::<(std::sync::Arc<deadpool_postgres::Pool>, DatabaseType)>()
            .cloned()
        {
            Some((_, db_type)) => match db_type {
                DatabaseType::Project(_) => match req.extensions().get::<entities::User>().cloned()
                {
                    Some(v) => Ok(RequiredUser(v)),
                    None => Err(adrastos_core::error::Error::Unauthorized),
                },
                _ => Err(adrastos_core::error::Error::BadRequest(
                    "Missing project ID".into(),
                )),
            },
            None => Err(adrastos_core::error::Error::BadRequest(
                "Invalid project ID".into(),
            )),
        };

        ready(result.map_err(|e| e.into()))
    }
}

impl std::ops::Deref for RequiredUser {
    type Target = entities::User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct SystemUser(Option<entities::SystemUser>);

impl FromRequest for SystemUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        ready(Ok(req
            .extensions()
            .get::<entities::SystemUser>()
            .cloned()
            .map(|v| SystemUser(Some(v)))
            .unwrap_or(SystemUser(None))))
    }
}

impl std::ops::Deref for SystemUser {
    type Target = Option<entities::SystemUser>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RequiredSystemUser(entities::SystemUser);

impl FromRequest for RequiredSystemUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let result = match req.extensions().get::<entities::SystemUser>().cloned() {
            Some(v) => Ok(RequiredSystemUser(v)),
            None => Err(adrastos_core::error::Error::Unauthorized),
        };

        ready(result.map_err(|e| e.into()))
    }
}

impl std::ops::Deref for RequiredSystemUser {
    type Target = entities::SystemUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct AnyUser(Option<entities::AnyUser>);

impl FromRequest for AnyUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let value = req.extensions().get::<entities::AnyUser>().cloned();
        let result = match value {
            Some(v) => Ok(AnyUser(Some(v))),
            None => Ok(AnyUser(None)),
        };

        ready(result)
    }
}

impl std::ops::Deref for AnyUser {
    type Target = Option<entities::AnyUser>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RequiredAnyUser(entities::AnyUser);

impl FromRequest for RequiredAnyUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let value = req.extensions().get::<entities::AnyUser>().cloned();
        let result = match value {
            Some(v) => Ok(RequiredAnyUser(v)),
            None => Err(adrastos_core::error::Error::Unauthorized.into()),
        };

        ready(result)
    }
}

impl std::ops::Deref for RequiredAnyUser {
    type Target = entities::AnyUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
