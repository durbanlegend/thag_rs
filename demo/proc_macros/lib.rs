#![allow(clippy::missing_panics_doc, dead_code, unused_imports)]
//! Procedural macros for generating enums and utilities for managing script categories.
//!
mod ansi_code_derive;
mod attrib_key_map_list;
mod attribute_basic;
mod const_demo;
mod const_demo_grail;
mod const_gen_str_demo;
mod custom_model;
mod derive_basic;
mod derive_deserialize_vec;
mod derive_doc_comment;
mod derive_key_map_list;
mod expander_demo;
mod host_port_const;
mod into_string_hash_map;
mod load_static_map;
mod my_description;
mod organizing_code;
mod organizing_code_const;
mod organizing_code_tokenstream;
mod repeat_dash;

use crate::ansi_code_derive::ansi_code_derive_impl;
use crate::attrib_key_map_list::use_mappings_impl;
use crate::attribute_basic::attribute_basic_impl;
use crate::const_demo::const_demo_impl;
use crate::const_demo_grail::const_demo_grail_impl;
use crate::const_gen_str_demo::string_concat_impl;
use crate::custom_model::derive_custom_model_impl;
use crate::derive_basic::derive_basic_impl;
use crate::derive_deserialize_vec::derive_deserialize_vec_impl;
use crate::derive_doc_comment::derive_doc_comment_impl;
use crate::derive_key_map_list::derive_key_map_list_impl;
use crate::expander_demo::baz2;
use crate::host_port_const::host_port_const_impl;
use crate::into_string_hash_map::into_hash_map_impl;
use crate::load_static_map::load_static_map_impl;
use crate::my_description::my_derive;
use crate::organizing_code::organizing_code_impl;
use crate::organizing_code_const::organizing_code_const_impl;
use crate::organizing_code_tokenstream::organizing_code_tokenstream_impl;
use crate::repeat_dash::repeat_dash_impl;
use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use std::path::Path;
use syn::{
    parse::{Parse, ParseStream},
    parse_file, parse_macro_input, parse_str, DeriveInput, Expr, ExprArray, Ident, LitInt, LitStr,
    Token,
};

/// A derive macro that generates helpful methods for ANSI color enums.
///
/// This macro generates:
/// - A `name()` method that returns a human-readable name for each color variant
/// - A `FromStr` trait implementation to parse variants from `snake_case` strings
///
/// # Attributes
/// - `#[ansi_name("Custom Name")]`: Override the default name for a variant
///
/// # Example Usage
/// See `demo/proc_macro_ansi_code_derive.rs` for a complete example.
///
/// ```rust
/// #[derive(AnsiCodeDerive)]
/// enum Color {
///     Red,
///     #[ansi_name("Dark Gray")]
///     BrightBlack,
/// }
/// ```
#[proc_macro_derive(AnsiCodeDerive, attributes(ansi_name))]
pub fn ansi_code_derive(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "ansi_code_derive", &input, ansi_code_derive_impl)
}

/// A basic attribute macro that demonstrates attribute macro functionality.
///
/// This is a simple example of an attribute macro that can be applied to items.
/// See `demo/proc_macro_attribute_basic.rs` for usage examples.
///
/// # Example
/// ```rust
/// #[attribute_basic]
/// fn my_function() { }
/// ```
#[proc_macro_attribute]
pub fn attribute_basic(_attr: TokenStream, input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(true, "attribute_basic", &input, attribute_basic_impl)
}

/// A basic derive macro that generates a `new` constructor method.
///
/// This macro demonstrates derive macro functionality by generating a `new` method
/// for structs. The method takes parameters for all fields and returns a new instance.
///
/// # Attributes
/// - `#[expand_macro]`: When present, the macro expansion will be displayed during compilation
///
/// # Example
/// See `demo/proc_macro_derive_basic.rs` for usage.
///
/// ```rust
/// #[derive(DeriveBasic)]
/// struct MyStruct {
///     field: String,
/// }
/// // Generates: impl MyStruct { fn new(field: String) -> Self { ... } }
/// ```
#[proc_macro_derive(DeriveBasic, attributes(expand_macro))]
pub fn derive_basic(input: TokenStream) -> TokenStream {
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
        "derive_basic",
        &input_clone,
        derive_basic_impl,
    )
}

