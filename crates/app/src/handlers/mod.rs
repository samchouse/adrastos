use adrastos_core::{
    db::postgres::DatabaseType,
    entities::{SystemUser, User},
    error::Error,
};
use axum::{
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    Json,
};

use crate::{
    assets::handle_embedded_file,
    middleware::extractors::{AnyUser, Config, Database},
};

pub mod auth;
pub mod config;
pub mod storage;
pub mod tables;
pub mod teams;

pub async fn root(uri: Uri) -> impl IntoResponse {
    handle_embedded_file(uri.path())
}

pub async fn api(Config(config): Config) -> impl IntoResponse {
    (
        StatusCode::PERMANENT_REDIRECT,
        [
            (header::LOCATION, config.client_url),
            (header::CACHE_CONTROL, "no-cache".into()),
        ],
    )
}

pub async fn me(
    Database(db): Database,
    AnyUser(user, _): AnyUser,
) -> Result<impl IntoResponse, Error> {
    match db.1 {
        DatabaseType::System => {
            let system_user = SystemUser::find_by_id(&user.id).one(&db).await?;
            Ok(Json(system_user).into_response())
        }
        DatabaseType::Project(_) => {
            let user = User::find_by_id(&user.id).one(&db).await?;
            Ok(Json(user).into_response())
        }
    }
}
