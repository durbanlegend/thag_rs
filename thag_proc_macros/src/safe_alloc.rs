#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Result};

// Custom parser that can handle both expressions and statement blocks
struct SafeAllocInput {
    content: proc_macro2::TokenStream,
}

impl Parse for SafeAllocInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse everything as a token stream - this handles both expressions and statements
        let content = input.parse()?;
        Ok(Self { content })
    }
}

pub fn safe_alloc_impl(input: TokenStream) -> TokenStream {
    let SafeAllocInput { content } = parse_macro_input!(input as SafeAllocInput);

    let expanded = quote! {
        {
            let was_already_using_sys = mem_tracking::compare_exchange_using_system(false, true).is_err();

            let result = { #content };

            if !was_already_using_sys {
                mem_tracking::set_using_system(false);
            }

            result
        }
    };

    TokenStream::from(expanded)
}
