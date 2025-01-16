#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

pub fn ansi_code_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let Data::Enum(DataEnum { variants, .. }) = input.data else {
        panic!("AnsiName can only be derived for enums")
    };

    let match_arms = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let name_str = match variant_ident.to_string().as_str() {
            s if s.starts_with("Bright") => {
                // Insert space after "Bright"
                let mut chars = s.chars();
                let first = chars.next().unwrap();
                let rest: String = chars.collect();
                format!("{first} {rest}")
            }
            s => s.to_string(),
        };

        quote! {
            Self::#variant_ident => #name_str,
        }
    });

    let expanded = quote! {
        impl #name {
            /// Get a readable name for the ANSI color
            pub fn name(self) -> &'static str {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
