#![allow(clippy::missing_panics_doc, dead_code, unused_imports)]
//! # Procedural Macros Demo Collection
//!
//! This crate provides a curated collection of high-quality procedural macros demonstrating
//! various techniques and patterns for writing proc macros in Rust. It serves as
//! educational material for developers learning to create their own procedural macros.
//!
//! ## Overview
//!
//! The collection focuses on quality over quantity, featuring 7 carefully selected macros
//! that demonstrate progressive complexity and real-world utility:
//!
//! ### Core Macros
//!
//! 1. **[`DeriveConstructor`]** - Basic derive macro for generating constructor methods
//! 2. **[`DeriveGetters`]** - Intermediate derive macro for generating getter methods
//! 3. **[`DeriveBuilder`]** - Advanced derive macro implementing the builder pattern
//! 4. **[`DeriveDisplay`]** - Trait implementation macro for Display formatting
//! 5. **[`DeriveDocComment`]** - Advanced derive macro demonstrating attribute parsing
//! 6. **[`file_navigator`]** - Function-like macro for file system navigation
//! 7. **[`const_demo`]** - Complex macro using external crates for const generation
//!
//! ## Progressive Learning Path
//!
//! The macros are designed to provide a progressive learning experience:
//!
//! - **Basic**: Start with `DeriveConstructor` to understand derive macro fundamentals
//! - **Intermediate**: Progress to `DeriveGetters` for method generation patterns
//! - **Advanced**: Learn builder patterns with `DeriveBuilder` and trait implementation with `DeriveDisplay`
//! - **Expert**: Master attribute parsing with `DeriveDocComment`
//! - **Practical**: Explore function-like macros with `file_navigator`
//! - **Complex**: Study advanced techniques with `const_demo`
//!
//! ## Usage
//!
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
//! use thag_demo_proc_macros::{DeriveConstructor, DeriveGetters, DeriveBuilder, DeriveDisplay};
//! ```
//!
//! ## Examples
//!
//! Each macro has a comprehensive example file:
//! - `demo/proc_macro_derive_constructor.rs` - Basic derive macro usage
//! - `demo/proc_macro_derive_getters.rs` - Getter generation example
//! - `demo/proc_macro_derive_builder.rs` - Builder pattern implementation
//! - `demo/proc_macro_derive_display.rs` - Display trait generation
//! - `demo/proc_macro_derive_doc_comment.rs` - Attribute parsing demo
//! - `demo/proc_macro_file_navigator.rs` - Interactive file operations
//! - `demo/proc_macro_const_demo.rs` - Advanced const generation
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
//! - Attribute parsing techniques
//! - Error handling in proc macros
//! - Function-like macro patterns
//! - Integration with external crates

mod const_demo;
mod derive_builder;
mod derive_constructor;
mod derive_display;
mod derive_doc_comment;
mod derive_getters;
mod file_navigator;

use crate::const_demo::const_demo_impl;
use crate::derive_builder::derive_builder_impl;
use crate::derive_constructor::derive_constructor_impl;
use crate::derive_display::derive_display_impl;
use crate::derive_doc_comment::derive_doc_comment_impl;
use crate::derive_getters::derive_getters_impl;
use crate::file_navigator::file_navigator_impl;
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
    derive_doc_comment_impl(input)
    // maybe_expand_proc_macro(true, "derive_doc_comment", &input, derive_doc_comment_impl)
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
/// ```rust
/// file_navigator! {}
/// // Generates: FileNavigator struct, select_file function, save_to_file function, etc.
/// ```
#[proc_macro]
pub fn file_navigator(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(true, "file_navigator", &input, file_navigator_impl)
}

/// Advanced constant generation using external crates.
///
/// This macro demonstrates complex proc macro techniques by using the `const_gen_proc_macro`
/// crate for compile-time constant generation. It showcases integration with external
/// dependencies and advanced code generation patterns.
///
/// #### Features
/// - Compile-time constant generation
/// - Integration with external proc macro crates
/// - Complex object manipulation
/// - Advanced expression handling
///
/// #### Example
/// ```rust
/// const_demo!(
///     let math = math::new(10);
///     math.add(5);
///     let result = math.get();
/// );
/// ```
#[proc_macro]
pub fn const_demo(input: TokenStream) -> TokenStream {
    // const_demo_impl(input)
    maybe_expand_proc_macro(true, "const_demo", &input, const_demo_impl)
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
