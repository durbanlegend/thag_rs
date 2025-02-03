


In an earlier chat you provided this:

"Here's an approach for a generic compile-time data structure loader:

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

"

I'd like to try it. Could you provide the missing fn generate_value?

Here's an example implementation of `generate_value` that handles basic TOML value types:

```rust
fn generate_value(value: &toml::Value) -> proc_macro2::TokenStream {
    match value {
        toml::Value::String(s) => {
            quote! { Value::String(#s.to_string()) }
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
            let elements = arr.iter().map(|v| generate_value(v));
            quote! { Value::Array(vec![#(#elements),*]) }
        }
        toml::Value::Table(table) => {
            let entries = table.iter().map(|(k, v)| {
                let key = k.to_string();
                let value = generate_value(v);
                quote! { (#key.to_string(), #value) }
            });
            quote! {
                Value::Table(vec![#(#entries),*].into_iter().collect())
            }
        }
        toml::Value::Datetime(dt) => {
            let dt_str = dt.to_string();
            quote! { Value::Datetime(#dt_str.to_string()) }
        }
    }
}
```

You'll also need to define your own `Value` enum that matches the types you want to support:

```rust
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Table(HashMap<String, Value>),
    Datetime(String),
}
```

To use this, you'll need these dependencies in your `Cargo.toml`:

```toml
[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = "1.0"
toml = "0.5"
phf = "0.11"
serde = { version = "1.0", features = ["derive"] }
```

This implementation:
1. Takes a TOML value as input
2. Matches on its type
3. Generates the appropriate Rust code to create a corresponding `Value` enum instance
4. Handles nested structures (arrays and tables) recursively

Note that this is a basic implementation and you might want to:
- Add error handling
- Support more types
- Customize the `Value` enum to match your needs
- Add validation
- Optimize the generated code

Would you like me to explain any part in more detail or help you adapt it for your specific use case?

I'm calling it like this:
    load_static_map!("themes/built_in/none.toml");

It looks like this:
#[proc_macro]
pub fn load_static_map(input: TokenStream) -> TokenStream {
    intercept_and_debug(true, "load_static_map", &input, load_static_map_impl)
}

fn load_static_map_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);

    let path = input.value();

    // Read and parse the data at compile time
    let content = std::fs::read_to_string(path).expect("Failed to read file");

And I'm getting the error:
--> /Users/donf/projects/thag_rs/demo/proc_macro_load_static_map.rs:9:5
  |
9 |     load_static_map!("themes/built_in/none.toml");
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = help: message: Failed to read file: Os { code: 2, kind: NotFound, message: "No such file or directory" }

What am I doing wrong?

The issue is related to the working directory when the macro runs during compilation. When you use a relative path like "themes/built_in/none.toml", it's being interpreted relative to the current working directory during compilation, which might not be what you expect.

Here are a few ways to fix this:

1. Use an absolute path based on the project root using the `env!("CARGO_MANIFEST_DIR")` macro:

```rust
#[proc_macro]
pub fn load_static_map_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let relative_path = input.value();

    // Construct absolute path from project root
    let absolute_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative_path);

    // Read and parse the data at compile time
    let content = std::fs::read_to_string(&absolute_path)
        .expect(&format!("Failed to read file: {}", absolute_path));
    // ... rest of the implementation
}
```

Then call it the same way:
```rust
load_static_map!("themes/built_in/none.toml");
```

2. Or you could pass the absolute path directly when calling the macro:

```rust
load_static_map!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/built_in/none.toml"));
```

The first approach is usually cleaner as it keeps the macro calls simpler. The macro will look for the file relative to your project's root directory (where your Cargo.toml is located).

Also, for debugging, you might want to add more detailed error messages:

```rust
let content = std::fs::read_to_string(&absolute_path)
    .unwrap_or_else(|e| panic!("Failed to read file '{}': {}", absolute_path, e));
```

Does this help resolve the file not found error?

The expanded map looks like this:
static MAP: phf::Map<&'static str, Value> = phf::phf_map! {
    ("name", Value::String("None".to_string())), ("backgrounds",
    Value::Array(vec![Value::String("#ffffff".to_string())])),
...

and I'm getting the error:
error: unsupported key expression
  --> /Users/donf/projects/thag_rs/demo/proc_macro_load_static_map.rs:29:5
   |
29 |     load_static_map!("/Users/donf/projects/thag_rs/themes/built_in/none.toml");
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
Any ideas?

Ah, yes - the `phf::phf_map!` macro has specific requirements for its key expressions. Let's modify the code to use string literals directly for the keys. Here's the corrected version:

```rust
#[proc_macro]
pub fn load_static_map(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let path = input.value();
    let absolute_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path);

    let content = std::fs::read_to_string(&absolute_path)
        .expect(&format!("Failed to read file: {}", absolute_path));

    let data: HashMap<String, toml::Value> = toml::from_str(&content)
        .expect("Failed to parse TOML");

    // Generate the static map with string literals as keys
    let entries = data.iter().map(|(k, v)| {
        let key_str = k.as_str();
        let value = generate_value(v);
        quote! { #key_str => #value }  // Changed from tuple syntax to => syntax
    });

    quote! {
        static MAP: phf::Map<&'static str, Value> = phf::phf_map! {
            #(#entries),*
        };
    }.into()
}
```

The key change is in how we format the entries. Instead of using tuple syntax `(key, value)`, we use the `key => value` syntax that `phf_map!` expects.

The expanded code should now look something like this:
```rust
static MAP: phf::Map<&'static str, Value> = phf::phf_map! {
    "name" => Value::String("None".to_string()),
    "backgrounds" => Value::Array(vec![Value::String("#ffffff".to_string())]),
    // ...
};
```

This should resolve the "unsupported key expression" error. Let me know if you need any clarification or run into other issues!

Thanks, now I'm getting a bunch of these:
error[E0015]: cannot call non-const fn `<str as ToString>::to_string` in statics
  --> /Users/donf/projects/thag_rs/demo/proc_macro_load_static_map.rs:29:5
   |
29 |     load_static_map!("/Users/donf/projects/thag_rs/themes/built_in/none.toml");
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: calls in statics are limited to constant functions, tuple structs and tuple variants
   = note: consider wrapping this expression in `std::sync::LazyLock::new(|| ...)`
   = note: this error originates in the macro `load_static_map` (in Nightly builds, run with -Z macro-backtrace for more info)



Ah yes, for static initialization we need to use const-friendly approaches. Here's how we can modify the code to work with const initialization:

```rust
#[proc_macro]
pub fn load_static_map(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let path = input.value();
    let absolute_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path);

    let content = std::fs::read_to_string(&absolute_path)
        .expect(&format!("Failed to read file: {}", absolute_path));

    let data: HashMap<String, toml::Value> = toml::from_str(&content)
        .expect("Failed to parse TOML");

    // Generate the entries
    let entries = data.iter().map(|(k, v)| {
        let key_str = k.as_str();
        let value = generate_const_value(v);
        quote! { #key_str => #value }
    });

    quote! {
        use std::sync::LazyLock;
        static MAP: LazyLock<phf::Map<&'static str, Value>> = LazyLock::new(|| {
            phf::phf_map! {
                #(#entries),*
            }
        });
    }.into()
}

fn generate_const_value(value: &toml::Value) -> proc_macro2::TokenStream {
    match value {
        toml::Value::String(s) => {
            quote! { Value::String(#s) }  // Note: Using string literal directly
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
            quote! { Value::Array(&[#(#elements),*]) }  // Note: Using array slice
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
```

And you'll need to modify your `Value` enum to work with static references:

```rust
#[derive(Debug, Clone)]
pub enum Value {
    String(&'static str),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(&'static [Value]),
    Table(&'static [(&&'static str, Value)]),
    Datetime(&'static str),
}
```

Also, make sure to add `LazyLock` to your dependencies in `Cargo.toml`:

```toml
[dependencies]
once_cell = "1.0"
# ... other dependencies
```

The key changes are:
1. Using `LazyLock` for lazy initialization
2. Using static string literals instead of `String`
3. Using static array slices instead of `Vec`
4. Removing runtime allocations (`to_string()`, `vec!`, etc.)

Let me know if you need any clarification or run into other issues!

How would I retrieve a value from a Value::Table?

Here's how you could access values from a Value::Table. First, let's create a helper method on the Value enum:

```rust
impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Table(entries) => {
                entries.iter()
                    .find(|(k, _)| *k == key)
                    .map(|(_, v)| v)
            }
            _ => None
        }
    }

    // Optionally, add convenience methods for common types:
    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            Value::String(s) => Some(s),
            _ => None
        }
    }

    pub fn as_array(&self) -> Option<&'static [Value]> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None
        }
    }
}
```

Then you can use it like this:

```rust
// Assuming MAP is your static phf::Map
if let Some(value) = MAP.get("some_table") {
    if let Some(nested_value) = value.get("nested_key") {
        match nested_value {
            Value::String(s) => println!("Got string: {}", s),
            Value::Integer(i) => println!("Got integer: {}", i),
            // ... handle other variants
        }
    }
}

// Or using the convenience methods:
if let Some(value) = MAP.get("some_table") {
    if let Some(nested_value) = value.get("nested_key") {
        if let Some(s) = nested_value.as_str() {
            println!("Got string: {}", s);
        }
    }
}

// You could even chain them:
let nested_str = MAP.get("some_table")
    .and_then(|v| v.get("nested_key"))
    .and_then(|v| v.as_str());
```

Would you like me to add more helper methods or show other ways to work with the values?
