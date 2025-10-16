#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

pub fn attribute_basic_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = item.clone().into();
    let attr: proc_macro2::TokenStream = attr.clone().into();

    // Annotate the item with the requested attribute
    let expanded = quote! {
        #[#attr]
        #item
    };
    TokenStream::from(expanded)
}
