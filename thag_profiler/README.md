# thag_profiler

A straightforward, accurate, lightweight cross-platform profiling library for Rust applications that provides both time and memory profiling.

`thag_profiler` aims to lower the barriers to profiling by offering a quick and easy tool that produces clear and accurate flamegraphs for both synchronous and asynchronous code.

`thag_profiler` provides an `#[enable_profiling]` attribute for your main method and a #`[profiled]` attribute for every function to be profiled.

`thag_profiler` provides an automated instrumentation tool `thag-instrument` to add the profiling attribute macros to all functions of a module, and a corresponding tool `thag-remove` to remove them after profiling.

It also provides a `profile!` macro, in combination with an optional `end` call,  allowing _time_ profiling of any desired code section(s) within a function, including nested or even overlapping sections.

`thag_profiler`'s easy-to-use prompted analysis tool, `thag-analyze`, uses the `inquire` crate to help you select output for analysis and optionally filter out any unwanted functions, and the `inferno` crate to display the results in your browser as interactive flamegraphs and flamecharts. For memory profiles you can also choose to display memory statistics and an allocation size analysis.


## Features

- **Zero-cost abstraction**: No runtime overhead when `thag_profiler`'s profiling features are disabled

- **Time and memory profiling**: Track execution time or memory usage, or both.

- **Single-attribute detailed memory profiling**: A deep dive into your app's complete memory allocations is possible simply by specifying `#[enable_profiling(runtime)]` in `fn main` and specifying detailed profiling via a `THAG_PROFILE` environment variable.

- **Function and section profiling**: Time profiling can be applied to any number of specific code sections, down to single instructions.

- **Async support**: Seamlessly works with `tokio` or other async code.

- **Automatic instrumentation**: Tools to quickly bulk add and remove profiling annotations to/from a source without losing comments or formatting.

- **Interactive flamegraphs and flamecharts**: Visualize performance bottlenecks and easily do before-and-after comparisons using `inferno` differential analysis.

- **Cross-platform**: Works on macOs, Linux and Windows.

## Installation

Add `thag_profiler` to your `Cargo.toml`:

```toml
[dependencies]
# For instrumentation only (default)
thag_profiler = "0.1.0"

# For time profiling only
thag_profiler = { version = "0.1.0", features = ["time_profiling"] }

# OR for comprehensive profiling (memory and optionally time)
thag_profiler = { version = "0.1.0", features = ["full_profiling"] }
```

Install the profiling tools:

```bash
# Install all tools
cargo install thag_profiler --no-default-features --features=tools

# Or install tools individually
cargo install thag_profiler --no-default-features --features=instrument-tool --bin thag-instrument
cargo install thag_profiler --no-default-features --features=instrument-tool --bin thag-remove
cargo install thag_profiler --no-default-features --features=analyze-tool --bin thag-analyze
```

## Quick Start

### 1. Instrument Your Code

#### a. Automatically instrument your code for profiling:

Replace `2021` with your project's Rust edition:

```bash
thag-instrument 2021 < path/to/your/file.rs > path/to/your/instrumented_file.rs
```

* Ensure your original source is backed up before instrumenting.

* Replace `2021` with your project's Rust edition.

* Do not redirect the output to your source file! Trust Thag on this!

* Compare generated code with original to ensure correctness before overwriting
any original code with instrumented code.

Repeat for all modules you want to profile.

####     ... AND / OR ...

#### b. Manually add profiling annotations:

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
For a section in a profiled async function, it's best to add `async_fn` as a second argument as described further below, to tie the section to the async function instance in the flamegraphs, otherwise the section causes the parent function to appear a second time in the flamegraph without its async identifier, as we have no way to link the two automatically.

### 2. Enable Profiling in Your Build

#### Manifest info

##### In scripts run with the `thag` script runner

When using `thag_profiler` in scripts, you have two options:

**1. Enable via command line** (recommended):

  ```bash
  cargo run bank/mem_prof.rs --features=my_profiling
  ```

  With this script configuration:

  ```toml
  /*[toml]
  [dependencies]
  thag_profiler = { version = "0.1" }

  [features]
  # For time profiling only
  my_profiling = ["thag_profiler/time_profiling"]

  # OR for comprehensive profiling (time + memory)
  my_profiling = ["thag_profiler/full_profiling"]
  ```

**OR**

**2. Enable directly in the dependency**:

