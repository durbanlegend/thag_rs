#![allow(clippy::missing_panics_doc)]
#![warn(missing_docs)]
//! # `thag_proc_macros`
//!
//! Proc macros for `thag_rs` (`thag`) and `thag_profiler`.
//!
//! ## Features
//!
//! - `time_profiling`:     Enable time-based performance profiling (default)
//! - `full_profiling`:     Enable comprehensive profiling including time and memory usage
//! - `debug_logging`:      Enable debug logging of profiling functions
//! - `no_tls`:             Use atomic rather than the default thread-local storage for allocator tracking
//! - `analyze_tool`:       Include dependencies required only for `thag_profile` binary.
//! - `instrument_tool`:    Include dependencies required only for `thag_instrument` and `thag_uninstrument` binaries.
//!

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

// Remove the proc macro implementations since we're using the runtime approach instead

#[cfg(feature = "time_profiling")]
mod enable_profiling;

#[cfg(feature = "time_profiling")]
mod profiled;

#[cfg(feature = "time_profiling")]
mod profile;

#[cfg(feature = "time_profiling")]
mod end;

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

/// Generates a `Category` enum with predefined variants and utility implementations for use with the
/// `thag_gen_readme` utility to generate a README.md for a directory such as demo/.
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

/// Generates a `FileNavigator` to allow the user to navigate the file system and select files and directories
/// from a command-line interface.
///
/// Syntax:
///
/// ```Rust
///     file_navigator! {}
/// ```
///
#[proc_macro]
pub fn file_navigator(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "file_navigator", &input, file_navigator_impl)
}

#[doc(hidden)]
#[proc_macro]
pub fn generate_theme_types(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(
        false,
        "generate_theme_types",
        &input,
        generate_theme_types_impl,
    )
}

/// Preload visual themes into memory at compile time.
///
/// Syntax:
///
/// ```Rust
///     preload_themes! {}
/// ```
///
#[proc_macro]
pub fn preload_themes(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "preload_themes", &input, preload_themes_impl)
}

/// Define common errors for `thag` tools.
///
/// Syntax:
///
/// ```Rust
///     tool_errors! {}
/// ```
///
#[proc_macro]
pub fn tool_errors(input: TokenStream) -> TokenStream {
    maybe_expand_proc_macro(false, "tool_errors", &input, tool_errors_impl)
}

/// Attribute macro to give a function access to its own name by inserting the statement `let fn_name = <function name>;`.
///
/// Syntax:
///
/// ```Rust
/// #[fn_name]
/// fn my_function() {
///     ...
/// }
/// ```
///
#[proc_macro_attribute]
pub fn fn_name(attr: TokenStream, item: TokenStream) -> TokenStream {
    maybe_expand_attr_macro(false, "fn_name", &attr, &item, fn_name_impl)
}

/// Attribute macro for use with `thag_profiler`. This macro is intended to annotate the user `main`
/// function in order to to enable and control profiling of the user code.
///
/// Zero-cost abstraction: only alters function if feature `time_profiling` is enabled.
///
/// Syntax:
///
/// ```Rust
/// #[enable_profiling]
/// fn main() {
///     ...
/// }
/// ```
///
/// ## Arguments
///
/// | Argument | Description |
/// |----------|-------------|
/// | `time` | Enable time profiling. |
/// | `memory` | Enable memory profiling. |
/// | `both` | Enable time and memory profiling. |
/// | `yes` | (default) Same as "both". |
/// | `no` | Disable memory profiling. |
/// | `runtime` | Control profiling via `THAG_PROFILER` environment variable args at runtime. |
/// | `function(arg1 ...)` | Pass arguments applicable to the current function as per `profiled`: |
///
/// ### Function Arguments
///
/// | Argument | Description |
/// |----------|-------------|
/// | `time` | Enable time/performance profiling for this function. |
/// | `mem_summary` | Enable basic memory profiling for this function. |
/// | `mem_detail` | Enable detailed memory profiling for this function. |
/// | `both` | Enable time and basic memory profiling for this function. |
/// | `global` | Enable profiling for this function according to the global setting. |
/// | `test` | Enable clone of profile for test access. |
///
/// E.g.:
///
/// ```Rust
/// #[enable_profiling(runtime)]
/// fn main() {
///     ...
/// }
/// ```
///
#[proc_macro_attribute]
#[allow(unused_variables)]
pub fn enable_profiling(attr: TokenStream, item: TokenStream) -> TokenStream {
    #[cfg(feature = "time_profiling")]
    {
        maybe_expand_attr_macro(
            false,
            "enable_profiling",
            &attr,
            &item,
            enable_profiling_impl,
        )
    }

    #[cfg(not(feature = "time_profiling"))]
    {
        item
    }
}

