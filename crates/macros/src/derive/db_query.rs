use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Field, ItemStruct};

use crate::{
    attributes::{AttributeTokens, Token, TokenName},
    types::Type,
};

pub fn db_query(item: TokenStream) -> TokenStream {
    let ItemStruct {
        ident,
        fields,
        attrs,
        ..
    } = parse_macro_input!(item as ItemStruct);
    let attrs = AttributeTokens::from(attrs);

    let create_validator = attrs.get(TokenName::Validated).is_some().then_some(quote! {
        self.validate().map_err(|err| crate::error::Error::ValidationErrors {
            message: format!("An error occurred while validating the {}", Self::error_identifier()),
            errors: err,
        })?;
    });

    let create_columns = fields.iter().filter_map(|it| {
        let Field { ident, attrs, .. } = it;
        let attrs = AttributeTokens::from(attrs.clone());

        if attrs.get(TokenName::Join).is_some() {
            return None;
        }

        let str_ident = ident.clone().unwrap().to_string();
        Some(quote! { sea_query::Alias::new(#str_ident) })
    });

    let create_values = fields.iter().filter_map(|it| {
        let Field {
            ident, ty, attrs, ..
        } = it;
        let attrs = AttributeTokens::from(attrs.clone());
        let ty = Type::from(ty.clone());

        if attrs.get(TokenName::Join).is_some() {
            return None;
        }

        let value = match ty {
            Type::String => quote! { self.#ident.clone().into() },
            Type::Vec(generic) => match *generic {
                Type::String => quote! { self.#ident.clone().into() },
                _ => quote! {
                    self.#ident.iter()
                        .filter_map(|f| serde_json::to_string(f).ok())
                        .collect::<Vec<String>>()
                        .into()
                },
            },
            Type::Option(generic) => match *generic {
                Type::String => quote! { self.#ident.clone().into() },
                Type::Vec(generic) => match *generic {
                    Type::String => quote! { self.#ident.clone().into() },
                    _ => quote! {
                        self.#ident.iter()
                            .filter_map(|f| serde_json::to_string(f).ok())
                            .collect::<Option<Vec<String>>>()
                            .into()
                    },
                },
                _ => quote! { self.#ident.into() },
            },
            _ if attrs.get(TokenName::Json).is_some() => {
                quote! { serde_json::to_string(&self.#ident).unwrap().into() }
            }
            _ => quote! { self.#ident.into() },
        };

        let transform = attrs.get(TokenName::Transform).map(|t| {
            if let Token::Transform(name) = t {
                return name.clone();
            }

            panic!("Expected transform token")
        });

        if let Some(transform_ident) = transform {
            return Some(quote! { #transform_ident(#value)?.into() });
        }

        Some(value)
    });

    quote! {
        impl #ident {
            pub async fn create(&self, db_pool: &deadpool_postgres::Pool) -> Result<(), crate::error::Error> {
                #create_validator

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
