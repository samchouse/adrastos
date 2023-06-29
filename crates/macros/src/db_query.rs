use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{parse_macro_input, Field, ItemStruct, Meta, Type};

pub fn derive(item: TokenStream) -> TokenStream {
    let ItemStruct {
        ident,
        fields,
        attrs,
        ..
    } = parse_macro_input!(item as ItemStruct);

    let create_validator = attrs
        .iter()
        .filter(|it| {
            if let Meta::List(list_meta) = &it.meta {
                return list_meta.path.segments.first().unwrap().ident == "adrastos"
                    && list_meta.tokens.clone().into_iter().any(|it| {
                        if let TokenTree::Ident(ident) = it {
                            return ident == "validated";
                        }

                        false
                    });
            }

            false
        })
        .map(|_| {
            quote! {
                self.validate().map_err(|err| crate::error::Error::ValidationErrors {
                    message: format!("An error occurred while validating the {}", Self::error_identifier()),
                    errors: err,
                })?;
            }
        });

    let create_columns = fields.iter().filter_map(|it| {
        let Field { ident, attrs, .. } = it;

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
        if has_skip_attr {
            return None;
        }

        let str_ident = ident.clone().unwrap().to_string();
        Some(quote! { sea_query::Alias::new(#str_ident) })
    });

    let create_values = fields.iter().filter_map(|it| {
        let Field {
            ident, ty, attrs, ..
        } = it;

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
        if has_skip_attr {
            return None;
        }

        let value = {
            if let Type::Path(ty) = ty {
                let name = ty.path.segments.first();
                if let Some(name) = name {
                    match name.ident.to_string().as_str() {
                        "String" => Some(quote! { self.#ident.clone().into() }),
                        "Vec" => {
                            if let syn::PathArguments::AngleBracketed(name) = &name.arguments {
                                let second_name = name.args.first();
                                if let Some(syn::GenericArgument::Type(Type::Path(ty))) =
                                    second_name
                                {
                                    let second_name = ty.path.segments.first();
                                    if let Some(second_name) = second_name {
                                        if second_name.ident.to_string().as_str() == "String" {
                                            return Some(quote! { self.#ident.clone().into() });
                                        }
                                    }
                                }
                            }

                            Some(quote! {
                                self.#ident.iter()
                                    .filter_map(|f| serde_json::to_string(f).ok())
                                    .collect::<Vec<String>>()
                                    .into()
                            })
                        },
                        "Option" => {
                            if let syn::PathArguments::AngleBracketed(name) = &name.arguments {
                                let second_name = name.args.first();
                                if let Some(syn::GenericArgument::Type(Type::Path(ty))) =
                                    second_name
                                {
                                    let second_name = ty.path.segments.first();
                                    if let Some(second_name) = second_name {
                                        return match second_name.ident.to_string().as_str() {
                                            "String" => Some(quote! { self.#ident.clone().into() }),
                                            "Vec" => {
                                                if let syn::PathArguments::AngleBracketed(third_name) = &second_name.arguments {
                                                    let third_name = third_name.args.first();
                                                    if let Some(syn::GenericArgument::Type(Type::Path(ty))) =
                                                        third_name
                                                    {
                                                        let third_name = ty.path.segments.first();
                                                        if let Some(third_name) = third_name {
                                                            if third_name.ident.to_string().as_str() == "String" {
                                                                return Some(quote! { self.#ident.clone().into() });
                                                            }
                                                        }
                                                    }
                                                }

                                                Some(quote! {
                                                    self.#ident.iter()
                                                        .filter_map(|f| serde_json::to_string(f).ok())
                                                        .collect::<Option<Vec<String>>>()
                                                        .into()
                                                })
                                            },
                                            _ => Some(quote! { self.#ident.into() }),
                                        }
                                    }
                                }
                            }

                            Some(quote! { self.#ident.into() })
                        },
                        _ => Some(quote! { self.#ident.into() }),
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(value_ident) = value {
            let transform = attrs
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
                                    return ident == "transform";
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
                });

            if let Some(transform_ident) = transform {
                return Some(quote! { #transform_ident(#value_ident)?.into() });
            }

            return Some(value_ident);
        }

        None
    });

    quote! {
		impl #ident {
			pub async fn create(&self, db_pool: &deadpool_postgres::Pool) -> Result<(), crate::error::Error> {
				#(#create_validator)*

				let query = sea_query::Query::insert()
					.into_table(Self::table())
					.columns([
						#(#create_columns),*
					])
					.values_panic([
						#(#create_values),*
					])
					.to_string(sea_query::PostgresQueryBuilder);

				db_pool
					.get()
					.await
					.unwrap()
					.execute(&query, &[])
					.await
					.map_err(|e| {
						tracing::error!(error = ?e);
						crate::error::Error::InternalServerError(format!("Failed to create {}", Self::error_identifier()))
					})?;

				Ok(())
			}

			pub async fn delete(&self, db_pool: &deadpool_postgres::Pool) -> Result<(), crate::error::Error> {
				let query = sea_query::Query::delete()
					.from_table(Self::table())
					.and_where(sea_query::Expr::col(sea_query::Alias::new("id")).eq(self.id.clone()))
					.to_string(sea_query::PostgresQueryBuilder);

				db_pool
					.get()
					.await
					.unwrap()
					.execute(&query, &[])
					.await
					.map_err(|e| {
						tracing::error!(error = ?e);
						crate::error::Error::InternalServerError(format!("Failed to delete {}", Self::error_identifier()))
					})?;

				Ok(())
			}
		}
	}.into()
}
