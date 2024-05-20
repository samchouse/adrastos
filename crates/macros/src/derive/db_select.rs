use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Field, Fields, ItemStruct};

use crate::{
    attribute_parser::{AttributeTokens, Token, TokenName},
    types::Type,
};

pub fn db_select(item: TokenStream) -> TokenStream {
    let ItemStruct {
        ident,
        fields,
        attrs,
        ..
    } = parse_macro_input!(item as ItemStruct);

    let iden_ident = format_ident!("{}Iden", ident);
    let join_ident = format_ident!("{}Join", ident);
    let builder_ident = format_ident!("{}SelectBuilder", ident);

    let fields = match fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("Only structs with fields are supported"),
    };

    let aliases = fields.iter().filter_map(|it| {
        let Field { ident, attrs, .. } = it;
        let attrs = AttributeTokens::from(attrs.clone());

        if attrs.get(TokenName::Join).is_some() {
            return None;
        }

        let str_ident = ident.clone().unwrap().to_string();
        Some(quote! { sea_query::Alias::new(#str_ident) })
    });

    let impls = fields.iter().filter_map(|it| {
        let Field {
            ident, attrs, ty, ..
        } = it;
        let attrs = AttributeTokens::from(attrs.clone());
        let ty = Type::from(ty.clone());

        attrs.get(TokenName::Find)?;

        let by_ident = format_ident!("by_{}", ident.clone().unwrap());
        let str_ident = ident.clone().unwrap().to_string();

        let ty_ident = ty.into_ident();
        Some(quote! {
            pub fn #by_ident(&mut self, #ident: #ty_ident) -> &mut Self {
                self.query_builder.and_where(sea_query::Expr::col(sea_query::Alias::new(#str_ident)).eq(#ident));

                self
            }
        })
    });

    let enum_variants = fields.iter().filter_map(|it| {
        let Field { attrs, ident, .. } = it;
        let attrs = AttributeTokens::from(attrs.clone());

        attrs.get(TokenName::Join)?;

        Some((
            it,
            format_ident!(
                "{}",
                ident.clone().unwrap().to_string().to_upper_camel_case()
            ),
        ))
    });

    let variant_ident = enum_variants
        .clone()
        .map(|(_, variant_ident)| quote! { #variant_ident });

    let query_branches = enum_variants.clone().map(|(it, variant_ident)| {
        let Field { ty, .. } = it;
        let ty = Type::from(ty.clone());
        let attrs = AttributeTokens::from(attrs.clone());

        let str_ident = match attrs.get(TokenName::JoinIdent) {
            Some(Token::JoinIdent(ident)) => format!("{}_id", ident),
            _ => format!("{}_id", ident.clone().to_string().to_lowercase()),
        };
        let ident = match ty {
            Type::Vec(generic) => generic.into_ident(),
            Type::Option(generic) => match *generic {
                Type::Vec(generic) => generic.into_ident(),
                _ => generic.into_ident(),
            }
            _ => ty.into()
        };

        quote! { #join_ident::#variant_ident => #ident::find().and_where(vec![sea_query::Expr::col(sea_query::Alias::new(#str_ident)).equals((#ident::table(), sea_query::Alias::new(#str_ident)))]).to_string() }
    });

    let string_branches = enum_variants.clone().flat_map(|(it, variant_ident)| {
        let Field { ty, .. } = it;
        let ty = Type::from(ty.clone());

        let ident = format!(
            "{}s",
            match ty {
                Type::Vec(generic) => *generic,
                Type::Option(generic) => match *generic {
                    Type::Vec(generic) => *generic,
                    _ => *generic,
                },
                _ => ty,
            }
        )
        .to_snake_case();

        quote! { #join_ident::#variant_ident => #ident.to_string(), }
    });

    let join_enum = if !enum_variants.clone().collect::<Vec<_>>().is_empty() {
        quote! {
            pub enum #join_ident {
                #(#variant_ident),*
            }

            impl std::fmt::Display for #join_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let name = match self {
                        #(#string_branches)*
                    };

                    write!(f, "{}", name)
                }
            }
        }
    } else {
        quote! {}
    };

    let join_fn = if !enum_variants.clone().collect::<Vec<_>>().is_empty() {
        quote! {
            pub fn join(&mut self, join: #join_ident) -> &mut Self {
                let query = match join {
                    #(#query_branches),*
                };

                self.query_builder.expr(sea_query::Expr::cust(
                    format!("(SELECT json_agg({join}) FROM ({query}) {join}) as {join}").as_str(),
                ));

                self
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #join_enum

        #[derive(Debug, Clone)]
        pub struct #builder_ident {
            query_builder: sea_query::SelectStatement,
        }

        impl #builder_ident {
            fn by_id(&mut self, id: &str) -> &mut Self {
                self.query_builder.and_where(sea_query::Expr::col(#iden_ident::Id).eq(id));

                self
            }

            async fn finish(&mut self, db: &deadpool_postgres::Pool) -> Result<Vec<deadpool_postgres::tokio_postgres::Row>, crate::error::Error> {
                let rows = db
                    .get()
                    .await
                    .unwrap()
                    .query(self.to_string().as_str(), &[])
                    .await
                    .map_err(|e| {
                        let error = format!(
                            "An error occurred while fetching the {}: {e}",
                            #ident::error_identifier(),
                        );
                        crate::error::Error::InternalServerError(error)
                    })?;

                Ok(rows)
            }

            #(#impls)*

            pub fn and_where(&mut self, expressions: Vec<sea_query::SimpleExpr>) -> &mut Self {
                for expression in expressions {
                    self.query_builder.and_where(expression);
                }

                self
            }

            #join_fn

            pub async fn one(&mut self, db: &deadpool_postgres::Pool) -> Result<#ident, crate::error::Error> {
                self.query_builder.reset_limit().limit(1);

                Ok(self
                    .finish(db)
                    .await?
                    .into_iter()
                    .next()
                    .ok_or_else(|| {
                        let message = format!("No {} was found", #ident::error_identifier());
                        crate::error::Error::BadRequest(message)
                    })?
                    .into())
            }

            pub async fn all(&mut self, db: &deadpool_postgres::Pool) -> Result<Vec<#ident>, crate::error::Error> {
                self.query_builder.reset_limit();

                // TODO(@Xenfo): add pagination, etc.
                Ok(self
                    .finish(db)
                    .await?
                    .into_iter()
                    .map(|row| row.into())
                    .collect::<Vec<_>>())
            }

            pub fn to_string(&self) -> String {
                self.query_builder.to_string(sea_query::PostgresQueryBuilder)
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
    }.into()
}
