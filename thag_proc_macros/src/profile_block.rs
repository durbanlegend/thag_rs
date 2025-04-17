#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Expr, Result, Token,
};

struct ProfileBlockInput {
    name_expr: Expr,
    mode: syn::Ident,
    block_tokens: proc_macro2::TokenStream,
}

impl Parse for ProfileBlockInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse "name = expr"
        let name_ident: syn::Ident = input.parse()?;
        if name_ident.to_string() != "name" {
            return Err(syn::Error::new(name_ident.span(), "Expected 'name'"));
        }

        input.parse::<Token![=]>()?;
        let name_expr: Expr = input.parse()?;
        input.parse::<Token![,]>()?;

        // Parse profiling mode
        let mode_ident: syn::Ident = input.parse()?;
        let mode_str = mode_ident.to_string();
        // let detailed = match mode_str.as_str() {
        //     "detailed_memory" => true,
        //     "memory" => false,
        //     _ => {
        //         return Err(syn::Error::new(
        //             mode_ident.span(),
        //             "Expected 'detailed_memory' or 'memory'",
        //         ))
        //     }
        // };

        input.parse::<Token![,]>()?;

        // Parse the block as raw tokens
        let content;
        braced!(content in input);
        let block_tokens = content.parse::<proc_macro2::TokenStream>()?;

        Ok(ProfileBlockInput {
            name_expr,
            mode: mode_ident,
            block_tokens,
        })
    }
}

pub fn profile_block_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ProfileBlockInput);

    let name_expr = &input.name_expr;
    let block_tokens = &input.block_tokens;

    // Use Span::call_site() to get the call site location
    let span = proc_macro2::Span::call_site();

    // Generate the transformed code with explicit spans
    let expanded = quote_spanned! {span=>
        {
            // This should be the call site line
            let start_line = line!();

            // Get the end line by calling our function after the block
            let _profile_guard = ::thag_profiler::ProfileSection::new_with_detailed_memory(
                Some(#name_expr),
                start_line,
                end_line_func(),
                true,
                module_path!().to_string(),
            );

            // Original block statements
            #block_tokens

            // Create a function at the call site that returns the end line
            #[allow(non_snake_case)]
            #[inline(never)]
            fn end_line_func() -> u32 {
                line!()
            }
        }
    };

    expanded.into()
}
