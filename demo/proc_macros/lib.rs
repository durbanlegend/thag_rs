#![allow(clippy::missing_panics_doc, unused_imports)]
mod attrib_key_map_list;
mod const_demo;
mod const_gen_str_demo;
mod custom_model;
mod derive_deserialize_vec;
mod derive_key_map_list;
mod expander_demo;
mod into_string_hash_map;
mod my_description;
mod organizing_code;
mod organizing_code_const;
mod organizing_code_tokenstream;

use crate::attrib_key_map_list::use_mappings_impl;
use crate::const_demo::const_demo_impl;
use crate::const_gen_str_demo::string_concat_impl;
use crate::custom_model::derive_custom_model_impl;
use crate::derive_deserialize_vec::derive_deserialize_vec_impl;
use crate::derive_key_map_list::derive_key_map_list_impl;
use crate::expander_demo::baz2;
use crate::into_string_hash_map::into_hash_map_impl;
use crate::my_description::my_derive;
use crate::organizing_code::organizing_code_impl;
use crate::organizing_code_const::organizing_code_const_impl;
use crate::organizing_code_tokenstream::organizing_code_tokenstream_impl;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, ExprArray, Ident, LitInt, Token,
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
    use expander::{Edition, Expander};
    let output = const_demo_impl(tokens.clone());
    let _expanded = Expander::new("const_demo")
        .add_comment("This is generated code!".to_owned())
        .fmt(Edition::_2021)
        .verbose(true)
        .dry(false)
        .write_to_out_dir(output.clone().into())
        .unwrap_or_else(|e| {
            eprintln!("Failed to write to file: {:?}", e);
            output.into()
        });
    // Run it again to avoid "error: expected expression, found keyword `const`"
    const_demo_impl(tokens)
}

// #[proc_macro]
// pub fn vec_concat(tokens: TokenStream) -> TokenStream {
//     use const_gen_proc_macro::Object;
//     use const_gen_proc_macro::ObjectType;
//     use const_gen_proc_macro::Path;
//     use const_gen_proc_macro::ProcMacroEnv;

//     let mut vec_type = ObjectType::new();
//     vec_type.add_method(
//         "concat",
//         &(&|first: &mut Vec<_>, second: Vec<_>| -> Vec<_> {
//             first.to_owned().extend_from_slice(&second);
//             first.to_vec()
//         } as &dyn Fn(&Vec<_>, Vec<_>) -> Vec<_>),
//     );
//     // sealing the ObjectType means it is no longer mutable and can now instantiate objects
//     let vec_type = vec_type.seal();

//     let mut vec_path = Path::new();
//     let vec_new =
//         &|first: Vec<_>| -> Object { vec_type.new_instance(first) } as &dyn Fn(Vec<_>) -> Object;
//     vec_path.add_function("new", &vec_new);

//     let mut env = ProcMacroEnv::new();
//     env.add_path("vec", vec_path);
//     env.process(tokens)
// }
