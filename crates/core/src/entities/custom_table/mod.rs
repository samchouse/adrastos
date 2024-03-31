// TODO(@Xenfo): implement relations

use chrono::{DateTime, Utc};
use sea_query::{
    Alias, ColumnDef, Expr, ForeignKey, ForeignKeyAction, Keyword, PostgresQueryBuilder,
    SimpleExpr, Table, TableCreateStatement,
};
use serde_json::{json, Map};

use crate::error::Error;

use self::{
    fields::{FieldInfo, RelationTarget},
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
}

impl ColType {
    fn to_json(&self, column: &Map<String, serde_json::Value>, name: &str) -> serde_json::Value {
        let camel_case_name = heck::AsLowerCamelCase(name).to_string();
        let camel_case_name = camel_case_name.as_str();

        match self {
            ColType::String => {
                json!({ camel_case_name: column.get(name).unwrap().as_str() })
            }
            ColType::Number => {
                json!({ camel_case_name: column.get(name).unwrap().as_i64() })
            }
            ColType::Boolean => {
                json!({ camel_case_name: column.get(name).unwrap().as_bool().unwrap() })
            }
            ColType::Date => {
                let date =
                    serde_json::from_value::<DateTime<Utc>>(column.get(name).unwrap().clone()).ok();

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
        }
    }
}

pub struct CustomTableSelectBuilder {
    is_count: bool,
    schema: CustomTableSchema,
    query_builder: sea_query::SelectStatement,
}

impl CustomTableSelectBuilder {
    pub fn count(&mut self) -> Self {
        let mut builder = CustomTableSelectBuilder {
            is_count: true,
            query_builder: self.query_builder.clone(),
            schema: self.schema.clone(),
        };

        builder.query_builder.reset_limit();
        builder.query_builder.reset_offset();
        builder.query_builder.clear_selects();

        builder
            .query_builder
            .expr(sea_query::Func::count(Expr::col(sea_query::Asterisk)));

        builder
    }

    pub fn and_where(&mut self, expressions: Vec<SimpleExpr>) -> &mut Self {
        for expression in expressions {
            self.query_builder.and_where(expression);
        }

        self
    }

    pub fn paginate(&mut self, page: Option<u64>, limit: Option<u64>) -> &mut Self {
        self.query_builder.reset_limit();
        self.query_builder.reset_offset();

        if let Some(page) = page
            && let Some(limit) = limit
        {
            self.query_builder.limit(limit);
            self.query_builder.offset((page - 1) * limit);
        }

        self
    }

    pub fn join(&mut self) -> &mut Self {
        self.schema.fields.iter().for_each(|f| {
            let FieldInfo::Relation { table, target, .. } = &f.info else {
                return;
            };

            let json_func = if target == &RelationTarget::Single {
                "row_to_json"
            } else {
                "json_agg"
            };

            let where_clause = match target {
                RelationTarget::Single => format!("= {}.{}", self.schema.name, f.name),
                RelationTarget::Many => format!(
                    "IN (SELECT {} FROM {} WHERE {} = {}.id)",
                    format_args!("{}_id", table),
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
                    table = table
                )
                .as_str(),
            ));
        });

        self
    }

    pub async fn finish(
        &mut self,
        db: &deadpool_postgres::Pool,
    ) -> Result<serde_json::Value, Error> {
        let query = if self.is_count {
            self.query_builder.to_string(PostgresQueryBuilder)
        } else {
            format!(
                "SELECT json_agg(columns) as columns FROM ({}) as columns",
                self.query_builder.to_string(PostgresQueryBuilder)
            )
        };

        let row = db
            .get()
            .await
            .unwrap()
            .query(query.as_str(), &[])
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

        if self.is_count {
            let count = row.get::<_, i64>("count");
            return Ok(count.into());
        }

        let mut columns = vec![
            ("id", ColType::String),
            ("created_at", ColType::Date),
            ("updated_at", ColType::Date),
        ];

        self.schema.fields.iter().for_each(|f| match f.info {
            FieldInfo::String { .. } => columns.push((&f.name, ColType::String)),
            FieldInfo::Number { .. } => columns.push((&f.name, ColType::Number)),
            FieldInfo::Boolean => columns.push((&f.name, ColType::Boolean)),
            FieldInfo::Date { .. } => columns.push((&f.name, ColType::Date)),
            FieldInfo::Email { .. } => columns.push((&f.name, ColType::String)),
            FieldInfo::Url { .. } => columns.push((&f.name, ColType::String)),
            FieldInfo::Select { .. } => {
                columns.push((&f.name, ColType::Array(Box::new(ColType::String))))
            }
            FieldInfo::Relation { .. } => columns.push((
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

        schema.fields.iter().for_each(|field| match &field.info {
            FieldInfo::String { .. } => columns.push(Alias::new(&field.name)),
            FieldInfo::Number { .. } => columns.push(Alias::new(&field.name)),
            FieldInfo::Boolean => columns.push(Alias::new(&field.name)),
            FieldInfo::Date { .. } => columns.push(Alias::new(&field.name)),
            FieldInfo::Email { .. } => columns.push(Alias::new(&field.name)),
            FieldInfo::Url { .. } => columns.push(Alias::new(&field.name)),
            FieldInfo::Select { .. } => columns.push(Alias::new(&field.name)),
            FieldInfo::Relation { target, .. } => {
                if target == &RelationTarget::Single {
                    columns.push(Alias::new(&field.name));
                }
            }
        });

        CustomTableSelectBuilder {
            is_count: false,
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

        schema.fields.iter().for_each(|field| match &field.info {
            FieldInfo::Relation {
                table,
                cascade_delete,
                ..
            } => {
                let mut foreign_key = ForeignKey::create();

                if *cascade_delete {
                    foreign_key.on_delete(ForeignKeyAction::Cascade);
                }

                columns.push(field.column());
                builder.foreign_key(
                    foreign_key
                        .name(format!("FK_{}_{}", schema.name, field.name))
                        .from(Alias::new(&schema.name), Alias::new(&field.name))
                        .to(Alias::new(table), Alias::new("id"))
                        .on_update(ForeignKeyAction::Cascade),
                );
            }
            _ => columns.push(field.column()),
        });

        columns.iter_mut().for_each(|column| {
            builder.col(column);
        });

        builder.to_owned()
    }
}
