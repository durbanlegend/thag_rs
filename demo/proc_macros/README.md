# Procedural Macros Documentation

This directory contains a collection of procedural macros demonstrating various techniques and patterns for writing proc macros in Rust.

## Overview

The procedural macros in this crate showcase:

- **Derive macros**: Generate implementations for traits automatically

- **Attribute macros**: Transform or augment code with custom attributes

- **Function-like macros**: Generate code using function-like syntax

## Derive Macros

### `AnsiCodeDerive`

A derive macro that generates helpful methods for ANSI color enums.

This macro generates:
- A `name()` method that returns a human-readable name for each color variant
- A `FromStr` trait implementation to parse variants from `snake_case` strings

# Attributes
- `#[ansi_name("Custom Name")]`: Override the default name for a variant

# Example Usage
See `demo/proc_macro_ansi_code_derive.rs` for a complete example.

```rust
#[derive(AnsiCodeDerive)]
enum Color {
    Red,
    #[ansi_name("Dark Gray")]
    BrightBlack,
}
```

**Example Usage:** [proc_macro_ansi_code_derive.rs](../proc_macro_ansi_code_derive.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_ansi_code_derive.rs
```

---

### `DeriveBasic`

A basic derive macro that generates a `new` constructor method.

This macro demonstrates derive macro functionality by generating a `new` method
for structs. The method takes parameters for all fields and returns a new instance.

# Attributes
- `#[expand_macro]`: When present, the macro expansion will be displayed during compilation

# Example
See `demo/proc_macro_derive_basic.rs` for usage.

```rust
#[derive(DeriveBasic)]
struct MyStruct {
    field: String,
}
// Generates: impl MyStruct { fn new(field: String) -> Self { ... } }
```

**Example Usage:** [proc_macro_derive_basic.rs](../proc_macro_derive_basic.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_basic.rs
```

---

### `DeriveConst`

Derives constant generation functionality with adjustment capabilities.

This macro demonstrates compile-time constant generation with configurable
adjustments. See `demo/proc_macro_organizing_code_const.rs` for examples.

# Attributes
- `#[adjust]`: Configures value adjustments
- `#[use_mappings]`: Specifies mapping configurations

# Example
```rust
#[derive(DeriveConst)]
#[adjust(factor = 2)]
struct Config {
    base_value: u32,
}
```

---

### `DeriveCustomModel`

Derives a custom model with additional functionality.

This macro demonstrates more advanced derive macro capabilities with custom attributes.
See `demo/proc_macro_derive_custom_model.rs` for detailed usage.

# Attributes
- `#[custom_model]`: Configures the custom model generation

# Example
```rust
#[derive(DeriveCustomModel)]
#[custom_model]
struct MyModel {
    id: u32,
    name: String,
}
```

**Example Usage:** [proc_macro_derive_custom_model.rs](../proc_macro_derive_custom_model.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_custom_model.rs
```

---

### `DeriveKeyMapList`

Derives key-map list functionality with advanced attribute support.

This macro generates methods for working with key-mapped lists, demonstrating
complex derive macro patterns with the `deluxe` crate.
See `demo/proc_macro_derive_key_map_list.rs` for usage examples.

# Attributes
- `#[deluxe]`: Enables deluxe attribute parsing
- `#[use_mappings]`: Configures key mappings

# Example
```rust
#[derive(DeriveKeyMapList)]
#[use_mappings(key1 = "mapped_key1")]
struct KeyMap {
    key1: String,
    values: Vec<String>,
}
```

**Example Usage:** [proc_macro_derive_key_map_list.rs](../proc_macro_derive_key_map_list.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_key_map_list.rs
```

---

### `DeserializeVec`

Derives vector deserialization functionality using the `deluxe` crate.

This macro demonstrates advanced attribute parsing and generates deserialization
methods for vector types with custom mappings.

# Attributes
- `#[deluxe]`: Configures deluxe attribute parsing
- `#[use_mappings]`: Specifies field mappings for deserialization

# Example
```rust
#[derive(DeserializeVec)]
#[use_mappings(field1 = "alias1")]
struct Data {
    field1: String,
    items: Vec<String>,
}
```

---

### `DocComment`

Derives automatic documentation comment generation.

This macro demonstrates automatic generation of documentation comments
based on struct or enum definitions. See `demo/proc_macro_derive_doc_comment.rs`
for examples.

# Example
```rust
#[derive(DocComment)]
struct Example {
    field: String,
}
// Generates documentation comments automatically
```

---

### `HostPortConst`

Derives compile-time constants for host and port configurations.

This macro generates compile-time constants for network configuration,
demonstrating practical applications of const generation.
See `demo/proc_macro_host_port_const.rs` for usage.

# Attributes
- `#[const_value]`: Specifies the constant value configuration

# Example
```rust
#[derive(HostPortConst)]
#[const_value(host = "localhost", port = 8080)]
struct ServerConfig;
```

**Example Usage:** [proc_macro_host_port_const.rs](../proc_macro_host_port_const.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_host_port_const.rs
```

---

### `IntoStringHashMap`

Derives conversion functionality to convert structs into `HashMap<String, String>`.

This macro generates an implementation that converts struct fields into a string-based HashMap.
Useful for serialization scenarios or when you need a dynamic key-value representation.

# Example
```rust
#[derive(IntoStringHashMap)]
struct Person {
    name: String,
    age: u32,
}
// Generates methods to convert Person into HashMap<String, String>
```

---

### `MyDescription`

Derives description functionality with custom attributes.

This macro demonstrates using the `deluxe` crate for advanced attribute parsing.
See the implementation for details on how custom descriptions are generated.

# Attributes
- `#[my_desc]`: Specifies custom description attributes

# Example
```rust
#[derive(MyDescription)]
#[my_desc(description = "A sample struct")]
struct Sample {
    value: i32,
}
```

---

## Attribute Macros

### `attribute_basic`

A basic attribute macro that demonstrates attribute macro functionality.

This is a simple example of an attribute macro that can be applied to items.
See `demo/proc_macro_attribute_basic.rs` for usage examples.

# Example
```rust
#[attribute_basic]
fn my_function() { }
```

**Example Usage:** [proc_macro_attribute_basic.rs](../proc_macro_attribute_basic.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_attribute_basic.rs
```

---

### `baz`

An attribute macro demonstrating the `expander` crate functionality.

This macro showcases macro expansion capabilities using the expander crate.
See `demo/proc_macro_expander_demo.rs` for usage examples.

# Example
```rust
#[baz]
fn my_function() {
    // function body
}
```

---

### `use_mappings`

An attribute macro for configuring field mappings.

This macro processes mapping configurations and applies them to the decorated item.
Often used in conjunction with other derive macros for customization.

# Example
```rust
#[use_mappings(field1 = "alias1", field2 = "alias2")]
struct MappedStruct {
    field1: String,
    field2: i32,
}
```

---

## Function-like Macros

### `const_demo`

A function-like macro demonstrating compile-time constant generation.

This macro shows how to generate constants at compile time based on input parameters.
See `demo/proc_macro_const_demo.rs` for basic usage.

# Example
```rust
const_demo!(MyConst = 42);
```

**Example Usage:** [proc_macro_const_demo.rs](../proc_macro_const_demo.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_const_demo.rs
```

---

### `const_demo_debug`

A debug variant of `const_demo` with detailed expansion information.

This macro provides debugging capabilities for const generation, showing
detailed expansion information. See `demo/proc_macro_const_demo_debug.rs`.

# Example
```rust
const_demo_debug!(MyConst = 42);
```

**Example Usage:** [proc_macro_const_demo_debug.rs](../proc_macro_const_demo_debug.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_const_demo_debug.rs
```

---

### `const_demo_expand`

A variant of `const_demo` that shows macro expansion during compilation.

This version of the const demo macro displays the generated code during compilation,
useful for debugging and understanding macro output. See `demo/proc_macro_const_demo_expand.rs`.

# Example
```rust
const_demo_expand!(MyConst = 42);
```

**Example Usage:** [proc_macro_const_demo_expand.rs](../proc_macro_const_demo_expand.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_const_demo_expand.rs
```

---

### `const_demo_grail`

An advanced constant generation macro using the `const_gen` crate.

This macro demonstrates advanced compile-time constant generation techniques
using external crates. See the implementation for technical details.

# Example
```rust
const_demo_grail!(AdvancedConst = complex_computation());
```

---

### `embed_file`

A function-like macro for embedding file contents at compile time.

This macro reads a file at compile time and embeds its contents as a string literal.
Useful for including configuration files, templates, or other static content.

# Example
```rust
let content = embed_file!("config.txt");
```

---

### `end`

Creates a function that returns the line number where it's called.

This macro generates a function with a name based on the input string that
returns the current line number using the `line!()` macro. Useful for debugging
and testing scenarios. See `demo/proc_macro_end.rs` for usage.

# Example
```rust
end!("my_function");
// Generates: fn end_my_function() -> u32 { line!() }

fn main() {
    println!("Current line: {}", end_my_function());
}
```

**Example Usage:** [proc_macro_end.rs](../proc_macro_end.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_end.rs
```

---

### `function_like_basic`

A basic function-like macro that generates a constant value.

This demonstrates function-like macro syntax and generates a constant `VALUE` set to 42.
See `demo/proc_macro_functionlike_basic.rs` for usage.

# Example
```rust
function_like_basic!();
// Generates: pub const VALUE: usize = 42;
```

---

### `load_static_map`

An advanced macro for loading directory contents into static maps.

This macro can embed entire directory structures as static HashMap data,
useful for embedding web assets, templates, or configuration directories.
See `demo/proc_macro_load_static_map.rs` for usage examples.

# Example
```rust
load_static_map!("assets/");
// Generates a static HashMap with file paths as keys and contents as values
```

**Example Usage:** [proc_macro_load_static_map.rs](../proc_macro_load_static_map.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_load_static_map.rs
```

---

### `organizing_code`

A function-like macro demonstrating code organization patterns.

This macro is based on examples from https://github.com/tdimitrov/rust-proc-macro-post
and shows how to organize complex macro logic. See `demo/proc_macro_organizing_code.rs`
for usage examples.

# Example
```rust
organizing_code!{
    // Your code here
}
```

**Example Usage:** [proc_macro_organizing_code.rs](../proc_macro_organizing_code.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_organizing_code.rs
```

---

### `organizing_code_tokenstream`

A function-like macro demonstrating TokenStream manipulation.

This macro shows advanced TokenStream processing techniques, also based on examples
from https://github.com/tdimitrov/rust-proc-macro-post.
See `demo/proc_macro_organizing_code_tokenstream.rs` for usage.

# Example
```rust
organizing_code_tokenstream!{
    // TokenStream content
}
```

**Example Usage:** [proc_macro_organizing_code_tokenstream.rs](../proc_macro_organizing_code_tokenstream.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_organizing_code_tokenstream.rs
```

---

### `repeat_dash`

A function-like macro that generates repeated dash characters.

This macro demonstrates simple text generation and can be used to create
visual separators or formatting elements. See `demo/proc_macro_repeat_dash.rs`
for usage examples.

# Example
```rust
repeat_dash!(10); // Generates 10 dash characters
```

**Example Usage:** [proc_macro_repeat_dash.rs](../proc_macro_repeat_dash.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_repeat_dash.rs
```

---

### `string_concat`

A function-like macro for compile-time string concatenation.

This macro demonstrates compile-time string manipulation and concatenation.
See `demo/proc_macro_string_concat.rs` for usage examples.

# Example
```rust
string_concat!("Hello", " ", "World"); // Generates "Hello World"
```

**Example Usage:** [proc_macro_string_concat.rs](../proc_macro_string_concat.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_string_concat.rs
```

---

## Usage

To use these macros in your project:

```toml
[dependencies]
thag_demo_proc_macros = { path = "demo/proc_macros" }
```

Or when using `thag_rs`:

```rust
// "thag_demo_proc_macros" is automatically resolved
use thag_demo_proc_macros::{YourMacro};
```

## Running Examples

Each proc macro has a corresponding example file in the `demo/` directory. To run the examples:

```bash
# Set the development path for thag-auto resolution
export THAG_DEV_PATH=/path/to/thag_rs

# Run an example
cargo run --bin thag -- demo/proc_macro_ansi_code_derive.rs
```

Or use the URL runner for published examples:

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_ansi_code_derive.rs
```

## Development

### Building
```bash
cd demo/proc_macros
cargo build
```

### Documentation
Generate and view the documentation:
```bash
cargo doc --no-deps --open
```

### Testing
```bash
cargo test
```

### Macro Expansion
Many macros support the `expand` feature to show generated code during compilation:
```bash
cargo build --features expand
```

### Example Testing
Test individual examples (requires setting `THAG_DEV_PATH`):
```bash
export THAG_DEV_PATH=$(pwd)  # From thag_rs root directory
cargo run --bin thag -- demo/proc_macro_const_demo.rs
cargo run --bin thag -- demo/proc_macro_derive_basic.rs
```

