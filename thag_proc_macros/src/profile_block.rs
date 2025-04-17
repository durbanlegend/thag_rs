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

// pub fn profile_block_impl(input: TokenStream) -> TokenStream {
pub fn profile_block_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ProfileBlockInput);

    let name_expr = &input.name_expr;
    let block_tokens = &input.block_tokens;

    // Create the function name identifier
    let name_str = if let Expr::Lit(lit_expr) = name_expr {
        if let syn::Lit::Str(lit_str) = &lit_expr.lit {
            lit_str.value()
        } else {
            return syn::Error::new(name_expr.span(), "Expected string literal for section name")
                .to_compile_error()
                .into();
        }
    } else {
        return syn::Error::new(name_expr.span(), "Expected string literal for section name")
            .to_compile_error()
            .into();
    };
    let func_name = format!("end_{}", name_str);
    let func_ident = syn::Ident::new(&func_name, name_expr.span());

    // Use Span::call_site() to get the call site location
    let span = proc_macro2::Span::mixed_site();

    let my_func = quote_spanned! {span=>

        // Original block statements
        #block_tokens

        // Create a function at the call site that returns the end line
        #[allow(non_snake_case)]
        #[inline(never)]
        fn #func_ident() -> u32 {
            line!()
        }
    };

    // Generate the transformed code with explicit spans
    let expanded = quote_spanned! {span=>
        {
            // This should be the call site line
            let start_line = line!();
            let end_line = #func_ident();

            // Get the end line by calling our function after the block
            let _profile_guard = ::thag_profiler::ProfileSection::new_with_detailed_memory(
                Some(#name_expr),
                start_line,
                end_line,
                true,
                module_path!().to_string(),
            );

            #my_func
        }
    };

    expanded.into()
}
