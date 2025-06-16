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

/// For an enum of ANDI colors, generates a `name` method to generate a human-readable name for each color, and
/// a `from_str` trait implementation method to parse a variant from a `snake_case` name.
///
/// Used by `demo/proc_macro_ansi_code_derive.rs`
///
#[proc_macro_derive(AnsiCodeDerive, attributes(ansi_name))]
pub fn ansi_code_derive(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "ansi_code_derive", &input, ansi_code_derive_impl)
}

#[proc_macro_attribute]
pub fn attribute_basic(_attr: TokenStream, input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(true, "attribute_basic", &input, attribute_basic_impl)
}

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

#[proc_macro_derive(DeriveCustomModel, attributes(custom_model))]
pub fn derive_custom_model(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(
        true,
        "derive_custom_model",
        &input,
        derive_custom_model_impl,
    )
}

#[proc_macro_derive(IntoStringHashMap)]
pub fn into_hash_map(input: TokenStream) -> TokenStream {
    into_hash_map_impl(input)
}

#[proc_macro_derive(MyDescription, attributes(my_desc))]
pub fn derive_my_description(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    my_derive(input.into()).unwrap().into()
}

// Define the custom derive macro using `deluxe`
#[proc_macro_derive(DeserializeVec, attributes(deluxe, use_mappings))]
pub fn derive_deserialize_vec(input: TokenStream) -> TokenStream {
    derive_deserialize_vec_impl(input.into()).unwrap().into()
}

#[proc_macro_derive(DeriveKeyMapList, attributes(deluxe, use_mappings))]
pub fn derive_key_map_list(input: TokenStream) -> TokenStream {
    derive_key_map_list_impl(input.into()).unwrap().into()
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
    // organizing_code_const_impl(input.into()).unwrap().into()
    maybe_expand_proc_macro(true, "organizing_code_const", &input, |tokens| {
        organizing_code_const_impl(tokens.into()).unwrap().into()
    })
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
pub fn use_mappings(attr: TokenStream, input: TokenStream) -> TokenStream {
    use_mappings_impl(attr, input)
}

#[proc_macro]
pub fn repeat_dash(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(
        cfg!(feature = "expand"),
        "repeat_dash",
        &input,
        repeat_dash_impl,
    )
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

#[proc_macro]
pub fn const_demo_grail(tokens: TokenStream) -> TokenStream {
    const_demo_grail_impl(tokens)
}

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

#[proc_macro_derive(DocComment)]
pub fn derive_doc_comment(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(true, "derive_doc_comment", &input, |tokens| {
        derive_doc_comment_impl(tokens)
    })
}

/// Basic file embedding handles a file.
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

/// More advanced embedding can handle a directory.
#[proc_macro]
pub fn load_static_map(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(true, "load_static_map", &input, load_static_map_impl)
}

/// Creates a function with the name specified in the string literal
/// that returns the line number where the function is called.
///
/// # Example
///
/// ```
/// use your_crate::line_function;
///
/// line_function!("get_line");
///
/// fn main() {
///     println!("Current line: {}", get_line()); // prints the current line number
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
