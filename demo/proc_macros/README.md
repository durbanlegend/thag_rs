# Procedural Macros Documentation

This directory contains a collection of procedural macros demonstrating various techniques and patterns for writing proc macros in Rust.

## Overview

The procedural macros in this crate showcase:

- **Derive macros**: Generate implementations for traits automatically

- **Attribute macros**: Transform or augment code with custom attributes

- **Function-like macros**: Generate code using function-like syntax

## Derive Macros

### `DeriveBuilder`

Generates builder pattern implementation for structs.

Creates a separate builder struct with fluent API for step-by-step construction.
Demonstrates advanced derive macro concepts including struct generation and method chaining.

## Example
```rust
use thag_demo_proc_macros::DeriveBuilder;
#[derive(DeriveBuilder)]
struct Config {
    host: String,
    port: u16,
}
// Generates: ConfigBuilder with fluent API
// let config = Config::builder().host("localhost").port(8080).build()?;
```

**Example Usage:** [proc_macro_derive_builder.rs](../proc_macro_derive_builder.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_builder.rs
```

---

### `DeriveConstructor`

Generates constructor methods for structs.

Creates a `new` method that takes parameters for all fields and returns a new instance.
Demonstrates basic derive macro concepts including field iteration and code generation.

## Example
```rust
use thag_demo_proc_macros::DeriveConstructor;
#[derive(DeriveConstructor)]
struct Person {
    name: String,
    age: u32,
}
// Generates: impl Person { pub fn new(name: String, age: u32) -> Person { ... } }
```

**Example Usage:** [proc_macro_derive_constructor.rs](../proc_macro_derive_constructor.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_constructor.rs
```

---

### `DeriveDisplay`

Generates Display trait implementations for structs and enums.

Creates readable string representations with proper formatting for different struct types.
Demonstrates trait implementation generation and pattern matching for enums.

## Example
```rust
use thag_demo_proc_macros::DeriveDisplay;
#[derive(DeriveDisplay)]
struct Person {
    name: String,
    age: u32,
}
// Generates: impl Display for Person { ... }
// Output: "Person { name: Alice, age: 30 }"
```

**Example Usage:** [proc_macro_derive_display.rs](../proc_macro_derive_display.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_display.rs
```

---

### `DeriveDocComment`

Extracts compile-time documentation and makes it available at runtime.

Generates methods to access documentation strings from enum variants and struct fields.
Demonstrates advanced attribute parsing across multiple item types.

## Example
```rust
use thag_demo_proc_macros::DeriveDocComment;
#[derive(DeriveDocComment)]
enum Status {
    /// Operation completed successfully
    Success,
    /// An error occurred
    Error,
}
// Generates: impl Status { fn doc_comment(&self) -> &'static str { ... } }
```

**Example Usage:** [proc_macro_derive_doc_comment.rs](../proc_macro_derive_doc_comment.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_doc_comment.rs
```

---

### `DeriveGetters`

Generates getter methods for all struct fields.

Creates getter methods that return references to fields, avoiding unnecessary moves.
Demonstrates method generation patterns and type analysis.

## Example
```rust
use thag_demo_proc_macros::DeriveGetters;
#[derive(DeriveGetters)]
struct Person {
    name: String,
    age: u32,
}
// Generates: impl Person { pub fn name(&self) -> &String { &self.name } ... }
```

**Example Usage:** [proc_macro_derive_getters.rs](../proc_macro_derive_getters.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_getters.rs
```

---

## Attribute Macros

### `cached`

Adds automatic memoization to functions.

Wraps functions with caching logic using HashMap and Mutex for thread safety.
Demonstrates function transformation and caching patterns.

## Example
```rust
use thag_demo_proc_macros::cached;
#[cached]
fn fibonacci(n: u32) -> u32 {
    if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) }
}

// To see the generated code during development:
#[cached(expand)]
fn fibonacci_debug(n: u32) -> u32 {
    if n <= 1 { n } else { fibonacci_debug(n-1) + fibonacci_debug(n-2) }
}
```

**Example Usage:** [proc_macro_cached.rs](../proc_macro_cached.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_cached.rs
```

---

### `retry`

Adds automatic retry logic to functions.

Wraps functions with configurable retry attempts and backoff delays.
Demonstrates attribute parameter parsing and error handling patterns.

## Example
```ignore
use thag_demo_proc_macros::retry;
#[retry(times = 5)]
fn unreliable_operation() -> Result<String, std::io::Error> {
    // Network operation that might fail
    Ok("success".to_string())
}

// To see the generated code during development:
#[retry(times = 3, expand)]
fn debug_operation() -> Result<String, std::io::Error> {
    // Network operation that might fail
    Ok("success".to_string())
}
```

**Example Usage:** [proc_macro_retry.rs](../proc_macro_retry.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_retry.rs
```

---

### `timing`

Measures and displays function execution time.

Wraps functions to measure execution time and output results to console.
Demonstrates function signature preservation and performance measurement.

## Example
```rust
use thag_demo_proc_macros::timing;
#[timing]
fn slow_operation() -> i32 {
    std::thread::sleep(std::time::Duration::from_millis(100));
    42
}
// Output: Function 'slow_operation' took: 100.234ms

// To see the generated code during development:
#[timing(expand)]
fn debug_operation() -> i32 {
    std::thread::sleep(std::time::Duration::from_millis(100));
    42
}
```

**Example Usage:** [proc_macro_timing.rs](../proc_macro_timing.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_timing.rs
```

---

## Function-like Macros

### `compile_time_assert`

Generates compile-time assertions.

Creates compile-time checks that prevent compilation if conditions are not met.
Demonstrates compile-time validation and zero-runtime-cost assertions.

## Example
```rust
use thag_demo_proc_macros::compile_time_assert;
compile_time_assert!(std::mem::size_of::<usize>() == 8, "Requires 64-bit platform");
compile_time_assert!(1 + 1 == 2, "Basic math must work");
```

**Example Usage:** [proc_macro_compile_time_assert.rs](../proc_macro_compile_time_assert.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_compile_time_assert.rs
```

---

### `env_or_default`

Resolves environment variables at compile time with fallback defaults.

Reads environment variables during compilation and generates string literals.
Demonstrates compile-time environment processing and configuration management.

## Example
```rust
const DATABASE_URL: &str = env_or_default!("DATABASE_URL", "localhost:5432");
const DEBUG_MODE: &str = env_or_default!("DEBUG", "false");
```

**Example Usage:** [proc_macro_env_or_default.rs](../proc_macro_env_or_default.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_env_or_default.rs
```

---

### `file_navigator`

Generates interactive file system navigation functionality.

Creates structures and functions for file selection and directory navigation.
Demonstrates complex code generation and external crate integration.

## Example
```ignore
use thag_demo_proc_macros::file_navigator;
file_navigator! {}
// Generates: FileNavigator struct, select_file function, save_to_file function, etc.
```

**Example Usage:** [proc_macro_file_navigator.rs](../proc_macro_file_navigator.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_file_navigator.rs
```

---

### `generate_tests`

Generates multiple test functions from test data.

Creates test functions from data arrays to reduce boilerplate in test suites.
Demonstrates test automation and repetitive code generation patterns.

## Example
```rust
generate_tests! {
    test_addition: [
        (1, 2, 3),
        (5, 7, 12),
        (0, 0, 0),
    ] => |a, b, expected| assert_eq!(a + b, expected)
}
```

**Example Usage:** [proc_macro_generate_tests.rs](../proc_macro_generate_tests.rs)

**Run Example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_generate_tests.rs
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
cargo run --bin thag -- demo/proc_macro_derive_constructor.rs
```

