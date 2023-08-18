// TODO(@Xenfo): implement relations

use chrono::{DateTime, Utc};
use sea_query::{
    Alias, ColumnDef, ColumnType, Expr, ForeignKey, ForeignKeyAction, Keyword,
    PostgresQueryBuilder, SimpleExpr, Table, TableCreateStatement,
};
use serde_json::{json, Map};

use crate::error::Error;

use self::{
    fields::{Field, RelationType},
    mm_relation::ManyToManyRelationTable,
    schema::CustomTableSchema,
};

pub mod fields;
pub mod mm_relation;
pub mod schema;

#[derive(Clone, Debug)]
enum ColType {
    String,
    Number,
    Boolean,
    Date,
    Array(Box<ColType>),
    Relation(String),
    OptionalDate,
}

impl ColType {
    fn to_json(&self, column: &Map<String, serde_json::Value>, name: &str) -> serde_json::Value {
        let camel_case_name = heck::AsLowerCamelCase(name).to_string();
        let camel_case_name = camel_case_name.as_str();

        match self {
            ColType::String => {
                json!({ camel_case_name: column.get(name).unwrap().as_str().unwrap() })
            }
            ColType::Number => {
                json!({ camel_case_name: column.get(name).unwrap().as_i64().unwrap() })
            }
            ColType::Boolean => {
                json!({ camel_case_name: column.get(name).unwrap().as_bool().unwrap() })
            }
            ColType::Date => {
                let date =
                    serde_json::from_value::<DateTime<Utc>>(column.get(name).unwrap().clone())
                        .unwrap();

                json!({ camel_case_name: date })
            }
            ColType::Array(col_type) => match col_type.as_ref().to_owned() {
                ColType::String => {
                    let array: Vec<_> = column
                        .get(name)
                        .unwrap()
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect();

                    json!({ camel_case_name: array })
                }
                _ => todo!(),
            },
            ColType::Relation(name) => {
                json!({ camel_case_name: column.get(name).unwrap() })
            }
            ColType::OptionalDate => {
                let date =
                    serde_json::from_value::<DateTime<Utc>>(column.get(name).unwrap().clone()).ok();

                json!({ camel_case_name: date })
            }
        }
    }
}

pub struct CustomTableSelectBuilder {
    schema: CustomTableSchema,
    query_builder: sea_query::SelectStatement,
}

impl CustomTableSelectBuilder {
    pub fn and_where(&mut self, expressions: Vec<SimpleExpr>) -> &mut Self {
        for expression in expressions {
            self.query_builder.and_where(expression);
        }

        self
    }

    pub fn limit(&mut self, limit: Option<u64>) -> &mut Self {
        self.query_builder.reset_limit().limit(limit.unwrap_or(100));

        self
    }

    pub fn join(&mut self) -> &mut Self {
        self.schema
            .fields
            .iter()
            .filter_map(|f| {
                if let Field::Relation(f) = f {
                    return Some(f);
                }

                None
            })
            .for_each(|f| {
                let json_func = if f.relation_type == RelationType::Single {
                    "row_to_json"
                } else {
                    "json_agg"
                };

                let where_clause = match f.relation_type {
                    RelationType::Single => format!("= {}.{}", self.schema.name, f.name),
                    RelationType::Many => format!(
                        "IN (SELECT {} FROM {} WHERE {} = {}.id)",
                        format_args!("{}_id", f.table),
                        ManyToManyRelationTable::table_name(&self.schema, f),
                        format_args!("{}_id", self.schema.name),
                        self.schema.name,
                    ),
                };

                self.query_builder.expr(Expr::cust(
                    format!(
                    "(SELECT {}({table}) FROM (SELECT * FROM {table} WHERE id {}) {table}) as {}",
                    json_func,
                    where_clause,
                    format_args!("{}_relation_key", f.name),
                    table = f.table
                )
                    .as_str(),
                ));
            });

        self
    }

    pub async fn finish(
        &mut self,
        db_pool: &deadpool_postgres::Pool,
    ) -> Result<serde_json::Value, Error> {
        let row = db_pool
            .get()
            .await
            .unwrap()
            .query(
                format!(
                    "SELECT json_agg(columns) as columns FROM ({}) as columns",
                    self.query_builder.to_string(PostgresQueryBuilder)
                )
                .as_str(),
                &[],
            )
            .await
            .map_err(|e| {
                let error = format!(
                    "An error occurred while fetching the {}: {e}",
                    self.schema.name
                );
                Error::InternalServerError(error)
            })?
            .into_iter()
            .next()
            .ok_or_else(|| {
                let message = format!(
                    "No document was found for the custom table {}",
                    self.schema.name
                );
                Error::BadRequest(message)
            })?;

        let mut columns = vec![
            ("id", ColType::String),
            ("created_at", ColType::Date),
            ("updated_at", ColType::OptionalDate),
        ];

        self.schema.fields.iter().for_each(|f| match f {
            Field::String(f) => columns.push((&f.name, ColType::String)),
            Field::Number(f) => columns.push((&f.name, ColType::Number)),
            Field::Boolean(f) => columns.push((&f.name, ColType::Boolean)),
            Field::Date(f) => columns.push((&f.name, ColType::Date)),
            Field::Email(f) => columns.push((&f.name, ColType::String)),
            Field::Url(f) => columns.push((&f.name, ColType::String)),
            Field::Select(f) => columns.push((&f.name, ColType::Array(Box::new(ColType::String)))),
            Field::Relation(f) => columns.push((
                &f.name,
                ColType::Relation(format!("{}_relation_key", f.name)),
            )),
        });

        let data = serde_json::from_value::<Vec<Map<String, serde_json::Value>>>(
            row.try_get("columns")
                .unwrap_or(serde_json::Value::Array(vec![])),
        )
        .unwrap()
        .iter()
        .map(|col| {
            let mut data = json!({});

            columns.clone().into_iter().for_each(|(name, col_type)| {
                json_patch::merge(&mut data, &col_type.to_json(col, name));
            });

            data
        })
        .collect();

        Ok(data)
    }
}

