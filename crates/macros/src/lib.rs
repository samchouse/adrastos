use proc_macro::TokenStream;

mod db_identity;
mod db_schema;
mod db_select;
mod db_serialize;

#[proc_macro_derive(DbIdentity, attributes(adrastos))]
pub fn derive_db_identity(item: TokenStream) -> TokenStream {
    db_identity::derive(item)
}

#[proc_macro_derive(DbSchema, attributes(adrastos))]
pub fn derive_db_schema(item: TokenStream) -> TokenStream {
    db_schema::derive(item)
}

#[proc_macro_derive(DbDeserialize)]
pub fn derive_db_serialize(item: TokenStream) -> TokenStream {
    db_serialize::derive(item)
}

#[proc_macro_derive(DbSelect, attributes(adrastos))]
pub fn derive_db_select(item: TokenStream) -> TokenStream {
    db_select::derive(item)
}
