#![allow(clippy::missing_panics_doc, unused_imports)]
//! Procedural macros for generating enums and utilities for managing script categories.
//!
//! For detailed documentation on the `category_enum` macro, see the
//! [dedicated documentation](https://github.com/thag_rs/demo/proc_macros/docs/category_enum.md).
mod attrib_key_map_list;
mod attribute_basic;
mod category_enum;
mod const_demo;
mod const_demo_grail;
mod const_gen_str_demo;
mod custom_model;
mod derive_basic;
mod derive_deserialize_vec;
mod derive_key_map_list;
mod expander_demo;
mod host_port_const;
mod into_string_hash_map;
mod my_description;
mod organizing_code;
mod organizing_code_const;
mod organizing_code_tokenstream;
mod repeat_dash;

use crate::attrib_key_map_list::use_mappings_impl;
use crate::attribute_basic::attribute_basic_impl;
use crate::category_enum::category_enum_impl;
use crate::const_demo::const_demo_impl;
use crate::const_demo_grail::const_demo_grail_impl;
use crate::const_gen_str_demo::string_concat_impl;
use crate::custom_model::derive_custom_model_impl;
use crate::derive_basic::derive_basic_impl;
use crate::derive_deserialize_vec::derive_deserialize_vec_impl;
use crate::derive_key_map_list::derive_key_map_list_impl;
use crate::expander_demo::baz2;
use crate::host_port_const::host_port_const_impl;
use crate::into_string_hash_map::into_hash_map_impl;
use crate::my_description::my_derive;
use crate::organizing_code::organizing_code_impl;
use crate::organizing_code_const::organizing_code_const_impl;
use crate::organizing_code_tokenstream::organizing_code_tokenstream_impl;
use crate::repeat_dash::repeat_dash_impl;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_file, parse_macro_input, DeriveInput, ExprArray, Ident, LitInt, Token,
};

#[proc_macro_attribute]
pub fn attribute_basic(_attr: TokenStream, input: TokenStream) -> TokenStream {
    intercept_and_debug(cfg!(feature = "expand"), input, attribute_basic_impl)
}

#[proc_macro_derive(DeriveBasic)]
pub fn derive_basic(input: TokenStream) -> TokenStream {
    intercept_and_debug(cfg!(feature = "expand"), input, derive_basic_impl)
}

#[proc_macro]
pub fn function_like_basic(input: TokenStream) -> TokenStream {
    intercept_and_debug(cfg!(feature = "expand"), input, |_tokens| {
        // Original macro logic
        let expanded = quote! {
            pub const VALUE: usize = 42;
        };
        TokenStream::from(expanded)
    })
}