impl From<&CustomTableSchema> for CustomTableSelectBuilder {
    fn from(schema: &CustomTableSchema) -> Self {
        let mut columns = vec![
            Alias::new("id"),
            Alias::new("created_at"),
            Alias::new("updated_at"),
        ];

        schema.fields.iter().for_each(|field| match field {
            Field::String(field) => columns.push(Alias::new(&field.name)),
            Field::Number(field) => columns.push(Alias::new(&field.name)),
            Field::Boolean(field) => columns.push(Alias::new(&field.name)),
            Field::Date(field) => columns.push(Alias::new(&field.name)),
            Field::Email(field) => columns.push(Alias::new(&field.name)),
            Field::Url(field) => columns.push(Alias::new(&field.name)),
            Field::Select(field) => columns.push(Alias::new(&field.name)),
            Field::Relation(field) => {
                if field.relation_type == RelationType::Single {
                    columns.push(Alias::new(&field.name));
                }
            }
        });

        CustomTableSelectBuilder {
            schema: schema.clone(),
            query_builder: sea_query::Query::select()
                .from(Alias::new(&schema.name))
                .columns(columns)
                .limit(1)
                .to_owned(),
        }
    }
}

impl From<&CustomTableSchema> for TableCreateStatement {
    fn from(schema: &CustomTableSchema) -> Self {
        let mut builder = Table::create();

        builder
            .table(Alias::new(&schema.name))
            .if_not_exists()
            .col(
                ColumnDef::new(Alias::new("id"))
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(Alias::new("created_at"))
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Keyword::CurrentTimestamp),
            )
            .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone());

        let mut columns = vec![];

        schema.fields.iter().for_each(|field| {
            match field {
                Field::String(field) => columns.push(field.column()),
                Field::Number(field) => {
                    let mut column = ColumnDef::new(Alias::new(&field.name));
        
                    if field.is_required {
                        column.not_null();
                    }
                    if field.is_unique {
                        column.unique_key();
                    }
        
                    columns.push(column.integer().to_owned());
                },
                Field::Boolean(field) => {
                    columns.push(ColumnDef::new(Alias::new(&field.name)).boolean().to_owned());
                },
                Field::Date(field) => {
                    let mut column = ColumnDef::new(Alias::new(&field.name));
        
                    if field.is_required {
                        column.not_null();
                    }
                    if field.is_unique {
                        column.unique_key();
                    }
        
                    columns.push(column.timestamp_with_time_zone().to_owned());
                },
                Field::Email(field) => {
                    let mut column = ColumnDef::new(Alias::new(&field.name));
        
                    if field.is_required {
                        column.not_null();
                    }
                    if field.is_unique {
                        column.unique_key();
                    }
        
                    columns.push(column.string().to_owned());
                },
                Field::Url(field) => {
                    let mut column = ColumnDef::new(Alias::new(&field.name));
        
                    if field.is_required {
                        column.not_null();
                    }
                    if field.is_unique {
                        column.unique_key();
                    }
        
                    columns.push(column.string().to_owned());
                },
                Field::Select(field) => {
                    let mut column = ColumnDef::new(Alias::new(&field.name));
        
                    if field.is_required {
                        column.not_null();
                    }
                    if field.is_unique {
                        column.unique_key();
                    }
        
                    columns.push(column.array(ColumnType::String(None)).to_owned());
                },
                Field::Relation(field) => {
                    if field.relation_type == RelationType::Single {
                        let mut column = ColumnDef::new(Alias::new(&field.name));
                        let mut foreign_key = ForeignKey::create();
        
                        if field.is_required {
                            column.not_null();
                        }
                        if field.is_unique {
                            column.unique_key();
                        }
        
                        if field.cascade_delete {
                            foreign_key.on_delete(ForeignKeyAction::Cascade);
                        }
        
                        columns.push(column.string().to_owned());
        
                        builder.foreign_key(
                            foreign_key
                                .name(format!("FK_{}_{}", schema.name, field.name))
                                .from(Alias::new(&schema.name), Alias::new(&field.name))
                                .to(Alias::new(&field.table), Alias::new("id"))
                                .on_update(ForeignKeyAction::Cascade),
                        );
                    };
                }
            }
        });

        columns.iter_mut().for_each(|column| {
            builder.col(column);
        });

        builder.to_owned()
    }
}
