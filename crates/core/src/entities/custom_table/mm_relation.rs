use sea_query::{
    Alias, ColumnDef, ForeignKey, ForeignKeyAction, InsertStatement, Query, Table,
    TableCreateStatement,
};

use crate::id::Id;

use super::{
    fields::{Field, RelationField, RelationType},
    schema::CustomTableSchema,
};

pub struct ManyToManyRelationTable;

impl ManyToManyRelationTable {
    pub fn table_name(schema: &CustomTableSchema, field: &RelationField) -> String {
        format!("{}_{}_to_{}", schema.name, field.name, field.table)
    }

    pub fn create_queries(schema: &CustomTableSchema) -> Vec<TableCreateStatement> {
        schema
            .fields
            .iter()
            .filter_map(|field| {
                let Field::Relation(field) = field else {
                    return None;
                };

                if field.relation_type == RelationType::Many {
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
                                ColumnDef::new(Alias::new(format!("{}_id", field.table)))
                                    .string()
                                    .not_null(),
                            )
                            .foreign_key(
                                ForeignKey::create()
                                    .name(format!("FK_{}_{}_id", &name, schema.name))
                                    .from(
                                        Alias::new(&name),
                                        Alias::new(format!("{}_id", schema.name)),
                                    )
                                    .to(Alias::new(&schema.name), Alias::new("id"))
                                    .on_update(ForeignKeyAction::Cascade)
                                    .on_delete(ForeignKeyAction::Cascade),
                            )
                            .foreign_key(
                                ForeignKey::create()
                                    .name(format!("FK_{}_{}_id", &name, field.table))
                                    .from(
                                        Alias::new(&name),
                                        Alias::new(format!("{}_id", field.table)),
                                    )
                                    .to(Alias::new(&field.table), Alias::new("id"))
                                    .on_update(ForeignKeyAction::Cascade)
                                    .on_delete(ForeignKeyAction::Cascade),
                            )
                            .to_owned(),
                    );
                }

                None
            })
            .collect::<Vec<_>>()
    }

    pub fn insert_query(
        schema: &CustomTableSchema,
        field: &RelationField,
        row_id: String,
        relations: Vec<String>,
    ) -> Vec<InsertStatement> {
        if field.relation_type == RelationType::Many {
            let name = Self::table_name(schema, field);

            return relations
                .into_iter()
                .map(|relation| {
                    Query::insert()
                        .into_table(Alias::new(&name))
                        .columns(vec![
                            Alias::new("id"),
                            Alias::new(format!("{}_id", schema.name)),
                            Alias::new(format!("{}_id", field.table)),
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
