use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{delete, get, http::header, post, web, HttpResponse, Responder};
use adrastos_core::{config::Config, entities::Upload, error::Error, id::Id, s3::S3};
use chrono::Utc;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::middleware::{database::ProjectDatabase, project::RequiredProject, user::RequiredUser};

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "files[]")]
    files: Vec<TempFile>,
}

#[get("/get/{user_id}/{id}/{name}")]
pub async fn get_upload(
    s3: web::Data<S3>,
    db: ProjectDatabase,
    project: RequiredProject,
    path: web::Path<(String, String, String)>,
) -> actix_web::Result<impl Responder, Error> {
    let (user_id, id, name) = path.into_inner();
    Upload::find_by_id(&id)
        .by_name(name.clone())
        .one(&db)
        .await?;

    let file = s3.get(format!("{}/{}/{}", project.id, user_id, id)).await?;
    Ok(HttpResponse::Ok()
        .append_header(header::ContentDisposition {
            disposition: if file.content_type.clone().unwrap().starts_with("image/")
                || file.content_type.clone().unwrap().starts_with("video/")
            {
                header::DispositionType::Inline
            } else {
                header::DispositionType::Attachment
            },
            parameters: vec![header::DispositionParam::Filename(name)],
        })
        .content_type(file.content_type.unwrap())
        .body(file.body.collect().await.unwrap().into_bytes()))
}

#[post("/upload")]
pub async fn upload(
    s3: web::Data<S3>,
    user: RequiredUser,
    db: ProjectDatabase,
    project: RequiredProject,
    config: web::Data<RwLock<Config>>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> actix_web::Result<impl Responder, Error> {
    for temp_file in form.files.iter() {
        let id = Id::new().to_string();

        s3.upload(
            format!("{}/{}/{}", project.id, user.id, id),
            temp_file.content_type.clone().unwrap().to_string(),
            temp_file.file.path(),
        )
        .await?;

        Upload {
            id,
            name: temp_file.file_name.clone().unwrap(),
            created_at: Utc::now(),
        }
        .create(&db)
        .await?
    }

    Ok(HttpResponse::Ok().json(Value::Null))
}

#[delete("/delete/{id}")]
pub async fn delete(
    s3: web::Data<S3>,
    user: RequiredUser,
    db: ProjectDatabase,
    path: web::Path<String>,
    project: RequiredProject,
) -> actix_web::Result<impl Responder, Error> {
    s3.delete(format!("{}/{}/{}", project.id, user.id, path))
        .await?;

    Ok(HttpResponse::Ok().json(Value::Null))
}
