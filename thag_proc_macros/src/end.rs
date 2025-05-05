#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, Result,
};

// Define a parser that accepts an identifier
struct ProfileIdentifier {
    value: String,
}

impl Parse for ProfileIdentifier {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse as an identifier
        let ident: Ident = input.parse()?;
        Ok(Self {
            value: ident.to_string(),
        })
    }
}

pub fn end_impl(input: TokenStream) -> TokenStream {
    // Parse the input using our parser
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