#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

pub fn end_impl(input: TokenStream) -> TokenStream {
    use quote::format_ident;
    // Parse the input as a string literal
    let profile_id_lit = parse_macro_input!(input as LitStr);
    let profile_id_str = profile_id_lit.value();
    let profile_id = format_ident!("{profile_id_str}");

    // Convert the string to an identifier
    let func_name = format_ident!("end_{}", profile_id_str);

    // Generate the function that returns line!()
    let expanded = quote! {
        fn #func_name() -> u32 { line!() }

        ::thag_profiler::with_allocator(::thag_profiler::Allocator::System, || {
        if let Some(profile) = #profile_id {
            drop(profile);
        }
        });

    };

    // Return the generated code
    expanded.into()
}
