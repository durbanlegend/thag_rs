#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use std::fs::OpenOptions;
use std::io::Write;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, Token,
};

/// Parse the input for `generate_tests`! { test_name: [data] => |params| body }
struct GenerateTestsInput {
    test_name: Ident,
    test_data: Vec<Expr>,
    params: Vec<Ident>,
    body: Expr,
}

impl Parse for GenerateTestsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let test_name = input.parse()?;
        input.parse::<Token![:]>()?;

        // Parse the array of test data
        let content;
        syn::bracketed!(content in input);
        let mut test_data = Vec::new();

        while !content.is_empty() {
            test_data.push(content.parse()?);
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        input.parse::<Token![=>]>()?;

        // Parse the closure |params| body
        input.parse::<Token![|]>()?;
        let mut params = Vec::new();

        // Parse parameters between | |
        while !input.peek(Token![|]) {
            params.push(input.parse()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        input.parse::<Token![|]>()?;
        let body = input.parse()?;

        Ok(GenerateTestsInput {
            test_name,
            test_data,
            params,
            body,
        })
    }
}

pub fn generate_tests_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as GenerateTestsInput);
    let base_test_name = &input.test_name;
    let test_data = &input.test_data;
    let params = &input.params;
    let body = &input.body;

    // Generate individual test functions
    let test_functions = test_data.iter().enumerate().map(|(i, data)| {
        let test_fn_name = quote::format_ident!("{}_{}", base_test_name, i);

        // Extract tuple elements if the data is a tuple
        let param_assignments = if let Expr::Tuple(tuple) = data {
            tuple
                .elems
                .iter()
                .zip(params)
                .map(|(value, param)| {
                    quote! { let #param = #value; }
                })
                .collect::<Vec<_>>()
        } else {
            // Single parameter case
            vec![quote! { let #(#params)* = #data; }]
        };

        quote! {
            #[test]
            fn #test_fn_name() {
                #(#param_assignments)*
                #body
            }
        }
    });

    let expanded = quote! {
        #(#test_functions)*
    };

    // // Debug: Write generated code to file
    // let generated_code = expanded.to_string();
    // let debug_output = format!(
    //     "// Generated tests for {}\n{}\n\n",
    //     base_test_name, generated_code
    // );

    // if let Ok(mut file) = OpenOptions::new()
    //     .create(true)
    //     .append(true)
    //     .open("generated_tests_debug.rs")
    // {
    //     let _ = writeln!(file, "{}", debug_output);
    // }

    // Also print to stderr so it shows during compilation
    let generated_code = expanded.to_string();
    eprintln!(
        "Generated {} test functions for {}",
        test_data.len(),
        base_test_name
    );
    eprintln!("Code:\n{generated_code}");

    TokenStream::from(expanded)
}
