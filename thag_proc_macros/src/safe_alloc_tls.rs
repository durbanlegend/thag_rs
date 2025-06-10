#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Result};

// Custom parser that can handle both expressions and statement blocks
struct SafeAllocTlsInput {
    content: proc_macro2::TokenStream,
}

impl Parse for SafeAllocTlsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse everything as a token stream - this handles both expressions and statements
        let content = input.parse()?;
        Ok(Self { content })
    }
}

pub fn safe_alloc_tls_impl(input: TokenStream) -> TokenStream {
    let SafeAllocTlsInput { content } = parse_macro_input!(input as SafeAllocTlsInput);

    let expanded = quote! {
        {
            // Use thread-local storage for better async/threading isolation
            // Only change false->true, never true->false unless we set it
            let was_already_using_sys = crate::mem_tracking::get_tls_using_system();
            
            if !was_already_using_sys {
                crate::mem_tracking::set_tls_using_system(true);
            }

            // Execute the provided code (whether expression or statements)
            let result = {
                #content
            };

            // Restore flag only if we set it (was false before)
            if !was_already_using_sys {
                crate::mem_tracking::set_tls_using_system(false);
            }

            result
        }
    };

    TokenStream::from(expanded)
}