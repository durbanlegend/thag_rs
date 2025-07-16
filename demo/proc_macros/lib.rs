#![allow(clippy::missing_panics_doc, dead_code, unused_imports)]
//! # Procedural Macros Demo Collection
//!
//! A collection of 12 procedural macros demonstrating proc macro development techniques in Rust.
//! Each macro teaches specific concepts while solving practical problems.
//!
//! ## Collection Overview
//!
//! ### Derive Macros (5)
//! - [`DeriveConstructor`] - Basic constructor generation
//! - [`DeriveGetters`] - Getter method generation
//! - [`DeriveBuilder`] - Builder pattern implementation
//! - [`DeriveDisplay`] - Display trait implementation
//! - [`DeriveDocComment`] - Documentation extraction
//!
//! ### Attribute Macros (3)
//! - [`macro@cached`] - Function memoization (use `expand` flag to see generated code)
//! - [`macro@timing`] - Execution time measurement (use `expand` flag to see generated code)
//! - [`macro@retry`] - Automatic retry logic (use `expand` flag to see generated code)
//!
//! ### Function-like Macros (4)
//! - [`macro@file_navigator`] - File system navigation
//! - [`macro@compile_time_assert`] - Compile-time validation
//! - [`macro@env_or_default`] - Environment variable access
//! - [`macro@generate_tests`] - Test case generation
//!
//! ## Usage
//!
//! Add this crate to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! thag_demo_proc_macros = { path = "absolute/path/to/demo/proc_macros" }
//! ```
//!
//! Or with `thag_rs`:
//!
//! ```rust
//! use thag_demo_proc_macros::{DeriveBuilder, cached, timing};
//!
//! // Basic usage
//! #[cached]
//! fn fibonacci(n: u32) -> u32 { ... }
//!
//! // With expand flag to see generated code during compilation
//! #[timing(expand)]
//! fn slow_operation() -> i32 { ... }
//! ```
//!
//! ## Learning Path
//!
//! 1. **DeriveConstructor** - Basic derive macro concepts
//! 2. **DeriveGetters** - Method generation patterns
//! 3. **DeriveBuilder** - Complex struct generation
//! 4. **DeriveDisplay** - Trait implementation
//! 5. **DeriveDocComment** - Attribute parsing
//! 6. **cached** - Function transformation (try with `expand` flag)
//! 7. **timing** - Performance measurement (try with `expand` flag)
//! 8. **retry** - Error handling patterns (try with `expand` flag)
//! 9. **file_navigator** - Complex code generation
//! 10. **compile_time_assert** - Compile-time validation
//! 11. **env_or_default** - Environment processing
//! 12. **generate_tests** - Test automation
//!
//! ## Debugging Generated Code
//!
//! Attribute macros (`cached`, `timing`, `retry`) support an `expand` flag to display
//! the generated code during compilation. This is useful for learning and debugging:
//!
//! ```rust
//! #[cached(expand)]        // Shows caching implementation
//! #[timing(expand)]        // Shows timing measurement code
//! #[retry(times = 3, expand)]  // Shows retry logic with custom retry count
//! ```

mod cached;
mod compile_time_assert;
mod derive_builder;
mod derive_constructor;
mod derive_display;
mod derive_doc_comment;
mod derive_getters;
mod env_or_default;
mod file_navigator;
mod generate_tests;
mod retry;
mod timing;

use crate::cached::cached_impl;
use crate::compile_time_assert::compile_time_assert_impl;
use crate::derive_builder::derive_builder_impl;
use crate::derive_constructor::derive_constructor_impl;
use crate::derive_display::derive_display_impl;
use crate::derive_doc_comment::derive_doc_comment_impl;
use crate::derive_getters::derive_getters_impl;
use crate::env_or_default::env_or_default_impl;
use crate::file_navigator::file_navigator_impl;
use crate::generate_tests::generate_tests_impl;
use crate::retry::retry_impl;
use crate::timing::timing_impl;
use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use std::path::Path;
use syn::{
    parse::{Parse, ParseStream},
    parse_file, parse_macro_input, parse_str, DeriveInput, Expr, ExprArray, Ident, LitInt, LitStr,
    Token,
};

/// Simple argument parser for attribute macros to check for expand flag
#[derive(Default)]
struct AttrArgs {
    expand: bool,
}

