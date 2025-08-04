#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, Token,
};

pub(crate) struct StyleArgs {
    pub(crate) styles: Vec<StyleEntry>,
    pub(crate) expr: Expr,
}

pub(crate) enum StyleEntry {
    Flag(Ident),            // e.g. bold
    KeyValue(Ident, Ident), // e.g. fg = Red
}

impl Parse for StyleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut styles = Vec::new();

        while !input.peek(Token![=>]) {
            if input.peek(Ident) && input.peek2(Token![=]) {
                let key: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let val: Ident = input.parse()?;
                styles.push(StyleEntry::KeyValue(key, val));
            } else if input.peek(Ident) {
                let flag: Ident = input.parse()?;
                styles.push(StyleEntry::Flag(flag));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        input.parse::<Token![=>]>()?;
        let expr: Expr = input.parse()?;

        Ok(StyleArgs { styles, expr })
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
