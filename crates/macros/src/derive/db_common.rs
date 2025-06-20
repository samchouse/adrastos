use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Field, Fields, ItemStruct};

use crate::{
    attribute_parser::{AttributeTokens, Token, TokenName},
    types::Type,
};

pub fn db_common(item: TokenStream) -> TokenStream {
    let ItemStruct {
        ident,
        fields,
        attrs,
        ..
    } = parse_macro_input!(item as ItemStruct);
    let attrs = AttributeTokens::from(attrs);

    let table_name = attrs
        .get(TokenName::Rename)
        .map(|t| {
            if let Token::Rename(name) = t {
                return name.clone();
            }

            panic!("Expected rename token")
        })
        .unwrap_or(format!("{}s", ident.to_string().to_snake_case()));

    let iden_ident = format_ident!("{}Iden", ident);
    let error_identifier = table_name.replace('_', " ");
    let error_identifier = error_identifier
        .strip_suffix('s')
        .unwrap_or(&error_identifier);

    let fields = match fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("Only structs with fields are supported"),
    };

    let branches = fields.iter().map(|it| {
        let Field { ident, .. } = it;

        let ident = ident.clone().unwrap().to_string();
        let branch_ident = format_ident!("{}", ident.to_string().to_upper_camel_case());

        quote! { Self::#branch_ident => #ident }
    });

    let columns = fields.iter().filter_map(|it| {
        let Field { ident, attrs, ty, .. } = it;
        let attrs = AttributeTokens::from(attrs.clone());
        let ty = Type::from(ty.clone());

        let str_ident = ident.clone().unwrap().to_string();
        if str_ident == "id" || attrs.get(TokenName::Join).is_some() {
            return None;
        }

        let unique_attr = attrs.get(TokenName::Unique).is_some().then_some(quote! { .unique_key() });
        let props = match ty {
            Type::String => quote! { .string().not_null() },
            Type::Int64 => quote! { .big_integer().not_null() },
            Type::Bool => quote! { .boolean().not_null().default(false) },
            Type::DateTime => quote! { .timestamp_with_time_zone().not_null().default(sea_query::Keyword::CurrentTimestamp) },
            Type::Vec(_) if attrs.get(TokenName::Json).is_some() => {
                quote! { .array(sea_query::ColumnType::JsonBinary).not_null().default(vec![] as Vec<String>) }
            },
            Type::Vec(_) => quote! { .array(sea_query::ColumnType::String(None)).not_null().default(vec![] as Vec<String>) },
            Type::Option(generic) => {
                match *generic {
                    Type::Int64 => quote! { .big_integer() },
                    Type::DateTime => quote! { .timestamp_with_time_zone() },
                    Type::Vec(_) if attrs.get(TokenName::Json).is_some() => {
                        quote! { .array(sea_query::ColumnType::JsonBinary) }
                    },
                    Type::Vec(_) => quote! { .array(sea_query::ColumnType::String(None)) },
                    _ => quote! { .string() },
                }
            },
            _ if attrs.get(TokenName::Json).is_some() => {
                quote! { .json_binary().not_null() }
            },
            _ => quote! { .string().not_null() }
        };

        Some(quote! {
            .col(
                sea_query::ColumnDef::new(sea_query::Alias::new(#str_ident))
                #props
                #unique_attr
            )
        })
    });

    let foreign_keys = fields.iter().filter_map(|it| {
        let Field {
            ident: inner_ident,
            attrs,
            ..
        } = it;
        let attrs = AttributeTokens::from(attrs.clone());

        let str_ident = inner_ident.clone().unwrap().to_string();
        let fk_name = format!("FK_{}_{}", ident.to_string().to_snake_case(), str_ident);

        let relation = attrs.get(TokenName::Relation).map(|t| {
            if let Token::Relation(ident) = t {
                return ident;
            }

            panic!("Expected relation token")
        })?;

        Some(quote! {
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name(#fk_name)
                    .from(Self::table(), sea_query::Alias::new(#str_ident))
                    .to(#relation::table(), sea_query::Alias::new("id"))
                    .on_update(sea_query::ForeignKeyAction::Cascade)
                    .on_delete(sea_query::ForeignKeyAction::Cascade),
            )
        })
    });

    let row_fields = fields.iter().map(|it| {
        let Field { ident, ty, .. } = it;
        let ty = Type::from(ty.clone());

        let str_ident = ident.clone().unwrap().to_string();

        match ty {
            Type::Int64 | Type::String | Type::Bool | Type::DateTime => quote! { #ident: row.get(#str_ident) },
            Type::Vec(generic) => match *generic {
                Type::Int64 | Type::String | Type::Bool | Type::DateTime => quote! { #ident: row.get(#str_ident) },
                _ => quote!{ #ident: row.get::<_, Vec<String>>(#str_ident).iter().map(|s| serde_json::from_str(s).unwrap()).collect::<Vec<_>>() },
            },
            Type::Option(generic) => match *generic {
                Type::Int64 | Type::String | Type::Bool | Type::DateTime => quote! { #ident: row.get(#str_ident) },
                Type::Vec(generic) => {
                    match *generic {
                        Type::Int64 | Type::String | Type::Bool | Type::DateTime => quote! { #ident: row.get(#str_ident) },
                        _ => quote!{ #ident: row.try_get::<_, serde_json::Value>(#str_ident).ok().map(|v| serde_json::from_value(v).unwrap()) },
                    }
                },
                _ => quote! { #ident: row.try_get::<_, String>(#str_ident).ok().map(|v| serde_json::from_str(&v).unwrap()) },
            },
            _ => quote! { #ident: serde_json::from_value(row.get(#str_ident)).unwrap() },
        }
    });

    quote! {
        impl #ident {
            pub fn table() -> sea_query::Alias {
                sea_query::Alias::new(#iden_ident::Table.to_string())
            }

            pub fn error_identifier() -> String {
                #error_identifier.to_string()
            }

            pub fn init() -> String {
                sea_query::Table::create()
                    .table(Self::table())
                    .if_not_exists()
                    .col(
                        sea_query::ColumnDef::new(sea_query::Alias::new("id"))
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    #(#columns)*
                    #(#foreign_keys)*
                    .to_string(sea_query::PostgresQueryBuilder)
            }
        }

        impl From<deadpool_postgres::tokio_postgres::Row> for #ident {
            fn from(row: deadpool_postgres::tokio_postgres::Row) -> Self {
                #ident {
                    #(#row_fields),*
                }
            }
        }

        impl std::fmt::Display for #iden_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = match self {
                    Self::Table => #table_name,
                    #(#branches),*
                };

                write!(f, "{name}")
            }
        }
    }
    .into()
}
