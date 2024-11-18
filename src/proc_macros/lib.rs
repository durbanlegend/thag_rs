#![allow(clippy::missing_panics_doc)]
mod repeat_dash;
mod tui_keys;

use crate::repeat_dash::repeat_dash_impl;
use crate::tui_keys::key_impl;
use proc_macro::TokenStream;
use syn::parse_file;

// Not public API. This is internal and to be used only by `key!`.
#[doc(hidden)]
#[proc_macro]
pub fn key(input: TokenStream) -> TokenStream {
    key_impl(input)
}

#[proc_macro]
pub fn repeat_dash(input: TokenStream) -> TokenStream {
    // repeat_dash_impl(input)
    intercept_and_debug(false, &input, repeat_dash_impl)
}

fn intercept_and_debug<F>(expand: bool, input: &TokenStream, proc_macro: F) -> TokenStream
where
    F: Fn(TokenStream) -> TokenStream,
{
    use inline_colorization::{style_bold, style_reset};

    // Call the provided macro function
    let output = proc_macro(input.clone());

    if expand {
        // Pretty-print the expanded tokens
        let output: proc_macro2::TokenStream = output.clone().into();
        let token_str = output.to_string();
        match parse_file(&token_str) {
            Err(e) => eprintln!("Failed to parse tokens: {e:?}"),
            Ok(syn_file) => {
                let pretty_output = prettyplease::unparse(&syn_file);
                let dash_line = "-".repeat(70);
                eprintln!("{style_reset}{dash_line}{style_reset}");
                eprintln!("{style_bold}Expanded macro:{style_reset}");
                eprint!("{pretty_output}");
                eprintln!("{style_reset}{dash_line}{style_reset}");
            }
        }
    }

    output
}
