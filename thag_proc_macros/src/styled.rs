#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, Token,
};

struct StyleArgs {
    styles: Vec<StyleEntry>,
    expr: Expr,
}

enum StyleEntry {
    Flag(Ident),            // e.g. bold
    KeyValue(Ident, Ident), // e.g. fg = Red
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
                let val: Ident = input.parse()?;
                styles.push(StyleEntry::KeyValue(key, val));
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
            StyleEntry::KeyValue(key, val) => {
                expr_tokens = quote! { #expr_tokens.#key(Color::#val) };
            }
        }
    }

    TokenStream::from(quote! { #expr_tokens })
}
