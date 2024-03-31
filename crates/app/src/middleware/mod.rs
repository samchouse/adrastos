use std::{
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::Header,
    web, Error, HttpMessage,
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use adrastos_core::{
    auth::TokenType,
    config,
    db::postgres::{Database, DatabaseType, Databases},
    entities::{self, SystemUserJoin, UserJoin},
};
use futures_util::future::LocalBoxFuture;
use serde::Deserialize;

pub use self::cors::Cors;

pub mod cors;
pub mod database;
pub mod project;
pub mod user;

#[derive(Deserialize, Debug)]
struct ReqParams {
    auth: Option<String>,
}

pub struct Config {
    pub config: config::Config,
    pub databases: Arc<Databases>,
}

impl<S, B> Transform<S, ServiceRequest> for Config
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = Middleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(Middleware {
            service: Rc::new(service),
            config: self.config.clone(),
            databases: self.databases.clone(),
        }))
    }
}

pub struct Middleware<S> {
    service: Rc<S>,
    config: config::Config,
    databases: Arc<Databases>,
}

impl<S, B> Service<ServiceRequest> for Middleware<S>
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
        let databases = self.databases.clone();
        let authorization = Authorization::<Bearer>::parse(&req);

        Box::pin(async move {
            let system_db = databases.get(&DatabaseType::System, &config).await;
            let db_type = match req.headers().get("X-Project-Id") {
                Some(project_id) => {
                    match entities::Project::find_by_id(project_id.to_str().unwrap())
                        .one(&system_db)
                        .await
                        .ok()
                    {
                        Some(project) => {
                            req.extensions_mut().insert::<entities::Project>(project);

                            Some(DatabaseType::Project(
                                project_id.to_str().unwrap().to_string(),
                            ))
                        }
                        None => None,
                    }
                }
                None => Some(DatabaseType::System),
            };

            if let Some(db_type) = db_type {
                let db = databases.get(&db_type, &config).await;

                req.extensions_mut()
                    .insert::<Database>(Database(system_db.clone(), DatabaseType::System));
                req.extensions_mut()
                    .insert::<(Arc<deadpool_postgres::Pool>, DatabaseType)>((
                        db.clone(),
                        db_type.clone(),
                    ));

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
                        match db_type {
                            DatabaseType::Project(_) => {
                                if let Ok(user) =
                                    entities::User::find_by_id(&access_token.claims.sub)
                                        .join(UserJoin::Connections)
                                        .join(UserJoin::RefreshTokenTrees)
                                        .join(UserJoin::Passkeys)
                                        .one(&db)
                                        .await
                                {
                                    req.extensions_mut().insert::<entities::User>(user.clone());
                                    req.extensions_mut()
                                        .insert::<entities::AnyUser>(user.into());
                                }

                                if let Ok(system_user) =
                                    entities::SystemUser::find_by_id(&access_token.claims.sub)
                                        .join(SystemUserJoin::Connections)
                                        .join(SystemUserJoin::RefreshTokenTrees)
                                        .join(SystemUserJoin::Passkeys)
                                        .one(&system_db)
                                        .await
                                {
                                    req.extensions_mut()
                                        .insert::<entities::SystemUser>(system_user.clone());
                                    req.extensions_mut()
                                        .insert::<entities::AnyUser>(system_user.into());
                                }
                            }
                            DatabaseType::System => {
                                if let Ok(system_user) =
                                    entities::SystemUser::find_by_id(&access_token.claims.sub)
                                        .join(SystemUserJoin::Connections)
                                        .join(SystemUserJoin::RefreshTokenTrees)
                                        .join(SystemUserJoin::Passkeys)
                                        .one(&db)
                                        .await
                                {
                                    req.extensions_mut()
                                        .insert::<entities::SystemUser>(system_user.clone());
                                    req.extensions_mut()
                                        .insert::<entities::AnyUser>(system_user.into());
                                }
                            }
                        }
                    }
                }
            }

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}
