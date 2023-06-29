use proc_macro::TokenStream;

mod db_common;
mod db_query;
mod db_select;

#[proc_macro_derive(DbCommon, attributes(adrastos))]
pub fn derive_db_common(item: TokenStream) -> TokenStream {
    db_common::derive(item)
}

#[proc_macro_derive(DbQuery, attributes(query))]
pub fn derive_db_query(item: TokenStream) -> TokenStream {
    db_query::derive(item)
}

#[proc_macro_derive(DbSelect, attributes(adrastos))]
pub fn derive_db_select(item: TokenStream) -> TokenStream {
    db_select::derive(item)
}
