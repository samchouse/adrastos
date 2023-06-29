use heck::{AsSnakeCase, AsUpperCamelCase};
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Field, Fields, ItemStruct, Meta, Type};

pub fn derive(item: TokenStream) -> TokenStream {
    let ItemStruct {
        ident,
        fields,
        attrs,
        ..
    } = parse_macro_input!(item as ItemStruct);

    let table_name = attrs
        .iter()
        .find(|it| {
            if let Meta::List(list_meta) = &it.meta {
                return list_meta.path.segments.first().unwrap().ident == "adrastos";
            }

            false
        })
        .and_then(|it| {
            if let Meta::List(list_meta) = &it.meta {
                let index = list_meta
                    .tokens
                    .clone()
                    .into_iter()
                    .position(|it| {
                        if let TokenTree::Ident(ident) = it {
                            return ident == "rename";
                        }

                        false
                    })
                    .map(|i| i + 2);

                if let Some(i) = index {
                    if let TokenTree::Literal(literal) =
                        list_meta.tokens.clone().into_iter().nth(i).unwrap()
                    {
                        return Some(
                            literal
                                .to_string()
                                .parse::<String>()
                                .unwrap()
                                .replace('\"', ""),
                        );
                    }
                }
            }

            None
        })
        .unwrap_or(AsSnakeCase(ident.to_string()).to_string());

    let iden_ident = format_ident!("{}Iden", ident);
    let lowercase_ident = ident.to_string().to_lowercase();
    let table_identifier = format!("{}s", table_name);
    let error_identifier = table_name.replace('_', " ");

    let fields = match fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("Only structs with fields are supported"),
    };

    let branches = fields.iter().map(|it| {
        let Field { ident, .. } = it;

        let ident = ident.clone().unwrap().to_string();
        let branch_ident = format_ident!("{}", AsUpperCamelCase(ident.to_string()).to_string());

        quote! { Self::#branch_ident => #ident }
    });

    let columns = fields.iter().filter_map(|it| {
        let Field { ident, attrs, ty, .. } = it;

        let str_ident = ident.clone().unwrap().to_string();
        let has_skip_attr = attrs.iter().any(|it| {
            if let Meta::List(list_meta) = &it.meta {
                list_meta.path.segments.first().unwrap().ident == "adrastos"
                    && list_meta.tokens.clone().into_iter().any(|it| {
                        if let TokenTree::Ident(ident) = it {
                            ident == "skip"
                        } else {
                            false
                        }
                    })
            } else {
                false
            }
        });

        if has_skip_attr || str_ident == "id" {
            return None;
        }

        let unique_attr = attrs.iter().find(|it| {
            if let Meta::List(list_meta) = &it.meta {
                list_meta.path.segments.first().unwrap().ident == "adrastos"
                    && list_meta.tokens.clone().into_iter().any(|it| {
                        if let TokenTree::Ident(ident) = it {
                            ident == "unique"
                        } else {
                            false
                        }
                    })
            } else {
                false
            }
        }).map(|_| {
            quote! { .unique_key() }
        }).unwrap_or(quote! {});

        if let Type::Path(ty) = ty {
            let name = ty.path.segments.first();
            if let Some(name) = name {
                let props = match name.ident.to_string().as_str() {
                    "bool" => quote! { .boolean().not_null().default(false) },
                    "String" => quote! { .string().not_null() },
                    "DateTime" => quote! { .timestamp_with_time_zone().not_null().default(sea_query::Keyword::CurrentTimestamp) },
                    "Vec" => quote! { .array(sea_query::ColumnType::String(None)).not_null().default(vec![] as Vec<String>) },
                    "Option" => {
                        if let syn::PathArguments::AngleBracketed(name) = &name.arguments {
                            let second_name = name.args.first();
                            if let Some(syn::GenericArgument::Type(Type::Path(ty))) = second_name {
                                let second_name = ty.path.segments.first();
                                if let Some(second_name) = second_name {
                                    match second_name.ident.to_string().as_str() {
                                        "String" => quote! { .string() },
                                        "DateTime" => quote! { .timestamp_with_time_zone() },
                                        "Vec" => quote! { .array(sea_query::ColumnType::String(None)) },
                                        _ => quote! { .string() },
                                    }
                                } else {
                                    quote! {}
                                }
                            } else {
                                quote! {}
                            }
                        } else {
                            quote! {}
                        }
                    }
                    _ => quote! {},
                };

                return Some(quote! {
                    .col(
                        sea_query::ColumnDef::new(sea_query::Alias::new(#str_ident))
                        #props
                        #unique_attr
                    )
                });
            }
        }

        None
    });

    let foreign_keys = fields.iter().filter_map(|it| {
        let Field { ident, attrs, .. } = it;

        let str_ident = ident.clone().unwrap().to_string();
        let fk_name = format!("FK_{}_{}", lowercase_ident, str_ident);

        let relation = attrs
            .iter()
            .find(|it| {
                if let Meta::List(list_meta) = &it.meta {
                    return list_meta.path.segments.first().unwrap().ident == "adrastos";
                }

                false
            })
            .and_then(|it| {
                if let Meta::List(list_meta) = &it.meta {
                    let index = list_meta
                        .tokens
                        .clone()
                        .into_iter()
                        .position(|it| {
                            if let TokenTree::Ident(ident) = it {
                                return ident == "relation";
                            }

                            false
                        })
                        .map(|i| i + 2);

                    if let Some(i) = index {
                        if let TokenTree::Ident(ident) =
                            list_meta.tokens.clone().into_iter().nth(i).unwrap()
                        {
                            return Some(ident);
                        }
                    }
                }

                None
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
        let str_ident = ident.clone().unwrap().to_string();

        if let Type::Path(ty) = ty {
            let name = ty.path.segments.first();
            if let Some(name) = name {
                match name.ident.to_string().as_str() {
                    "String" | "u32" | "bool" | "DateTime" => return quote! { #ident: row.get(#str_ident) },
                    "Vec" => {
                        if let syn::PathArguments::AngleBracketed(name) = &name.arguments {
                            let second_name = name.args.first();
                            if let Some(syn::GenericArgument::Type(Type::Path(ty))) = second_name {
                                let second_name = ty.path.segments.first();
                                if let Some(second_name) = second_name {
                                    match second_name.ident.to_string().as_str() {
                                        "String" | "u32" | "bool" => return quote! { #ident: row.get(#str_ident) },
                                        _ => {}
                                    }
                                }
                            }
                        }

                        return quote!{ #ident: row.get::<_, Vec<String>>(#str_ident).iter().map(|s| serde_json::from_str(s).unwrap()).collect::<Vec<_>>() }
                    },
                    "Option" => {
                        if let syn::PathArguments::AngleBracketed(name) = &name.arguments {
                            let second_name = name.args.first();
                            if let Some(syn::GenericArgument::Type(Type::Path(ty))) = second_name {
                                let second_name = ty.path.segments.first();
                                if let Some(second_name) = second_name {
                                    match second_name.ident.to_string().as_str() {
                                        "String" | "u32" | "bool" | "DateTime" => return quote! { #ident: row.get(#str_ident) },
                                        "Vec" => {
                                            if let syn::PathArguments::AngleBracketed(name) = &second_name.arguments {
                                                let third_name = name.args.first();
                                                if let Some(syn::GenericArgument::Type(Type::Path(ty))) = third_name {
                                                    let third_name = ty.path.segments.first();
                                                    if let Some(third_name) = third_name {
                                                        match third_name.ident.to_string().as_str() {
                                                            "String" | "u32" | "bool" => return quote! { #ident: row.get(#str_ident) },
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                            }

                                            return quote!{ #ident: row.try_get::<_, Option<Vec<String>>>(#str_ident).ok().flatten().map(|v| v.iter().map(|s| serde_json::from_str(&s).unwrap()).collect::<Vec<_>>()) }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        return quote! { #ident: row.try_get(#str_ident).ok().map(|v| serde_json::from_value(v).unwrap()) }
                    },
                    _ => {}
                }
            }
        }

        quote! { #ident: serde_json::from_str(row.get(#str_ident)).unwrap() }
    });

    quote! {
        impl #ident {
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

        impl Identity for #ident {
            fn table() -> sea_query::Alias {
                sea_query::Alias::new(#iden_ident::Table.to_string())
            }

            fn error_identifier() -> String {
                #error_identifier.to_string()
            }
        }

        impl std::fmt::Display for #iden_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = match self {
                    Self::Table => #table_identifier,
                    #(#branches),*
                };

                write!(f, "{name}")
            }
        }

        impl From<deadpool_postgres::tokio_postgres::Row> for #ident {
            fn from(row: deadpool_postgres::tokio_postgres::Row) -> Self {
                #ident {
                    #(#row_fields),*
                }
            }
        }
    }
    .into()
}
