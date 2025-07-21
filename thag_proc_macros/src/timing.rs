#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub fn timing_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Get function details
    let fn_name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let generics = &input.sig.generics;
    let where_clause = &input.sig.generics.where_clause;
    let vis = &input.vis;
    let block = &input.block;
    let attrs = &input.attrs;

    let fn_name_str = fn_name.to_string();

    let result = quote! {
        #(#attrs)*
        #vis fn #fn_name #generics(#inputs) #output #where_clause {
            let _start_time = std::time::Instant::now();

            // Execute the original function
            let result = {
                #block
            };

            let _elapsed = _start_time.elapsed();
            println!("⏱️  Function '{}' took: {:.0?}", #fn_name_str, _elapsed);

            result
        }
    };

    result.into()
}
