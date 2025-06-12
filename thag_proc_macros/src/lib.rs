#![allow(clippy::missing_panics_doc)]
mod ansi_code_derive;
mod category_enum;
mod file_navigator;
mod fn_name;
mod generate_theme_types;
mod palette_methods;
mod preload_themes;
mod repeat_dash;
mod tool_errors;

#[cfg(feature = "full_profiling")]
mod safe_alloc;

#[cfg(feature = "tui")]
mod tui_keys;

#[cfg(feature = "time_profiling")]
mod enable_profiling;

#[cfg(feature = "time_profiling")]
mod profiled;

#[cfg(feature = "time_profiling")]
mod profile;

#[cfg(feature = "time_profiling")]
mod end;

use crate::ansi_code_derive::ansi_code_derive_impl;
use crate::category_enum::category_enum_impl;
use crate::file_navigator::file_navigator_impl;
use crate::fn_name::fn_name_impl;
use crate::generate_theme_types::generate_theme_types_impl;
use crate::palette_methods::palette_methods_impl;
use crate::preload_themes::preload_themes_impl;
use crate::repeat_dash::repeat_dash_impl;
use crate::tool_errors::tool_errors_impl;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_file, parse_str, Expr};

#[cfg(feature = "full_profiling")]
use crate::safe_alloc::safe_alloc_impl;

#[cfg(feature = "time_profiling")]
use crate::enable_profiling::enable_profiling_impl;

#[cfg(feature = "time_profiling")]
use crate::profiled::profiled_impl;

#[cfg(feature = "time_profiling")]
use crate::profile::profile_impl;

#[cfg(feature = "time_profiling")]
use crate::end::end_impl;

#[cfg(feature = "tui")]
use crate::tui_keys::key_impl;

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
    // Parse the input to check for the `expand_macro` attribute
    // let should_expand = input.clone().into_iter().any(|token| {
    //     // Very basic check - you might want something more robust
    //     token.to_string().contains("expand_macro")
    // });

    maybe_expand_proc_macro(false, "category_enum", &input, category_enum_impl)
}

// Not public API. This is internal and to be used only by `key!`.
#[cfg(feature = "tui")]
#[doc(hidden)]
#[proc_macro]
pub fn key(input: TokenStream) -> TokenStream {
    key_impl(input)
}

/// Generates a constant `DASH_LINE` consisting of a dash (hyphen) repeated the number of times specified by the integer literal argument `n`.
///
/// Syntax:
///
/// ```Rust
///     repeat_dash!(<n>);
/// ```
///
/// E.g.:
///
/// ```Rust
/// repeat_dash!(70);
/// cvprtln!(Lvl::EMPH, V::Q, "{DASH_LINE}");
/// ```
///
#[proc_macro]
pub fn repeat_dash(input: TokenStream) -> TokenStream {
    // repeat_dash_impl(input)
    maybe_expand_proc_macro(false, "repeat_dash", &input, repeat_dash_impl)
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

/// Generates repetitive methods for all 14 `Style` fields of the `Palette` struct
/// instead of hand-coding them.
#[proc_macro_derive(PaletteMethods)]
pub fn palette_methods(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "palette_methods", &input, palette_methods_impl)
}

#[proc_macro_derive(AnsiCodeDerive, attributes(ansi_name))]
pub fn ansi_code_derive(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "ansi_code_derive", &input, ansi_code_derive_impl)
}

#[proc_macro]
pub fn file_navigator(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "file_navigator", &input, file_navigator_impl)
}

#[proc_macro]
pub fn generate_theme_types(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(
        false,
        "generate_theme_types",
        &input,
        generate_theme_types_impl,
    )
}

#[proc_macro]
pub fn preload_themes(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "preload_themes", &input, preload_themes_impl)
}

#[proc_macro]
pub fn tool_errors(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "tool_errors", &input, tool_errors_impl)
}

#[proc_macro_attribute]
pub fn fn_name(attr: TokenStream, item: TokenStream) -> TokenStream {
    maybe_expand_attr_macro(false, "fn_name", &attr, &item, fn_name_impl)
}

#[proc_macro_attribute]
#[cfg(feature = "time_profiling")]
pub fn enable_profiling(attr: TokenStream, item: TokenStream) -> TokenStream {
    maybe_expand_attr_macro(
        false,
        "enable_profiling",
        &attr,
        &item,
        enable_profiling_impl,
    )
}

#[cfg(not(feature = "time_profiling"))]
#[proc_macro_attribute]
pub fn enable_profiling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[cfg(feature = "time_profiling")]
#[proc_macro_attribute]
pub fn profiled(attr: TokenStream, item: TokenStream) -> TokenStream {
    // eprintln!("DEBUGLIB: profiled attribute macro called");
    // Set to true to enable macro expansion output
    maybe_expand_attr_macro(false, "profiled", &attr, &item, profiled_impl)
}

#[cfg(not(feature = "time_profiling"))]
#[proc_macro_attribute]
pub fn profiled(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Creates a function with the name specified in the string literal
/// that returns the line number where the function is called.
///
/// # Example
///
/// ```
/// use thag_profiler::end;
///
/// // Intended for use with `profile!(my_section, detailed_memory)`,
/// // so `profile!` can get section end line number
/// println!("Current line: {}", end_my_section()); // prints the `end!` line number
///
/// end!(my_section);
///
/// ```
#[cfg(feature = "time_profiling")]
#[proc_macro]
pub fn end(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "end", &input, end_impl)
}

#[cfg(not(feature = "time_profiling"))]
#[proc_macro]
pub fn end(_input: TokenStream) -> TokenStream {
    // Return an empty token stream to make this a no-op
    TokenStream::new()
}

#[cfg(feature = "time_profiling")]
#[proc_macro]
pub fn profile(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "profile", &input, profile_impl)
}

#[cfg(not(feature = "time_profiling"))]
#[proc_macro]
pub fn profile(_input: TokenStream) -> TokenStream {
    // Return an empty token stream to make this a no-op
    TokenStream::new()
}

#[cfg(not(feature = "full_profiling"))]
#[proc_macro]
pub const fn safe_alloc(input: TokenStream) -> TokenStream {
    input
}

#[cfg(feature = "full_profiling")]
#[proc_macro]
pub fn safe_alloc(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "safe_alloc", &input, safe_alloc_impl)
}
