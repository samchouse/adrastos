use proc_macro::TokenStream;

mod db_select;
mod db_serialize;

#[proc_macro_derive(DbDeserialize)]
pub fn derive_db_serialize(item: TokenStream) -> TokenStream {
    db_serialize::derive(item)
}

#[proc_macro_derive(DbSelect, attributes(select))]
pub fn derive_db_select(item: TokenStream) -> TokenStream {
    db_select::derive(item)
}
