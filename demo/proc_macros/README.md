# Procedural Macros Demo Collection

A collection of 12 procedural macros for learning proc macro development in Rust. Each macro aims to demonstrate specific techniques while solving practical problems.

The macros are presented here in the recommended order for learning.

The collection covers all three proc macro types: **Derive Macros** (5), **Attribute Macros** (3), and **Function-like Macros** (4).

By default, each macro displays its generated code to `stderr` at compile time, formatted by `prettyplease` when possible. This helps understand macro behavior and debug compilation errors. The expansion feature is controlled by the `maybe_expand_attr_macro` and `maybe_expand_proc_macro` functions in `lib.rs`.

## The Collection

### 1. DeriveConstructor - Basic Derive Macro

**File**: `derive_constructor.rs` | **Demo**: `../proc_macro_derive_constructor.rs`

Generates constructor methods for structs.

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

Generates getter methods that return references to struct fields.

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

Generates builder pattern implementation for structs.

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

Generates Display trait implementations for structs and enums.

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

Extracts compile-time documentation and makes it available at runtime.

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

### 6. cached - Attribute Macro

**File**: `cached.rs` | **Demo**: `../proc_macro_cached.rs`

Adds automatic memoization to functions.

**What it teaches:**

- Function wrapping and transformation
- Thread-safe caching with HashMap and Mutex
- Compile-time cache key generation
- Performance optimization patterns

**Example:**

```rust
#[cached]
fn fibonacci(n: u32) -> u64 {
    if n <= 1 { n as u64 } else { fibonacci(n-1) + fibonacci(n-2) }
}
```

### 7. timing - Attribute Macro

**File**: `timing.rs` | **Demo**: `../proc_macro_timing.rs`

Measures and displays function execution time.

**What it teaches:**

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
// Output: Function 'slow_operation' took: 100.234ms
```

### 8. retry - Attribute Macro

**File**: `retry.rs` | **Demo**: `../proc_macro_retry.rs`

Adds automatic retry logic to functions with configurable attempts and backoff.

**What it teaches:**

- Attribute macro parameter parsing
- Error handling and resilience patterns
- Panic catching and retry logic
- Progress reporting

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

Generates interactive file system navigation functionality.

**What it teaches:**

- Function-like macro patterns
- Complex code generation
- Interactive user interface generation
- External crate integration

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

### 11. env_or_default - Function-like Macro

**File**: `env_or_default.rs` | **Demo**: `../proc_macro_env_or_default.rs`

Resolves environment variables at compile time with fallback defaults.

**What it teaches:**

- Compile-time environment variable processing
- String literal generation
- Configuration management patterns
- Zero-overhead environment access

**Example:**

```rust
const DATABASE_URL: &str = env_or_default!("DATABASE_URL", "postgresql://localhost:5432/myapp");
const SERVER_PORT: &str = env_or_default!("PORT", "8080");
```

### 12. generate_tests - Function-like Macro

**File**: `generate_tests.rs` | **Demo**: `../proc_macro_generate_tests.rs`

Generates multiple test functions from test data to reduce boilerplate.

**What it teaches:**

- Test automation patterns
- Parameter unpacking from tuples
- Repetitive code generation
- Test function generation

**Example:**

```rust
generate_tests! {
    test_addition: [
        (1, 2, 3),
        (5, 7, 12),
        (0, 0, 0),
    ] => |a, b, expected| assert_eq!(a + b, expected)
}
// Generates: test_addition_0, test_addition_1, test_addition_2 functions
```

## Learning Path

Follow this progression for systematic learning:

1. **DeriveConstructor** - Learn fundamentals
2. **DeriveGetters** - Understand method generation
3. **DeriveBuilder** - Master builder pattern generation
4. **DeriveDisplay** - Understand trait implementation
5. **DeriveDocComment** - Master attribute parsing
6. **cached** - Learn function transformation
7. **timing** - Understand performance measurement
8. **retry** - Master error handling patterns
9. **file_navigator** - See practical applications
10. **compile_time_assert** - Learn compile-time validation
11. **env_or_default** - Master environment processing
12. **generate_tests** - Understand test automation

## Running the Examples

Each macro has a demo file. Run them with:

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
cargo run -- demo/proc_macro_env_or_default.rs
cargo run -- demo/proc_macro_generate_tests.rs
```

## Key Concepts Demonstrated

- **Syntax Parsing**: Using `syn` to parse Rust syntax trees
- **Code Generation**: Using `quote!` to generate code
- **Error Handling**: Proper error reporting in proc macros
- **Type Analysis**: Working with different Rust types and patterns
- **Attribute Processing**: Extracting and using custom attributes
- **Function Transformation**: Wrapping and modifying function behavior
- **Performance Optimization**: Caching, timing, and retry patterns
- **Compile-time Validation**: Zero-runtime-cost assertions
- **Documentation Generation**: Runtime access to compile-time docs
- **Environment Processing**: Compile-time configuration management
- **Test Automation**: Reducing test boilerplate

## Why These 12?

Each macro was selected for specific educational value:

- **Progressive Complexity**: From simple field processing to advanced patterns
- **Complete Coverage**: All three proc macro types represented
- **Practical Utility**: Each solves real development problems
- **Clean Implementation**: Production-ready code without experimental features
- **Teaching Value**: Each demonstrates distinct proc macro concepts
- **Balanced Distribution**: Even coverage across macro types

## Building and Testing

```bash
# Check the library builds correctly
cd demo/proc_macros
cargo check

# Run tests
cargo test

# Build documentation
cargo doc --open
```

## Contributing

When adding new macros to this collection, ensure they:

1. **Solve real problems** - Have practical utility
2. **Teach unique concepts** - Demonstrate distinct techniques
3. **Include comprehensive examples** - Show practical usage
4. **Work reliably** - Are thoroughly tested
5. **Fit the educational progression** - Add value to the learning sequence
