use heck::{AsUpperCamelCase, AsSnakeCase};
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
                list_meta.path.segments.first().unwrap().ident == "adrastos"
                    && list_meta.tokens.clone().into_iter().any(|it| {
                        if let TokenTree::Ident(ident) = it {
                            ident == "join"
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
    let alias_2 = aliases.clone();

    let impls = fields.iter().filter_map(|it| {
        let Field {
            ident, attrs, ty, ..
        } = it;

        let by_ident = format_ident!("by_{}", ident.clone().unwrap());
        let str_ident = ident.clone().unwrap().to_string();
        let has_find_attr = attrs.iter().any(|it| {
            if let Meta::List(list_meta) = &it.meta {
                list_meta.path.segments.first().unwrap().ident == "adrastos"
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

    let join_ident = format_ident!("{}Join", ident);

    let enum_variants = fields.iter().filter(|it| {
        let Field { attrs, .. } = it;

        let has_join_attr = attrs.iter().any(|it| {
            if let Meta::List(list_meta) = &it.meta {
                list_meta.path.segments.first().unwrap().ident == "adrastos"
                    && list_meta.tokens.clone().into_iter().any(|it| {
                        if let TokenTree::Ident(ident) = it {
                            ident == "join"
                        } else {
                            false
                        }
                    })
            } else {
                false
            }
        });

        has_join_attr
    });

    let e = enum_variants.clone().map(|it| {
        let Field { ident, .. } = it;

        let variant_ident = format_ident!(
            "{}",
            AsUpperCamelCase(ident.clone().unwrap().to_string()).to_string()
        );

        Some(quote! { #variant_ident })
    });

    let ea = enum_variants.clone().filter_map(|it| {
        let Field { ident, ty, .. } = it;

        let str_ident = ident.clone().unwrap().to_string();
        let variant_ident = format_ident!(
            "{}",
            AsUpperCamelCase(ident.clone().unwrap().to_string()).to_string()
        );

        if let Type::Path(ty) = ty {
            let name = ty.path.segments.first();
            if let Some(name) = name {
                match name.ident.to_string().as_str() {
                    "Vec" => {
                        if let syn::PathArguments::AngleBracketed(name) = &name.arguments {
                            let second_name = name.args.first();
                            if let Some(syn::GenericArgument::Type(Type::Path(ty))) = second_name {
                                let second_name = ty.path.segments.first();
                                if let Some(second_name) = second_name {
                                    let ident = second_name.ident.to_string();
                                    return Some(quote! { #join_ident::#variant_ident => #ident::find().and_where(vec![sea_query::Expr::col(sea_query::Alias::new(#str_ident)).equals((#ident::table(), sea_query::Alias::new("id")))]).to_string() });
                                }
                            }
                        }
                    }
                    "Option" => {
                        if let syn::PathArguments::AngleBracketed(name) = &name.arguments {
                            let second_name = name.args.first();
                            if let Some(syn::GenericArgument::Type(Type::Path(ty))) = second_name {
                                let second_name = ty.path.segments.first();
                                if let Some(second_name) = second_name {
                                    if second_name.ident.to_string().as_str() == "Vec" {
                                        if let syn::PathArguments::AngleBracketed(name) =
                                            &second_name.arguments
                                        {
                                            let third_name = name.args.first();
                                            if let Some(syn::GenericArgument::Type(
                                                Type::Path(ty),
                                            )) = third_name
                                            {
                                                let third_name = ty.path.segments.first();
                                                if let Some(third_name) = third_name {
                                                    let ident = &third_name.ident;
                                                    return Some(quote! {
                                                        #join_ident::#variant_ident => #ident::find().and_where(vec![sea_query::Expr::col(sea_query::Alias::new(#str_ident)).equals((#ident::table(), sea_query::Alias::new("id")))]).to_string()
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        let ident = &name.ident;
                        return Some(quote! { #join_ident::#variant_ident => #ident::find().and_where(vec![sea_query::Expr::col(sea_query::Alias::new(#str_ident)).equals((#ident::table(), sea_query::Alias::new("id")))]).to_string() });
                    }
                }
            }
        }

        return None;
    });

    let eaa = enum_variants.clone().flat_map(|it| {
        let Field { ident, ty, .. } = it;
        
        let variant_ident = format_ident!(
            "{}",
            AsUpperCamelCase(ident.clone().unwrap().to_string()).to_string()
        );

        if let Type::Path(ty) = ty {
            let name = ty.path.segments.first();
            if let Some(name) = name {
                match name.ident.to_string().as_str() {
                    "Vec" => {
                        if let syn::PathArguments::AngleBracketed(name) = &name.arguments {
                            let second_name = name.args.first();
                            if let Some(syn::GenericArgument::Type(Type::Path(ty))) = second_name {
                                let second_name = ty.path.segments.first();
                                if let Some(second_name) = second_name {
                                    let ident = AsSnakeCase(second_name.ident.to_string()).to_string();
                                    return Some(quote! { #join_ident::#variant_ident => #ident });
                                }
                            }
                        }
                    }
                    "Option" => {
                        if let syn::PathArguments::AngleBracketed(name) = &name.arguments {
                            let second_name = name.args.first();
                            if let Some(syn::GenericArgument::Type(Type::Path(ty))) = second_name {
                                let second_name = ty.path.segments.first();
                                if let Some(second_name) = second_name {
                                    if second_name.ident.to_string().as_str() == "Vec" {
                                        if let syn::PathArguments::AngleBracketed(name) =
                                            &second_name.arguments
                                        {
                                            let third_name = name.args.first();
                                            if let Some(syn::GenericArgument::Type(
                                                Type::Path(ty),
                                            )) = third_name
                                            {
                                                let third_name = ty.path.segments.first();
                                                if let Some(third_name) = third_name {
                                                    let ident = AsSnakeCase(third_name.ident.to_string()).to_string();
                                                    return Some(quote! { #join_ident::#variant_ident => #ident });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        let ident = AsSnakeCase(name.ident.to_string()).to_string();
                        return Some(quote! { #join_ident::#variant_ident => #ident });
                    }
                }
            }
        }

        None
    });

    let ev_2 = enum_variants.clone();
    let ev_3 = enum_variants.clone();

    let other_ident = ident.to_string().to_lowercase();
    let oath = format!("{}s", other_ident);

    let aasdfadf = if enum_variants.clone().collect::<Vec<_>>().len() > 0 {
        quote! {
            pub enum #join_ident {
                #(#e),*
            }
    
            impl #join_ident {
                fn to_string(&self) -> String {
                    match self {
                        #(#eaa.to_string()),*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    let ass = if enum_variants.collect::<Vec<_>>().len() > 0 {
        quote! {
            pub fn join(&mut self, join: #join_ident) -> &mut Self {
                let query = match join {
                    #(#ea),*
                };

                self.query_builder.expr(sea_query::Expr::cust(
                    format!(
                        "(SELECT json_agg({}) FROM ({query}) {}) as {}",
                        join.to_string(),
                        join.to_string(),
                        format!("{}s", join.to_string())
                    )
                    .as_str(),
                ));

                self
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #aasdfadf

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
                    .query(self.to_string().as_str(), &[])
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

            #ass

            pub async fn one(&mut self, db_pool: &deadpool_postgres::Pool) -> Result<#ident, crate::error::Error> {
                self.query_builder.reset_limit().limit(1);

                self.finish(db_pool).await
            }

            pub async fn all(&mut self, db_pool: &deadpool_postgres::Pool) -> Result<#ident, crate::error::Error> {
                self.query_builder.reset_limit();

                self.finish(db_pool).await
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
