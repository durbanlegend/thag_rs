#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::HashMap;
use syn::Ident;

#[allow(clippy::too_many_lines)]
pub fn preload_themes_impl(_input: TokenStream) -> TokenStream {
    let themes_dir = "themes/built_in";
    let mut theme_indices = Vec::new();
    let mut bg_to_names = HashMap::new();

    for entry in std::fs::read_dir(themes_dir).unwrap() {
        let path = entry.unwrap().path();
        // Skip hidden files like .DS_Store and read only .toml files
        if path.file_name().and_then(|n| n.to_str()).is_none_or(|n| {
            n.starts_with('.')
                || !std::path::Path::new(n)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
        }) {
            continue;
        }

        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Error reading {}", path.display()));
        let value: toml::Value = toml::from_str(&content)
            .unwrap_or_else(|_| panic!("Bad toml at path {}", path.display()));
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();

        // // Only process true_color themes
        // let min_color_support = value
        //     .get("min_color_support")
        //     .and_then(|v| v.as_str())
        //     .unwrap_or("basic");

        // if min_color_support != "true_color" {
        //     continue; // Skip non-true_color themes
        // }

        if let Some(bg_array) = value.get("backgrounds").and_then(|v| v.as_array()) {
            let backgrounds: Vec<_> = bg_array
                .iter()
                .filter_map(|v| v.as_str())
                .filter_map(hex_to_rgb)
                .collect();

            let bg_rgbs = backgrounds.iter().map(|(r, g, b)| {
                quote! { (#r, #g, #b) }
            });

            let term_bg_luma = to_upper_camel_case(
                value
                    .get("term_bg_luma")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dark"),
            );

            // let min_color_support = to_upper_camel_case(
            //     value
            //         .get("min_color_support")
            //         .and_then(|v| v.as_str())
            //         .unwrap_or("basic"),
            // );

            let term_bg_luma_ident = Ident::new(&term_bg_luma, Span::call_site());
            // let min_color_support_ident = Ident::new(&min_color_support, Span::call_site());

            let theme_index = quote! {
                #name => ThemeIndex {
                    name: #name,
                    bg_rgbs: &[#(#bg_rgbs),*],
                    term_bg_luma: TermBgLuma::#term_bg_luma_ident,
                    min_color_support: ColorSupport::TrueColor,  // Will generate 256 if needed at run time
                    content: #content,
                }
            };
            theme_indices.push(theme_index);

            // Build bg->names mapping
            for bg_rgb in backgrounds {
                bg_to_names
                    .entry(bg_rgb)
                    .or_insert_with(Vec::new)
                    .push(name.clone());
            }
        }
    }

    // eprintln!("Building index by background...");

    let bg_lookup_entries = bg_to_names.iter().map(|((r, g, b), names)| {
        let hex = format!("{r:02x}{g:02x}{b:02x}");
        quote! {
            #hex => &[#(#names),*]
        }
    });

    // eprintln!("Done!");

    quote! {
        #[derive(Debug)]
        pub struct ThemeIndex {
            pub name: &'static str,
            pub bg_rgbs: &'static [(u8, u8, u8)],
            pub term_bg_luma: TermBgLuma,
            pub min_color_support: ColorSupport,
            pub content: &'static str,
        }

        impl ThemeIndex {
            fn matches_background(&self, bg: (u8, u8, u8)) -> bool {
                // eprintln!("bg={bg:?}, self.bg_rgbs={:?}", self.bg_rgbs);
                self.bg_rgbs.iter().any(|&theme_bg| {
                    bg == theme_bg
                })
            }

            // New method to get theme with specific color support
            fn get_theme_with_color_support(&self, color_support: ColorSupport) -> Theme {
                let mut theme = Theme::get_builtin(self.name).expect("Could not get theme");
                if color_support != ColorSupport::TrueColor {
                    theme.convert_to_color_support(color_support);
                }
                theme
            }
        }

        static THEME_INDEX: phf::Map<&'static str, ThemeIndex> = phf::phf_map! {
            #(#theme_indices),*
        };

        static BG_LOOKUP: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
            #(#bg_lookup_entries),*
        };

        #[must_use]
        pub fn rgb_to_hex((r, g, b): &(u8, u8, u8)) -> String {
            format!("#{r:02x}{g:02x}{b:02x}")
        }

        #[must_use]
        pub fn rgb_to_bare_hex((r, g, b): &(u8, u8, u8)) -> String {
            format!("{r:02x}{g:02x}{b:02x}")
        }

    }
    .into()
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
