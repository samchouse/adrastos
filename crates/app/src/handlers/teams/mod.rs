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

pub mod projects;

#[derive(Serialize, Deserialize)]
pub struct CreateBody {
    name: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/create", post(create))
        .route("/delete/:id", delete(remove))
        .route("/projects/:id", get(projects::get_by_id))
        .nest("/:team_id/projects", projects::routes())
}

pub async fn list(
    _: SystemUser,
    SystemDatabase(db): SystemDatabase,
) -> Result<impl IntoResponse, Error> {
    let teams = entities::Team::find().all(&db).await?;
    Ok(Json(teams))
}

pub async fn create(
    _: SystemUser,
    SystemDatabase(db): SystemDatabase,
    Json(body): Json<CreateBody>,
) -> Result<impl IntoResponse, Error> {
    let team = entities::Team {
        id: Id::new().to_string(),
        name: body.name.clone(),
        created_at: Utc::now(),
        ..Default::default()
    };

    team.create(&db).await?;
    Ok(Json(team))
}

pub async fn remove(
    _: SystemUser,
    Path(id): Path<String>,
    SystemDatabase(db): SystemDatabase,
) -> Result<impl IntoResponse, Error> {
    entities::Team::find_by_id(&id)
        .one(&db)
        .await?
        .delete(&db)
        .await?;
    Ok(Json(Value::Null))
}
