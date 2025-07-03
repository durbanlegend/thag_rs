#![allow(clippy::missing_panics_doc, dead_code, unused_imports)]
//! # Procedural Macros Demo Collection
//!
//! This crate provides a curated collection of high-quality procedural macros demonstrating
//! various techniques and patterns for writing proc macros in Rust. It serves as
//! educational material for developers learning to create their own procedural macros.
//!
//! ## Overview
//!
//! The collection focuses on quality over quantity, featuring 12 carefully selected macros
//! that demonstrate progressive complexity and real-world utility:
//!
//! ### Derive Macros (5)
//!
//! 1. **[`DeriveConstructor`]** - Basic derive macro for generating constructor methods
//! 2. **[`DeriveGetters`]** - Intermediate derive macro for generating getter methods
//! 3. **[`DeriveBuilder`]** - Advanced derive macro implementing the builder pattern
//! 4. **[`DeriveDisplay`]** - Trait implementation macro for Display formatting
//! 5. **[`DeriveDocComment`]** - Advanced derive macro demonstrating attribute parsing
//!
//! ### Attribute Macros (3)
//!
//! 6. **[`cached`]** - Attribute macro for automatic function memoization
//! 7. **[`timing`]** - Attribute macro for execution time measurement
//! 8. **[`retry`]** - Attribute macro for automatic retry logic
//!
//! ### Function-like Macros (4)
//!
//! 9. **[`file_navigator`]** - Function-like macro for file system navigation
//! 10. **[`compile_time_assert`]** - Function-like macro for compile-time validation
//! 11. **[`env_or_default`]** - Environment variable access with default fallback
//! 12. **[`generate_tests`]** - Automatic test case generation from data
//!
//! ## Progressive Learning Path
//!
//! The macros are designed to provide a progressive learning experience:
//!
//! - **Basic**: Start with `DeriveConstructor` to understand derive macro fundamentals
//! - **Intermediate**: Progress to `DeriveGetters` for method generation patterns
//! - **Advanced**: Learn builder patterns with `DeriveBuilder` and trait implementation with `DeriveDisplay`
//! - **Expert**: Master attribute parsing with `DeriveDocComment`
//! - **Function-like**: Explore `file_navigator` and `compile_time_assert` for utility macros
//! - **Attribute Wrapping**: Learn function transformation with `cached`, `timing`, and `retry`
//!
//! ## Usage
//!
//! The `demo/proc_macros` library is packaged for use in development. You may copy the macros for reuse as
//! you see fit, but to use them in place, you will need the `demo/proc_macros` library in your path.
//! Add this crate to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! thag_demo_proc_macros = { path = "demo/proc_macros" }
//! ```
//!
//! Or when using with `thag_rs`:
//!
//! ```rust
//! use thag_demo_proc_macros::{DeriveBuilder, DeriveConstructor, DeriveDisplay, DeriveDocComment, DeriveGetters, cached, compile_time_assert, file_navigator, retry, timing};
//! ```
//!
//! ## Examples
//!
//! Each macro has a comprehensive example file:
//!
//! **Derive Macros:**
//! - `demo/proc_macro_derive_constructor.rs` - Basic derive macro usage
//! - `demo/proc_macro_derive_getters.rs` - Getter generation example
//! - `demo/proc_macro_derive_builder.rs` - Builder pattern implementation
//! - `demo/proc_macro_derive_display.rs` - Display trait generation
//! - `demo/proc_macro_derive_doc_comment.rs` - Enhanced attribute parsing demo
//!
//! **Attribute Macros:**
//! - `demo/proc_macro_cached.rs` - Automatic function memoization
//! - `demo/proc_macro_timing.rs` - Execution time measurement
//! - `demo/proc_macro_retry.rs` - Automatic retry logic
//!
//! **Function-like Macros:**
//! - `demo/proc_macro_file_navigator.rs` - Interactive file operations
//! - `demo/proc_macro_compile_time_assert.rs` - Compile-time validation
//!
//! ## Educational Value
//!
//! This collection demonstrates:
//! - Clean proc macro structure and organization
//! - Working with `syn` for parsing Rust syntax
//! - Using `quote` for code generation
//! - Field iteration and type analysis
//! - Method generation with documentation
//! - Builder pattern implementation
//! - Trait implementation generation (Display)
//! - Complex struct and enum handling
//! - Attribute parsing techniques across multiple item types
//! - Function wrapping and transformation
//! - Caching and memoization patterns
//! - Performance measurement and retry logic
//! - Compile-time validation and assertions
//! - Environment variable processing
//! - Test automation and generation
//! - Error handling in proc macros
//! - Function-like macro patterns

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

