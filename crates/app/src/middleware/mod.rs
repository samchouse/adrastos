use std::sync::Arc;

use adrastos_core::{
    auth::TokenType,
    config::Config,
    db::postgres::{Database, DatabaseType},
    entities::{self, System, SystemUserJoin, UserJoin},
};
use axum::{
    extract::{Query, Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::Deserialize;
use tracing_unwrap::{OptionExt, ResultExt};

use crate::state::{AppState, Flag};

pub mod cors;
pub mod extractors;
pub mod size_limiter;
pub mod trace;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReqParams {
    auth: Option<String>,
    project_id: Option<String>,
}

pub async fn run(
    Query(req_params): Query<ReqParams>,
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    State(AppState {
        config,
        databases,
        flags,
        ..
    }): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let flags = flags
        .iter()
        .find(|flag| req.uri().path().starts_with(&flag.0))
        .cloned()
        .map(|f| f.1)
        .unwrap_or_default();

    let project_id = req
        .headers()
        .get("x-project-id")
        .map(|h| h.to_str().unwrap().to_string())
        .or(if flags.contains(&Flag::AllowProjectIdParam) {
            req_params.project_id
        } else {
            None
        });

    let system_db = databases.get(&DatabaseType::System, &config).await;
    let db_type = match project_id {
        Some(project_id) => {
            match entities::Project::find_by_id(&project_id)
                .one(&system_db)
                .await
                .ok()
            {
                Some(project) => {
                    req.extensions_mut().insert::<entities::Project>(project);

                    Some(DatabaseType::Project(project_id))
                }
                None => None,
            }
        }
        None => Some(DatabaseType::System),
    };

    if let Some(db_type) = db_type {
        let db = databases.get(&db_type, &config).await;

        let mut updated_config = config.clone();
        updated_config.attach_system(
            &db.get()
                .await
                .unwrap_or_log()
                .query(&System::get(), &[])
                .await
                .unwrap_or_log()
                .into_iter()
                .next()
                .unwrap_or_log()
                .into(),
        );
        req.extensions_mut().insert::<Config>(updated_config);

        req.extensions_mut()
            .insert::<Database>(Database(system_db.clone(), DatabaseType::System));
        req.extensions_mut()
            .insert::<(Arc<deadpool_postgres::Pool>, DatabaseType)>((db.clone(), db_type.clone()));

        let authorization = authorization.map(|auth| auth.token().to_string()).or({
            if flags.contains(&Flag::AllowAuthParam) {
                req_params.auth
            } else {
                None
            }
        });

        if let Some(token) = authorization {
            if let Ok(access_token) = TokenType::verify(&config, token) {
                match db_type {
                    DatabaseType::Project(_) => {
                        if let Ok(user) = entities::User::find_by_id(&access_token.claims.sub)
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

    next.run(req).await
}
