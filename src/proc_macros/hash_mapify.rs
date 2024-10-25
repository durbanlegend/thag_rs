#![allow(dead_code)]
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Lit, LitStr, Token, Type};

pub struct ParsedMapEntry(String, proc_macro2::TokenStream);

pub struct ParsedMap {
    value_type: Type,
    entries: Vec<ParsedMapEntry>,
}

impl ToTokens for ParsedMapEntry {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let key = self.0.clone();
        let value = self.1.clone();

        tokens.extend(quote!(String::from(#key), #value));
    }
}

impl Parse for ParsedMap {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut entries = Vec::<ParsedMapEntry>::new();

        // Check if input is empty (no arguments are passed). If not, then
        // panic as we cannot continue further.
        assert!(
            !input.is_empty(),
            "At least a type must be specified for an empty hashmap"
        );

        // Since the first argument should be of type `Type`, you try
        // to parse `Type` out of input and returns an error otherwise.
        let ty = input.parse::<Type>()?;

        // Next, parse the `,` token, which you expect to be used to
        // separate the arguments.
        input.parse::<Token![,]>()?;

        // Loop until the input is empty (there is nothing else
        // left to parse).
        while !input.is_empty() {
            // Try to parse the key as an identifier
            let key = if let Ok(key) = input.parse::<syn::Ident>() {
                key.to_string()
                // If it's not an identifier, try to parse it as
                // a string literal
            } else if let Ok(key) = input.parse::<LitStr>() {
                key.value()
                // If it's neither an identifier nor a string literal,
                // it is not a valid key, so panic with appropriate
                // error.
            } else {
                panic!("Key must be either a string literal or an identifier!");
            };

            // Parse the `=` sign, which should be the next token after
            // a key.
            input.parse::<Token![=]>()?;

            // Next, try to parse the value as an identifier. If it is, it
            // means that it's a variable, so we should convert it to token
            // stream directly.
            let value = if let Ok(value) = input.parse::<syn::Ident>() {
                value.to_token_stream()
                // If the input isn't an identifier, try to parse it as a
                // literal value such as `"string"` for strings, `42`
                // for numbers `false` for boolean value, etc.
            } else if let Ok(value) = input.parse::<Lit>() {
                value.to_token_stream()
            } else {
                // If the input is neither an identifier nor a literal value
                // panic with appropriate error.
                panic!("Value must be either a literal or an identifier!");
            };

            // Push the parsed key value pair to our list.
            entries.push(ParsedMapEntry(key, value));

            // Check if next token is a comma, without advancing the stream
            if input.peek(Token![,]) {
                // If it is, then parse it out and advance the stream before
                // moving on to the next key-value pair
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ParsedMap {
            value_type: ty,
            entries,
        })
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn hash_mapify_impl(item: TokenStream) -> TokenStream {
    // Parse input token stream as `ParsedMap` defined by us.
    // This will use the logic from parse trait we implemented
    // earlier.
    let input = parse_macro_input!(item as ParsedMap);

    let key_value_pairs = input.entries;
    let ty = input.value_type;

    // Generate the output hashmap inside a code block so that
    // we don't shadow any existing variables. Return the hashmap
    // from the block.
    quote!({
        // Create a new hashmap with `String` for key type and `#ty` for
        // value type, which parsed from the macro input arguments.
        let mut hash_map = std::collections::HashMap::<String, #ty>::new();

        // Insert all key-value pairs into the hashmap.
        #(
            hash_map.insert(#key_value_pairs);
        )*

        // Return the generated hashmap
        hash_map
    })
    .into()
}
