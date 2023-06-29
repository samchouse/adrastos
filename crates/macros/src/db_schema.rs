use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{parse_macro_input, Field, Fields, ItemStruct, Meta, Type};

pub fn derive(item: TokenStream) -> TokenStream {
    let ItemStruct { ident, fields, .. } = parse_macro_input!(item as ItemStruct);

    let lowercase_ident = ident.to_string().to_lowercase();

    let fields = match fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("Only structs with fields are supported"),
    };

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
    }
    .into()
}