/// A basic derive macro that generates a `new` constructor method.
///
/// This macro demonstrates derive macro fundamentals by generating a `new` method
/// for structs. The method takes parameters for all fields and returns a new instance.
/// Perfect for learning basic derive macro concepts.
///
/// #### Features
/// - Generates constructor methods for structs with named fields
/// - Proper error handling for unsupported types
/// - Clean, readable generated code
/// - Comprehensive documentation
///
/// #### Example
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

    // If the `expand` feature is enabled, check if the `expand_macro` attribute
    // is present
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

/// Derives a builder pattern for struct construction.
///
/// This macro generates a builder struct and methods that allow for step-by-step
/// construction of the target struct. It demonstrates advanced derive macro concepts
/// including struct generation, method chaining, and error handling.
///
/// #### Features
/// - Generates a separate builder struct with optional fields
/// - Fluent API with method chaining
/// - Build-time validation ensuring all fields are set
/// - Comprehensive error messages for missing fields
/// - Default trait implementation for builder
///
/// #### Example
/// ```rust
/// use thag_demo_proc_macros::DeriveBuilder;
/// #[derive(DeriveBuilder)]
/// struct Config {
///     host: String,
///     port: u16,
///     timeout: u64,
/// }
/// // Generates: ConfigBuilder with fluent API
/// // let config = Config::builder().host("localhost").port(8080).timeout(5000).build()?;
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

/// Derives a Display trait implementation for structs and enums.
///
/// This macro automatically generates Display implementations that provide
/// readable string representations. It demonstrates trait implementation generation,
/// pattern matching for enums, and field formatting techniques.
///
/// #### Features
/// - Supports structs with named fields, tuple structs, and unit structs
/// - Handles enums with all variant types (unit, tuple, struct)
/// - Automatic field formatting with proper separators
/// - Type-aware formatting for different structures
/// - Clean, readable output format
///
/// #### Example
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

/// Derives getter methods for all fields in a struct.
///
/// This macro automatically generates getter methods that return references to each field,
/// avoiding unnecessary moves. It demonstrates intermediate derive macro concepts including
/// field iteration, type analysis, and method generation with documentation.
///
/// #### Features
/// - Automatic getter generation for all struct fields
/// - Returns references to avoid unnecessary moves
/// - Generated documentation for each getter method
/// - Proper error handling for unsupported types
/// - Type-aware return types
///
/// #### Example
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

    // If the `expand` feature is enabled, check if the `expand_macro` attribute
    // is present
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

/// Derives documentation methods for enum variants.
///
/// This macro demonstrates advanced derive macro techniques including attribute parsing
/// and documentation extraction. It generates a `doc_comment` method that returns
/// the documentation string for each enum variant.
///
/// #### Features
/// - Extracts documentation from enum variant attributes
/// - Generates match expressions for documentation access
/// - Demonstrates attribute parsing techniques
/// - Error handling for missing documentation
///
/// #### Example
/// ```rust
/// use thag_demo_proc_macros::DeriveDocComment;
/// #[derive(DeriveDocComment)]
/// enum Status {
///     /// The operation completed successfully
///     Success,
///     /// An error occurred during processing
///     Error,
/// }
/// // Generates: impl Status { fn doc_comment(&self) -> &'static str { ... } }
/// ```
#[proc_macro_derive(DeriveDocComment)]
pub fn derive_doc_comment(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(true, "derive_doc_comment", &input, derive_doc_comment_impl)
}

