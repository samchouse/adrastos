use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{delete, get, http::header, post, web, HttpResponse, Responder};
use adrastos_core::{
    entities::{SizeUnit, UploadMetadata},
    error::Error,
    id::Id,
    s3::S3,
};
use chrono::Utc;
use serde_json::{json, Value};

use crate::middleware::{
    config::Config, database::ProjectDatabase, project::RequiredProject, user::RequiredAnyUser,
};

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "files[]")]
    files: Vec<TempFile>,
}

#[get("/list")]
pub async fn list(
    s3: web::Data<S3>,
    db: ProjectDatabase,
    project: RequiredProject,
) -> actix_web::Result<impl Responder, Error> {
    let all_metadata = UploadMetadata::find().all(&db).await?;
    let files = s3.list(project.id.clone()).await;

    let mut response = vec![];
    for meta in all_metadata.iter() {
        let path = format!("{}/{}/{}", project.id, meta.user_id, meta.id);
        let content_type = s3.head(path.clone()).await.unwrap().content_type.unwrap();

        let files = &files.as_ref().unwrap().contents;
        let file = files
            .as_ref()
            .unwrap()
            .iter()
            .find(|file| file.key.clone().unwrap() == path)
            .unwrap();

        response.push(json!({
            "id": meta.id,
            "name": meta.name,
            "type": content_type,
            "size": file.size.unwrap(),
            "createdAt": meta.created_at
        }))
    }

    Ok(HttpResponse::Ok().json(response))
}

#[get("/get/{id}/{name}")]
pub async fn get_upload(
    s3: web::Data<S3>,
    db: ProjectDatabase,
    project: RequiredProject,
    path: web::Path<(String, String)>,
) -> actix_web::Result<impl Responder, Error> {
    let (id, name) = path.into_inner();
    let upload_meta = UploadMetadata::find_by_id(&id)
        .by_name(name.clone())
        .one(&db)
        .await?;

    let file = s3
        .get(format!("{}/{}/{}", project.id, upload_meta.user_id, id))
        .await?;
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
    config: Config,
    s3: web::Data<S3>,
    db: ProjectDatabase,
    user: RequiredAnyUser,
    project: RequiredProject,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> actix_web::Result<impl Responder, Error> {
    let mut max_file_size = config.max_file_size.unwrap() * 1000 * 1000;
    if config.size_unit.clone().unwrap() == SizeUnit::Gb {
        max_file_size *= 1000
    }

    for temp_file in form.files.iter() {
        if temp_file.size > max_file_size as usize {
            return Err(Error::BadRequest(format!(
                "File is too large, max is {} {}",
                config.max_file_size.unwrap(),
                config.size_unit.clone().unwrap()
            )));
        }

        let id = Id::new().to_string();

        s3.upload(
            format!("{}/{}/{}", project.id, user.id, id),
            temp_file.content_type.clone().unwrap().to_string(),
            temp_file.file.path(),
        )
        .await?;

        UploadMetadata {
            id,
            user_id: user.id.clone(),
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
    db: ProjectDatabase,
    user: RequiredAnyUser,
    path: web::Path<String>,
    project: RequiredProject,
) -> actix_web::Result<impl Responder, Error> {
    s3.delete(format!("{}/{}/{}", project.id, user.id, path))
        .await?;

    UploadMetadata::find_by_id(&path)
        .one(&db)
        .await?
        .delete(&db)
        .await?;

    Ok(HttpResponse::Ok().json(Value::Null))
}
