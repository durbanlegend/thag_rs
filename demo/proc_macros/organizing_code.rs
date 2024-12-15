#![allow(clippy::module_name_repetitions)]
// From https://github.com/tdimitrov/rust-proc-macro-post
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, ExprMacro};

pub fn organizing_code_impl(input: TokenStream) -> TokenStream {
    let progress = progress_message("Thinking about the answer".to_string());
    let answer = answer(input);

    println!("answer={answer:#?}");

    quote!(
        #progress;
        #answer;
    )
}

fn progress_message(msg: String) -> ExprMacro {
    parse_quote!(println!(#msg))
}

fn answer(result: TokenStream) -> ExprMacro {
    parse_quote!(println!("Answer: {}", #result))
}
