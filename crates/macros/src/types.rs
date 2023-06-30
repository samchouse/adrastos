use core::fmt;

use proc_macro2::Ident;
use quote::format_ident;
use syn::PathSegment;

#[derive(Debug, PartialEq)]
pub enum Type {
    Bool,
    String,
    DateTime,
    Vec(Box<Type>),
    Option(Box<Type>),
    Unknown(String),
}

impl Type {
    pub fn into_ident(self) -> Ident {
        self.into()
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Type::Bool => "bool",
            Type::String => "String",
            Type::DateTime => "DateTime",
            Type::Vec(_) => "Vec",
            Type::Option(_) => "Option",
            Type::Unknown(name) => name,
        };

        write!(f, "{}", name)
    }
}

impl From<Type> for Ident {
    fn from(value: Type) -> Self {
        format_ident!("{}", value.to_string())
    }
}

impl From<syn::Type> for Type {
    fn from(value: syn::Type) -> Self {
        match value {
            syn::Type::Path(ty) => {
                let seg = ty.path.segments.first().unwrap();
                match seg.ident.to_string().as_str() {
                    "bool" => Type::Bool,
                    "String" => Type::String,
                    "DateTime" => Type::DateTime,
                    "Vec" => Type::Vec(recurse_segment(seg)),
                    "Option" => Type::Option(recurse_segment(seg)),
                    _ => Type::Unknown(seg.ident.to_string()),
                }
            }
            _ => unimplemented!(),
        }
    }
}

fn recurse_segment(seg: &PathSegment) -> Box<Type> {
    if let syn::PathArguments::AngleBracketed(brackets) = &seg.arguments {
        let generic = brackets.args.first();
        if let Some(syn::GenericArgument::Type(syn::Type::Path(ty))) = generic {
            let seg = ty.path.segments.first();
            if let Some(seg) = seg {
                return Box::new(match seg.ident.to_string().as_str() {
                    "bool" => Type::Bool,
                    "String" => Type::String,
                    "DateTime" => Type::DateTime,
                    "Vec" => Type::Vec(recurse_segment(seg)),
                    "Option" => Type::Option(recurse_segment(seg)),
                    _ => Type::Unknown(seg.ident.to_string()),
                });
            }
        }
    }

    panic!("Something went wrong parsing the type")
}
