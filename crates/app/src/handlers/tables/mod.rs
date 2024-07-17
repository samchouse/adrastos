use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use adrastos_core::{
    config,
    db::postgres::{self, Database},
    entities::{
        custom_table::{
            fields::{Field, FieldInfo, RelationTarget},
            mm_relation::ManyToManyRelationTable,
            permissions::Permissions,
            schema::{CustomTableSchema, UpdateCustomTableSchema},
        },
        Historical, WebhookConfig, WebhookProvider,
    },
    error::Error,
    id::Id,
    task_queue::TaskQueue,
};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use heck::ToSnakeCase;
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventPayload, WebhookEventType};
use regex::Regex;
use ring::hmac;
use sea_query::{
    Alias, Expr, ForeignKeyAction, Order, PostgresQueryBuilder, Table, TableCreateStatement,
    TableForeignKey,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use tracing_unwrap::ResultExt;

use crate::{
    middleware::extractors::{AnyUser, Config, ProjectDatabase},
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

/// {
///     "build_timestamp": Date,
///     "permissions": {
///         Name: {
///             "view": String,
///             "create": String,
///             "update": String,
///             "delete": String
///         }
///     }
/// }

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsResponse {
    pub build_timestamp: DateTime<Utc>,
    pub permissions: HashMap<String, Permissions>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/create", post(create))
        .route("/update/:name", patch(update))
        .route("/delete/:name", delete(remove))
        .route("/permissions-webhook", get(permissions_webhook))
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
    Config(config): Config,
    ProjectDatabase(db): ProjectDatabase,
    State(AppState { task_queue, .. }): State<AppState>,
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

    if let Some(webhook_config) = config.webhook_config.clone() {
        fetch_permissions(task_queue, db, config, webhook_config).await;
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
                                                    .expr(Expr::cust(format!("'{}'", Id::new())))
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
    if let Some(permissions) = body.permissions {
        if !permissions.strict {
            update.permissions = Some(permissions);
        }
    }

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

pub async fn permissions_webhook(
    headers: HeaderMap,
    Config(config): Config,
    ProjectDatabase(db): ProjectDatabase,
    State(AppState { task_queue, .. }): State<AppState>,
    body: Bytes,
) -> Result<impl IntoResponse, Error> {
    let Some(webhook_config) = config.webhook_config.clone() else {
        return Err(Error::BadRequest("Webhooks aren't setup".into()));
    };

    match webhook_config.provider.clone() {
        WebhookProvider::GitHub { branch, secret } => {
            if let Some(secret) = secret {
                let signature_header = headers
                    .get("X-Hub-Signature-256")
                    .unwrap()
                    .to_str()
                    .unwrap();

                if !signature_header.starts_with("sha256=") {
                    return Err(Error::BadRequest("Invalid header format".into()));
                }
                let signature = &signature_header[7..];

                let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
                let expected_tag =
                    format!("sha256={}", hex::encode(hmac::sign(&key, &body).as_ref()));

                hmac::verify(
                    &key,
                    &hex::decode(signature).unwrap(),
                    expected_tag.as_ref(),
                )
                .map_err(|_| Error::BadRequest("Invalid signature".into()))?;
            }

            let header = headers.get("X-GitHub-Event").unwrap().to_str().unwrap();
            let event = WebhookEvent::try_from_header_and_body(header, &body).unwrap();

            if !matches!(event.kind, WebhookEventType::Push) {
                return Err(Error::BadRequest("Invalid event kind".to_string()));
            }

            let WebhookEventPayload::Push(payload) = event.specific else {
                return Err(Error::BadRequest("Invalid event payload".to_string()));
            };

            if branch != payload.base_ref.unwrap() {
                return Ok(StatusCode::ACCEPTED);
            }
        }
    }

    fetch_permissions(task_queue, db, config, webhook_config).await;
    Ok(StatusCode::ACCEPTED)
}

async fn fetch_permissions(
    task_queue: Arc<RwLock<TaskQueue>>,
    db: Database,
    config: config::Config,
    webhook_config: WebhookConfig,
) {
    task_queue.write().await.add_task(move |clear_task| {
        let db = db.clone();
        let config = config.clone();
        let webhook_config = webhook_config.clone();
        Box::pin(async move {
            let response = reqwest::Client::new()
                .get(webhook_config.permissions_url.clone())
                .bearer_auth(webhook_config.key.clone())
                .send()
                .await
                .unwrap()
                .json::<PermissionsResponse>()
                .await
                .unwrap();

            let mut hasher = DefaultHasher::new();
            serde_json::to_string(&serde_json::to_value(&response).unwrap())
                .unwrap()
                .hash(&mut hasher);
            let hash = hasher.finish();

            if let Some(historical) = webhook_config.historical.clone() {
                if response.build_timestamp <= historical.build_timestamp || hash == historical.hash
                {
                    return;
                }
            }

            let system = config.system();
            let Some(mut system) = system.clone() else {
                return;
            };

            system.webhook_config = Some(WebhookConfig {
                historical: Some(Historical {
                    hash,
                    build_timestamp: response.build_timestamp,
                }),
                ..webhook_config.clone()
            });

            db.get()
                .await
                .unwrap()
                .execute(&system.set(), &[])
                .await
                .unwrap();

            let tables = CustomTableSchema::find().all(&db).await.unwrap();
            for table in tables {
                if table.permissions.strict
                    && let Some(permissions) = response.permissions.get(&table.name)
                {
                    table
                        .update(
                            &db,
                            UpdateCustomTableSchema {
                                permissions: Some(Permissions {
                                    strict: true,
                                    ..permissions.clone()
                                }),
                                ..Default::default()
                            },
                        )
                        .await
                        .unwrap();
                };
            }

            clear_task().await;
        })
    });
}
