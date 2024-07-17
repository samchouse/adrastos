use adrastos_core::{entities, error::Error, id::Id};
use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    middleware::extractors::{SystemDatabase, SystemUser},
    state::AppState,
};

#[derive(Serialize, Deserialize)]
pub struct CreateBody {
    name: String,
    hostnames: Vec<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/create", post(create))
        .route("/delete/:id", delete(remove))
}

pub async fn get_by_id(
    _: SystemUser,
    Path(path): Path<String>,
    SystemDatabase(db): SystemDatabase,
) -> Result<impl IntoResponse, Error> {
    let project = entities::Project::find_by_id(&path).one(&db).await?;
    Ok(Json(project))
}

pub async fn list(
    _: SystemUser,
    SystemDatabase(db): SystemDatabase,
) -> Result<impl IntoResponse, Error> {
    let projects = entities::Project::find().all(&db).await?;
    Ok(Json(projects))
}

pub async fn create(
    _: SystemUser,
    Path(team_id): Path<String>,
    SystemDatabase(db): SystemDatabase,
    Json(body): Json<CreateBody>,
) -> Result<impl IntoResponse, Error> {
    let project = entities::Project {
        id: Id::new().to_string(),
        name: body.name.clone(),
        hostnames: body.hostnames.clone(),
        team_id: team_id.to_string(),
        created_at: Utc::now(),
        ..Default::default()
    };

    project.create(&db).await?;
    Ok(Json(project))
}

pub async fn remove(
    _: SystemUser,
    Path((_, id)): Path<(String, String)>,
    SystemDatabase(db): SystemDatabase,
) -> Result<impl IntoResponse, Error> {
    entities::Project::find_by_id(&id)
        .one(&db)
        .await?
        .delete(&db)
        .await?;
    Ok(Json(Value::Null))
}
