#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Result};

// Custom parser that handles format string and arguments like println!
struct DebugLogInput {
    format_args: proc_macro2::TokenStream,
}

impl Parse for DebugLogInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse all tokens as format arguments (same as format! macro)
        let format_args = input.parse()?;
        Ok(Self { format_args })
    }
}

pub fn debug_log_impl(input: TokenStream) -> TokenStream {
    let DebugLogInput { format_args } = parse_macro_input!(input as DebugLogInput);

    // Check if debug logging is enabled via feature
    #[cfg(feature = "debug_logging")]
    let expanded = quote! {
        {
            ::thag_profiler::safe_alloc! {
                if let Some(logger) = ::thag_profiler::DebugLogger::get() {
                    use std::io::Write;
                    let _write_result = {
                        let mut locked_writer = logger.lock();
                        writeln!(locked_writer, "{}", format!(#format_args))
                    };
                    // No auto-flush to prevent deadlocks - rely on explicit flush calls
                }
            }
        }
    };

    // When debug logging is disabled, compile to nothing
    #[cfg(not(feature = "debug_logging"))]
    let expanded = quote! {
        {
            // Zero-cost: compile to nothing when debug_logging feature is disabled
        }
    };

    TokenStream::from(expanded)
}