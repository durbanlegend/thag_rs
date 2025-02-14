#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
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
            if ident != "type" {
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

    let profile_type = match args.profile_type {
        Some(ProfileTypeOverride::Time) => quote! { crate::profiling::ProfileType::Time },
        Some(ProfileTypeOverride::Memory) => quote! { crate::profiling::ProfileType::Memory },
        Some(ProfileTypeOverride::Both) | None => quote! { crate::profiling::ProfileType::Both },
    };

    // Create the new function body
    let original_body = input.block;
    input.block = parse_quote! {{
            crate::profiling::set_profiling_enabled(true);

            crate::profiling::enable_profiling(true, #profile_type)
                .expect("Failed to enable profiling");

            let result = (|| #original_body)();

            crate::profiling::enable_profiling(false, #profile_type)
                .expect("Failed to disable profiling");

            crate::profiling::set_profiling_enabled(false);

            result
    }};

    quote! {
        #input
    }
    .into()
}
