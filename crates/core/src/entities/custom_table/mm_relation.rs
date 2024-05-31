use sea_query::{
    Alias, ColumnDef, ForeignKey, ForeignKeyAction, InsertStatement, Query, Table,
    TableCreateStatement,
};

use crate::id::Id;

use super::{
    fields::{Field, FieldInfo, RelationTarget},
    schema::CustomTableSchema,
};

pub struct ManyToManyRelationTable;

impl ManyToManyRelationTable {
    pub fn table_name(schema: &CustomTableSchema, field: &Field) -> String {
        let FieldInfo::Relation { table, .. } = &field.info else {
            panic!("Field is not a relation");
        };

        format!("{}_{}_to_{}", schema.name, field.name, table)
    }

    pub fn create_query(schema: &CustomTableSchema, field: &Field) -> Option<TableCreateStatement> {
        let FieldInfo::Relation { table, target, .. } = &field.info else {
            return None;
        };

        if target == &RelationTarget::Many {
            let name = Self::table_name(schema, field);

            return Some(
                Table::create()
                    .table(Alias::new(&name))
                    .col(ColumnDef::new(Alias::new("id")).string().primary_key())
                    .col(
                        ColumnDef::new(Alias::new(format!("{}_id", schema.name)))
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new(format!("{}_id", table)))
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(sea_query::Keyword::CurrentTimestamp),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(format!("FK_{}_{}_id", &name, schema.name))
                            .from(Alias::new(&name), Alias::new(format!("{}_id", schema.name)))
                            .to(Alias::new(&schema.name), Alias::new("id"))
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(format!("FK_{}_{}_id", &name, table))
                            .from(Alias::new(&name), Alias::new(format!("{}_id", table)))
                            .to(Alias::new(table), Alias::new("id"))
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            );
        }

        None
    }

    pub fn create_queries(schema: &CustomTableSchema) -> Vec<TableCreateStatement> {
        schema
            .fields
            .iter()
            .filter_map(|f| Self::create_query(schema, f))
            .collect::<Vec<_>>()
    }

    pub fn insert_query(
        schema: &CustomTableSchema,
        f: &Field,
        row_id: String,
        relations: Vec<String>,
    ) -> Vec<InsertStatement> {
        let FieldInfo::Relation { table, target, .. } = &f.info else {
            panic!("Field is not a relation");
        };

        if target == &RelationTarget::Many {
            let name = Self::table_name(schema, f);

            return relations
                .into_iter()
                .map(|relation| {
                    Query::insert()
                        .into_table(Alias::new(&name))
                        .columns(vec![
                            Alias::new("id"),
                            Alias::new(format!("{}_id", schema.name)),
                            Alias::new(format!("{}_id", table)),
                        ])
                        .values_panic([
                            Id::new().to_string().into(),
                            row_id.clone().into(),
                            relation.into(),
                        ])
                        .to_owned()
                })
                .collect();
        }

        vec![]
    }
}
