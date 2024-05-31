use adrastos_core::{
    db::postgres,
    entities::custom_table::{
        fields::{Field, FieldInfo, RelationTarget},
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
use sea_query::{
    Alias, Expr, ForeignKeyAction, Order, PostgresQueryBuilder, Table, TableCreateStatement,
    TableForeignKey,
};
use serde::Deserialize;
use serde_json::Value;
use tracing_unwrap::ResultExt;

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
            .unwrap_or_log();
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
    let mut additional_queries = vec![];
    let mut more_queries = vec![];
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
                            if let FieldInfo::Relation {
                                table,
                                target,
                                cascade_delete,
                                ..
                            } = &field.info
                            {
                                let FieldInfo::Relation {
                                    target: old_target, ..
                                } = f.info
                                else {
                                    todo!()
                                };

                                if *target != old_target {
                                    match target {
                                        RelationTarget::Single => {
                                            alter_query.add_column(&mut field.column());
                                            altered = true;

                                            let mut foreign_key = TableForeignKey::new();

                                            if *cascade_delete {
                                                foreign_key.on_delete(ForeignKeyAction::Cascade);
                                            }

                                            alter_query.add_foreign_key(
                                                foreign_key
                                                    .name(format!(
                                                        "FK_{}_{}",
                                                        custom_table.name, field.name
                                                    ))
                                                    .from_tbl(Alias::new(&custom_table.name))
                                                    .from_col(Alias::new(&field.name))
                                                    .to_tbl(Alias::new(table))
                                                    .to_col(Alias::new("id"))
                                                    .on_update(ForeignKeyAction::Cascade),
                                            );

                                            additional_queries.push((
                                                format!(
                                                    "{}_{}_to_{}",
                                                    custom_table.name, field.name, table
                                                ),
                                                format!(
                                                    "WITH LastRelations AS ({}) {} FROM LastRelations WHERE {}.id = LastRelations.{}_id",
                                                    sea_query::Query::select()
                                                        .from(Alias::new(format!(
                                                            "{}_{}_to_{}",
                                                            custom_table.name, field.name, table
                                                        )))
                                                        .distinct_on([Alias::new(format!(
                                                            "{}_id",
                                                            custom_table.name
                                                        ))])
                                                        .columns([
                                                            Alias::new(format!(
                                                                "{}_id",
                                                                custom_table.name
                                                            )),
                                                            Alias::new(format!("{}_id", table)),
                                                        ])
                                                        .order_by_columns([
                                                            (
                                                                Alias::new(format!(
                                                                    "{}_id",
                                                                    custom_table.name
                                                                )),
                                                                Order::Asc
                                                            ),
                                                            (Alias::new("created_at"), Order::Desc)
                                                        ])
                                                        .to_string(PostgresQueryBuilder),
                                                    sea_query::Query::update()
                                                        .table(Alias::new(&custom_table.name))
                                                        .values([(
                                                            Alias::new(&field.name),
                                                            Expr::cust(format!(
                                                                "LastRelations.{}_id",
                                                                table
                                                            )),
                                                        )])
                                                        .to_string(PostgresQueryBuilder),
                                                    custom_table.name,
                                                    custom_table.name
                                                )
                                            ));
                                        }
                                        RelationTarget::Many => {
                                            if let Some(q) = ManyToManyRelationTable::create_query(
                                                &custom_table,
                                                field,
                                            ) {
                                                more_queries
                                                    .push(q.to_string(PostgresQueryBuilder));
                                            }

                                            alter_query.drop_column(Alias::new(&field.name));
                                            alter_query.drop_foreign_key(Alias::new(format!(
                                                "FK_{}_{}",
                                                custom_table.name, field.name
                                            )));
                                            altered = true;

                                            more_queries.push(format!("{} {}",
                                                sea_query::Query::insert()
                                                    .into_table(Alias::new(format!(
                                                        "{}_{}_to_{}",
                                                        custom_table.name, field.name, table
                                                    )))
                                                    .columns(vec![
                                                        Alias::new("id"),
                                                        Alias::new(format!("{}_id", custom_table.name)),
                                                        Alias::new(format!("{}_id", table)),
                                                    ]).to_string(PostgresQueryBuilder),
                                                sea_query::Query::select()
                                                    .from(Alias::new(&custom_table.name))
                                                    .expr(Expr::cust(&format!("'{}'", Id::new())))
                                                    .expr(Expr::col(Alias::new("id")))
                                                    .expr(Expr::col(Alias::new(&field.name)))
                                                    .to_string(PostgresQueryBuilder),
                                            ));
                                        }
                                    }
                                }
                            } else if update.name != field.name {
                                alter_query.rename_column(
                                    Alias::new(&update.name),
                                    Alias::new(&field.name),
                                );
                                altered = true;
                            }

                            return field.clone();
                        }

                        f
                    })
                    .collect();
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

    for query in more_queries {
        db.get()
            .await
            .unwrap()
            .execute(query.as_str(), &[])
            .await
            .unwrap_or_log();
    }

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

    for additional_query in additional_queries {
        db.get()
            .await
            .unwrap()
            .execute(additional_query.1.as_str(), &[])
            .await
            .unwrap();

        db.get()
            .await
            .unwrap()
            .execute(
                sea_query::Table::drop()
                    .table(Alias::new(additional_query.0))
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

    for field in custom_table.fields {
        if let FieldInfo::Relation { table, target, .. } = field.info {
            if matches!(target, RelationTarget::Many) {
                db.get()
                    .await
                    .unwrap()
                    .execute(
                        Table::drop()
                            .table(Alias::new(format!(
                                "{}_{}_to_{}",
                                custom_table.name, field.name, table
                            )))
                            .to_string(PostgresQueryBuilder)
                            .as_str(),
                        &[],
                    )
                    .await
                    .unwrap();
            }
        }
    }

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