impl Parse for AttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();

        // Handle empty case
        if input.is_empty() {
            return Ok(args);
        }

        // Parse as a list of flags
        let mut first = true;

        while !input.is_empty() {
            if !first {
                let _: syn::Token![,] = input.parse()?;
            }
            first = false;

            // Parse as flag
            let flag: syn::Ident = input.parse()?;
            match flag.to_string().as_str() {
                "expand" => args.expand = true,
                _ => {
                    // Ignore unknown flags for now to maintain compatibility
                }
            }
        }

        Ok(args)
    }
}

/// Argument parser for retry macro to handle both times parameter and expand flag
#[derive(Default)]
struct RetryArgs {
    times: Option<u32>,
    expand: bool,
}

impl Parse for RetryArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();

        // Handle empty case
        if input.is_empty() {
            return Ok(args);
        }

        // Parse as a list of arguments
        let mut first = true;

        while !input.is_empty() {
            if !first {
                let _: syn::Token![,] = input.parse()?;
            }
            first = false;

            // Try to parse as "key = value" or just "key"
            let key: syn::Ident = input.parse()?;

            if input.peek(syn::Token![=]) {
                let _: syn::Token![=] = input.parse()?;
                let value: syn::Expr = input.parse()?;

                match key.to_string().as_str() {
                    "times" => {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Int(lit_int),
                            ..
                        }) = value
                        {
                            args.times = Some(lit_int.base10_parse().unwrap_or(3));
                        }
                    }
                    _ => {
                        // Ignore unknown key=value pairs for now
                    }
                }
            } else {
                // This is just a flag
                match key.to_string().as_str() {
                    "expand" => args.expand = true,
                    _ => {
                        // Ignore unknown flags for now to maintain compatibility
                    }
                }
            }
        }

        Ok(args)
    }
}

/// Generates constructor methods for structs.
///
/// Creates a `new` method that takes parameters for all fields and returns a new instance.
/// Demonstrates basic derive macro concepts including field iteration and code generation.
///
/// ## Example
/// ```rust
/// use thag_demo_proc_macros::DeriveConstructor;
/// #[derive(DeriveConstructor)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
/// // Generates: impl Person { pub fn new(name: String, age: u32) -> Person { ... } }
/// ```
#[proc_macro_derive(DeriveConstructor, attributes(expand_macro))]
pub fn derive_constructor(input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let check_input = parse_macro_input!(input as DeriveInput);

    #[cfg(feature = "expand")]
    let should_expand = check_input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("expand_macro"));
    #[cfg(not(feature = "expand"))]
    let should_expand = false;

    maybe_expand_proc_macro(
        should_expand,
        "derive_constructor",
        &input_clone,
        derive_constructor_impl,
    )
}

/// Generates builder pattern implementation for structs.
///
/// Creates a separate builder struct with fluent API for step-by-step construction.
/// Demonstrates advanced derive macro concepts including struct generation and method chaining.
///
/// ## Example
/// ```rust
/// use thag_demo_proc_macros::DeriveBuilder;
/// #[derive(DeriveBuilder)]
/// struct Config {
///     host: String,
///     port: u16,
/// }
/// // Generates: ConfigBuilder with fluent API
/// // let config = Config::builder().host("localhost").port(8080).build()?;
/// ```
#[proc_macro_derive(DeriveBuilder, attributes(expand_macro))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let check_input = parse_macro_input!(input as DeriveInput);

    #[cfg(feature = "expand")]
    let should_expand = check_input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("expand_macro"));
    #[cfg(not(feature = "expand"))]
    let should_expand = false;

    maybe_expand_proc_macro(
        should_expand,
        "derive_builder",
        &input_clone,
        derive_builder_impl,
    )
}

/// Generates Display trait implementations for structs and enums.
///
/// Creates readable string representations with proper formatting for different struct types.
/// Demonstrates trait implementation generation and pattern matching for enums.
///
/// ## Example
/// ```rust
/// use thag_demo_proc_macros::DeriveDisplay;
/// #[derive(DeriveDisplay)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
/// // Generates: impl Display for Person { ... }
/// // Output: "Person { name: Alice, age: 30 }"
/// ```
#[proc_macro_derive(DeriveDisplay, attributes(expand_macro))]
pub fn derive_display(input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let check_input = parse_macro_input!(input as DeriveInput);

    #[cfg(feature = "expand")]
    let should_expand = check_input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("expand_macro"));
    #[cfg(not(feature = "expand"))]
    let should_expand = false;

    maybe_expand_proc_macro(
        should_expand,
        "derive_display",
        &input_clone,
        derive_display_impl,
    )
}

