use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_derive(EntityBuilder)]
pub fn derive_entity_builder(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);

    let struct_name = &item.ident;
    let builder_struct_name = format_ident!("{}Builder", &item.ident);

    let expanded = quote! {
        #[derive(Debug)]
        struct #builder_struct_name(sea_query::SelectStatement);

        impl #builder_struct_name {
            pub fn by_id(&mut self, id: &str) -> &mut Self {
                self.0.and_where(sea_query::Expr::col(<#struct_name as core::Identity>::Iden::Id).eq(id));

                self
            }

            fn finish(&self) -> String {
                self.0.to_string(sea_query::PostgresQueryBuilder)
            }
        }

        impl #struct_name {
            fn select() -> #builder_struct_name {
                #builder_struct_name(sea_query::Query::select())
            }
        }
    };

    expanded.into()
}
