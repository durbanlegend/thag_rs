#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn attribute_basic_impl(tokens: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = tokens.clone().into();

    // Attach logic to the input
    let expanded = quote! {
        #[allow(unused_variables)]
        #tokens
    };
    TokenStream::from(expanded)
}
