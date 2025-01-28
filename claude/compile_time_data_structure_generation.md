One of the tricks that's worked really well has been when you wrote the proc macro called `generate_theme_types` which pre-loads all available built-in colour theme signatures from toml files into a hashmap at compile time and stores it as a static variable.

I know that this one relies on logic in build.rs to pre-load the theme contents into another phf Map, but I've been wondering a. if this is strictly necessary, and more importantly, b. to what extent we could generalise this technique to provide a proc macro or if necessarily a family of them, to pre-load common data structure types at compile time for the benefit of the user. I'm looking for something pretty generic for loading, say, maps, dictionaries or lookup tables at compile time as a performance benefit for the user.

This is a great question about compile-time data structure generation! Let's explore how we could create a more generic solution.

Here's an approach for a generic compile-time data structure loader:

```rust
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr};
use std::collections::HashMap;
use serde::Deserialize;

#[proc_macro]
pub fn load_static_map(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let path = input.value();

    // Read and parse the data at compile time
    let content = std::fs::read_to_string(&path)
        .expect("Failed to read file");

    // Example for TOML, but could be made generic
    let data: HashMap<String, Value> = toml::from_str(&content)
        .expect("Failed to parse TOML");

    // Generate the static map
    let entries = data.iter().map(|(k, v)| {
        let key = k.to_string();
        let value = generate_value(v);
        quote! { (#key, #value) }
    });

    quote! {
        static MAP: phf::Map<&'static str, Value> = phf::phf_map! {
            #(#entries),*
        };
    }
}

// More generic version supporting different formats and types
#[proc_macro]
pub fn load_static_data<T: Deserialize>(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as DataLoaderConfig);

    match config.format {
        Format::Toml => generate_from_toml::<T>(&config.path),
        Format::Json => generate_from_json::<T>(&config.path),
        Format::Yaml => generate_from_yaml::<T>(&config.path),
    }
}

// Usage example:
#[derive(Deserialize)]
struct MyConfig {
    name: String,
    values: HashMap<String, i32>,
}

load_static_data! {
    type: MyConfig,
    path: "config.toml",
    format: toml
}
```

Here's a more complete example showing how to make it generic over different data structures:

```rust
use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;

pub enum StaticContainer {
    Map,
    Vec,
    Set,
}

#[proc_macro]
pub fn load_static_container(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as ContainerConfig);

    match config.container_type {
        StaticContainer::Map => generate_static_map(&config),
        StaticContainer::Vec => generate_static_vec(&config),
        StaticContainer::Set => generate_static_set(&config),
    }
}

fn generate_static_map(config: &ContainerConfig) -> TokenStream2 {
    quote! {
        static MAP: phf::Map<&'static str, Value> = phf::phf_map! {
            // Generated key-value pairs
        };
    }
}

// Example usage:
load_static_container! {
    container: Map,
    key_type: String,
    value_type: i32,
    source: "data.toml"
}
```

And here's how we could make it work with different file types and validation:

```rust
#[proc_macro]
pub fn load_validated_data(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as ValidatedDataConfig);

    // Validate the data at compile time
    let data = load_and_validate(&config);

    // Generate the appropriate static container
    match config.container_type {
        ContainerType::Map => quote! {
            static DATA: phf::Map<_, _> = phf::phf_map! {
                #(#data)*
            };
        },
        ContainerType::Array => quote! {
            static DATA: &[_] = &[
                #(#data),*
            ];
        },
    }
}

// Helper traits for validation
pub trait CompileTimeValidate {
    fn validate(&self) -> Result<(), String>;
}
```

Some key features to consider implementing:

1. Type Safety:
```rust
// Ensure type safety at compile time
pub trait StaticLoadable: Deserialize + CompileTimeValidate {
    type Key;
    type Value;
}
```

2. Custom Validation:
```rust
#[proc_macro_attribute]
pub fn validate_static_data(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Add compile-time validation rules
}
```

3. Error Handling:
```rust
fn compile_error(msg: &str) -> TokenStream2 {
    quote! {
        compile_error!(#msg);
    }
}
```

To use this system, you might write something like:

```rust
#[derive(Deserialize)]
struct Config {
    settings: HashMap<String, Value>,
}

load_static_data! {
    type: Config,
    source: "config.toml",
    container: Map,
    validate: true
}
```

This approach offers several advantages:
1. No build.rs dependency required
2. Type-safe at compile time
3. Flexible for different data structures
4. Support for validation
5. Better error messages at compile time

The main limitations to consider:
1. Compile time increases with data size
2. Memory usage during compilation
3. Limited to static data

Remember to handle:
- File paths relative to the project
- Different serialization formats
- Type conversion and validation
- Error messages
- Documentation and examples

This system could be packaged as a separate crate for reuse across projects.

Thanks, that looks great. I'll come back to it.

A few additional thoughts that might be helpful when you do come back to this:

1. Consider using the `const fn` feature for any helper functions where possible, as this can help with compile-time evaluation.

2. The `phf` crate is great for static maps, but depending on your use case, you might also want to look at `rustc_hash` or other alternatives for different performance characteristics.

3. Don't forget about the `include_str!` and `include_bytes!` macros - they can be very useful for this kind of compile-time data loading.

Let me know if you need more specific details when you return to this topic!
