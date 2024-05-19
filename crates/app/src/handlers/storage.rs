use std::io::{Seek, SeekFrom};

use adrastos_core::{
    entities::{SizeUnit, UploadMetadata},
    error::Error,
    id::Id,
};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Path, State},
    http::header,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use chrono::Utc;
use serde_json::{json, Value};
use tempfile::NamedTempFile;
use tokio_util::io::ReaderStream;
use tower::ServiceBuilder;

use crate::{
    middleware::{
        self,
        extractors::{AnyUser, Config, Project, ProjectDatabase},
    },
    state::AppState,
};

#[derive(TryFromMultipart)]
pub struct UploadForm {
    #[form_data(limit = "unlimited", field_name = "files[]")]
    files: Vec<FieldData<NamedTempFile>>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/get/:id/:name", get(get_upload))
        .route(
            "/upload",
            post(upload).layer(
                ServiceBuilder::new()
                    .layer(DefaultBodyLimit::disable())
                    .layer(axum::middleware::from_fn(middleware::size_limiter::run)),
            ),
        )
        .route("/delete/:id", delete(remove))
}

pub async fn list(
    _: AnyUser,
    Project(project): Project,
    ProjectDatabase(db): ProjectDatabase,
    State(AppState { s3, .. }): State<AppState>,
) -> Result<impl IntoResponse, Error> {
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

    Ok(Json(response))
}

pub async fn get_upload(
    // _: AnyUser, // TODO(@Xenfo): re-enable this
    Project(project): Project,
    ProjectDatabase(db): ProjectDatabase,
    State(AppState { s3, .. }): State<AppState>,
    Path((id, name)): Path<(String, String)>,
) -> Result<impl IntoResponse, Error> {
    let upload_meta = UploadMetadata::find_by_id(&id)
        .by_name(name.clone())
        .one(&db)
        .await?;

    let file = s3
        .get(format!("{}/{}/{}", project.id, upload_meta.user_id, id))
        .await?;
    Ok((
        [
            (
                header::CONTENT_DISPOSITION,
                format!(
                    "{}; filename=\"{}\"",
                    if file.content_type.clone().unwrap().starts_with("image/")
                        || file.content_type.clone().unwrap().starts_with("video/")
                    {
                        "inline"
                    } else {
                        "attachement"
                    },
                    name
                ),
            ),
            (header::CONTENT_TYPE, file.content_type.unwrap()),
        ],
        Body::from_stream(ReaderStream::new(file.body.into_async_read())),
    ))
}

pub async fn upload(
    Config(config): Config,
    AnyUser(user, _): AnyUser,
    Project(project): Project,
    ProjectDatabase(db): ProjectDatabase,
    State(AppState { s3, .. }): State<AppState>,
    TypedMultipart(UploadForm { mut files }): TypedMultipart<UploadForm>,
) -> Result<impl IntoResponse, Error> {
    let mut max_file_size = config.max_file_size.unwrap() * 1000 * 1000;
    if config.size_unit.clone().unwrap() == SizeUnit::Gb {
        max_file_size *= 1000
    }

    for file in files.iter_mut() {
        if file.contents.seek(SeekFrom::End(0)).unwrap() as usize > max_file_size as usize {
            return Err(Error::BadRequest(format!(
                "File is too large, max is {} {}",
                config.max_file_size.unwrap(),
                config.size_unit.clone().unwrap()
            )));
        }

        let id = Id::new().to_string();

        s3.upload(
            format!("{}/{}/{}", project.id, user.id, id),
            file.metadata.content_type.clone().unwrap().to_string(),
            file.contents.path(),
        )
        .await?;

        UploadMetadata {
            id,
            user_id: user.id.clone(),
            name: file.metadata.file_name.clone().unwrap(),
            created_at: Utc::now(),
        }
        .create(&db)
        .await?
    }

    Ok(Json(Value::Null))
}

pub async fn remove(
    AnyUser(user, _): AnyUser,
    Path(path): Path<String>,
    Project(project): Project,
    ProjectDatabase(db): ProjectDatabase,
    State(AppState { s3, .. }): State<AppState>,
) -> Result<impl IntoResponse, Error> {
    s3.delete(format!("{}/{}/{}", project.id, user.id, path))
        .await?;

    UploadMetadata::find_by_id(&path)
        .one(&db)
        .await?
        .delete(&db)
        .await?;

    Ok(Json(Value::Null))
}