/// A basic function-like macro that generates a constant value.
///
/// This demonstrates function-like macro syntax and generates a constant `VALUE` set to 42.
/// See `demo/proc_macro_functionlike_basic.rs` for usage.
///
/// # Example
/// ```rust
/// function_like_basic!();
/// // Generates: pub const VALUE: usize = 42;
/// ```
#[proc_macro]
pub fn function_like_basic(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(
        cfg!(feature = "expand"),
        "function_like_basic",
        &input,
        |_tokens| {
            // Original macro logic
            let expanded = quote! {
                pub const VALUE: usize = 42;
            };
            TokenStream::from(expanded)
        },
    )
}

/// Derives a custom model with additional functionality.
///
/// This macro demonstrates more advanced derive macro capabilities with custom attributes.
/// See `demo/proc_macro_derive_custom_model.rs` for detailed usage.
///
/// # Attributes
/// - `#[custom_model]`: Configures the custom model generation
///
/// # Example
/// ```rust
/// #[derive(DeriveCustomModel)]
/// #[custom_model]
/// struct MyModel {
///     id: u32,
///     name: String,
/// }
/// ```
#[proc_macro_derive(DeriveCustomModel, attributes(custom_model))]
pub fn derive_custom_model(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(
        true,
        "derive_custom_model",
        &input,
        derive_custom_model_impl,
    )
}

/// Derives conversion functionality to convert structs into `HashMap<String, String>`.
///
/// This macro generates an implementation that converts struct fields into a string-based HashMap.
/// Useful for serialization scenarios or when you need a dynamic key-value representation.
///
/// # Example
/// ```rust
/// #[derive(IntoStringHashMap)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
/// // Generates methods to convert Person into HashMap<String, String>
/// ```
#[proc_macro_derive(IntoStringHashMap)]
pub fn into_hash_map(input: TokenStream) -> TokenStream {
    into_hash_map_impl(input)
}

/// Derives description functionality with custom attributes.
///
/// This macro demonstrates using the `deluxe` crate for advanced attribute parsing.
/// See the implementation for details on how custom descriptions are generated.
///
/// # Attributes
/// - `#[my_desc]`: Specifies custom description attributes
///
/// # Example
/// ```rust
/// #[derive(MyDescription)]
/// #[my_desc(description = "A sample struct")]
/// struct Sample {
///     value: i32,
/// }
/// ```
#[proc_macro_derive(MyDescription, attributes(my_desc))]
pub fn derive_my_description(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    my_derive(input.into()).unwrap().into()
}

/// Derives vector deserialization functionality using the `deluxe` crate.
///
/// This macro demonstrates advanced attribute parsing and generates deserialization
/// methods for vector types with custom mappings.
///
/// # Attributes
/// - `#[deluxe]`: Configures deluxe attribute parsing
/// - `#[use_mappings]`: Specifies field mappings for deserialization
///
/// # Example
/// ```rust
/// #[derive(DeserializeVec)]
/// #[use_mappings(field1 = "alias1")]
/// struct Data {
///     field1: String,
///     items: Vec<String>,
/// }
/// ```
#[proc_macro_derive(DeserializeVec, attributes(deluxe, use_mappings))]
pub fn derive_deserialize_vec(input: TokenStream) -> TokenStream {
    derive_deserialize_vec_impl(input.into()).unwrap().into()
}

/// Derives key-map list functionality with advanced attribute support.
///
/// This macro generates methods for working with key-mapped lists, demonstrating
/// complex derive macro patterns with the `deluxe` crate.
/// See `demo/proc_macro_derive_key_map_list.rs` for usage examples.
///
/// # Attributes
/// - `#[deluxe]`: Enables deluxe attribute parsing
/// - `#[use_mappings]`: Configures key mappings
///
/// # Example
/// ```rust
/// #[derive(DeriveKeyMapList)]
/// #[use_mappings(key1 = "mapped_key1")]
/// struct KeyMap {
///     key1: String,
///     values: Vec<String>,
/// }
/// ```
#[proc_macro_derive(DeriveKeyMapList, attributes(deluxe, use_mappings))]
pub fn derive_key_map_list(input: TokenStream) -> TokenStream {
    derive_key_map_list_impl(input.into()).unwrap().into()
}

/// A function-like macro demonstrating code organization patterns.
///
/// This macro is based on examples from https://github.com/tdimitrov/rust-proc-macro-post
/// and shows how to organize complex macro logic. See `demo/proc_macro_organizing_code.rs`
/// for usage examples.
///
/// # Example
/// ```rust
/// organizing_code!{
///     // Your code here
/// }
/// ```
#[proc_macro]
pub fn organizing_code(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    organizing_code_impl(input).into()
}