/// Generates a `FileNavigator` for interactive file system navigation.
///
/// This function-like macro demonstrates practical proc macro applications by generating
/// code for file system navigation with a command-line interface. It creates structures
/// and functions for selecting files and directories interactively.
///
/// #### Features
/// - Interactive file selection with filtering
/// - Directory navigation capabilities
/// - File saving functionality
/// - Cross-platform compatibility
///
/// #### Example
/// ```ignore
/// use std::path::PathBuf;
/// use thag_demo_proc_macros::file_navigator;
/// file_navigator! {}
/// // Generates: FileNavigator struct, select_file function, save_to_file function, etc.
/// ```
#[proc_macro]
pub fn file_navigator(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(true, "file_navigator", &input, file_navigator_impl)
}

//// Attribute macro that adds automatic memoization/caching to functions.
///
/// This macro demonstrates advanced attribute macro techniques by wrapping functions
/// with caching logic. It automatically stores function results and returns cached
/// values for repeated calls with the same parameters.
///
/// #### Features
/// - Automatic result caching using HashMap
/// - Thread-safe cache with Mutex
/// - Supports functions with multiple parameters
/// - Compile-time cache key generation
/// - Clone trait bounds for parameters and return types
///
/// #### Example
/// ```rust
/// use thag_demo_proc_macros::cached;
/// #[cached]
/// fn expensive_computation(n: u32) -> u32 {
///     // Expensive operation here
///     std::thread::sleep(std::time::Duration::from_secs(1));
///     n * n
/// }
/// ```
#[proc_macro_attribute]
pub fn cached(attr: TokenStream, item: TokenStream) -> TokenStream {
    maybe_expand_attr_macro(true, "cached", &attr, &item, cached_impl)
}

/// Attribute macro that adds automatic timing measurement to functions.
///
/// This macro wraps functions to measure and display their execution time.
/// It demonstrates simple attribute macro patterns and is useful for performance
/// analysis and debugging.
///
/// #### Features
/// - Automatic execution time measurement
/// - Console output with function name and duration
/// - Zero runtime overhead when not applied
/// - Works with any function signature
///
/// #### Example
/// ```rust
/// use thag_demo_proc_macros::timing;
/// #[timing]
/// fn slow_function() -> i32 {
///     std::thread::sleep(std::time::Duration::from_millis(100));
///     42
/// }
/// // Output: ⏱️  Function 'slow_function' took: 100.234ms
/// ```
#[proc_macro_attribute]
pub fn timing(attr: TokenStream, item: TokenStream) -> TokenStream {
    maybe_expand_attr_macro(true, "timing", &attr, &item, timing_impl)
}

/// Attribute macro that adds automatic retry logic to functions.
///
/// This macro wraps functions with retry logic that will automatically retry
/// failed function calls. It demonstrates attribute macro parameter parsing
/// and error handling patterns.
///
/// #### Features
/// - Configurable retry count with `times` parameter
/// - Automatic backoff delay between retries
/// - Panic catching and retry logic
/// - Progress reporting for retry attempts
/// - Graceful failure after max retries
///
/// #### Example
/// ```ignore
/// use thag_demo_proc_macros::retry;
/// #[retry(times = 5)]
/// fn unreliable_network_call() -> Result<String, std::io::Error> {
///     // Simulated unreliable operation
///     if rand::random::<f32>() < 0.7 {
///         Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Failed"))
///     } else {
///         Ok("Success".to_string())
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn retry(attr: TokenStream, item: TokenStream) -> TokenStream {
    maybe_expand_attr_macro(true, "retry", &attr, &item, retry_impl)
}