#[proc_macro_derive(DeriveCustomModel, attributes(custom_model))]
pub fn derive_custom_model(input: TokenStream) -> TokenStream {
    derive_custom_model_impl(input)
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
    organizing_code_const_impl(input.into()).unwrap().into()
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
    intercept_and_debug(cfg!(feature = "expand"), input, repeat_dash_impl)
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

fn intercept_and_debug<F>(expand: bool, input: TokenStream, proc_macro: F) -> TokenStream
where
    F: Fn(TokenStream) -> TokenStream,
{
    use inline_colorization::{style_bold, style_reset};

    // Call the provided macro function
    let output = proc_macro(input.clone());

    if expand {
        // Pretty-print the expanded tokens
        let output: proc_macro2::TokenStream = output.clone().into();
        let token_str = output.to_string();
        match parse_file(&token_str) {
            Err(e) => eprintln!("Failed to parse tokens: {e:?}"),
            Ok(syn_file) => {
                let pretty_output = prettyplease::unparse(&syn_file);
                let dash_line = "-".repeat(70);
                eprintln!("{style_reset}{dash_line}{style_reset}");
                eprintln!("{style_bold}Expanded macro:{style_reset}");
                eprint!("{pretty_output}");
                eprintln!("{style_reset}{dash_line}{style_reset}");
            }
        }
    }

    output
}

/// Generates a `Category` enum with predefined variants and utility implementations.
///
/// The `category_enum` macro defines an enum `Category` with a hardcoded set of variants.
/// This ensures consistency across all callers and centralizes control over the available categories.
///
/// Additionally, it generates:
/// - A `FromStr` implementation to parse strings into the `Category` enum.
/// - A utility method `Category::all_categories()` to return a list of all available category names.
///
/// # Usage
///
/// Simply invoke the macro in your project:
///
/// ```rust
/// use demo_proc_macros::category_enum;
///
/// category_enum!();
/// ```
///
/// This generates:
///
/// ```rust
/// pub enum Category {
///     AST,
///     CLI,
///     REPL,
///     Async,
///     Basic,
///     BigNumbers,
///     Crates,
///     Educational,
///     ErrorHandling,
///     Exploration,
///     Macros,
///     Math,
///     ProcMacros,
///     Prototype,
///     Recreational,
///     Reference,
///     Technique,
///     Testing,
///     Tools,
///     TypeIdentification,
/// }
///
/// impl std::str::FromStr for Category {
///     type Err = String;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         match s {
///             "AST" => Ok(Category::AST),
///             "CLI" => Ok(Category::CLI),
///             "REPL" => Ok(Category::REPL),
///             "Async" => Ok(Category::Async),
///             // ... other variants ...
///             _ => Err(format!("Invalid category: {s}")),
///         }
///     }
/// }
///
/// impl Category {
///     pub fn all_categories() -> Vec<&'static str> {
///         vec![
///             "AST", "CLI", "REPL", "Async", "Basic", "BigNumbers", "Crates",
///             "Educational", "ErrorHandling", "Exploration", "Macros", "Math",
///             "ProcMacros", "Prototype", "Recreational", "Reference", "Technique",
///             "Testing", "Tools", "TypeIdentification",
///         ]
///     }
/// }
/// ```
///
/// # Benefits
///
/// - Consistency: The hardcoded list ensures uniformity across all callers.
/// - Convenience: Auto-generated utility methods simplify working with the categories.
/// - Safety: Enums prevent invalid values at compile time.
///
/// # Use Cases
///
/// This macro is ideal for scenarios requiring centralized control over predefined categories,
/// such as filtering demo scripts or generating reports.
#[proc_macro]
pub fn category_enum(input: TokenStream) -> TokenStream {
    intercept_and_debug(false, input, |tokens| category_enum_impl(tokens))
}

// mod category_enum {
//     pub(crate) fn category_enum_impl(_input: TokenStream) -> TokenStream {
//         let expanded = quote! {
//             use std::str::FromStr;

//             #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//             enum Category {
//                 AST,
//                 CLI,
//                 REPL,
//                 Async,
//                 Basic,
//                 BigNumbers,
//                 Crates,
//                 Educational,
//                 ErrorHandling,
//                 Exploration,
//                 Macros,
//                 Math,
//                 ProcMacros,
//                 Prototype,
//                 Recreational,
//                 Reference,
//                 Technique,
//                 Testing,
//                 Tools,
//                 TypeIdentification,
//             }

//             impl FromStr for Category {
//                 type Err = String;

//                 fn from_str(s: &str) -> Result<Self, Self::Err> {
//                     match s.to_lowercase().as_str() {
//                         "ast" => Ok(Category::AST),
//                         "cli" => Ok(Category::CLI),
//                         "repl" => Ok(Category::REPL),
//                         "async" => Ok(Category::Async),
//                         "basic" => Ok(Category::Basic),
//                         "big_numbers" => Ok(Category::BigNumbers),
//                         "crates" => Ok(Category::Crates),
//                         "educational" => Ok(Category::Educational),
//                         "error_handling" => Ok(Category::ErrorHandling),
//                         "exploration" => Ok(Category::Exploration),
//                         "macros" => Ok(Category::Macros),
//                         "math" => Ok(Category::Math),
//                         "proc_macros" => Ok(Category::ProcMacros),
//                         "prototype" => Ok(Category::Prototype),
//                         "recreational" => Ok(Category::Recreational),
//                         "reference" => Ok(Category::Reference),
//                         "technique" => Ok(Category::Technique),
//                         "testing" => Ok(Category::Testing),
//                         "tools" => Ok(Category::Tools),
//                         "type_identification" => Ok(Category::TypeIdentification),
//                         _ => Err(format!("Invalid category: {}", s)),
//                     }
//                 }
//             }

//             /// Returns a vector of all valid category names as strings.
//             ///
//             /// This function is automatically generated by the `category_enum` macro and provides
//             /// a complete list of categories, making it convenient for validation, UI prompts, or filtering.
//             ///
//             /// # Example
//             ///
//             /// ```rust
//             /// use demo_proc_macros::category_enum;
//             ///
//             /// category_enum! {
//             ///     AST, CLI, REPL, async, basic, big_numbers, crates, educational,
//             ///     error_handling, exploration, macros, math, proc_macros, prototype,
//             ///     recreational, reference, technique, testing, tools, type_identification
//             /// }
//             ///
//             /// let categories = Category::all_categories();
//             /// assert_eq!(categories, vec![
//             ///     "AST", "CLI", "REPL", "Async", "Basic", "BigNumbers", "Crates",
//             ///     "Educational", "ErrorHandling", "Exploration", "Macros", "Math",
//             ///     "ProcMacros", "Prototype", "Recreational", "Reference", "Technique",
//             ///     "Testing", "Tools", "TypeIdentification"
//             /// ]);
//             /// ```
//             pub fn all_categories() -> Vec<String> {
//                 vec![
//                     "AST".to_string(),
//                     "CLI".to_string(),
//                     "REPL".to_string(),
//                     "async".to_string(),
//                     "basic".to_string(),
//                     "big_numbers".to_string(),
//                     "crates".to_string(),
//                     "educational".to_string(),
//                     "error_handling".to_string(),
//                     "exploration".to_string(),
//                     "macros".to_string(),
//                     "math".to_string(),
//                     "proc_macros".to_string(),
//                     "prototype".to_string(),
//                     "recreational".to_string(),
//                     "reference".to_string(),
//                     "technique".to_string(),
//                     "testing".to_string(),
//                     "tools".to_string(),
//                     "type_identification".to_string(),
//                 ]
//             }
//         };
//         TokenStream::from(expanded)
//     }
// }
