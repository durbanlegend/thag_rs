//! Internal documentation attribute macro.
//!
//! This module provides a proc macro attribute that simplifies the process of marking
//! items as internal documentation that should be hidden from public API docs.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};

/// Proc macro attribute for marking items as internal documentation.
///
/// This is a convenience macro that applies the `#[cfg_attr(not(feature = "internal_docs"), doc(hidden))]`
/// attribute to items, making them hidden from public API documentation but visible in internal docs.
///
/// # Examples
///
/// ```rust
/// use thag_proc_macros::internal_doc;
///
/// #[internal_doc]
/// pub fn internal_utility_function() {
///     // This function will be hidden from public API docs
///     // but visible when the `internal_docs` feature is enabled
/// }
/// ```
///
/// This is equivalent to:
///
/// ```rust
/// #[cfg_attr(not(feature = "internal_docs"), doc(hidden))]
/// pub fn internal_utility_function() {
///     // Implementation...
/// }
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn internal_doc_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input as a generic item
    let item = parse_macro_input!(input as Item);

    // We don't use args for this macro, but we need to accept it for the attribute signature
    let _ = args;

    // Generate the expanded code with the cfg_attr attribute
    let expanded = quote! {
        #[cfg_attr(not(feature = "internal_docs"), doc(hidden))]
        #item
    };

    TokenStream::from(expanded)
}
