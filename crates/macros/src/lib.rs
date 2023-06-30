use proc_macro::TokenStream;

mod attributes;
mod derive;
mod types;

#[proc_macro_derive(DbCommon, attributes(adrastos))]
pub fn derive_db_common(item: TokenStream) -> TokenStream {
    derive::db_common(item)
}

#[proc_macro_derive(DbQuery, attributes(query))]
pub fn derive_db_query(item: TokenStream) -> TokenStream {
    derive::db_query(item)
}

#[proc_macro_derive(DbSelect, attributes(adrastos))]
pub fn derive_db_select(item: TokenStream) -> TokenStream {
    derive::db_select(item)
}
