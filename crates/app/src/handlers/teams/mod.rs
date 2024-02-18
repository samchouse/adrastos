use actix_web::{delete, get, post, web, HttpResponse, Responder};
use adrastos_core::{entities::Team, error::Error, id::Id};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::middleware::{database::SystemDatabase, user::RequiredSystemUser};

pub mod projects;

#[derive(Serialize, Deserialize)]
struct CreateBody {
    name: String,
}

#[get("/list")]
pub async fn list(
    db: SystemDatabase,
    _: RequiredSystemUser,
) -> actix_web::Result<impl Responder, Error> {
    let teams = Team::find().all(&db).await?;
    Ok(HttpResponse::Ok().json(teams))
}

#[post("/create")]
pub async fn create(
    db: SystemDatabase,
    _: RequiredSystemUser,
    body: web::Json<CreateBody>,
) -> actix_web::Result<impl Responder, Error> {
    let team = Team {
        id: Id::new().to_string(),
        name: body.name.clone(),
        created_at: Utc::now(),
        ..Default::default()
    };

    team.create(&db).await?;
    Ok(HttpResponse::Ok().json(team))
}

#[delete("/delete/{id}")]
pub async fn delete(
    db: SystemDatabase,
    _: RequiredSystemUser,
    id: web::Path<String>,
) -> actix_web::Result<impl Responder, Error> {
    Team::find_by_id(&id).one(&db).await?.delete(&db).await?;
    Ok(HttpResponse::Ok().json(Value::Null))
}