/// Generates getter methods for all struct fields.
///
/// Creates getter methods that return references to fields, avoiding unnecessary moves.
/// Demonstrates method generation patterns and type analysis.
///
/// ## Example
/// ```rust
/// use thag_demo_proc_macros::DeriveGetters;
/// #[derive(DeriveGetters)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
/// // Generates: impl Person { pub fn name(&self) -> &String { &self.name } ... }
/// ```
#[proc_macro_derive(DeriveGetters, attributes(expand_macro))]
pub fn derive_getters(input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let check_input = parse_macro_input!(input as DeriveInput);

    #[cfg(feature = "expand")]
    let should_expand = check_input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("expand_macro"));
    #[cfg(not(feature = "expand"))]
    let should_expand = false;

    maybe_expand_proc_macro(
        should_expand,
        "derive_getters",
        &input_clone,
        derive_getters_impl,
    )
}

/// Extracts compile-time documentation and makes it available at runtime.
///
/// Generates methods to access documentation strings from enum variants and struct fields.
/// Demonstrates advanced attribute parsing across multiple item types.
///
/// ## Example
/// ```rust
/// use thag_demo_proc_macros::DeriveDocComment;
/// #[derive(DeriveDocComment)]
/// enum Status {
///     /// Operation completed successfully
///     Success,
///     /// An error occurred
///     Error,
/// }
/// // Generates: impl Status { fn doc_comment(&self) -> &'static str { ... } }
/// ```
#[proc_macro_derive(DeriveDocComment)]
pub fn derive_doc_comment(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "derive_doc_comment", &input, derive_doc_comment_impl)
}

/// Generates interactive file system navigation functionality.
///
/// Creates structures and functions for file selection and directory navigation.
/// Demonstrates complex code generation and external crate integration.
///
/// ## Example
/// ```ignore
/// use thag_demo_proc_macros::file_navigator;
/// file_navigator! {}
/// // Generates: FileNavigator struct, select_file function, save_to_file function, etc.
/// ```
#[proc_macro]
pub fn file_navigator(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "file_navigator", &input, file_navigator_impl)
}

/// Adds automatic memoization to functions.
///
/// Wraps functions with caching logic using HashMap and Mutex for thread safety.
/// Demonstrates function transformation and caching patterns.
///
/// ## Example
/// ```rust
/// use thag_demo_proc_macros::cached;
/// #[cached]
/// fn fibonacci(n: u32) -> u32 {
///     if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) }
/// }
///
/// // To see the generated code during development:
/// #[cached(expand)]
/// fn fibonacci_debug(n: u32) -> u32 {
///     if n <= 1 { n } else { fibonacci_debug(n-1) + fibonacci_debug(n-2) }
/// }
/// ```
#[proc_macro_attribute]
pub fn cached(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse::<AttrArgs>(attr.clone()).unwrap_or_default();
    maybe_expand_attr_macro(args.expand, "cached", &attr, &item, cached_impl)
}

/// Measures and displays function execution time.
///
/// Wraps functions to measure execution time and output results to console.
/// Demonstrates function signature preservation and performance measurement.
///
/// ## Example
/// ```rust
/// use thag_demo_proc_macros::timing;
/// #[timing]
/// fn slow_operation() -> i32 {
///     std::thread::sleep(std::time::Duration::from_millis(100));
///     42
/// }
/// // Output: Function 'slow_operation' took: 100.234ms
///
/// // To see the generated code during development:
/// #[timing(expand)]
/// fn debug_operation() -> i32 {
///     std::thread::sleep(std::time::Duration::from_millis(100));
///     42
/// }
/// ```
#[proc_macro_attribute]
pub fn timing(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse::<AttrArgs>(attr.clone()).unwrap_or_default();
    maybe_expand_attr_macro(args.expand, "timing", &attr, &item, timing_impl)
}

/// Adds automatic retry logic to functions.
///
/// Wraps functions with configurable retry attempts and backoff delays.
/// Demonstrates attribute parameter parsing and error handling patterns.
///
/// ## Example
/// ```ignore
/// use thag_demo_proc_macros::retry;
/// #[retry(times = 5)]
/// fn unreliable_operation() -> Result<String, std::io::Error> {
///     // Network operation that might fail
///     Ok("success".to_string())
/// }
///
/// // To see the generated code during development:
/// #[retry(times = 3, expand)]
/// fn debug_operation() -> Result<String, std::io::Error> {
///     // Network operation that might fail
///     Ok("success".to_string())
/// }
/// ```
#[proc_macro_attribute]
pub fn retry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse::<RetryArgs>(attr.clone()).unwrap_or_default();
    maybe_expand_attr_macro(args.expand, "retry", &attr, &item, retry_impl)
}

