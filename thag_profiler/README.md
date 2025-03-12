# User Guide to Application Profiling with `thag_profiler`

## Introduction

Profiling is key to optimizing your Rust applications, but it tends to be time-consuming (no pun intended). `thag_profiler` provides quick, straightforward and consistent run-time and memory profiling for your Rust project or script across all platforms.

`thag_profiler` uses intrusive profiling, meaning you need to instrument your code for profiling. While you can do this manually, `thag_profiler` also provides automated instrumentation tools to help. This instrumentation is "lossless", preserving original code with its comments and formatting, adding only necessary `#[profiled]` function attributes (and `#[enable_profiling]` for `fn main`) using the `rust-analyzer` syntax tree library.

The run-time overhead of instrumentation is zero when the `profiling` feature is disabled. When enabled, a wrapper is transparently added around functions to instantiate a `Profile` object to measure execution time and/or memory usage. Profiling overhead is excluded from the reported execution time.

`thag_profiler` works with both async and non-async code. It uses Rust's `backtrace` crate to populate the call stack for each Profile instance, stripping out non-profiled functions including scaffolding from async runtimes like `tokio`.

## Installation

### Adding to Your Project

Add `thag_profiler` to your project dependencies:

```toml
[dependencies]
thag_profiler = "0.1"

# Optional: Enable profiling for your project
[features]
profiling = ["thag_profiler/profiling"]
```

### Installing the Profiling Tools

```bash
# Install all tools
cargo install thag_profiler --features=tools

# Or install individual tools
cargo install thag_profiler --features=instrument-tool --bin thag-instrument
cargo install thag_profiler --features=remove-tool --bin thag-remove
cargo install thag_profiler --features=analyze-tool --bin thag-analyze
```

## Quick-start Guide

1. **Instrument your code**:

  Replace `2021` with your project's Rust edition:

   ```bash
   thag-instrument 2021 < src/any_module.rs > src/any_module_instrumented.rs
   ```

  Compare the instrumented code to the original before replacing the original with the instrumented version.

  Repeat for all modules you want to profile.

2. **Enable profiling** via one of these methods:

   - Add the `#[enable_profiling]` attribute to your main function. This is done automatically if you instrument `main.rs` as in step 1.
   - Enable the `profiling` feature when running: `cargo run --features profiling`

3. **Run your application** to generate profile data

4. **Analyze the results**:

   ```bash
   thag-analyze
   ```

## Enabling Profiling

There are several ways to enable profiling:

### 1. Using the `#[enable_profiling]` Attribute

```rust
use thag_profiler::profiled;

#[enable_profiling]  // Default: profiles both time and memory
fn main() {
    // Your code here
}

// Or specify the profiling type
#[enable_profiling(profile_type = "time")]  // Options: "time", "memory", "both"
fn main() {
    // Your code here
}
```

### 2. Using the `profiling` Feature Flag

In your Cargo.toml or script's toml block:

```toml
[features]
profiling = ["thag_profiler/profiling"]
```

Then run with:
```bash
cargo run --features profiling
```

### 3. Programmatically

```rust
use thag_profiler::{enable_profiling, ProfileType};

fn main() {
    // Enable profiling programmatically
    enable_profiling(true, ProfileType::Both).expect("Failed to enable profiling");

    // Your code here
}
```

## Instrumenting Your Code

### Automatic Instrumentation

For easier profiling, `thag_profiler` provides a streaming tool to automatically instrument your code:

  * Ensure your original source is backed up before instrumenting.

  * Replace `2021` with your project's Rust edition.

  * Do not redirect the output to your source file! Trust Thag!

   ```bash
   thag-instrument 2021 < path/to/your/file.rs > path/to/yourinstrumented/file.rs
   ```

  Compare the instrumented code to the original before replacing the original with the instrumented version.

  Repeat for all modules you want to profile.

This will add `#[profiled]` attributes to functions and methods, and `#[enable_profiling]` to the main function if present.

When you're done profiling, remove the instrumentation:

  ```bash
  thag-remove 2021 < path/to/your/instrumented/file.rs > path/to/your/de-instrumented/file.rs
  ```
Again, always verify the output before replacing the original file.

### Manual Instrumentation

#### Using the `#[profiled]` Attribute