```rust
/*[toml]
[dependencies]
thag_profiler = { version = "0.1", features = ["time_profiling"] }
*/
```

##### In Regular Cargo Projects

In standard Cargo projects, the same options apply, only directly in Cargo.toml:

**1. Use feature propagation**:
   ```toml
   [dependencies]
   thag_profiler = { version = "0.1" }

   [features]
   my_profiling = ["thag_profiler/time_profiling"]
   ```

   Then run with:
   ```bash
   cargo run --features my_profiling
   ```

**OR**

**2. Enable directly in the dependency**:
   ```toml
   [dependencies]
   thag_profiler = { version = "0.1", features = ["full_profiling"] }
   ```

#### In code

**EITHER**

**1. With an attribute (recommended)**

Enable profiling by adding the `#[enable_profiling]` attribute to your `main` function.

The attribute is recommended because it is the only way to obtain the zero-cost abstraction of unused profiling code, and to ensure that in the case of memory profiling, profiler code is excluded at compile time from interfering with the tracking of memory allocations and deallocations by profiled code.

NB: the `#[enable_profiling]` attribute also profiles the annotated function, so the `#[profiled]` attribute should not be specified on the same function.

**#[enable_profiling] arguments**
The following optional arguments are available:

- `both`: Specifies both time and memory profiling.

- `memory`: Specifies memory profiling only.

- `time`: Specifies time profiling only.

- `no`: Disables profiling.

- `yes`: (Default) Enables profiling according to the feature specified in the `thag_profiler` dependency, which must be either `full_profiling` or `time_profiling`.

- `runtime`: Specifies that a detailed specification will be provided via the `THAG_PROFILER` environment variable.

E.g.:

```rust
#[enable_profiling(memory)]
fn main() {
...
}
```

**Format of the `THAG_PROFILER` environment variable to be used with `#[enable_profiling(runtime)]`**

    THAG_PROFILER=<profile_type>,[<output_dir>],{<debug_level>],[<detail>]

    where `<profile_type>` = `both`, `memory` or `time`
          `<output_dir>` (optional) = output directory for `.folded` files.
          `<debug_level>` (optional) = `none` (default) - no debug log
                                       `announce` - display debug log path in user output
                                       `quiet` - log without displaying location.
                Debug log output will be written to `std::env::temp_dir()/thag_profiler`
                with the log name in the format `<program_stem>-yyyymmdd-HHmmss-debug.log`.
          `<detail>` (optional, for `memory` or `both` only) = `true` for detailed allocation and deallocation `.folded` file generation,
          otherwise `false` (default).

E.g.:

```bash
THAG_PROFILER=both,$TMPDIR,announce,true cargo run

Specifies both memory and time profiling, `.folded` files to $TMPDIR, debug log path to be written to user program output, extra `.folded` files for detailed memory allocations and deallocations required.


THAG_PROFILER=time cargo run

Specifies time profiling only, `.folded` files to current directory, no debug log, no detailed memory files as not applicable to time profiling.


THAG_PROFILE=memory,,quiet thag demo/document_pipeline_profile_minimal.rs  -ft

Runs `thag` demo script document_pipeline_profile_minimal.rs with forced rebuild (-f) and timings (-t),
memory profiling only, debug logging without announcing the log file path, and no detailed output `.folded` files.
```

The `main` function will be taken to be the root of the profiling callstack.

```rust
#[thag_profiler::enable_profiling]
fn main() {
    // Your program...
}
```

**OR**


**2. Programmatically**

This is not recommended as it cannot be as clean and efficient as an attribute macro
and lacks the same rich set of options.
Currently it is kept around as an adjunct to the attribute macro to allow turning profiling on and off on the fly - which itself is problematic with async code. Thag may yet decide to give it the boot, so please don't build your hopes and dreams on it.

```rust
use thag_profiler::{profiling::enable_profiling, ProfileType};

fn main() {
    // Enable both time and memory profiling
    enable_profiling(true, ProfileType::Both).expect("Failed to enable profiling");

    // Your program...
}
```

### 3. Run Your Application

### 4. Analyze Results

After running your application with profiling enabled, folded stack files will be generated in the current working directory, unless that location is overridden by the second argument of a `THAG_PROFILER` environment variable used in conjunction with `#[enable_profiling(runtime)]`.

Use the included analysis tool to visualize the results:

```bash
thag-analyze <output_dir>
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

The `#[profiled]` attribute supports a profile_type option:

