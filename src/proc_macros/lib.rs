mod custom_model;
mod into_string_hash_map;
mod tui_keys;

use crate::custom_model::derive_custom_model_impl;
use crate::into_string_hash_map::into_hash_map_impl;
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
