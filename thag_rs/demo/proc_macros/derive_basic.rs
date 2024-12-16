#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_basic_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let struct_name = input.ident;

    let expanded = quote! {
        impl #struct_name {
            pub fn new(field: String) -> MyStruct {
                Self { field }
            }
        }
    };
    TokenStream::from(expanded)
}
