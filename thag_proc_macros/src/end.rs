#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitStr, Result,
};

// Define a custom parser that accepts either a string literal or an identifier
struct ProfileIdentifier {
    value: String,
}

impl Parse for ProfileIdentifier {
    fn parse(input: ParseStream) -> Result<Self> {
        // Try parsing as a string literal first
        if input.peek(LitStr) {
            let lit: LitStr = input.parse()?;
            return Ok(Self { value: lit.value() });
        }

        // If not a string, try parsing as an identifier
        let ident: Ident = input.parse()?;
        Ok(Self {
            value: ident.to_string(),
        })
    }
}

pub fn end_impl(input: TokenStream) -> TokenStream {
    // Parse the input using our custom parser
    let profile_identifier = parse_macro_input!(input as ProfileIdentifier);
    let profile_id_str = profile_identifier.value;
    let profile_id = format_ident!("{profile_id_str}");

    // Convert the string to an identifier
    let func_name = format_ident!("end_{profile_id_str}");

    #[cfg(feature = "full_profiling")]
    let expanded = quote! {
        fn #func_name() -> u32 { line!() }

        ::thag_profiler::with_allocator(::thag_profiler::Allocator::System, || {
            if let Some(profile) = #profile_id {
                drop(profile);
            }
        });
    };

    #[cfg(not(feature = "full_profiling"))]
    let expanded = quote! {
        fn #func_name() -> u32 { line!() }

        if let Some(profile) = #profile_id {
            drop(profile);
        }
    };

    expanded.into()
}
