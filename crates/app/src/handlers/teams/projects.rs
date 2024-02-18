use actix_web::{delete, get, post, web, HttpResponse, Responder};
use adrastos_core::{entities::Project, error::Error, id::Id};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::middleware::{database::SystemDatabase, user::RequiredSystemUser};

#[derive(Serialize, Deserialize)]
struct DeletePath {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct CreateBody {
    name: String,
    hostnames: Vec<String>,
}

#[get("/projects/{id}")]
pub async fn get(
    db: SystemDatabase,
    _: RequiredSystemUser,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder, Error> {
    let project = Project::find_by_id(&path).one(&db).await?;
    Ok(HttpResponse::Ok().json(project))
}

#[get("/list")]
pub async fn list(
    db: SystemDatabase,
    _: RequiredSystemUser,
) -> actix_web::Result<impl Responder, Error> {
    let projects = Project::find().all(&db).await?;
    Ok(HttpResponse::Ok().json(projects))
}

#[post("/create")]
pub async fn create(
    db: SystemDatabase,
    _: RequiredSystemUser,
    body: web::Json<CreateBody>,
    team_id: web::Path<String>,
) -> actix_web::Result<impl Responder, Error> {
    let project = Project {
        id: Id::new().to_string(),
        name: body.name.clone(),
        hostnames: body.hostnames.clone(),
        team_id: team_id.to_string(),
        created_at: Utc::now(),
        ..Default::default()
    };

    project.create(&db).await?;
    Ok(HttpResponse::Ok().json(project))
}

#[delete("/delete/{id}")]
pub async fn delete(
    db: SystemDatabase,
    _: RequiredSystemUser,
    path: web::Path<DeletePath>,
) -> actix_web::Result<impl Responder, Error> {
    Project::find_by_id(&path.id)
        .one(&db)
        .await?
        .delete(&db)
        .await?;
    Ok(HttpResponse::Ok().json(Value::Null))
}
