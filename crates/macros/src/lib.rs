use proc_macro::TokenStream;
use quote::quote;

mod db_identity;
mod db_schema;
mod db_select;
mod db_serialize;

#[proc_macro_derive(DbCommon, attributes(adrastos))]
pub fn derive_db_common(item: TokenStream) -> TokenStream {
    let identity: proc_macro2::TokenStream = db_identity::derive(item.clone()).into();
    let schema: proc_macro2::TokenStream = db_schema::derive(item.clone()).into();
    let serialize: proc_macro2::TokenStream = db_serialize::derive(item).into();

    quote! {
        #identity
        #schema
        #serialize
    }
    .into()
}

#[proc_macro_derive(DbSelect, attributes(adrastos))]
pub fn derive_db_select(item: TokenStream) -> TokenStream {
    db_select::derive(item)
}