/// A function-like macro demonstrating TokenStream manipulation.
///
/// This macro shows advanced TokenStream processing techniques, also based on examples
/// from https://github.com/tdimitrov/rust-proc-macro-post.
/// See `demo/proc_macro_organizing_code_tokenstream.rs` for usage.
///
/// # Example
/// ```rust
/// organizing_code_tokenstream!{
///     // TokenStream content
/// }
/// ```
#[proc_macro]
pub fn organizing_code_tokenstream(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    organizing_code_tokenstream_impl(input).into()
}

/// Derives constant generation functionality with adjustment capabilities.
///
/// This macro demonstrates compile-time constant generation with configurable
/// adjustments. See `demo/proc_macro_organizing_code_const.rs` for examples.
///
/// # Attributes
/// - `#[adjust]`: Configures value adjustments
/// - `#[use_mappings]`: Specifies mapping configurations
///
/// # Example
/// ```rust
/// #[derive(DeriveConst)]
/// #[adjust(factor = 2)]
/// struct Config {
///     base_value: u32,
/// }
/// ```
#[proc_macro_derive(DeriveConst, attributes(adjust, use_mappings))]
pub fn organizing_code_const(input: TokenStream) -> TokenStream {
    // organizing_code_const_impl(input.into()).unwrap().into()
    maybe_expand_proc_macro(true, "organizing_code_const", &input, |tokens| {
        organizing_code_const_impl(tokens.into()).unwrap().into()
    })
}

/// An attribute macro demonstrating the `expander` crate functionality.
///
/// This macro showcases macro expansion capabilities using the expander crate.
/// See `demo/proc_macro_expander_demo.rs` for usage examples.
///
/// # Example
/// ```rust
/// #[baz]
/// fn my_function() {
///     // function body
/// }
/// ```
#[proc_macro_attribute]
pub fn baz(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // wrap as per usual for `proc-macro2::TokenStream`, here dropping `attr` for simplicity
    baz2(input.into()).into()
}

/// An attribute macro for configuring field mappings.
///
/// This macro processes mapping configurations and applies them to the decorated item.
/// Often used in conjunction with other derive macros for customization.
///
/// # Example
/// ```rust
/// #[use_mappings(field1 = "alias1", field2 = "alias2")]
/// struct MappedStruct {
///     field1: String,
///     field2: i32,
/// }
/// ```
#[proc_macro_attribute]
pub fn use_mappings(attr: TokenStream, input: TokenStream) -> TokenStream {
    use_mappings_impl(attr, input)
}

/// A function-like macro that generates repeated dash characters.
///
/// This macro demonstrates simple text generation and can be used to create
/// visual separators or formatting elements. See `demo/proc_macro_repeat_dash.rs`
/// for usage examples.
///
/// # Example
/// ```rust
/// repeat_dash!(10); // Generates 10 dash characters
/// ```
#[proc_macro]
pub fn repeat_dash(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(
        cfg!(feature = "expand"),
        "repeat_dash",
        &input,
        repeat_dash_impl,
    )
}

/// A function-like macro for compile-time string concatenation.
///
/// This macro demonstrates compile-time string manipulation and concatenation.
/// See `demo/proc_macro_string_concat.rs` for usage examples.
///
/// # Example
/// ```rust
/// string_concat!("Hello", " ", "World"); // Generates "Hello World"
/// ```
#[proc_macro]
pub fn string_concat(tokens: TokenStream) -> TokenStream {
    string_concat_impl(tokens)
}

/// A function-like macro demonstrating compile-time constant generation.
///
/// This macro shows how to generate constants at compile time based on input parameters.
/// See `demo/proc_macro_const_demo.rs` for basic usage.
///
/// # Example
/// ```rust
/// const_demo!(MyConst = 42);
/// ```
#[proc_macro]
pub fn const_demo(tokens: TokenStream) -> TokenStream {
    const_demo_impl(tokens)
}

/// A variant of `const_demo` that shows macro expansion during compilation.
///
/// This version of the const demo macro displays the generated code during compilation,
/// useful for debugging and understanding macro output. See `demo/proc_macro_const_demo_expand.rs`.
///
/// # Example
/// ```rust
/// const_demo_expand!(MyConst = 42);
/// ```
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

