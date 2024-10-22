mod custom_model;
mod deserialize_vec_derive;
mod into_string_hash_map;
mod key_map_list_derive;
mod my_description;
mod tui_keys;

use crate::custom_model::derive_custom_model_impl;
use crate::deserialize_vec_derive::deserialize_vec_derive_impl;
use crate::into_string_hash_map::into_hash_map_impl;
use crate::key_map_list_derive::key_map_list_derive_impl;
use crate::my_description::my_derive;
use crate::tui_keys::key_impl;
use proc_macro::TokenStream;

// Not public API. This is internal and to be used only by `key!`.
#[doc(hidden)]
#[proc_macro]
pub fn key(input: TokenStream) -> TokenStream {
    key_impl(input)
}

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
