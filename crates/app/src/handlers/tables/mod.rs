use adrastos_core::{
    db::postgres,
    entities::custom_table::{
        fields::Field,
        mm_relation::ManyToManyRelationTable,
        permissions::Permissions,
        schema::{CustomTableSchema, UpdateCustomTableSchema},
    },
    error::Error,
    id::Id,
};
use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::Utc;
use heck::ToSnakeCase;
use regex::Regex;
use sea_query::{Alias, PostgresQueryBuilder, Table, TableCreateStatement};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    middleware::extractors::{AnyUser, ProjectDatabase},
    state::AppState,
};

pub mod custom;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBody {
    name: String,
    fields: Vec<Field>,
    permissions: Permissions,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "action")]
enum Action {
    Create { field: Field },
    Update { field: Field },
    Delete,
}

#[derive(Deserialize, Debug)]
struct UpdateField {
    name: String,
    #[serde(flatten)]
    action: Action,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBody {
    name: Option<String>,
    fields: Option<Vec<UpdateField>>,
    permissions: Option<Permissions>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/create", post(create))
        .route("/update/:name", patch(update))
        .route("/delete/:name", delete(remove))
        .nest("/:name", custom::routes())
}

pub async fn list(
    _: AnyUser,
    ProjectDatabase(db): ProjectDatabase,
) -> Result<impl IntoResponse, Error> {
    let tables = CustomTableSchema::find().all(&db).await?;
    Ok(Json(tables))
}

pub async fn create(
    _: AnyUser,
    ProjectDatabase(db): ProjectDatabase,
    Json(body): Json<CreateBody>,
) -> Result<impl IntoResponse, Error> {
    let custom_table = CustomTableSchema {
        id: Id::new().to_string(),
        name: body.name.to_snake_case(),
        fields: body
            .fields
            .into_iter()
            .map(|f| Field {
                name: f.name.to_snake_case(),
                info: f.info,
            })
            .collect(),
        permissions: body.permissions,
        created_at: Utc::now(),
        updated_at: None,
    };

    let found_table = CustomTableSchema::find()
        .by_name(custom_table.name.clone())
        .one(&db)
        .await;
    if found_table.is_ok() {
        return Err(Error::BadRequest(
            "A table with this name already exists".into(),
        ));
    }

    db.get()
        .await
        .unwrap()
        .execute(
            TableCreateStatement::from(&custom_table)
                .to_string(PostgresQueryBuilder)
                .as_str(),
            &[],
        )
        .await
        .map_err(|error| {
            let Some(db_error) = error.as_db_error() else {
                return Error::InternalServerError("Unable to convert error".to_string());
            };
            let Some(routine) = db_error.routine() else {
                return Error::InternalServerError("Unable to get error info".to_string());
            };
            let Some(error) = postgres::Error::try_from(routine).ok() else {
                return Error::InternalServerError("Unsupported database error code".to_string());
            };

            match error {
                postgres::Error::NonExistentTable => {
                    let pre = Regex::new(r#"".+""#).unwrap();

                    let Some(matched) = pre.find(db_error.message()) else {
                        return Error::InternalServerError("Invalid error details".to_string());
                    };

                    let table_name = matched.as_str().replace('\"', "");

                    Error::BadRequest(format!("Table '{}' doesn't exist", table_name))
                }
                _ => todo!(),
            }
        })?;

    custom_table.create(&db).await?;

    for query in ManyToManyRelationTable::create_queries(&custom_table) {
        db.get()
            .await
            .unwrap()
            .execute(query.to_string(PostgresQueryBuilder).as_str(), &[])
            .await
            .unwrap();
    }

    Ok(Json(custom_table))
}

pub async fn update(
    _: AnyUser,
    Path(path): Path<String>,
    ProjectDatabase(db): ProjectDatabase,
    Json(body): Json<UpdateBody>,
) -> Result<impl IntoResponse, Error> {
    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db)
        .await?;

    let mut altered = false;
    let mut alter_query = Table::alter();
    let mut update = UpdateCustomTableSchema {
        ..Default::default()
    };

    if let Some(name) = body.name {
        if name != custom_table.name {
            let found_table = CustomTableSchema::find()
                .by_name(name.clone())
                .one(&db)
                .await;
            if found_table.is_ok() {
                return Err(Error::BadRequest(
                    "A table with this name already exists".into(),
                ));
            }

            update.name = Some(name);
        }
    }
    if let Some(fields) = body.fields {
        let mut updated_fields = custom_table.fields.clone();

        fields.iter().for_each(|update| match &update.action {
            Action::Create { field } => {
                updated_fields.push(field.clone());
                alter_query.add_column(&mut field.column());
                altered = true;
            }
            Action::Update { field } => {
                updated_fields = updated_fields
                    .clone()
                    .into_iter()
                    .map(|f| {
                        if f.name == update.name {
                            return field.clone();
                        }

                        f
                    })
                    .collect();

                if update.name != field.name {
                    alter_query.rename_column(Alias::new(&update.name), Alias::new(&field.name));
                    altered = true;
                }
            }
            Action::Delete => {
                updated_fields = updated_fields
                    .clone()
                    .into_iter()
                    .filter(|f| f.name != update.name)
                    .collect();
                alter_query.drop_column(Alias::new(&update.name));
                altered = true;
            }
        });

        update.fields = Some(updated_fields);
    }

    update.permissions = body.permissions;
    custom_table.update(&db, update.clone()).await?;

    if altered {
        db.get()
            .await
            .unwrap()
            .execute(
                alter_query
                    .table(Alias::new(&custom_table.name))
                    .to_string(PostgresQueryBuilder)
                    .as_str(),
                &[],
            )
            .await
            .unwrap();
    }

    if let Some(name) = update.name {
        db.get()
            .await
            .unwrap()
            .execute(
                Table::rename()
                    .table(Alias::new(&custom_table.name), Alias::new(name.clone()))
                    .to_string(PostgresQueryBuilder)
                    .as_str(),
                &[],
            )
            .await
            .unwrap();
    }

    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db)
        .await?;

    Ok(Json(custom_table))
}

pub async fn remove(
    _: AnyUser,
    Path(path): Path<String>,
    ProjectDatabase(db): ProjectDatabase,
) -> Result<impl IntoResponse, Error> {
    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db)
        .await?;

    custom_table.delete(&db).await?;

    db.get()
        .await
        .unwrap()
        .execute(
            Table::drop()
                .table(Alias::new(custom_table.name))
                .to_string(PostgresQueryBuilder)
                .as_str(),
            &[],
        )
        .await
        .unwrap();

    Ok(Json(Value::Null))
}