/// Attribute macro for use with `thag_profiler`. This macro is intended to annotate user
/// functions other than `main` in order to to control profiling of each function
/// individually.
///
/// Zero-cost abstraction: only alters function if feature `time_profiling` is enabled.
///
/// Syntax:
///
/// ```Rust
/// #[profiled]
/// fn my_function() {
///     ...
/// }
/// ```
///
/// ## Arguments
///
/// | Argument | Description |
/// |----------|-------------|
/// | `time` | Enable time/performance profiling for this function. |
/// | `mem_summary` | Enable basic memory profiling for this function. |
/// | `mem_detail` | Enable detailed memory profiling for this function. |
/// | `both` | Enable time and basic memory profiling for this function. |
/// | `global` | Enable profiling for this function according to the global setting. |
/// | `test` | Enable clone of profile for test access. |
///
/// E.g.:
///
/// ```Rust
/// #[profiled(both)]
/// fn my_function() {
///     ...
/// }
/// ```
///
#[proc_macro_attribute]
#[allow(unused_variables)]
pub fn profiled(attr: TokenStream, item: TokenStream) -> TokenStream {
    #[cfg(feature = "time_profiling")]
    {
        maybe_expand_attr_macro(false, "profiled", &attr, &item, profiled_impl)
    }

    #[cfg(not(feature = "time_profiling"))]
    {
        item
    }
}

/// Proc macro for use with `thag_profiler`. This macro defines the end of the scope of a `profile!`
/// macro with the same name argument, and also explicitly drops the profile in ring-fenced profiler code.
///
/// Creates a function `end_<name>` that returns its own starting line number for the `profile!`
/// macro to call to determine where its own scope ends.
///
/// Zero-cost abstraction: no-op unless `time_profiling` is enabled.
///
/// Syntax:
///
/// ```Rust
/// end!(name);
/// ```
/// where `name` is the same as the `name` argument to the preceding `profile!(name)` macro.
///
/// # Example
///
/// ```Rust
/// use thag_profiler::{end, profile};
///
/// profile!(my_section, mem_summary);
/// // User code section
/// ...
/// // Show off the `end_<name>` function generated for internal use by `profile!`
/// println!("This section ends on line: {}", end_my_section()); // prints the `end!` line number
/// ...
/// end!(my_section);
///
/// ```
#[proc_macro]
#[allow(unused_variables)]
pub fn end(input: TokenStream) -> TokenStream {
    #[cfg(feature = "time_profiling")]
    {
        maybe_expand_proc_macro(false, "end", &input, end_impl)
    }

    #[cfg(not(feature = "time_profiling"))]
    {
        // Return an empty token stream to make this a no-op
        TokenStream::new()
    }
}

/// Proc macro for use with `thag_profiler`. This macro profiles a section of user code between itself
/// and an optional `end!` macro with a matching name argument.
///
/// If no matching `end!` macro is provided, the `unbounded` flag is required, to confirm that the
/// section ends at the very end of the function. It is the user's responsibility to ensure that
/// the `profile!` macro is scoped to the very end of the function and not within any inner block,
/// as the `Profile` it generates will be implicitly dropped when it goes out of scope, and if this
/// is before the end of the function it will cause any profiling data from that point to the end
/// of the function to be lost.
///
/// Since `unbounded` means the profile is unavoidably dropped implicitly in user code, `end!`
/// is preferred in order to guarantee ring fencing of the profiler code.
///
/// `profile!` scopes must not be nested or overlapped.
///
/// Section profiles as provided by this macro are of limited usefulness, as they may have parent
/// functions but not child functions in the callstack, since they are grafted on to the backtrace
/// mechanism that produces the callstack. A function called from within a profiled section will
/// appear in flamegraphs, not as a child of the section but as a child of the parent function and
/// a sibling of the section.
///
/// Zero-cost abstraction: no-op unless `time_profiling` is enabled.
///
/// Syntax:
///
/// ```Rust
/// profile!(name[, flag1[, flag2[, ...]]]);
/// ```
///
/// | Flag | Description |
/// |------|-------------|
/// | `time` | Enable time profiling for this section |
/// | `mem_summary` | Enable basic memory allocation tracking |
/// | `mem_detail` | Enable detailed memory allocation tracking |
/// | `async_fn` | Mark that this profile is for an async function |
/// | `unbounded` | This is equivalent to an `end!` macro at the end of the function |
///
/// # Example
///
/// ```Rust
/// use thag_profiler::profile;
/// fn my_function() -> Result(()) {
///     ...
///     profile!(my_section, mem_detail, unbounded);
///     // User code section
///     ...
///
///     Ok(())
/// } // my_section ends at end of function
///
/// ```
#[proc_macro]
#[allow(unused_variables)]
pub fn profile(input: TokenStream) -> TokenStream {
    #[cfg(feature = "time_profiling")]
    {
        maybe_expand_proc_macro(false, "profile", &input, profile_impl)
    }

    #[cfg(not(feature = "time_profiling"))]
    {
        // Return an empty token stream to make this a no-op
        TokenStream::new()
    }
}

/// Internal proc macro for use by `thag_profiler` code to ring-fence profiler code.
///
/// If the `full_profiling` feature is enabled, sets TLS / atomic allocator control variable
/// to system allocator in order to ensure as far as possible that any memory allocations
/// in the included code will be handled by the system allocator, and not by the tracking
/// allocator intended for user code profiling.
///
/// Zero-cost abstraction: no change to code unless `full_profiling` is enabled.
///
/// Syntax:
///
/// ```Rust
///     safe_alloc! {
///       // Profiler code
///     }
/// ```
///
#[proc_macro]
pub const fn safe_alloc(input: TokenStream) -> TokenStream {
    #[cfg(feature = "full_profiling")]
    {
        maybe_expand_proc_macro(false, "safe_alloc", &input, safe_alloc_impl)
    }

    #[cfg(not(feature = "full_profiling"))]
    {
        input
    }
}