/// Function-like macro for compile-time assertions.
///
/// This macro generates compile-time checks that will cause compilation to fail
/// if the specified condition is not true. It demonstrates function-like macro
/// parsing with multiple parameters and compile-time validation techniques.
///
/// #### Features
/// - Compile-time condition evaluation
/// - Custom error messages for failed assertions
/// - Zero runtime overhead (assertions are checked at compile time)
/// - Supports any boolean expression that can be evaluated at compile time
///
/// #### Example
/// ```rust
/// use thag_demo_proc_macros::compile_time_assert;
/// compile_time_assert!(std::mem::size_of::<usize>() == 8, "This code requires 64-bit systems");
/// compile_time_assert!(1 + 1 == 2, "Basic math must work");
/// ```
#[proc_macro]
pub fn compile_time_assert(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(
        true,
        "compile_time_assert",
        &input,
        compile_time_assert_impl,
    )
}

/// Function-like macro for environment variable access with default fallback.
///
/// This macro reads environment variables at compile time and provides fallback
/// values when variables are not set. It demonstrates compile-time environment
/// variable processing and conditional value generation.
///
/// #### Features
/// - Compile-time environment variable resolution
/// - Automatic fallback to default values
/// - Zero runtime overhead
/// - Configuration management patterns
///
/// #### Example
/// ```rust
/// const DATABASE_URL: &str = env_or_default!("DATABASE_URL", "localhost:5432");
/// const DEBUG_MODE: &str = env_or_default!("DEBUG", "false");
/// ```
#[proc_macro]
pub fn env_or_default(input: TokenStream) -> TokenStream {
    // env_or_default_impl(input)
    maybe_expand_proc_macro(true, "env_or_default", &input, env_or_default_impl)
}

/// Function-like macro for generating repetitive test cases.
///
/// This macro generates multiple test functions from a list of test data,
/// reducing boilerplate code in test suites. It demonstrates repetitive
/// code generation and test automation patterns.
///
/// #### Features
/// - Automatic test function generation
/// - Support for multiple test cases
/// - Parameter unpacking from tuples
/// - Reduces test code duplication
///
/// #### Example
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
    maybe_expand_proc_macro(true, "generate_tests", &input, generate_tests_impl)
}

/// A helper function for conditional macro expansion.
///
/// This utility function demonstrates how to conditionally expand proc macros
/// for debugging purposes, which is useful during development.
fn maybe_expand_proc_macro<F>(
    expand: bool,
    name: &str,
    input: &TokenStream,
    proc_macro: F,
) -> TokenStream
where
    F: Fn(TokenStream) -> TokenStream,
{
    // Call the provided macro function
    let output = proc_macro(input.clone());

    if expand {
        expand_output(name, &output);
    }

    output
}

/// A helper function for conditional attribute macro expansion.
///
/// Similar to `maybe_expand_proc_macro` but specifically for attribute macros
/// that take both attribute and item token streams.
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
    // Call the provided macro function
    let output = attr_macro(attr.clone(), item.clone());

    if expand {
        expand_output(name, &output);
    }

    output
}

fn expand_output(name: &str, output: &TokenStream) {
    // Pretty-print the expanded tokens
    use inline_colorization::{color_cyan, color_reset, style_bold, style_reset, style_underline};
    let output: proc_macro2::TokenStream = output.clone().into();
    let token_str = output.to_string();
    let dash_line = "─".repeat(70);

    // First try to parse as a file
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
        // If parsing as a file fails, try parsing as an expression
        Err(_) => match parse_str::<Expr>(&token_str) {
            Ok(expr) => {
                // For expressions, we don't have a pretty printer, so just output the token string
                eprintln!("{style_reset}{dash_line}{style_reset}");
                eprintln!(
                            "{style_bold}{style_underline}Expanded macro{style_reset} {style_bold}{color_cyan}{name}{color_reset} (as expression):{style_reset}\n"
                        );
                eprintln!("{}", quote!(#expr));
                eprintln!("{style_reset}{dash_line}{style_reset}");
            }
            Err(_e) => {
                // eprintln!("Failed to parse tokens as file or expression: {e:?}");
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
