#![allow(clippy::missing_panics_doc, unused_imports)]
mod attrib_key_map_list;
mod const_demo;
mod const_demo_grail;
mod const_gen_str_demo;
mod custom_model;
mod derive_deserialize_vec;
mod derive_key_map_list;
mod expander_demo;
mod host_port_const;
mod into_string_hash_map;
mod my_description;
mod organizing_code;
mod organizing_code_const;
mod organizing_code_tokenstream;

use crate::attrib_key_map_list::use_mappings_impl;
use crate::const_demo::const_demo_impl;
use crate::const_demo_grail::const_demo_grail_impl;
use crate::const_gen_str_demo::string_concat_impl;
use crate::custom_model::derive_custom_model_impl;
use crate::derive_deserialize_vec::derive_deserialize_vec_impl;
use crate::derive_key_map_list::derive_key_map_list_impl;
use crate::expander_demo::baz2;
use crate::host_port_const::host_port_const_impl;
use crate::into_string_hash_map::into_hash_map_impl;
use crate::my_description::my_derive;
use crate::organizing_code::organizing_code_impl;
use crate::organizing_code_const::organizing_code_const_impl;
use crate::organizing_code_tokenstream::organizing_code_tokenstream_impl;
// use macro_utils::expand_macro_with;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_file, parse_macro_input, DeriveInput, ExprArray, Ident, LitInt, Token,
};

#[proc_macro_derive(DeriveCustomModel, attributes(custom_model))]
pub fn derive_custom_model(item: TokenStream) -> TokenStream {
    derive_custom_model_impl(item)
}

#[proc_macro_derive(IntoStringHashMap)]
pub fn into_hash_map(item: TokenStream) -> TokenStream {
    into_hash_map_impl(item)
}

#[proc_macro_derive(MyDescription, attributes(my_desc))]
pub fn derive_my_description(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    my_derive(item.into()).unwrap().into()
}

// Define the custom derive macro using `deluxe`
#[proc_macro_derive(DeserializeVec, attributes(deluxe, use_mappings))]
pub fn derive_deserialize_vec(input: TokenStream) -> TokenStream {
    derive_deserialize_vec_impl(input.into()).unwrap().into()
}

#[proc_macro_derive(DeriveKeyMapList, attributes(deluxe, use_mappings))]
pub fn derive_key_map_list(item: TokenStream) -> TokenStream {
    derive_key_map_list_impl(item.into()).unwrap().into()
}

// From https://github.com/tdimitrov/rust-proc-macro-post
#[proc_macro]
pub fn organizing_code(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    organizing_code_impl(input).into()
}

// From https://github.com/tdimitrov/rust-proc-macro-post
#[proc_macro]
pub fn organizing_code_tokenstream(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    organizing_code_tokenstream_impl(input).into()
}

#[proc_macro_derive(DeriveConst, attributes(adjust, use_mappings))]
pub fn organizing_code_const(input: TokenStream) -> TokenStream {
    organizing_code_const_impl(input.into()).unwrap().into()
}

#[proc_macro_attribute]
pub fn baz(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // wrap as per usual for `proc-macro2::TokenStream`, here dropping `attr` for simplicity
    baz2(input.into()).into()
}

#[proc_macro_attribute]
pub fn use_mappings(attr: TokenStream, item: TokenStream) -> TokenStream {
    use_mappings_impl(attr, item)
}

#[proc_macro]
pub fn repeat_dash(input: TokenStream) -> TokenStream {
    // Parse the input as a literal integer
    let input = parse_macro_input!(input as LitInt);
    let len = input
        .base10_parse::<usize>()
        .expect("Expected a usize integer");

    // Generate the repeated dash string
    let dash_line = "-".repeat(len);

    // expanded a constant string definition
    TokenStream::from(quote! {
        const DASH_LINE: &str = #dash_line;
    })
}

#[proc_macro]
pub fn string_concat(tokens: TokenStream) -> TokenStream {
    string_concat_impl(tokens)
}

#[proc_macro]
pub fn const_demo(tokens: TokenStream) -> TokenStream {
    const_demo_impl(tokens)
}

#[proc_macro]
pub fn const_demo_expand(tokens: TokenStream) -> TokenStream {
    let output = const_demo_impl(tokens.clone());
    let token_str = output.to_string();

    // Parse and prettify
    let _pretty_output = match syn::parse_file(&token_str) {
        Err(e) => {
            eprintln!("failed to prettify token_str: {e:?}");
            token_str
        }
        Ok(syn_file) => {
            let token_str = prettyplease::unparse(&syn_file);
            eprintln!("Expanded macro:\n{}", token_str);
            token_str
        }
    };
    output
}

#[proc_macro]
pub fn const_demo_debug(tokens: TokenStream) -> TokenStream {
    expand_macro_with(
        tokens.clone().into(),
        |_arg0: TokenStream| const_demo(tokens.clone()).into(),
        true,
    )
    .into()
}

fn expand_macro_with<F>(tokens: TokenStream, proc_macro: F, expand: bool) -> TokenStream
where
    F: Fn(TokenStream) -> TokenStream,
{
    let output = proc_macro(tokens.clone());
    if expand {
        let token_str = output.clone().to_string();
        match syn::parse_file(&token_str) {
            Err(e) => eprintln!("failed to prettify token_str: {e:?}"),
            Ok(syn_file) => {
                let pretty_output = prettyplease::unparse(&syn_file);
                eprintln!("Expanded macro:\n{}", pretty_output);
            }
        }
    }
    output.into()
}

#[proc_macro]
pub fn const_demo_grail(tokens: TokenStream) -> TokenStream {
    const_demo_grail_impl(tokens)
}

#[proc_macro_derive(HostPortConst, attributes(const_value))]
pub fn host_port_const(tokens: TokenStream) -> TokenStream {
    host_port_const_impl(tokens.into()).into()
}

fn intercept_and_debug<F>(expand: bool, input: TokenStream, proc_macro: F) -> TokenStream
where
    F: Fn(TokenStream) -> TokenStream,
{
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
                eprintln!("Expanded macro:\n{pretty_output}");
            }
        }
    }

    output
}

#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    intercept_and_debug(cfg!(feature = "expand"), input, |_tokens| {
        // let tokens: proc_macro2::TokenStream = tokens.clone().into();

        // Original macro logic
        let expanded = quote! {
            pub const VALUE: usize = 42;
        };
        TokenStream::from(expanded)
    })
}

// use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyDerive)]
pub fn my_derive_macro(input: TokenStream) -> TokenStream {
    intercept_and_debug(cfg!(feature = "expand"), input, |tokens| {
        let input = parse_macro_input!(tokens as DeriveInput);
        let struct_name = input.ident;

        // Macro expansion logic
        let expanded = quote! {
            impl #struct_name {
                pub const CONST_VALUE: usize = 42;
            }
        };
        TokenStream::from(expanded)
    })
}

#[proc_macro_attribute]
pub fn my_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    intercept_and_debug(cfg!(feature = "expand"), item, |tokens| {
        let tokens: proc_macro2::TokenStream = tokens.clone().into();

        // Attach logic to the input
        let expanded = quote! {
            #[allow(unused_variables)]
            #tokens
        };
        TokenStream::from(expanded)
    })
}
