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
            // Inline the sys_alloc logic directly (no function call)
            let was_already_using_sys = crate::mem_tracking::USING_SYSTEM_ALLOCATOR
                .swap(true, std::sync::atomic::Ordering::SeqCst);

            // Execute the provided code (whether expression or statements)
            let result = {
                #content
            };

            // Restore flag only if we set it
            if !was_already_using_sys {
                crate::mem_tracking::USING_SYSTEM_ALLOCATOR
                    .store(false, std::sync::atomic::Ordering::SeqCst);
            }

            result
        }
    };

    TokenStream::from(expanded)
}
