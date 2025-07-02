# Refined Procedural Macros Demo Collection

A curated collection of 11 high-quality procedural macros designed for learning proc macro development in Rust.

## Philosophy

This collection prioritizes **quality over quantity**, featuring carefully selected macros that demonstrate progressive complexity and real-world utility. Each macro serves as a clear teaching example without the confusion of experimental or incomplete code.

The collection is organized into three categories: **Derive Macros** (5), **Attribute Macros** (3), and **Function-like Macros** (3), providing comprehensive coverage of all proc macro types.

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


**File**: `file_navigator.rs` | **Demo**: `../proc_macro_file_navigator.rs`

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


**File**: `const_demo.rs` | **Demo**: `../proc_macro_const_demo.rs`

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


## Running the Examples

Generates interactive file system navigation with comprehensive file operations workflow.

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

### 10. const_demo - Expert-level Macro


## Key Concepts Demonstrated

Advanced constant generation using external proc macro crates for complex compile-time computation.

**What it teaches:**

- Integration with external proc macro libraries
- Complex compile-time computation
- Advanced token stream manipulation
- Real-world proc macro ecosystem usage

**Example:**

```rust
const_demo!(
    let math = math::new(10);
    math.add(5);
    let result = math.get();
);
```

### 11. compile_time_assert - Function-like Macro


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
11. **Analyze const_demo** - Understand advanced external integration

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
cargo run -- demo/proc_macro_const_demo.rs
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

## Why These 11?

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

## What Was Removed

This refined collection replaced 20+ experimental macros with these 7 high-quality examples. Removed macros included:

- Incomplete experiments (`const_demo_grail`, `derive_key_map_list`)
- External dependency showcases (`custom_model`, `my_description`)
- Overly simple examples (`repeat_dash`, `attribute_basic`)
- Third-party code samples (`organizing_code*`)
- Redundant functionality (`ansi_code_derive`, `into_string_hash_map`)

## Contributing

When adding new macros to this collection, ensure they:

1. **Solve a real problem** - Have practical utility
2. **Teach something new** - Demonstrate unique concepts
3. **Are well documented** - Include comprehensive examples
4. **Are thoroughly tested** - Work reliably
5. **Fit the progression** - Add educational value to the sequence

This collection prioritizes educational value and code quality over feature quantity.
