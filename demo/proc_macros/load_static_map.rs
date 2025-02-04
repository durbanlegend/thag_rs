use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{parse_macro_input, LitStr};
use toml::Value;

pub fn load_static_map_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);

    let relative_path = input.value();

    eprintln!("env!(CARGO_MANIFEST_DIR)={}", env!("CARGO_MANIFEST_DIR"));

    // Construct absolute path from project root. The manifest dir will be that of the proc macros crate.
    // We assume the project dir is two steps up from the proc macros dir.
    let absolute_path = format!("{}/../../{}", env!("CARGO_MANIFEST_DIR"), relative_path);
    eprintln!("absolute_path={absolute_path}");

    // Read and parse the data at compile time
    let content = std::fs::read_to_string(&absolute_path)
        .unwrap_or_else(|e| panic!("Failed to read file '{}': {}", absolute_path, e));

    // Example for TOML, but could be made generic
    let data: HashMap<String, Value> = toml::from_str(&content).expect("Failed to parse TOML");

    // Generate the static map
    let entries = data.iter().map(|(k, v)| {
        let key_str = k.as_str();
        let value = generate_const_value(v);
        quote! { #key_str => #value } // Changed from tuple syntax to => syntax
    });

    quote! {
        static MAP: phf::Map<&'static str, Value> = phf::phf_map! {
            #(#entries),*
        };
    }
    .into()
}

fn generate_const_value(value: &toml::Value) -> proc_macro2::TokenStream {
    match value {
        toml::Value::String(s) => {
            quote! { Value::String(#s) } // Note: Using string literal directly
        }
        toml::Value::Integer(i) => {
            quote! { Value::Integer(#i) }
        }
        toml::Value::Float(f) => {
            quote! { Value::Float(#f) }
        }
        toml::Value::Boolean(b) => {
            quote! { Value::Boolean(#b) }
        }
        toml::Value::Array(arr) => {
            let elements = arr.iter().map(|v| generate_const_value(v));
            quote! { Value::Array(&[#(#elements),*]) } // Note: Using array slice
        }
        toml::Value::Table(table) => {
            let entries = table.iter().map(|(k, v)| {
                let key = k.as_str();
                let value = generate_const_value(v);
                quote! { (#key, #value) }
            });
            quote! {
                Value::Table(&[#(#entries),*])  // Note: Using array slice
            }
        }
        toml::Value::Datetime(dt) => {
            let dt_str = dt.to_string();
            quote! { Value::Datetime(#dt_str) }
        }
    }
}
