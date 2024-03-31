use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use adrastos_core::{
    db::postgres,
    entities::custom_table::{
        fields::Field,
        mm_relation::ManyToManyRelationTable,
        schema::{CustomTableSchema, UpdateCustomTableSchema},
    },
    error::Error,
    id::Id,
};
use chrono::Utc;
use heck::AsSnakeCase;
use regex::Regex;
use sea_query::{Alias, PostgresQueryBuilder, Table, TableCreateStatement};
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;

use crate::middleware::{database::ProjectDatabase, user::RequiredAnyUser};

pub mod custom;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBody {
    name: String,
    fields: Vec<Field>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum Action {
    Create,
    Update,
    Delete,
}

#[derive(Deserialize, Debug)]
struct UpdateField {
    name: String,
    field: Field,
    action: Action,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBody {
    name: Option<String>,
    fields: Option<Vec<UpdateField>>,
}

#[get("/list")]
pub async fn list(
    _: RequiredAnyUser,
    db: ProjectDatabase,
) -> actix_web::Result<impl Responder, Error> {
    let tables = CustomTableSchema::find().all(&db).await?;
    Ok(HttpResponse::Ok().json(tables))
}

#[utoipa::path(path = "/tables")]
#[post("/create")]
pub async fn create(
    _: RequiredAnyUser,
    db: ProjectDatabase,
    body: web::Json<CreateBody>,
) -> actix_web::Result<impl Responder, Error> {
    let body = body.into_inner();

    let custom_table = CustomTableSchema {
        id: Id::new().to_string(),
        name: AsSnakeCase(body.name).to_string(),
        fields: body
            .fields
            .into_iter()
            .map(|f| Field {
                name: AsSnakeCase(f.name).to_string(),
                info: f.info,
            })
            .collect(),
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

    Ok(HttpResponse::Ok().json(custom_table))
}

#[patch("/update/{name}")]
pub async fn update(
    _: RequiredAnyUser,
    db: ProjectDatabase,
    path: web::Path<String>,
    body: web::Json<UpdateBody>,
) -> actix_web::Result<impl Responder, Error> {
    let body = body.into_inner();

    let custom_table = CustomTableSchema::find()
        .by_name(path.clone())
        .one(&db)
        .await?;

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

        fields.iter().for_each(|update| match update.action {
            Action::Create => {
                updated_fields.push(update.field.clone());
            }
            Action::Update => {
                updated_fields = updated_fields
                    .clone()
                    .into_iter()
                    .map(|f| {
                        if f.name == update.name {
                            return update.field.clone();
                        }

                        f
                    })
                    .collect();

                if update.name != update.field.name {
                    alter_query
                        .rename_column(Alias::new(&update.name), Alias::new(&update.field.name));
                }
            }
            Action::Delete => {
                updated_fields = updated_fields
                    .clone()
                    .into_iter()
                    .filter(|f| f.name != update.name)
                    .collect();
            }
        });

        update.fields = Some(updated_fields);
    }

    custom_table.update(&db, update.clone()).await?;

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

    Ok(HttpResponse::Ok().json(custom_table))
}

#[delete("/delete/{name}")]
pub async fn delete(
    _: RequiredAnyUser,
    db: ProjectDatabase,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder, Error> {
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

    Ok(HttpResponse::Ok().json(Value::Null))
}