/// A debug variant of `const_demo` with detailed expansion information.
///
/// This macro provides debugging capabilities for const generation, showing
/// detailed expansion information. See `demo/proc_macro_const_demo_debug.rs`.
///
/// # Example
/// ```rust
/// const_demo_debug!(MyConst = 42);
/// ```
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

/// An advanced constant generation macro using the `const_gen` crate.
///
/// This macro demonstrates advanced compile-time constant generation techniques
/// using external crates. See the implementation for technical details.
///
/// # Example
/// ```rust
/// const_demo_grail!(AdvancedConst = complex_computation());
/// ```
#[proc_macro]
pub fn const_demo_grail(tokens: TokenStream) -> TokenStream {
    const_demo_grail_impl(tokens)
}

/// Derives compile-time constants for host and port configurations.
///
/// This macro generates compile-time constants for network configuration,
/// demonstrating practical applications of const generation.
/// See `demo/proc_macro_host_port_const.rs` for usage.
///
/// # Attributes
/// - `#[const_value]`: Specifies the constant value configuration
///
/// # Example
/// ```rust
/// #[derive(HostPortConst)]
/// #[const_value(host = "localhost", port = 8080)]
/// struct ServerConfig;
/// ```
#[proc_macro_derive(HostPortConst, attributes(const_value))]
pub fn host_port_const(tokens: TokenStream) -> TokenStream {
    host_port_const_impl(tokens.into()).into()
}

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

/// Derives automatic documentation comment generation.
///
/// This macro demonstrates automatic generation of documentation comments
/// based on struct or enum definitions. See `demo/proc_macro_derive_doc_comment.rs`
/// for examples.
///
/// # Example
/// ```rust
/// #[derive(DocComment)]
/// struct Example {
///     field: String,
/// }
/// // Generates documentation comments automatically
/// ```
#[proc_macro_derive(DocComment)]
pub fn derive_doc_comment(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(true, "derive_doc_comment", &input, |tokens| {
        derive_doc_comment_impl(tokens)
    })
}

/// A function-like macro for embedding file contents at compile time.
///
/// This macro reads a file at compile time and embeds its contents as a string literal.
/// Useful for including configuration files, templates, or other static content.
///
/// # Example
/// ```rust
/// let content = embed_file!("config.txt");
/// ```
#[proc_macro]
pub fn embed_file(input: TokenStream) -> TokenStream {
    println!("The current directory is {:#?}", std::env::current_dir());

    println!("vars={:#?}", std::env::vars());

    #[cfg(target_os = "windows")]
    let pwd = std::env::var("pwd").expect("Could not resolve $pwd");

    #[cfg(not(target_os = "windows"))]
    let pwd = std::env::var("PWD").expect("Could not resolve $PWD");

    println!("PWD={pwd}");
    let embed = parse_macro_input!(input as LitStr).value();
    let path = std::path::PathBuf::from(pwd).join(&embed);
    println!("path={path:#?}");
    let content = fs::read_to_string(Path::new(&path)).expect("Failed to read file");

    quote! {
        #content
    }
    .into()
}

/// An advanced macro for loading directory contents into static maps.
///
/// This macro can embed entire directory structures as static HashMap data,
/// useful for embedding web assets, templates, or configuration directories.
/// See `demo/proc_macro_load_static_map.rs` for usage examples.
///
/// # Example
/// ```rust
/// load_static_map!("assets/");
/// // Generates a static HashMap with file paths as keys and contents as values
/// ```
#[proc_macro]
pub fn load_static_map(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(true, "load_static_map", &input, load_static_map_impl)
}

/// Creates a function that returns the line number where it's called.
///
/// This macro generates a function with a name based on the input string that
/// returns the current line number using the `line!()` macro. Useful for debugging
/// and testing scenarios. See `demo/proc_macro_end.rs` for usage.
///
/// # Example
/// ```rust
/// end!("my_function");
/// // Generates: fn end_my_function() -> u32 { line!() }
///
/// fn main() {
///     println!("Current line: {}", end_my_function());
/// }
/// ```
#[proc_macro]
pub fn end(input: TokenStream) -> TokenStream {
    use quote::format_ident;
    // Parse the input as a string literal
    let func_name_lit = parse_macro_input!(input as LitStr);
    let func_name_str = func_name_lit.value();

    // Convert the string to an identifier
    let func_name = format_ident!("end_{}", func_name_str);

    // Generate the function that returns line!()
    let expanded = quote! {
        fn #func_name() -> u32 {
            line!()
        }
    };

    // Return the generated code
    expanded.into()
}
