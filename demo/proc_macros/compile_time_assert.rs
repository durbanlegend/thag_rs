#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Expr, LitStr, Token};

/// Parse the input for `compile_time_assert!(condition`, "message")
struct CompileTimeAssertInput {
    condition: Expr,
    message: LitStr,
}

impl Parse for CompileTimeAssertInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let condition = input.parse()?;
        input.parse::<Token![,]>()?;
        let message = input.parse()?;
        Ok(Self { condition, message })
    }
}

pub fn compile_time_assert_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as CompileTimeAssertInput);
    let condition = &input.condition;
    let message = &input.message;

    // Generate a const assertion that will fail compilation if the condition is false
    let expanded = quote! {
        const _: () = {
            if !(#condition) {
                panic!(#message);
            }
        };
    };

    TokenStream::from(expanded)
}
