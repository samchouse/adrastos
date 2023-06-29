use heck::{AsSnakeCase, AsUpperCamelCase};
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Field, Fields, ItemStruct, Meta};

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
    let error_identifier = table_name.replace('_', " ");
    let table_identifier = format!("{}s", table_name);

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

    quote! {
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
    }
    .into()
}