```rust
// Override the profile type for a specific function (time, memory, or both)
#[profiled(profile_type = "both")]
fn allocating_function() { /* ... */ }
```

#### Order of attributes

If both `#[enable_profiling]` and `#[profiled]` attributes are used, they should be specified in that order.

```rust
#[enable_profiling]
#[profiled]
fn main() { /* ... */ }
```

If used to decorate a main function that has the attribute `#[tokio::main]`, they should come before `#[tokio::main]`.

```rust
#[enable_profiling]
#[profiled]
#[tokio::main]
async fn main() { /* ... */ }
```

### Controlling Profiling at Runtime

At least for the time being, you can programmatically control profiling with the `thag_profile::enable_profiling` and `disable_profiling` functions.

```rust
use thag_profiler::{disable_profiling, enable_profiling, ProfileType};

fn main() {
    // Enable profiling programmatically
    enable_profiling(true, Some(ProfileType::Time));

    // Run code with profiling...

    // Disable profiling for a section
    disable_profiling();
    run_unprofiled_section();

    // Re-enable for another section (profile type according to feature "full_profiling" or "time_profiling"))
    enable_profiling(true, None);
    run_profiled_section();
}
```
This should be straightforward for synchronous code, but be careful if doing this for async code, because the
`enable_profiling` and `disable_profiling` functions will respectively switch profiling on and off in real time for all
instrumented functions and sections in all threads, instead of only for child nodes in the abstract syntax tree of the
code, which is likely not what you want.

### Code Section Profiling with `profile!`

Use the `profile!` macro to profile specific sections of code:

```rust
use thag_profiler::profile;

fn complex_function() {
    // Basic usage
    let section = profile!("initialization");
    initialize_things();
    section.end();
}

// Profiling a section of an async function, provide an `async_fn` arg for accurate collation
asyn fn async_complex_function() {
    // Basic usage
    let section = profile!("initialization", async_fn);
    initialize_things();
    section.end();

}
```

### Nesting and Overlapping Section Profiles

Section profiles can be nested to track hierarchical operations:

```rust
use thag_profiler::profile;

fn complex_operation() {
    let meal_section = profile!("3_course_meal");

    let starter_section = profile!("starter");
    // Starter course code...
    starter_section.end();

    let mains_section = profile!("main_course");
    // Main course code...
    mains_section.end();

    let dessert_section = profile!("dessert");
    // Dessert course code...
    dessert_section.end();  // Optional if about to go out of scope anyway

    meal_section.end();  // Optional if about to go out of scope anywa
}
```

There is nothing preventing you from overlapping section profiles if you so desire. They will of course still appear separately in the detailed time flamechart in the order of completion.

### Conditional Profiling

You can conditionally enable profiling based on build configuration:

**1. Attribute macro example**

```rust
// Only apply profiling when a feature is enabled
#[cfg_attr(feature = "my_profile_feature", profiled)]
fn expensive_calculation() { /* ... */ }

// Only profile in debug builds
#[cfg_attr(debug_assertions, profiled)]
fn complex_operation() { /* ... */ }
```


**2. Declarative macro example**

```rust
fn process_data(data: &[u8]) {
    // Only include profiling in debug builds
    #[cfg(debug_assertions)]
    let process_section = profile!("process_data");

    // Your code here...

    #[cfg(debug_assertions)]
    process_section.end();

    ...
}
```

## How It Works

### Time Profiling

Time profiling measures the wall-clock time between profile creation and destruction. It has minimal overhead and is suitable for most performance investigations.

### Memory Profiling

`thag_profiler` memory profiling aims to provide a practical and convenient solution to memory profiling that is compatible with async operation.

Memory profiling (available via the `full_profiling` feature) accurately tracks every heap allocation and deallocation requested by profiled user code, including reallocations, using a global memory allocator in conjunction with attribute macros to exclude `thag_profiler`'s own code from interfering with the analysis. It uses the official Rust `backtrace` crate to identify the source of the allocation or deallocation request.

**Notes:** Memory profiling is about memory analysis, not about speed. `thag_profiler` memory profiling has distinctly higher overhead than time profiling and will
noticeably affect performance.
It's recommended to use it selectively for occasional health checks and targeted investigations in development rather than leave it enabled indefinitely.

While time profiling is fast, memory profiling is slower but richer in detail, and optionally fully detailed.

Memory profiling (the optional `full_profiling` feature) requires `thag_profiler` to use a custom global allocator for user code.

