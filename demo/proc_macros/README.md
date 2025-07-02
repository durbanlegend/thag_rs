# Procedural Macros Demo Collection

A collection of 10 procedural macros designed with two goals in mind:

 1. To be generally useful.

 2. To help teach proc macro development in Rust.

The original collection was based on my own prototypes and experiments and was itself something of a prototype. The revised collection is focused on sharing knowledge and useful code.
I used Claude (Sonnet 4) to help select and enhance a core of the best original proc macros, to suggest additional macros of the 3 types to flesh out the collection, to implement them and to draft the documentation (as you may be able to tell from the style 🙂).

This Readme file describes each of the proc macros in the collection, in order from simpler to more complex examples, followed by general information about the collection, such as how to running the demos.

The collection is organized into three categories: **Derive Macros** (5), **Attribute Macros** (3), and **Function-like Macros** (2), providing coverage of all proc macro types.

By default the output of each macro is automatically displayed in Rust source form to `stderr` at compile time, formatted if possible by `prettyplease`. The purpose is to help understand what the macros are doing and help debug any compiler errors that point to the macro invocation.

This "expansion" is carried out by the functions `maybe_expand_attr_macro` and `maybe_expand_proc_macro` at the end of `lib.rs`. As the naming "maybe_" implies, this may be turned on or off by the first argument to the function, `expand: boolean`. `lib.rs` demonstrates 2 methods of doing so, most simply by hard-coding the argument; alternatively, most of the derive macros are designed to enable the expansion on receiving the attribute `#[expand_macro]` from the caller. Other possibilities not demonstrated are to accept the boolean value from an environment variable or configuration option.

## The Collection

### 1. DeriveConstructor - Basic Derive Macro

**File**: `derive_constructor.rs` | **Demo**: `../proc_macro_derive_constructor.rs`

A fundamental derive macro that generates constructor methods for structs.

**What it teaches:**

- Basic derive macro structure
- Field iteration and processing
- Simple code generation with `quote!`
- Error handling for unsupported types

**Example:**

```rust
#[derive(DeriveConstructor)]
struct Person {
    name: String,
    age: u32,
}
// Generates: impl Person { pub fn new(name: String, age: u32) -> Person { ... } }
```

### 2. DeriveGetters - Intermediate Derive Macro

**File**: `derive_getters.rs` | **Demo**: `../proc_macro_derive_getters.rs`

Automatically generates getter methods that return references to struct fields.

**What it teaches:**

- Method generation patterns
- Type analysis (references vs owned types)
- Documentation generation in macros
- Field naming and identifier handling

**Example:**

```rust
#[derive(DeriveGetters)]
struct Config {
    host: String,
    port: u16,
}
// Generates getter methods: host(), port(), etc.
```

### 3. DeriveBuilder - Advanced Derive Macro

**File**: `derive_builder.rs` | **Demo**: `../proc_macro_derive_builder.rs`

Generates a complete builder pattern implementation for structs.

**What it teaches:**

- Builder pattern generation
- Fluent API design with method chaining
- Separate struct generation
- Build-time validation and error handling
- Default trait implementation

**Example:**

```rust
#[derive(DeriveBuilder)]
struct Config {
    host: String,
    port: u16,
}
// Generates: ConfigBuilder with fluent API
// let config = Config::builder().host("localhost").port(8080).build()?;
```

### 4. DeriveDisplay - Trait Implementation Macro

**File**: `derive_display.rs` | **Demo**: `../proc_macro_derive_display.rs`

Automatically generates Display trait implementations for structs and enums.

**What it teaches:**

- Trait implementation generation
- Pattern matching for enums
- Field formatting with proper separators
- Handling different struct types (named, tuple, unit)
- Type-aware formatting

**Example:**

```rust
#[derive(DeriveDisplay)]
struct Person {
    name: String,
    age: u32,
}
// Generates: impl Display for Person { ... }
// Output: "Person { name: Alice, age: 30 }"
```

### 5. DeriveDocComment - Enhanced Derive Macro

**File**: `derive_doc_comment.rs` | **Demo**: `../proc_macro_derive_doc_comment.rs`

Extracts documentation from structs, enums, and all variant types, making compile-time documentation available at runtime.

**What it teaches:**

- Advanced attribute parsing across multiple item types
- Working with structs, enums, tuple structs, and unit structs
- Documentation extraction from fields and variants
- Runtime access to compile-time documentation
- Complex pattern matching generation

**Example:**

```rust
#[derive(DeriveDocComment)]
enum Status {
    /// Operation completed successfully
    Success,
    /// An error occurred
    Error,
}
// Generates: impl Status { fn doc_comment(&self) -> &'static str { ... } }
```

**Example - Struct:**

```rust
/// A user configuration
#[derive(DeriveDocComment)]
struct Config {
    /// The server hostname
    host: String,
    /// The server port number
    port: u16,
}
// Generates: impl Config { pub fn field_doc(name: &str) -> Option<&'static str> { ... } }
```

### 6. cached - Attribute Macro

**File**: `cached.rs` | **Demo**: `../proc_macro_cached.rs`

Adds automatic memoization/caching to functions for significant performance improvements.

**What it teaches:**

- Advanced attribute macro techniques
- Function wrapping and transformation
- Thread-safe caching with HashMap and Mutex
- Compile-time cache key generation
- Performance optimization patterns

**Example:**

