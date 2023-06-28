use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Field, Fields, ItemStruct, Meta, Type};

pub fn derive(item: TokenStream) -> TokenStream {
    let ItemStruct { ident, fields, .. } = parse_macro_input!(item as ItemStruct);

    let builder_ident = format_ident!("{}SelectBuilder", ident);
    let iden_ident = format_ident!("{}Iden", ident);

    let fields = match fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("Only structs with fields are supported"),
    };

    let aliases = fields.iter().filter_map(|it| {
        let Field { ident, attrs, .. } = it;

        let str_ident = ident.clone().unwrap().to_string();
        let has_skip_attr = attrs.iter().any(|it| {
            if let Meta::List(list_meta) = &it.meta {
                list_meta.path.segments.first().unwrap().ident == "select"
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

        Some(quote! { Alias::new(#str_ident) })
    });
    let impls = fields.iter().filter_map(|it| {
        let Field {
            ident, attrs, ty, ..
        } = it;

        let by_ident = format_ident!("by_{}", ident.clone().unwrap());
        let str_ident = ident.clone().unwrap().to_string();
        let has_find_attr = attrs.iter().any(|it| {
            if let Meta::List(list_meta) = &it.meta {
                list_meta.path.segments.first().unwrap().ident == "select"
                    && list_meta.tokens.clone().into_iter().any(|it| {
                        if let TokenTree::Ident(ident) = it {
                            ident == "find"
                        } else {
                            false
                        }
                    })
            } else {
                false
            }
        });

        if !has_find_attr {
            return None;
        }

        if let Type::Path(ty) = ty {
            let ty_ident = ty.path.segments.first().unwrap().ident.clone();

            return Some(quote! {
                pub fn #by_ident(&mut self, #ident: #ty_ident) -> &mut Self {
                    self.query_builder.and_where(Expr::col(Alias::new(#str_ident)).eq(#ident));

                    self
                }
            });
        }

        None
    });

    let expanded = quote! {
        #[derive(Debug, Clone)]
        pub struct #builder_ident {
            query_builder: sea_query::SelectStatement,
        }

        impl #builder_ident {
            fn by_id(&mut self, id: &str) -> &mut Self {
                self.query_builder.and_where(sea_query::Expr::col(#iden_ident::Id).eq(id));

                self
            }

            async fn finish(&mut self, db_pool: &deadpool_postgres::Pool) -> Result<#ident, crate::error::Error> {
                let row = db_pool
                    .get()
                    .await
                    .unwrap()
                    .query(self.query_builder.to_string(sea_query::PostgresQueryBuilder).as_str(), &[])
                    .await
                    .map_err(|e| {
                        let error = format!(
                            "An error occurred while fetching the {}: {e}",
                            #ident::error_identifier(),
                        );
                        crate::error::Error::InternalServerError(error)
                    })?
                    .into_iter()
                    .next()
                    .ok_or_else(|| {
                        let message = format!("No {} was found", #ident::error_identifier());
                        crate::error::Error::BadRequest(message)
                    })?;

                Ok(row.into())
            }

            #(#impls)*

            pub fn and_where(&mut self, expressions: Vec<sea_query::SimpleExpr>) -> &mut Self {
                for expression in expressions {
                    self.query_builder.and_where(expression);
                }

                self
            }

            pub async fn one(&mut self, db_pool: &deadpool_postgres::Pool) -> Result<#ident, crate::error::Error> {
                self.query_builder.reset_limit().limit(1);

                self.finish(db_pool).await
            }

            pub async fn all(&mut self, db_pool: &deadpool_postgres::Pool) -> Result<#ident, crate::error::Error> {
                self.query_builder.reset_limit();

                self.finish(db_pool).await
            }
        }

        impl #ident {
            pub fn find() -> #builder_ident {
                #builder_ident {
                    query_builder: sea_query::Query::select()
                        .from(Self::table())
                        .columns([
                            #(#aliases,)*
                        ])
                        .to_owned(),
                }
            }

            pub fn find_by_id(id: &str) -> #builder_ident {
                let mut builder = Self::find();
                builder.by_id(id).to_owned()
            }
        }
    };

    expanded.into()
}