/// Generates compile-time assertions.
///
/// Creates compile-time checks that prevent compilation if conditions are not met.
/// Demonstrates compile-time validation and zero-runtime-cost assertions.
///
/// ## Example
/// ```rust
/// use thag_demo_proc_macros::compile_time_assert;
/// compile_time_assert!(std::mem::size_of::<usize>() == 8, "Requires 64-bit platform");
/// compile_time_assert!(1 + 1 == 2, "Basic math must work");
/// ```
#[proc_macro]
pub fn compile_time_assert(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(
        false,
        "compile_time_assert",
        &input,
        compile_time_assert_impl,
    )
}

/// Resolves environment variables at compile time with fallback defaults.
///
/// Reads environment variables during compilation and generates string literals.
/// Demonstrates compile-time environment processing and configuration management.
///
/// ## Example
/// ```rust
/// const DATABASE_URL: &str = env_or_default!("DATABASE_URL", "localhost:5432");
/// const DEBUG_MODE: &str = env_or_default!("DEBUG", "false");
/// ```
#[proc_macro]
pub fn env_or_default(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "env_or_default", &input, env_or_default_impl)
}

/// Generates multiple test functions from test data.
///
/// Creates test functions from data arrays to reduce boilerplate in test suites.
/// Demonstrates test automation and repetitive code generation patterns.
///
/// ## Example
/// ```rust
/// generate_tests! {
///     test_addition: [
///         (1, 2, 3),
///         (5, 7, 12),
///         (0, 0, 0),
///     ] => |a, b, expected| assert_eq!(a + b, expected)
/// }
/// ```
#[proc_macro]
pub fn generate_tests(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "generate_tests", &input, generate_tests_impl)
}

/// Conditionally expands proc macros for debugging.
///
/// Utility function for displaying generated code during development.
fn maybe_expand_proc_macro<F>(
    expand: bool,
    name: &str,
    input: &TokenStream,
    proc_macro: F,
) -> TokenStream
where
    F: Fn(TokenStream) -> TokenStream,
{
    let output = proc_macro(input.clone());

    if expand {
        expand_output(name, &output);
    }

    output
}

/// Conditionally expands attribute macros for debugging.
///
/// Utility function for displaying generated code from attribute macros.
fn maybe_expand_attr_macro<F>(
    expand: bool,
    name: &str,
    attr: &TokenStream,
    item: &TokenStream,
    attr_macro: F,
) -> TokenStream
where
    F: Fn(TokenStream, TokenStream) -> TokenStream,
{
    let output = attr_macro(attr.clone(), item.clone());

    if expand {
        expand_output(name, &output);
    }

    output
}

fn expand_output(name: &str, output: &TokenStream) {
    use inline_colorization::{color_cyan, color_reset, style_bold, style_reset, style_underline};
    let output: proc_macro2::TokenStream = output.clone().into();
    let token_str = output.to_string();
    let dash_line = "─".repeat(70);

    match parse_file(&token_str) {
        Ok(syn_file) => {
            let pretty_output = prettyplease::unparse(&syn_file);
            eprintln!("{style_reset}{dash_line}{style_reset}");
            eprintln!(
                "{style_bold}{style_underline}Expanded macro{style_reset} {style_bold}{color_cyan}{name}{color_reset}:{style_reset}\n"
            );
            eprint!("{pretty_output}");
            eprintln!("{style_reset}{dash_line}{style_reset}");
        }
        Err(_) => match parse_str::<Expr>(&token_str) {
            Ok(expr) => {
                eprintln!("{style_reset}{dash_line}{style_reset}");
                eprintln!(
                    "{style_bold}{style_underline}Expanded macro{style_reset} {style_bold}{color_cyan}{name}{color_reset} (as expression):{style_reset}\n"
                );
                eprintln!("{}", quote!(#expr));
                eprintln!("{style_reset}{dash_line}{style_reset}");
            }
            Err(_) => {
                eprintln!("{style_reset}{dash_line}{style_reset}");
                eprintln!(
                    "{style_bold}{style_underline}Expanded macro{style_reset} {style_bold}{color_cyan}{name}{color_reset} (as token string):{style_reset}\n"
                );
                eprintln!("{token_str}");
                eprintln!("{style_reset}{dash_line}{style_reset}");
            }
        },
    }
}