```rust
#[cached]
fn expensive_fibonacci(n: u32) -> u64 {
    if n <= 1 { n as u64 } else { expensive_fibonacci(n-1) + expensive_fibonacci(n-2) }
}
```

### 7. timing - Attribute Macro

**File**: `timing.rs` | **Demo**: `../proc_macro_timing.rs`

Automatically measures and displays function execution time for performance analysis.

**What it teaches:**

- Simple but effective attribute macro patterns
- Function signature preservation
- Performance measurement techniques
- Console output integration

**Example:**

```rust
#[timing]
fn slow_operation() -> String {
    std::thread::sleep(std::time::Duration::from_millis(100));
    "completed".to_string()
}
// Output: ⏱️ Function 'slow_operation' took: 100.234ms
```

### 8. retry - Attribute Macro

**File**: `retry.rs` | **Demo**: `../proc_macro_retry.rs`

Adds automatic retry logic to functions with configurable attempts and backoff delays.

**What it teaches:**

- Attribute macro parameter parsing
- Error handling and resilience patterns
- Panic catching and retry logic
- Progress reporting and user feedback

**Example:**

```rust
#[retry(times = 5)]
fn unreliable_network_call() -> Result<String, std::io::Error> {
    // Network operation that might fail
    Ok("success".to_string())
}
```

### 9. file_navigator - Function-like Macro

**File**: `file_navigator.rs` | **Demo**: `../proc_macro_file_navigator.rs`

Generates interactive file system navigation from the command line using `inquire`, with comprehensive file operations workflow.

**What it teaches:**

- Function-like macro patterns
- Complex code generation
- Real-world utility creation
- Interactive user interface generation

**Example:**

```rust
file_navigator! {}
// Generates: FileNavigator struct, select_file function, save_to_file function, etc.
```

### 10. compile_time_assert - Function-like Macro

**File**: `compile_time_assert.rs` | **Demo**: `../proc_macro_compile_time_assert.rs`

Generates compile-time assertions that prevent compilation if conditions are not met.

**What it teaches:**

- Function-like macro parsing with multiple parameters
- Compile-time validation techniques
- Zero-runtime-cost assertions
- Custom error message generation

**Example:**

```rust
compile_time_assert!(std::mem::size_of::<usize>() == 8, "Requires 64-bit platform");
compile_time_assert!(1 + 1 == 2, "Basic math must work");
```

## Learning Path

Follow this progression for the best learning experience:

1. **Start with DeriveConstructor** - Learn the fundamentals
2. **Progress to DeriveGetters** - Understand method generation
3. **Study DeriveBuilder** - Master builder pattern generation
4. **Learn DeriveDisplay** - Understand trait implementation
5. **Explore DeriveDocComment** - Master attribute parsing across types
6. **Try cached** - Learn function transformation and caching
7. **Use timing** - Understand performance measurement
8. **Apply retry** - Master error handling and resilience
9. **Practice file_navigator** - See practical applications
10. **Test compile_time_assert** - Learn compile-time validation

## Running the Examples

Each macro has a comprehensive demo file. Run them with:

```bash
# Set up the development environment
export THAG_DEV_PATH=/path/to/thag_rs

# Derive Macros
cargo run -- demo/proc_macro_derive_constructor.rs
cargo run -- demo/proc_macro_derive_getters.rs
cargo run -- demo/proc_macro_derive_builder.rs
cargo run -- demo/proc_macro_derive_display.rs
cargo run -- demo/proc_macro_derive_doc_comment.rs

# Attribute Macros
cargo run -- demo/proc_macro_cached.rs
cargo run -- demo/proc_macro_timing.rs
cargo run -- demo/proc_macro_retry.rs

# Function-like Macros
cargo run -- demo/proc_macro_file_navigator.rs
cargo run -- demo/proc_macro_compile_time_assert.rs
```

## Key Concepts Demonstrated

- **Syntax Parsing**: Using `syn` to parse Rust syntax trees
- **Code Generation**: Using `quote!` to generate clean, readable code
- **Error Handling**: Proper error reporting in proc macros
- **Type Analysis**: Working with different Rust types and patterns
- **Attribute Processing**: Extracting and using custom attributes
- **Function Transformation**: Wrapping and modifying function behavior
- **Performance Optimization**: Caching, timing, and retry patterns
- **Compile-time Validation**: Zero-runtime-cost assertions
- **Documentation Generation**: Runtime access to compile-time docs
- **External Integration**: Using proc macro ecosystem crates

## Why These 10?

Each macro was chosen for specific pedagogical value:

- **Progressive Complexity**: From simple field processing to advanced external integration
- **Complete Coverage**: All three proc macro types represented
- **Real-world Utility**: All solve actual problems developers face
- **Clean Implementation**: No experimental code or incomplete features
- **Teaching Value**: Each demonstrates distinct proc macro concepts
- **Ecosystem Integration**: Shows how to work with existing crate ecosystems

## Building and Testing

```bash
# Check the library builds correctly
cd demo/proc_macros
cargo check

# Run tests (if any are added)
cargo test

# Build documentation
cargo doc --open
```

## Contributing

When adding new macros to this collection, ensure they:

1. **Solve a real problem** - Have practical utility
2. **Teach something new** - Demonstrate unique concepts
3. **Are well documented** - Include comprehensive examples
4. **Are thoroughly tested** - Work reliably
5. **Fit the progression** - Add educational value to the sequence
