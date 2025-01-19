#![allow(clippy::missing_panics_doc)]
mod ansi_code_derive;
mod category_enum;
// mod generate_theme_signatures;
mod palette_methods;
mod repeat_dash;

use crate::ansi_code_derive::ansi_code_derive_impl;
use crate::category_enum::category_enum_impl;
// use crate::generate_theme_signatures::generate_theme_signatures_impl;
use crate::palette_methods::palette_methods_impl;
use crate::repeat_dash::repeat_dash_impl;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_file, parse_macro_input, ItemFn};

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
    let should_expand = input.clone().into_iter().any(|token| {
        // Very basic check - you might want something more robust
        token.to_string().contains("expand_macro")
    });

    intercept_and_debug(should_expand, &input, category_enum_impl)
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
    intercept_and_debug(false, &input, repeat_dash_impl)
}

fn intercept_and_debug<F>(expand: bool, input: &TokenStream, proc_macro: F) -> TokenStream
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

#[proc_macro_attribute]
pub fn profile(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let body = &input.block;

    quote! {
        #(#attrs)*
        #vis #sig {
            let _profile = ::thag_rs::Profile::new(concat!(
                module_path!(), "::",
                stringify!(#fn_name)
            ));
            #body
        }
    }
    .into()
}

#[proc_macro_derive(PaletteMethods)]
pub fn palette_methods(input: TokenStream) -> TokenStream {
    // Parse the input to check for the `expand_macro` attribute
    let should_expand = input.clone().into_iter().any(|token| {
        // Very basic check - you might want something more robust
        token.to_string().contains("expand_macro")
    });

    intercept_and_debug(should_expand, &input, palette_methods_impl)
}

#[proc_macro_derive(AnsiCodeDerive, attributes(ansi_name))]
pub fn ansi_code_derive(input: TokenStream) -> TokenStream {
    // Parse the input to check for the `expand_macro` attribute
    let should_expand = input.clone().into_iter().any(|token| {
        // Very basic check - you might want something more robust
        token.to_string().contains("expand_macro")
    });

    intercept_and_debug(should_expand, &input, ansi_code_derive_impl)
}

use proc_macro2::Span;
use std::fs;
use syn::Ident;

#[proc_macro]
pub fn generate_theme_types(input: TokenStream) -> TokenStream {
    // generate_theme_types_impl()
    intercept_and_debug(false, &input, generate_theme_types_impl)
}

fn generate_theme_types_impl(_input: TokenStream) -> TokenStream {
    // First, collect theme signatures from TOML files
    let mut theme_entries = Vec::new();

    if let Ok(entries) = fs::read_dir("themes/built_in") {
        for entry in entries.filter_map(Result::ok) {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(theme) = content.parse::<toml::Value>() {
                        let name = entry
                            .path()
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        eprintln!("name={name}");
                        if let Some(bg) = theme.get("background").and_then(|v| v.as_str()) {
                            eprintln!("bg={bg}");
                            if let Some((r, g, b)) = hex_to_rgb(bg) {
                                eprintln!("rbg={r}, {g}, {b}");
                                let term_bg_luma = to_upper_camel_case(
                                    theme
                                        .get("term_bg_luma")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("dark"),
                                );

                                let min_color_support = to_upper_camel_case(
                                    theme
                                        .get("min_color_support")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("basic"),
                                );

                                let term_bg_luma = Ident::new(&term_bg_luma, Span::call_site());
                                let min_color_support =
                                    Ident::new(&min_color_support, Span::call_site());

                                theme_entries.push(quote! {
                                    m.insert(
                                        #name.to_string(),
                                        ThemeSignature {
                                            bg_rgb: (#r, #g, #b),
                                            term_bg_luma: TermBgLuma::#term_bg_luma,
                                            min_color_support: ColorSupport::#min_color_support,
                                        }
                                    );
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Generate both structures and their implementations
    let expanded = quote! {
        /// Theme definition loaded from TOML files
        #[derive(Debug, Clone, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub struct ThemeDefinition {
            name: String,
            #[serde(skip)]
            pub filename: PathBuf, // e.g., "themes/built_in/dracula.toml"
            #[serde(skip)]
            pub is_builtin: bool, // true for built-in themes, false for custom    pub term_bg_luma: TermBgLuma,
            /// Light or dark background requirement
            pub term_bg_luma: String,
            /// Minimum color support required
            pub min_color_support: String,
            /// Theme background color in hex format
            pub background: Option<String>,
            /// Theme description
            pub description: String,
            /// Color palette configuration
            pub palette: PaletteConfig,
        }

        impl ThemeDefinition {
            /// Get the background luminance requirement
            pub fn term_bg_luma(&self) -> &str {
                &self.term_bg_luma
            }

            /// Get the minimum color support requirement
            pub fn min_color_support(&self) -> &str {
                &self.min_color_support
            }

            /// Get the background color if specified
            pub fn background(&self) -> Option<&str> {
                self.background.as_deref()
            }
        }

        /// Runtime theme signature for matching
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct ThemeSignature {
            /// RGB values of theme background
            pub bg_rgb: (u8, u8, u8),
            /// Light or dark background requirement
            pub term_bg_luma: TermBgLuma,
            /// Minimum color support required
            pub min_color_support: ColorSupport,
        }

        impl ThemeSignature {
            /// Get signatures for all built-in themes
            pub fn get_signatures() -> ::std::collections::HashMap<String, ThemeSignature> {
                let mut m = ::std::collections::HashMap::new();
                #(#theme_entries)*
                m
            }

            // /// Calculate color distance for theme matching
            // pub fn color_distance(&self, other_rgb: (u8, u8, u8)) -> f32 {
            //     let (r1, g1, b1) = self.bg_rgb;
            //     let (r2, g2, b2) = other_rgb;
            //     let dr = (r1 as f32 - r2 as f32) * 0.30;
            //     let dg = (g1 as f32 - g2 as f32) * 0.59;
            //     let db = (b1 as f32 - b2 as f32) * 0.11;
            //     (dr * dr + dg * dg + db * db).sqrt()
            // }
        }

        // // Initialize the static theme signatures
        // lazy_static::lazy_static! {
        //     pub static ref THEME_SIGNATURES: ::std::collections::HashMap<String, ThemeSignature> =
        //         ThemeSignature::get_signatures();
        // }

        // Use OnceLock instead of lazy_static
        pub static THEME_SIGNATURES: ::std::sync::OnceLock<::std::collections::HashMap<String, ThemeSignature>> =
            ::std::sync::OnceLock::new();

        // Helper function to get or initialize signatures
        pub fn get_theme_signatures() -> &'static ::std::collections::HashMap<String, ThemeSignature> {
            THEME_SIGNATURES.get_or_init(ThemeSignature::get_signatures)
        }

    };

    TokenStream::from(expanded)
}

fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

fn to_upper_camel_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.extend(c.to_lowercase());
        }
    }

    result
}
