#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Parser},
    parse_macro_input,
    spanned::Spanned,
    Block, Expr, Result, Token,
};

struct ProfileBlockInput {
    name_expr: Expr,
    detailed: bool,
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
        let detailed = match mode_str.as_str() {
            "detailed_memory" => true,
            "memory" => false,
            _ => {
                return Err(syn::Error::new(
                    mode_ident.span(),
                    "Expected 'detailed_memory' or 'memory'",
                ))
            }
        };

        input.parse::<Token![,]>()?;

        // Parse the block as raw tokens
        let content;
        braced!(content in input);
        let block_tokens = content.parse::<proc_macro2::TokenStream>()?;

        Ok(ProfileBlockInput {
            name_expr,
            detailed,
            block_tokens,
        })
    }
}

pub fn profile_block_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ProfileBlockInput);

    let name_expr = &input.name_expr;
    let detailed = input.detailed;
    let block_tokens = &input.block_tokens;

    // // Extract string literal if the expression is a string literal
    // let name_str = if let Expr::Lit(expr_lit) = name_expr {
    //     if let syn::Lit::Str(lit_str) = &expr_lit.lit {
    //         lit_str.value()
    //     } else {
    //         return syn::Error::new(
    //             expr_lit.lit.span(),
    //             "Expected string literal for section name",
    //         )
    //         .to_compile_error()
    //         .into();
    //     }
    // } else {
    //     return syn::Error::new(name_expr.span(), "Expected string literal for section name")
    //         .to_compile_error()
    //         .into();
    // };

    // let end_fn_name = format!("end_{}", name_str);
    // let end_fn_ident = syn::Ident::new(&end_fn_name, name_expr.span());

    // Generate the transformed code
    let expanded = quote! {
       {
            // Use paste to call the end function
            let end_line = ::thag_profiler::paste::paste! {
                [<end_ #name_expr>]()
            };

            let _profile_guard = {
                let start_line = line!();

                ::thag_profiler::ProfileSection::new_with_detailed_memory(
                    Some(#name_expr),
                    start_line,
                    end_line,
                    #detailed,
                    module_path!().to_string(),
                )
            };

            // Original block statements as raw tokens
            #block_tokens
            ::thag_profiler::end!(#name_expr);
        }
    };

    expanded.into()
}
