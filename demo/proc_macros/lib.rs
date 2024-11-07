#![allow(clippy::missing_panics_doc)]
mod custom_model;
mod deserialize_vec_derive;
mod expander_demo;
mod into_string_hash_map;
mod key_map_list_attrib;
mod key_map_list_derive;
mod my_description;
mod my_proc;

use crate::custom_model::derive_custom_model_impl;
use crate::deserialize_vec_derive::deserialize_vec_derive_impl;
use crate::expander_demo::baz2;
use crate::into_string_hash_map::into_hash_map_impl;
use crate::key_map_list_attrib::use_mappings_impl;
use crate::key_map_list_derive::key_map_list_derive_impl;
use crate::my_description::my_derive;
use proc_macro::TokenStream;
use syn::parse_macro_input;

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
pub fn deserialize_vec_derive(input: TokenStream) -> TokenStream {
    deserialize_vec_derive_impl(input.into()).unwrap().into()
}

#[proc_macro_derive(DeriveKeyMapList, attributes(deluxe, use_mappings))]
pub fn key_map_list_derive(item: TokenStream) -> TokenStream {
    key_map_list_derive_impl(item.into()).unwrap().into()
}

// From https://github.com/tdimitrov/rust-proc-macro-post
#[proc_macro]
pub fn my_proc_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    my_proc::my_proc_impl(input).into()
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
    use_mappings_impl(attr, item) /* .unwrap() */
}
