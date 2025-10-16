#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, LitInt, LitStr, Token,
};

struct StyleArgs {
    styles: Vec<StyleEntry>,
    expr: Expr,
}

enum StyleEntry {
    Flag(Ident),                // e.g. bold
    KeyValue(Ident, ColorSpec), // e.g. fg = Red, fg = Color256(196), fg = "#ff0000"
}

enum ColorSpec {
    Basic(Ident),                // Red, Green, etc.
    Color256(LitInt),            // Color256(196)
    Rgb(LitInt, LitInt, LitInt), // Rgb(255, 0, 0)
    Hex(LitStr),                 // "#ff0000"
}

impl Parse for StyleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the first argument as the expression
        let expr: Expr = input.parse()?;

        let mut styles = Vec::new();

        // If there's a comma, consume it and continue parsing style entries
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            if input.peek(Ident) && input.peek2(Token![=]) {
                // key = value
                let key: Ident = input.parse()?;
                input.parse::<Token![=]>()?;

                // Parse different color specifications
                let color_spec = if input.peek(Ident) {
                    let ident: Ident = input.parse()?;
                    let ident_str = ident.to_string();

                    if ident_str == "Color256" && input.peek(syn::token::Paren) {
                        // Color256(n)
                        let content;
                        syn::parenthesized!(content in input);
                        let n: LitInt = content.parse()?;
                        ColorSpec::Color256(n)
                    } else if ident_str == "Rgb" && input.peek(syn::token::Paren) {
                        // Rgb(r, g, b)
                        let content;
                        syn::parenthesized!(content in input);
                        let r: LitInt = content.parse()?;
                        content.parse::<Token![,]>()?;
                        let g: LitInt = content.parse()?;
                        content.parse::<Token![,]>()?;
                        let b: LitInt = content.parse()?;
                        ColorSpec::Rgb(r, g, b)
                    } else {
                        // Basic color like Red, Green, etc.
                        ColorSpec::Basic(ident)
                    }
                } else if input.peek(LitStr) {
                    // Hex string like "#ff0000"
                    let hex: LitStr = input.parse()?;
                    ColorSpec::Hex(hex)
                } else {
                    return Err(input.error(
                        "Expected color specification (Red, Color256(n), Rgb(r,g,b), or \"#hex\")",
                    ));
                };

                styles.push(StyleEntry::KeyValue(key, color_spec));
            } else if input.peek(Ident) {
                // single flag
                let flag: Ident = input.parse()?;
                styles.push(StyleEntry::Flag(flag));
            } else {
                return Err(input.error("Expected identifier or key=value style entry"));
            }
        }

        Ok(Self { styles, expr })
    }
}

pub fn styled_impl(input: TokenStream) -> TokenStream {
    let StyleArgs { styles, expr } = parse_macro_input!(input as StyleArgs);

    let mut expr_tokens = quote! { (#expr).style() };

    for style in styles {
        match style {
            StyleEntry::Flag(flag) => {
                expr_tokens = quote! { #expr_tokens.#flag() };
            }
            StyleEntry::KeyValue(key, color_spec) => {
                match color_spec {
                    ColorSpec::Basic(ident) => {
                        expr_tokens = quote! { #expr_tokens.#key(Color::#ident) };
                    }
                    ColorSpec::Color256(n) => {
                        expr_tokens = quote! { #expr_tokens.#key(Color::Color256(#n)) };
                    }
                    ColorSpec::Rgb(r, g, b) => {
                        expr_tokens = quote! { #expr_tokens.#key(Color::Rgb(#r, #g, #b)) };
                    }
                    ColorSpec::Hex(hex_str) => {
                        // Convert hex to RGB at compile time
                        let hex = hex_str.value();
                        let hex = hex.trim_start_matches('#');
                        if hex.len() != 6 {
                            return TokenStream::from(quote! {
                                compile_error!("Hex color must be 6 characters (e.g., \"#ff0000\")")
                            });
                        }

                        // Parse hex to RGB values
                        match (
                            u8::from_str_radix(&hex[0..2], 16),
                            u8::from_str_radix(&hex[2..4], 16),
                            u8::from_str_radix(&hex[4..6], 16),
                        ) {
                            (Ok(r), Ok(g), Ok(b)) => {
                                expr_tokens = quote! { #expr_tokens.#key(Color::Rgb(#r, #g, #b)) };
                            }
                            _ => {
                                return TokenStream::from(quote! {
                                    compile_error!("Invalid hex color format. Use \"#rrggbb\" format.")
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    TokenStream::from(quote! { #expr_tokens })
}
