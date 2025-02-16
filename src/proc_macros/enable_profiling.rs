#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Ident, ItemFn, LitStr, Token,
};

#[derive(Debug)]
enum ProfileTypeOverride {
    Time,
    Memory,
    Both,
}

struct ProfileArgs {
    profile_type: Option<ProfileTypeOverride>,
}

// Custom parsing for the attribute arguments
impl Parse for ProfileArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let profile_type = if input.is_empty() {
            None
        } else {
            let ident: Ident = input.parse()?;
            if ident != "profile_type" {
                return Err(syn::Error::new(ident.span(), "Expected 'type'"));
            }
            let _: Token![=] = input.parse()?;
            let type_str: LitStr = input.parse()?;
            Some(match type_str.value().as_str() {
                "time" => ProfileTypeOverride::Time,
                "memory" => ProfileTypeOverride::Memory,
                "both" => ProfileTypeOverride::Both,
                _ => {
                    return Err(syn::Error::new(
                        type_str.span(),
                        "Invalid profile type. Expected 'time', 'memory', or 'both'",
                    ))
                }
            })
        };

        Ok(Self { profile_type })
    }
}

pub fn enable_profiling_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ProfileArgs);
    let mut input = parse_macro_input!(item as ItemFn);

    let crate_path = get_crate_path();
    let profile_type = match args.profile_type {
        Some(ProfileTypeOverride::Time) => quote! { ProfileType::Time },
        Some(ProfileTypeOverride::Memory) => quote! { ProfileType::Memory },
        Some(ProfileTypeOverride::Both) | None => quote! { ProfileType::Both },
    };

    // Create the new function body
    let original_body = input.block;
    input.block = parse_quote! {{
        use #crate_path::profiling::{enable_profiling, ProfileType};

        enable_profiling(true, #profile_type)
            .expect("Failed to enable profiling");

        let result = #original_body;  // Just use the body directly

        enable_profiling(false, #profile_type)
            .expect("Failed to disable profiling");

        result
    }};

    quote! {
        #input
    }
    .into()
}

fn get_crate_path() -> proc_macro2::TokenStream {
    if std::env::var("CARGO_BIN_NAME").is_ok() {
        quote! { thag_rs }
    } else {
        match crate_name("thag_rs") {
            Ok(FoundCrate::Itself) => quote! { crate },
            Ok(FoundCrate::Name(name)) => {
                let ident = format_ident!("{}", name);
                quote! { #ident }
            }
            Err(_) => quote! { thag_rs },
        }
    }
}
