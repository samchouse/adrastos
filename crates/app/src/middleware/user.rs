// TODO(@Xenfo): add admin only middleware

use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::Header,
    web, Error, FromRequest, HttpMessage,
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use adrastos_core::{
    auth::TokenType,
    config,
    entities::{self, UserJoin},
};
use futures_util::future::LocalBoxFuture;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ReqParams {
    auth: Option<String>,
}

pub struct GetUser {
    pub config: config::Config,
    pub db_pool: deadpool_postgres::Pool,
}

impl<S, B> Transform<S, ServiceRequest> for GetUser
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = GetUserMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(GetUserMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
            db_pool: self.db_pool.clone(),
        }))
    }
}

pub struct GetUserMiddleware<S> {
    service: Rc<S>,
    config: config::Config,
    db_pool: deadpool_postgres::Pool,
}

impl<S, B> Service<ServiceRequest> for GetUserMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let config = self.config.clone();
        let service = self.service.clone();
        let db_pool = self.db_pool.clone();
        let authorization = Authorization::<Bearer>::parse(&req);

        Box::pin(async move {
            let authorization = authorization
                .ok()
                .map(|a| a.into_scheme().token().to_owned())
                .or(req
                    .extract::<web::Query<ReqParams>>() // TODO(@Xenfo): should mark this token as used in the database
                    .await
                    .map(|q| q.auth.clone())
                    .ok()
                    .flatten());

            if let Some(token) = authorization {
                if let Ok(access_token) = TokenType::verify(&config, token) {
                    let user = entities::User::find_by_id(&access_token.claims.sub)
                        .join(UserJoin::Connections)
                        .join(UserJoin::RefreshTokenTrees)
                        .join(UserJoin::Passkeys)
                        .one(&db_pool)
                        .await;
                    if let Ok(user) = user {
                        req.extensions_mut().insert::<entities::User>(user);
                    }
                }
            }

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

pub struct User(Option<entities::User>);

impl FromRequest for User {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let value = req.extensions().get::<entities::User>().cloned();
        let result = match value {
            Some(v) => Ok(User(Some(v))),
            None => Ok(User(None)),
        };

        ready(result)
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
        let value = req.extensions().get::<entities::User>().cloned();
        let result = match value {
            Some(v) => Ok(RequiredUser(v)),
            None => Err(adrastos_core::error::Error::Unauthorized.into()),
        };

        ready(result)
    }
}

impl std::ops::Deref for RequiredUser {
    type Target = entities::User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
