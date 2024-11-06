#![allow(clippy::missing_panics_doc)]
mod tui_keys;

use crate::tui_keys::key_impl;
use proc_macro::TokenStream;

// Not public API. This is internal and to be used only by `key!`.
#[doc(hidden)]
#[proc_macro]
pub fn key(input: TokenStream) -> TokenStream {
    key_impl(input)
}
