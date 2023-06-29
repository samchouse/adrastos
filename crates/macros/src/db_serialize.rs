use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Field, Fields, ItemStruct, Type};

pub fn derive(item: TokenStream) -> TokenStream {
    let ItemStruct { ident, fields, .. } = parse_macro_input!(item as ItemStruct);

    let fields = match fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("Only structs with fields are supported"),
    };

    let fields = fields.iter().map(|it| {
        let Field { ident, ty, .. } = it;
        let str_ident = ident.clone().unwrap().to_string();

        if let Type::Path(ty) = ty {
            let name = ty.path.segments.first();
            if let Some(name) = name {
                match name.ident.to_string().as_str() {
                    "String" | "u32" | "bool" | "DateTime" => return quote! { #ident: row.get(#str_ident) },
                    "Vec" => {
                        let second_name = ty.path.segments.iter().nth(2);
                        if let Some(second_name) = second_name {
                            match second_name.ident.to_string().as_str() {
                                "String" | "u32" | "bool" => return quote! { #ident: row.get(#str_ident) },
                                _ => {}
                            }
                        }

                        return quote!{ #ident: row.get::<_, Vec<String>>(#str_ident).iter().map(|s| serde_json::from_str(s).unwrap()).collect::<Vec<_>>() }
                    },
                    "Option" => {
                        let second_name = ty.path.segments.iter().nth(2);
                        if let Some(second_name) = second_name {
                            match second_name.ident.to_string().as_str() {
                                "String" | "u32" | "bool" | "DateTime" => return quote! { #ident: row.get(#str_ident) },
                                "Vec" => {
                                    let third_name = ty.path.segments.iter().nth(3);
                                    if let Some(third_name) = third_name {
                                        match third_name.ident.to_string().as_str() {
                                            "String" | "u32" | "bool" => return quote! { #ident: row.get(#str_ident) },
                                            _ => {}
                                        }
                                    }

                                    return quote!{ #ident: row.try_get::<_, Option<Vec<String>>>(#str_ident).ok().map(|v| v.iter().map(|s| serde_json::from_str(s).unwrap()).collect::<Vec<_>>()) }
                                }
                                _ => {}
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
        impl From<deadpool_postgres::tokio_postgres::Row> for #ident {
            fn from(row: deadpool_postgres::tokio_postgres::Row) -> Self {
                #ident {
                    #(#fields),*
                }
            }
        }
    }
    .into()
}
