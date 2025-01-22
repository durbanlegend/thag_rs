#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitInt};

pub fn repeat_dash_impl(tokens: TokenStream) -> TokenStream {
    // Parse the input as a literal integer
    let input = parse_macro_input!(tokens as LitInt);
    let len = input
        .base10_parse::<usize>()
        .expect("Expected a usize integer");

    // Generate the repeated dash string
    let dash_line = "─".repeat(len);

    // expanded a constant string definition
    TokenStream::from(quote! {
        const DASH_LINE: &str = #dash_line;
    })
}