```rust
use thag_profiler::profiled;

#[profiled]
fn expensive_calculation() -> u64 {
    // Function code
}

// Works with async functions too
#[profiled]
async fn fetch_data() -> Result<String, Error> {
    // Async operations
}
```

#### The `#[profiled]` Attribute Options

```rust
// Add implementation type name to profile output
#[profiled(imp = "MyStruct")]
fn my_method(&self) { /* ... */ }

// Add trait name to profile output
#[profiled(trait_name = "MyTrait")]
fn my_method(&self) { /* ... */ }

// Specify profiling type
#[profiled(profile_type = "memory")]  // Options: "global", "time", "memory", "both"
fn memory_heavy_function() { /* ... */ }
```

#### Profiling Code Sections

For fine-grained control, you can profile specific sections of code:

```rust
fn my_function() {
    // Regular code

    // Start a profiled section
    let section = profile!("expensive part");

    // Code to profile
    expensive_operation();

    // End the profiled section
    // This isoptional - otherwise it will end when `section` goes out of scope.
    section.end();

    // More code (not profiled)
}
```

The `profile!` macro supports various options:

```rust
// Basic usage (profiles time)
let section = profile!("section_name");

// Specify profile type
let section = profile!("memory_section", memory);  // Options: time, memory, both

// Profile async code
let section = profile!("async_section", async);
let section = profile!("async_memory", memory, async);

// Profile methods (auto-detects name from backtrace)
let section = profile!(method);
let section = profile!(method, memory);
let section = profile!(method, async);
let section = profile!(method, both, async);
```

## Conditional Profiling

You can make profiling conditional:

```rust
// Only profile in debug builds
#[cfg_attr(debug_assertions, profiled)]
fn expensive_calculation() { /* ... */ }

// Only profile when a feature is enabled
#[cfg_attr(feature = "profile_enabled", profiled)]
fn complex_operation() { /* ... */ }

// Combine with options
#[cfg_attr(feature = "profile_enabled", profiled(imp = "MyStruct"))]
fn my_method(&self) { /* ... */ }
```

## Analyzing Profile Results

### Profile Output

Profiling generates folded stack files in your current directory:

- `{executable_name}-{timestamp}.folded` - For time profiling
- `{executable_name}-{timestamp}-memory.folded` - For memory profiling

### Using the Profiling Analyzer

```bash
thag-analyze
```

This interactive tool lets you:
- View individual time or memory profiles
- Compare profiles to see optimization impacts
- Generate flamecharts for visual analysis
- Filter and search profile data
- View detailed statistics

### Interpreting Flamecharts

Flamecharts provide an intuitive visualization of your profiling data:
- The horizontal axis shows function execution chronologically
- The wider a function appears, the more time it takes
- Interactive features let you zoom, search, and explore details

## Advanced Features

### Working with Async Code

The `#[profiled]` attribute works seamlessly with async functions:

```rust
#[profiled]
async fn process_data() -> Result<Data, Error> {
    // The profiler will correctly track time spent in this future
    let result = fetch_data().await?;
    process_result(result).await
}
```

### Error Handling with Profile Sections

```rust
fn fallible_operation() -> Result<(), Error> {
    let section = profile!("critical_operation");

    match risky_function() {
        Ok(value) => {
            // Process value
            section.end();
            Ok(())
        }
        Err(e) => {
            // End profiling before returning error
            section.end();
            Err(e)
        }
    }
}
```

### Checking if Profiling is Active

```rust
fn my_function() {
    let section = profile!("expensive_part");

    if section.is_active() {
        println!("Profiling is enabled");
    }

    // Process data
    section.end();
}
```

## Best Practices

1. **Profile representative workloads**: Ensure your test cases represent real-world usage
2. **Focus on hot paths**: Look for the widest blocks in flamecharts - these are your bottlenecks
3. **Compare before/after**: Always profile before and after optimizations
4. **Use serial testing**: When testing profiled functions, use the `serial_test` crate:

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_profiled_function() {
    // Tests using profiled functions
}
```

5. **Verify automated changes**: Always review changes made by the instrumentation tools

## Troubleshooting

### Common Issues

1. **Missing profile output**: Ensure profiling is enabled and you have write permissions
2. **Test failures**: Use `#[serial]` for tests with profiled functions
3. **Overhead concerns**: For extremely performance-sensitive code, be aware of profiling overhead
4. **Instrumentation safety**: Never redirect instrumentation tool output directly back to input files

