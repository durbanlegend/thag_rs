#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, LitStr};

pub fn ansi_code_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Enum(DataEnum { variants, .. }) = input.data else {
        panic!("AnsiCodeDerive can only be derived for enums")
    };

    // Generate name() method match arms
    let name_match_arms = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;

        // Look for custom name in attributes
        let custom_name = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("ansi_name"))
            .and_then(|attr| attr.parse_args::<LitStr>().ok().map(|lit| lit.value()));

        // if let Some(custom_name) = custom_name.clone() {
        //     eprintln!("custom_name={custom_name}");
        // }

        // Use custom name or generate from variant name
        let name_str = custom_name.unwrap_or_else(|| match variant_ident.to_string().as_str() {
            s if s.starts_with("Bright") => {
                let chars = s.chars();
                let rest: String = chars.skip(6).collect();
                // let rest: String = chars.collect();
                format!("Bright {rest}")
            }
            s => s.to_string(),
        });
        // eprintln!("name_str={name_str}");

        quote! {
            Self::#variant_ident => #name_str,
        }
    });

    // Generate FromStr match arms
    let from_str_match_arms = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        // Convert camelCase to snake_case for matching
        let str_name = variant_ident
            .to_string()
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i > 0 && c.is_uppercase() {
                    format!("_{}", c.to_lowercase())
                } else {
                    c.to_lowercase().to_string()
                }
            })
            .collect::<String>();

        quote! {
            #str_name => Ok(Self::#variant_ident),
        }
    });

    // Generate documentation
    let doc = format!(
        r#" Get a human-readable name for the ANSI color.

Returns a static string representing the color name.

# Examples
```
use thag_rs::styling::AnsiCode;
assert_eq!({name}::Red.name(), "Red");
assert_eq!({name}::BrightBlue.name(), "Bright Blue");
```
"#,
    );

    let expanded = quote! {
        impl #name {
            #[doc = #doc]
            pub fn name(self) -> &'static str {
                match self {
                    #(#name_match_arms)*
                }
            }
        }

        impl std::str::FromStr for #name {
            type Err = ThagError;
            fn from_str(s: &str) -> ThagResult<Self> {
                match s.to_lowercase().as_str() {
                    #(#from_str_match_arms)*
                    _ => Err(ThemeError::InvalidAnsiCode(s.to_string()).into()),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
