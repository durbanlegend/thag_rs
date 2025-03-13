# thag_profiler

A straightforward, lightweight profiling library for Rust applications that provides time and memory profiling with minimal overhead.

## Features

- **Zero-cost abstraction**: No runtime overhead when profiling is disabled
- **Time and memory profiling**: Track execution time or memory usage, or both
- **Function and section profiling**: Profile entire functions or specific code sections
- **Async support**: Seamlessly works with async code
- **Automatic instrumentation**: Tools to add and remove profiling code
- **Interactive glamegraphs and flamecharts**: Visualize performance bottlenecks
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

Automatically instrument your code for profiling:

Replace `2021` with your project's Rust edition:

```bash
thag-instrument 2021 < path/to/your/file.rs > path/to/your/instrumented_file.rs
```

* Ensure your original source is backed up before instrumenting.

* Replace `2021` with your project's Rust edition.

* Do not redirect the output to your source file! Trust Thag!

* Compare generated code with original to ensure correctness before

Repeat for all modules you want to profile.


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

#### Manifest info

##### In Scripts Run with thag

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

##### In Regular Cargo Projects

In standard Cargo projects, the same options apply, only directly in Cargo.toml:

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

#### In code

Enable profiling by adding the #[enable_profiling] attribute to your main function:

```rust
use thag_profiler::profiled;

#[enable_profiling]
fn main() {
    // Your program...
}
```

Or programmatically:

```rust
use thag_profiler::{profiling::enable_profiling, ProfileType};

fn main() {
    // Enable both time and memory profiling
    enable_profiling(true, ProfileType::Both).expect("Failed to enable profiling");

    // Your program...
}
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

The `#[profiled]` and `#[enable_profiling]` attributes support a profile_type option:

```rust
// Specify at the global level what to profile (time, memory, or both)
#[enable_profiling(profile_type = "memory")]
fn main() { /* ... */ }

// Override the profile type for a specific function (time, memory, or both)
#[profiled(profile_type = "both")]
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

Thag includes three command-line tools for working with profiles.

Please take care to back up and protect your code before instrumenting or removing instrumentation.
By using the tools you take full responsibility for any consequences.

#### Instrumentation: thag-instrument and thag-remove

Automatically add or remove profiling attributes to/from code, outputting to a different file.

These tools aim to be lossless, i.e. preserving comments and formatting intact. For this purpose
they rely on `rust-analyzer`'s `ra_ap_syntax` and `ra-ap-rustc_lexer` crates.

Input is from `stdin` and output is to `stdout`.

***NB NB NB*** always direct output to a different file.

Replace `2021` with your project's Rust edition (2015, 2018, 2021, 2024) as required by the `rust_analyzer` crates:

***thag-instrument:*** Add profiling attributes to code
```bash
thag-instrument 2021 < path/to/your/file.rs > path/to/your/instrumented_file.rs
```

***thag-remove:*** Remove profiling attributes from code
```bash
thag-remove 2021 < path/to/your/instrumented_file.rs > path/to/your/de-instrumented_file.rs
```

* Ensure your original source is safely backed up or committed before instrumenting.

* Replace `2021` with your project's Rust edition.

* Do not redirect the output to your source file! Trust Thag!

* In the case of thag-remove, you may need to remove the relevant imports manually.

* Compare the original and instrumented files to ensure correctness, especially if
you're using a custom edition.


  E.g.  Comparing before and after with `vimdiff`:

    ```
    vimdiff demo/factorial_ibig_product.rs demo/factorial_ibig_product_profile.rs
    ```

    ![vimdiff](../assets/vimdiff_profile_instrument.png)

    If you're profiling a project source file, at this point you'd want to replace the uninstrumented code with the instrumented version.


Repeat for all modules you want to profile.

#### 3. Analysis: thag-analyze

Interactive analysis of profiling results:

```bash
thag-analyze
```
![Main menu](../assets/thag-analyze_main.png)

***Important notice:***
By using the tools, you agree to the license terms. Take precautions not to overwrite your code when using the instrumenting tools.

[License reminder](../assets/dont_make_me_tap_the_sign.jpg)

## Profile Analysis Features

The analyzer provides:

1. **Statistical Summary**: Shows function calls, total time, average time
2. **Interactive Flamegraphs and Flamecharts**: Visual representation of performance data, both cumulative and detailed
3. **Differential Analysis**: Compare before/after optimizations (cumulative)
4. **Memory Allocation Tracking**: Identify memory usage patterns

### Flamegraphs and Flamecharts

Cumulative flamegraphs and detailed flamecharts provide an intuitive interactive visualization of your profiling data. The wider a function appears, the more time it takes relative to the total execution.

Flamegraphs and flamecharts are interactive SVGs that allow you to:

- Zoom in on specific functions

- Hover over functions to see detailed information

- Search for specific functions

- Compare before/after optimizations

![Example Flamechart](../assets/flamechart_time_20250312-081119.png)

You can interact with the above example [here](../assets/flamechart_time_20250312-081119.svg).

You may be more familiar with flamegraphs than flamecharts. Flamecharts are distinguished by laying out data on the horizontal axis chronologically instead of alphabetically.
Flamecharts provide a detailed view that reflects the sequence of events, in particular for the execution timeline. For memory profiling the sequence will be the sequence of `drop` events,
since this is the point at which `thag` profiling records the allocation and deallocation.

`thag` uses the `inferno` crate to generate flamecharts.
For the execution timeline, the analysis tool allows you to choose the `inferno` color scheme to use.
For the memory flamechart, it adheres to `inferno`'s memory-optimized color scheme.


## Best Practices

1. **Profile representative workloads**: Make sure your test cases represent real-world usage
2. **Focus on hot paths**: Look for the widest blocks in your flamechart - these are your performance bottlenecks
3. **Compare before/after**: Always compare profiles before and after optimization
4. **Watch for memory bloat**: Use memory profiling to identify excessive allocations
5. **Verify changes**: Always verify automated changes with a diff tool

## Testing with Profiled Code

TODO Re-check:
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

1. **Missing profile output**: Ensure profiling is enabled and you have write permissions in the current directory
2. **Test failures**: TODO confirm: Profiled tests must use the `#[serial]` attribute
3. **Performance impact**: Memory profiling adds some overhead
4. **File redirect issues**: Never redirect output from the instrumentation tools back to the input file

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