## Inspecting Profile Files

The folded stack files are human-readable:

```bash
head your-executable-timestamp.folded
```

## Conclusion

`thag_profiler` provides a simple but effective cross-platform solution for understanding and optimizing your Rust code. By combining easy instrumentation, detailed analysis, and interactive visualizations, it helps you make your code faster and more efficient.

Get started quickly by running `thag-instrument` to auto-instrument your code, enable profiling, run your application, and analyze the results with `thag-analyze`.



# Readme.md for thag_profiler

I've updated the guide to reflect the new structure and features of `thag_profiler`. Here's the revised markdown for your README.md:

```md
# thag_profiler

A straightforward, lightweight profiling library for Rust applications that provides time and memory profiling with minimal overhead.

## Features

- **Zero-cost abstraction**: No runtime overhead when profiling is disabled
- **Time and memory profiling**: Track execution time or memory usage, or both
- **Function and section profiling**: Profile entire functions or specific code sections
- **Async support**: Seamlessly works with async code
- **Automatic instrumentation**: Tools to add and remove profiling code
- **Interactive flamecharts**: Visualize performance bottlenecks
- **Cross-platform**: Works on all platforms supported by Rust

## Quick Start

### Installation

Add `thag_profiler` to your `Cargo.toml`:

```toml
[dependencies]
thag_profiler = "0.1"

# If you want to enable profiling:
[features]
my_profile_feature = ["thag_profiler/profiling"]
```


Install the profiling tools:

```bash
# Install all tools
cargo install thag_profiler --features=tools

# Or install individual tools
cargo install thag_profiler --features=instrument-tool --bin thag-instrument
cargo install thag_profiler --features=remove-tool --bin thag-remove
cargo install thag_profiler --features=analyze-tool --bin thag-analyze
```

### Instrumenting Your Code

Automatically instrument your code:

```bash
thag-instrument path/to/your/file.rs > path/to/your/instrumented_file.rs
```

Or manually add profiling annotations:

```rust
use thag_profiler::{profiled, profile};

// Instrument a function
#[profiled]
fn expensive_calculation() -> u64 {
    // Function code...
    42
}

// Profile a specific section
fn complex_operation() {
    // Some code...

    let section = profile!("expensive_part");
    // Code to profile
    expensive_operation();
    section.end();

    // More code...
}
```

### Enabling Profiling

#### In Scripts Run with thag

When using `thag_profiler` in scripts, you have two options:

1. **Enable via command line** (recommended):
   ```bash
   cargo run bank/mem_prof.rs --features=profile
   ```

   With this script configuration:
   ```rust
   /*[toml]
   [dependencies]
   thag_profiler = { version = "0.1" }

   [features]
   profile = ["thag_profiler/profiling"]
   */
   ```

2. **Enable directly in the dependency**:
   ```rust
   /*[toml]
   [dependencies]
   thag_profiler = { version = "0.1", features = ["profiling"] }
   */
   ```

#### In Regular Cargo Projects

In standard Cargo projects, the same options apply:

1. **Use feature propagation**:
   ```toml
   [dependencies]
   thag_profiler = { version = "0.1" }

   [features]
   my_profiling = ["thag_profiler/profiling"]
   ```

   Then run with:
   ```bash
   cargo run --features my_profiling
   ```

2. **Enable directly in the dependency**:
   ```toml
   [dependencies]
   thag_profiler = { version = "0.1", features = ["profiling"] }
   ```
```

### Analyzing Results

After running your program, analyze the results:

```bash
thag-analyze
```

This will open an interactive menu to explore your profiling data.

## Detailed Usage

### Function Profiling with `#[profiled]`

Add the `#[profiled]` attribute to any function you want to profile:

```rust
use thag_profiler::profiled;

// Regular function
#[profiled]
fn expensive_calculation() -> u64 {
    // Function code...
}

// Works with async functions too
#[profiled]
async fn fetch_data() -> Result<String, Error> {
    // Async operations...
}

// Methods in implementations
impl MyStruct {
    #[profiled]
    fn process(&self, data: &[u8]) {
        // Method code...
    }
}
```

#### Attribute Options

The `#[profiled]` attribute supports several options:

```rust
// Specify the implementation type for better method profiling
#[profiled(imp = "MyStruct")]
fn my_method(&self) { /* ... */ }

// Specify the trait being implemented
#[profiled(trait_name = "MyTrait")]
fn trait_method(&self) { /* ... */ }

// Specify what to profile (time, memory, or both)
#[profiled(profile_type = "memory")]
fn allocating_function() { /* ... */ }
```

### Code Section Profiling with `profile!`

Use the `profile!` macro to profile specific sections of code:

```rust
use thag_profiler::profile;

fn complex_function() {
    // Basic usage
    let section = profile!("initialization");
    initialize_things();
    section.end();

    // Profile a method
    let section = profile!(method);
    self.do_something();
    section.end();

    // Profile memory usage
    let section = profile!("allocation", memory);
    let data = vec![0; 1_000_000];
    section.end();

    // Profile async code
    let section = profile!("async_operation", async);
    async_operation().await;
    section.end();

    // Combined options
    let section = profile!(method, both, async);
    self.complex_async_operation().await;
    section.end();
}
```

### Conditional Compilation

You can conditionally apply profiling:

```rust
// Only apply profiling when a feature is enabled
#[cfg_attr(feature = "my_profile_feature", profiled)]
fn expensive_calculation() { /* ... */ }

// Only profile in debug builds
#[cfg_attr(debug_assertions, profiled)]
fn complex_operation() { /* ... */ }
```

### Profiling Tools

Thag includes three command-line tools for working with profiles:

#### 1. thag-instrument

Automatically adds profiling attributes to your code:

```bash
thag-instrument [options] <path>
```

Options:
- `-r, --recursive`: Process directories recursively
- `-e, --edition <year>`: Specify Rust edition (2015, 2018, 2021)

#### 2. thag-remove

Removes profiling attributes from your code:

```bash
thag-remove [options] <path>
```

Options:
- `-r, --recursive`: Process directories recursively
- `-e, --edition <year>`: Specify Rust edition (2015, 2018, 2021)

#### 3. thag-analyze

Interactive analysis of profiling results:

```bash
thag-analyze [options]
```

Options:
- `-f, --file <path>`: Analyze a specific profile file
- `-i, --interactive`: Start in interactive mode (default)

## Profile Analysis Features

The analyzer provides:

1. **Statistical Summary**: Shows function calls, total time, average time
2. **Interactive Flamechart**: Visual representation of performance data
3. **Differential Analysis**: Compare before/after optimizations
4. **Memory Allocation Tracking**: Identify memory usage patterns

### Flamecharts

Flamecharts provide an intuitive visualization of your profiling data. The wider a function appears, the more time it takes relative to the total execution.

Flamecharts are interactive SVGs that allow you to:
- Zoom in on specific functions
- Hover over functions to see detailed information
- Search for specific functions
- Compare before/after optimizations

![Example Flamechart](https://raw.githubusercontent.com/yourusername/thag_profiler/main/assets/flamechart_example.png)

## Best Practices

1. **Profile representative workloads**: Make sure your test cases represent real-world usage
2. **Focus on hot paths**: Look for the widest blocks in your flamechart - these are your performance bottlenecks
3. **Compare before/after**: Always compare profiles before and after optimization
4. **Watch for memory bloat**: Use memory profiling to identify excessive allocations
5. **Verify changes**: Always verify automated changes with a diff tool

## Testing with Profiled Code

When writing tests that use profiled functions, use the `serial_test` crate:

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_profiled_function() {
    // Tests using profiled functions
}
```

This is important because `thag_profiler` maintains global state that isn't thread-safe.

## Troubleshooting

### Common Issues

1. **Missing profile output**: Ensure profiling is enabled and you have write permissions
2. **Test failures**: Profiled tests must use the `#[serial]` attribute
3. **Performance impact**: Memory profiling adds some overhead
4. **File redirect issues**: Never redirect output back to the input file

### Inspecting Profile Files

The folded stack files are human-readable:

```bash
head your-executable-timestamp.folded
```

## License

SPDX-License-Identifier: Apache-2.0 OR MIT

Licensed under either of

    Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

or

    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

as you prefer.
`
## Contributing

Contributions will be considered (under MIT/Apache 2 license) if they align with the aims of the project.
