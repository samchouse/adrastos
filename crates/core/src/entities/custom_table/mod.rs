// TODO(@Xenfo): implement relations

use actix_web::web;
use chrono::{DateTime, Utc};
use sea_query::{
    Alias, ColumnDef, ColumnType, Keyword, PostgresQueryBuilder, SimpleExpr, Table,
    TableCreateStatement,
};
use serde_json::json;
use tokio_postgres::Row;

use crate::error::Error;

use self::schema::CustomTableSchema;

pub mod schema;

#[derive(Clone, Debug)]
enum ColType {
    String,
    Number,
    Boolean,
    Date,
    Array(Box<ColType>),
    OptionalDate,
}

impl ColType {
    fn patch_from_row_column(row: &Row, name: &str, col_type: ColType) -> serde_json::Value {
        let camel_case_name = heck::AsLowerCamelCase(name).to_string();
        let camel_case_name = camel_case_name.as_str();

        match col_type {
            ColType::String => json!({ camel_case_name: row.get::<_, String>(name) }),
            ColType::Number => json!({ camel_case_name: row.get::<_, i64>(name) }),
            ColType::Boolean => json!({ camel_case_name: row.get::<_, bool>(name) }),
            ColType::Date => json!({ camel_case_name: row.get::<_, DateTime<Utc>>(name) }),
            ColType::Array(col_type) => match col_type.as_ref().to_owned() {
                ColType::String => json!({ camel_case_name: row.get::<_, Vec<String>>(name) }),
                _ => todo!(),
            },
            ColType::OptionalDate => {
                json!({ camel_case_name: row.get::<_, Option<DateTime<Utc>>>(name) })
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
        self.query_builder.limit(limit.unwrap_or(100));

        self
    }

    pub async fn finish(
        &mut self,
        db_pool: &web::Data<deadpool_postgres::Pool>,
    ) -> Result<serde_json::Value, Error> {
        let row = db_pool
            .get()
            .await
            .unwrap()
            .query(
                self.query_builder.to_string(PostgresQueryBuilder).as_str(),
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

        let mut data = json!({});

        let mut columns = vec![
            ("id", ColType::String),
            ("created_at", ColType::Date),
            ("updated_at", ColType::OptionalDate),
        ];

        self.schema
            .string_fields
            .iter()
            .for_each(|f| columns.push((&f.name, ColType::String)));
        self.schema
            .number_fields
            .iter()
            .for_each(|f| columns.push((&f.name, ColType::Number)));
        self.schema
            .boolean_fields
            .iter()
            .for_each(|f| columns.push((&f.name, ColType::Boolean)));
        self.schema
            .date_fields
            .iter()
            .for_each(|f| columns.push((&f.name, ColType::Date)));
        self.schema
            .email_fields
            .iter()
            .for_each(|f| columns.push((&f.name, ColType::String)));
        self.schema
            .url_fields
            .iter()
            .for_each(|f| columns.push((&f.name, ColType::String)));
        self.schema
            .select_fields
            .iter()
            .for_each(|f| columns.push((&f.name, ColType::Array(Box::new(ColType::String)))));

        columns.into_iter().for_each(|(name, col_type)| {
            json_patch::merge(
                &mut data,
                &ColType::patch_from_row_column(&row, name, col_type),
            )
        });

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

        schema.string_fields.iter().for_each(|field| {
            columns.push(Alias::new(&field.name));
        });
        schema.number_fields.iter().for_each(|field| {
            columns.push(Alias::new(&field.name));
        });
        schema.boolean_fields.iter().for_each(|field| {
            columns.push(Alias::new(&field.name));
        });
        schema.date_fields.iter().for_each(|field| {
            columns.push(Alias::new(&field.name));
        });
        schema.email_fields.iter().for_each(|field| {
            columns.push(Alias::new(&field.name));
        });
        schema.url_fields.iter().for_each(|field| {
            columns.push(Alias::new(&field.name));
        });
        schema.select_fields.iter().for_each(|field| {
            columns.push(Alias::new(&field.name));
        });
        schema.relation_fields.iter().for_each(|field| {
            columns.push(Alias::new(&field.name));
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

        schema.string_fields.iter().for_each(|field| {
            columns.push(field.column());
        });
        schema.number_fields.iter().for_each(|field| {
            let mut column = ColumnDef::new(Alias::new(&field.name));

            if field.is_required {
                column.not_null();
            }
            if field.is_unique {
                column.unique_key();
            }

            columns.push(column.integer().to_owned());
        });
        schema.boolean_fields.iter().for_each(|field| {
            columns.push(ColumnDef::new(Alias::new(&field.name)).boolean().to_owned());
        });
        schema.date_fields.iter().for_each(|field| {
            let mut column = ColumnDef::new(Alias::new(&field.name));

            if field.is_required {
                column.not_null();
            }
            if field.is_unique {
                column.unique_key();
            }

            columns.push(column.timestamp_with_time_zone().to_owned());
        });
        schema.email_fields.iter().for_each(|field| {
            let mut column = ColumnDef::new(Alias::new(&field.name));

            if field.is_required {
                column.not_null();
            }
            if field.is_unique {
                column.unique_key();
            }

            columns.push(column.string().to_owned());
        });
        schema.url_fields.iter().for_each(|field| {
            let mut column = ColumnDef::new(Alias::new(&field.name));

            if field.is_required {
                column.not_null();
            }
            if field.is_unique {
                column.unique_key();
            }

            columns.push(column.string().to_owned());
        });
        schema.select_fields.iter().for_each(|field| {
            let mut column = ColumnDef::new(Alias::new(&field.name));

            if field.is_required {
                column.not_null();
            }
            if field.is_unique {
                column.unique_key();
            }

            columns.push(column.array(ColumnType::String(None)).to_owned());
        });

        columns.iter_mut().for_each(|column| {
            builder.col(column);
        });

        builder.to_owned()
    }
}
