#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::fs;
use syn::Ident;

#[allow(clippy::too_many_lines)]
pub fn generate_theme_types_impl(_input: TokenStream) -> TokenStream {
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

                        // Only handle backgrounds array
                        if let Some(bg_array) = theme.get("backgrounds").and_then(|v| v.as_array())
                        {
                            let backgrounds: Vec<_> = bg_array
                                .iter()
                                .filter_map(|v| v.as_str())
                                .filter_map(hex_to_rgb)
                                .collect();

                            if let Some(first_bg) = backgrounds.first() {
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

                                let (r, g, b) = *first_bg;
                                let bg_entries = backgrounds.iter().map(|(r, g, b)| {
                                    quote! { (#r, #g, #b) }
                                });

                                theme_entries.push(quote! {
                                    m.insert(
                                        #name.to_string(),
                                        ThemeSignature {
                                            bg_rgb: (#r, #g, #b),
                                            bg_rgbs: vec![#(#bg_entries),*],
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
        /// Runtime theme signature for matching
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct ThemeSignature {
            /// RGB values of primary theme background (first in list)
            pub bg_rgb: (u8, u8, u8),
            /// All possible RGB values for theme background
            pub bg_rgbs: Vec<(u8, u8, u8)>,
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

            /// Check if a given background color exactly matches any of the theme's backgrounds
            fn matches_background(&self, bg: (u8, u8, u8)) -> bool {
                self.bg_rgbs.iter().any(|&theme_bg| {
                    bg == theme_bg
                })
            }

            /// Check if a given background color matches any of the theme's backgrounds within the threshold
            pub fn bg_within_threshold(&self, bg: (u8, u8, u8)) -> bool {
                self.bg_rgbs.iter().any(|&theme_bg| {
                    color_distance(bg, theme_bg) < THRESHOLD
                })
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
