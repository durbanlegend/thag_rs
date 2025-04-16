#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

pub fn end_impl(input: TokenStream) -> TokenStream {
    use quote::format_ident;
    // Parse the input as a string literal
    let func_name_lit = parse_macro_input!(input as LitStr);
    let func_name_str = func_name_lit.value();

    // Convert the string to an identifier
    let func_name = format_ident!("end_{}", func_name_str);

    // Generate the function that returns line!()
    let expanded = quote! {
        fn #func_name() -> u32 {
            line!()
        }
    };

    // Return the generated code
    expanded.into()
}