1. This is incompatible with specifying your own global allocator.

2. It is also incompatible with std::thread_local storage (TLS) in your code or its dependencies. You will know if you see an error: "fatal runtime error: the global allocator may not use TLS with destructors".

    This is a known issue with `async_std`, but not with its official replacement `smol`, nor with `tokio`.

### Detailed memory profiling with a single attribute

The combination of `#[enable_profiling(runtime)]` on `fn main` and the runtime environment `THAG_PROFILE=memory,<dir>,<log_level>,true` will accurately expose every run-time memory allocation and de-allocation in separate flamegraph (`.folded`) format files.

Obviously this is the slowest profiling option and may be prohibitively slow for some applications.

To mitigate this, `thag_profiler` provides a `SIZE_TRACKING_THRESHOLD=<bytes>` environment variable allowing you to track only individual allocations that exceed the specified threshold size (default value 0). This is obviously at the cost of accuracy, particularly if your app mainly does allocations below the threshold. To get a good idea of s suitable threshold, you can first do _detailed_ memory profiling (cancel if you need to once you see significant detailed output being generated) and select `Show Allocation Size Distribution` from the `thag-analyze` tool for the profile. This needs to be the detailed allocations `.folded` file because the normal memory profiling shows aggregated values per function rather than the detailed values being tracked.


### Profile Output

Profiles generate "folded" stack traces in the output directory:

- `your_program-<yyyymmdd>-<hhmmss>.folded`: Time profiling data

- `your_program-<yyyymmdd>-<hhmmss>-memory.folded`: Memory profiling data (if enabled)

These files can be visualized with the included analyzer or with tools like [Inferno](https://github.com/jonhoo/inferno).


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

* Do not redirect the output back to your source file in the same command! Trust Thag!

* In the case of `thag-remove`, you may need to remove the relevant imports manually.
`thag-remove` may leave the occasional trailing space and one or two blank lines at the very top of the file.

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

**1. Statistical Summary**: Shows function calls, total time, average time

**2. Interactive Flamegraphs and Flamecharts**: Visual representation of performance data, both cumulative and detailed

**3. Differential Analysis**: Compare before/after optimizations (cumulative) using `inferno` differential flamegraphs module.

**4. Memory Allocation Tracking**: Identify memory usage patterns

### Flamegraphs and Flamecharts

Cumulative flamegraphs and detailed flamecharts provide an intuitive interactive visualization of your profiling data. The wider a function appears, the more time (or allocated / deallocated memory) it represents relative to the total for the execution.

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

`thag` uses the `inferno` crate to generate flamegraphs and flamecharts.
For the execution timeline, the analysis tool allows you to choose the `inferno` color scheme to use.
For memory flamegraphs and flamecharts, it adheres to `inferno`'s memory-optimized color scheme.


## Best Practices

**1. Profile representative workloads**: Make sure your test cases represent real-world usage

**2. Focus on hot paths**: Look for the widest blocks in your flamechart - these are your performance bottlenecks

**3. Compare before/after**: Always compare profiles before and after optimization

**4. Watch for memory bloat**: Use memory profiling to identify excessive allocations

**5. Verify changes**: Always verify automated changes with a diff tool

## Testing with Profiled Code

When writing tests that use profiled functions, it's recommended to use a serialization mechanism. The `#[enable_profiling]` attribute and the current tests use the `thag_profiler::PROFILING_MUTEX` to ensure that only one instance runs at a time for thread safety:

TODO update:
```rust
use serial_test::serial;

#[test]
#[serial]
fn test_profiled_function() {
    // Tests using profiled functions
}
```

This is important because `thag_profiler` maintains some global state that isn't thread-safe (although this shouldn't affect async profiling per se).

## Troubleshooting

### Common Issues

**1. Missing profile output**: Ensure profiling is enabled and you have write permissions in the current directory

**2. Test failures**: Profiled tests must use serialization

**3. Performance impact**: Memory profiling adds significant overhead

**4. File redirect issues**: Never redirect output from the instrumentation tools back to the input file

### Inspecting Profile Files

The folded stack files are human-readable:

```bash
head your_executable-<yyyymmdd>-<hhmmss>.folded
```

## License

SPDX-License-Identifier: Apache-2.0 OR MIT

Licensed under either of

    Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

or

    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

as you prefer.

## Contributing

Contributions will be considered (under MIT/Apache 2 license) if they align with the aims of the project.
