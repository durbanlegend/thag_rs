#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::fs;
use syn::Ident;

pub fn generate_theme_types_impl(_input: TokenStream) -> TokenStream {
    // First, collect theme signatures from TOML files
    let mut theme_entries = Vec::new();

    if let Ok(entries) = fs::read_dir("themes/built_in") {
        for entry in entries.filter_map(Result::ok) {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("toml") {
                // eprintln!("entry.path()={}", entry.path().display());
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // eprintln!("... content read");
                    if let Ok(theme) = content.parse::<toml::Value>() {
                        let name = entry
                            .path()
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        // eprintln!("name={name}");
                        if let Some(bg) = theme.get("background").and_then(|v| v.as_str()) {
                            // eprintln!("bg={bg}");
                            if let Some((r, g, b)) = hex_to_rgb(bg) {
                                // eprintln!("rbg={r}, {g}, {b}");
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
