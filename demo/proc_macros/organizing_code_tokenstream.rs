#![allow(
    dead_code,
    unused_variables,
    clippy::redundant_pub_crate,
    clippy::module_name_repetitions
)]
// From https://github.com/tdimitrov/rust-proc-macro-post
use proc_macro2::TokenStream;
use quote::quote;

pub fn organizing_code_tokenstream_impl(input: TokenStream) -> TokenStream {
    let mut result = Vec::new();

    result.push(progress_message("Thinking about the answer".to_string()));
    result.push(answer(input));

    println!("result={result:#?}");

    quote!(
        #(#result);*
    )
}

fn progress_message(msg: String) -> TokenStream {
    quote!(println!(#msg))
}

fn answer(result: TokenStream) -> TokenStream {
    quote!(println!("Answer: {}", #result))
}
