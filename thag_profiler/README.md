# thag_profiler

A straightforward, accurate lightweight cross-platform profiling library for Rust applications that provides time and/or memory profiling.

`thag_profiler` aims to lower the barriers to profiling by offering a quick and easy tool that produces clear and accurate flamegraphs for both synchronous and asynchronous code.

`thag_profiler` provides an `#[enable_profiling]` attribute for your main method, a #`[profiled]` attribute for every function to be profiled, a combinination of `profile!`and optional `end!` macros for code sections, and a choice of detailed or summary memory profiling, allowing you to detect memory hotspots and break them out in detail.

`thag_profiler` provides an automated instrumentation tool `thag-instrument` to add the profiling attribute macros to all functions of a module, and a corresponding tool `thag-remove` to remove them after profiling.

Profiling options are highly configurable, with global runtime options as well as lower-level profile type overrides.

`thag_profiler`'s easy-to-use prompted analysis tool, `thag-analyze`, uses the `inquire` crate to help you select output for analysis and optionally filter out any unwanted functions, and the `inferno` crate to display the results in your browser as interactive flamegraphs and flamecharts. For memory profiles you can also choose to display memory statistics and an allocation size analysis.


## Features

- **Zero-cost abstraction**: No runtime overhead when `thag_profiler`'s profiling features are disabled

- **Execution time profiling**: Low-overhead profiling to highlight hotspots.

- **Accurate memory profiling**: Memory allocations are accurately tracked at line number level and ring-fenced from profiler code to avoid interference. They may be summarized by function or section, or broken out in detail where desired.

- **Function and section profiling**: Profiling can be applied to any number of specific code sections, down to single instructions.

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

Replace `2021` below with your project's Rust edition:

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

// Profile a specific section with `profile!` and matching `end!`
fn complex_operation() {
    // Some code...

    profile!("expensive_part");
    // Code to profile
    expensive_operation();
    end!("expensive_part");

    // More code...
}

// Profile a specific section of an async function
async fn complex_async_operation() {
    // Some code...

    profile!("expensive_part", async_fn);
    // Code to profile
    expensive_operation();
    end!("expensive_part");

    // More code...
}

// Profile the remainder of a function
fn complex_operation() {
    // Some code...

    // Must be scoped to end of function
    profile!("rest_of_function", unbounded);
    // All code to end of function will be profiled
}

// INCORRECT:
fn complex_operation() {
    // Some code...

    {
        // Unbounded keyword misused here
        profile!("rest_of_block", unbounded);
    }  // Profile will be dropped here unknown to allocation tracker

    // The following section profiling may not work correctly due to the above
    profile!("another_section");
    expensive_operation();
    end!("another_section");
}
```

For a section in a profiled async function, it's best to add `async_fn` as a second argument as described further below, to tie the section to the async function instance in the flamegraphs, otherwise the section causes the parent function to appear a second time in the flamegraph without its async identifier, as we have no way to link the two automatically.

### 2. Enable the Profiling Feature

The profiling feature must be enabled via manifest info, which may specify it as a default or require it to be specified at runtime.

#### In regular Cargo projects

You have two options:

  **1. Cargo.toml and command line**:

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

  **2. Cargo.toml only**:

    ```toml
    [dependencies]
    thag_profiler = { version = "0.1", features = ["full_profiling"] }
    ```

#### In scripts run with the `thag` script runner

When using `thag_profiler` in `thag` scripts, you have to same two options, only using a `toml` block in place of a `Cargo.toml`:

**1. Manifest and command line**:

  Sample script configuration:

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

  Then run with:

  ```bash
  cargo run bank/mem_prof.rs --features=my_profiling
  ```

**OR**

**2. Manifest only**:

  ```rust
  /*[toml]
  [dependencies]
  thag_profiler = { version = "0.1", features = ["time_profiling"] }
  */
  ```

### 3. Enable Profiling at Runtime

**EITHER**

**1. Enable profiling with an attribute (recommended)**

Enable profiling by adding the `#[enable_profiling]` attribute to your `main` function.

The attribute is recommended because it is the only way to obtain the zero-cost abstraction of unused profiling code, and because it ensures the clean compile-time ring-fencing of memory profiler code from profiled code.

NB: the `#[enable_profiling]` attribute also profiles the annotated function, so the `#[profiled]` attribute need not and should not be specified on the same function.

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

    where `<profile_type>`           = `both`, `memory` or `time`
          `<output_dir>` (optional)  = output directory for `.folded` files.
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


**2. Enable profiling programmatically**

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

### 4. Run Your Application

### 5. Analyze Results

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
#[profiled(both)]
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

1. **Easy to enable/disable profiling globally**: Developers can quickly turn on/off profiling without changing every profile section

2. **Clean code organization**: Section profiling clearly shows intent and what *could* be profiled, even if it's currently overridden

3. **Two-layer configuration**: Gives both fine-grained (per-section) and coarse-grained (global) control

4. **Simplicity**: No need for complex conditional logic in each profiling section

NB: Section profiling modes will be overridden by the program defaults set by `#[enable_profiling]`.

#### Format

```Rust
profile!("name" [, flag1, flag2, ...]);
```

Parameters

- **name**: A string literal or a `&str` variable that identifies the profiling section
- **flags**: Optional comma-separated identifiers that control profiling behavior

#### Available Flags

| Flag | Description |
|------|-------------|
| `time` | Enable time profiling for this section |
| `mem_summary` | Enable basic memory usage tracking |
| `mem_detail` | Enable detailed memory allocation tracking |
| `async_fn` | Mark that this profile is for an async function |
| `unbounded` | This is equivalent to an `end!` macro at the end of the function |

#### Profile Types

The macro automatically determines the type of profiling based on the flags provided:

- **Time only**: When only the `time` flag is present
- **Memory only**: When `mem_summary` or `mem_detail` is present without `time`
- **Both**: When `time` is combined with either `mem_summary` or `mem_detail`

#### Examples

```rust
// Basic time profiling
profile!("calculate_result", time);

// Memory usage summary
profile!("load_data", mem_summary);

// Detailed memory tracking
profile!("process_image", mem_detail);

// Both time and memory profiling
profile!("generate_report", time, mem_detail);

// Async function profiling
profile!("fetch_data", time, async_fn);

// Unbounded memory profile (must be manually ended)
profile!("long_running_task", mem_summary, unbounded);
```

#### Notes

- The macro captures source location information automatically for accurate profiling results

Section profiling requires either:

1. Recommended: An `end!(<identifier>)` macro to drop the profile outside of user code and to mark the end of the section so that memory allocations can be
accurately attributed to the correct section by line number. This macro invocation must not be outside the normal Rust scope of the `profile!` macro.

The `<identifier>` must be a string literal or &str value identical to that used in the matching `profile!` macro call.

OR:

2. An `unbounded` argument to allow the profile to be dropped at the end of the _function_ and to assist memory profiling. This is not preferred because:

a. The profile inevitably gets dropped in user code, leaving it up to the tracker to identify and filter out its allocations in the first place. This is not as clean and precise as using the `end!` mechanism, and thus creates more overhead and greater exposure to any potential loopholes in the filtering algorithm.

b. It has limited applicability and is open to misuse. It may only be used to profile the remainder of a function. For more limited scopes you must use an `end!` macro.

The 'unbounded` option may be dropped in future.

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
    profile!("process_data");

    // Your code here...

    #[cfg(debug_assertions)]
    end!("process_data");

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

### Memory Profiling Limitations and Considerations

- **Performance Impact**: Memory profiling introduces significant overhead compared to time profiling. Expect your application to run significantly more slowly when memory profiling is enabled.

- **Allocation Attribution**: Memory profiling attempts to attribute allocations to the correct task using stack traces, but in complex async code or highly concurrent applications, some
allocations may be attributed to parent tasks rather than to the exact function that requested them.

- **Thread-Safety Considerations**: Memory profiling uses global state protected by mutexes. While this works for most cases, extremely high-concurrency applications may experience contention.

- **Implementation Details**: Memory tracking is implemented using a global allocator that intercepts all memory allocations. This has several consequences:
  - Incompatible with custom global allocators
  - May experience issues with thread-local storage with destructors

 - **Complete Allocation Tracking**: All allocations, including those from libraries and dependencies, are tracked and included in profiling data. This provides a comprehensive view of memory usage
   across your entire application stack, revealing hidden costs from dependencies like async runtimes.

Detailed memory profiling will allow you to drill down into these allocations as well as the resulting deallocations.

### Windows Memory Profiling

For memory profiling on Windows, your application requires:

1. Debug information in the executable, which can be enabled with:

   ```toml
   [profile.release]
   debug = true
   strip = false
   ```

2. PDB files generated by the build must be distributed alongside the executable.
   These files contain the debug information needed for accurate profiling.

### Async Compatibility

`thag_profiler` supports profiling async code with some considerations:

- **Basic Time Profiling**: Works well with most async runtimes including tokio and smol.

- **Memory Profiling with Async**: Memory profiling in async contexts is more complex:
  - Works with tokio and smol for most common patterns
  - Not compatible with async_std due to TLS limitations
  - Task attribution may be less precise in highly concurrent async code
  - For best results in async code, use explicit section profiling with `profile!("section_name", async)`

- **Runtime Control**: Enabling/disabling profiling at runtime in async code affects all instrumented code across all threads, which may not align with the logical structure of async tasks. Plan
your profiling strategy accordingly.

### Implementation Details (For Advanced Users)

`thag_profiler` uses several internal mechanisms to track profiling data:

- **Task Tracking**: Memory profiling uses a task-based system to attribute allocations to the correct code path, even in async contexts.

- **Thread-Safety**: The profiler uses atomic operations and mutex-protected shared state to coordinate profiling across threads.

- **Guard Objects**: TaskGuard objects help manage the lifetime of profiling tasks and ensure proper cleanup when tasks complete.

- **Stack Introspection**: The profiler examines stack traces to attribute allocations to the correct task, using pattern matching and similarity scoring.

- **Profile Code Ring-Fencing**: The profiler carefully isolates its own allocations and operations from user code through the use of a dual-allocator system. This ensures that profiling overhead
  doesn't contaminate the results, providing clean separation between the measurement apparatus and the code being measured.

Note that deallocations are not reported for normal memory profiling, as they invite a fruitless attempt to identify memory leaks by matching them up by function against the allocations, whereas the deallocations are often done by a parent function. However, deallocations are reported for detailed memory profiling in order to give a complete picture, so this is a better tool for identifying memory leaks, although still not a walk in the park.

The reporting takes two forms:

a. For regular memory profiling, the allocations are accumulated to a mutex-protected collection with the key of the identified `task_id` which in turn is associated with a `Profile` that created it before the execution of the function began. When the function completes execution the `Profile` goes out of scope and is automatically dropped, and its `drop` trait method retrieves all the accumulated allocations for the associated `task_id` and writes them to the `-memory.folded` file.

b. For detailed memory profiling, allocations and deallocations alike are not accumulated or even tracked back to a `Profile`, but immediately written with a lightly tidied-up stack to the `-memory_detail.folded` and `-memory_detail_dealloc.folded` files respectively.

Being the default, this allocator is automatically used for user code and must not be used for profiler code.

To avoid getting caught up in the default mechanism and polluting the user allocation data with its own allocations, all of the profiler's own code that runs during memory profiling execution is passed directly to the untracked System allocator in a closure or function via a `with_allocator()` function (`pub fn with_allocator<T, F: FnOnce() -> T>(req_alloc: Allocator, f: F) -> T`).

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

`thag` uses the `inferno` crate to generate flamegraphs and flamecharts.
For the execution timeline, the analysis tool allows you to choose the `inferno` color scheme to use.
For memory flamegraphs and flamecharts, it adheres to `inferno`'s memory-optimized color scheme.


 ### Flamegraphs vs. Flamecharts

`thag_profiler` can generate both flamegraphs and flamecharts:

- **Flamegraphs** aggregate all executions of a function into one, making them ideal for identifying which functions consume the most resources overall. Use flamegraphs when you want to identify your application's hottest functions regardless of when they occur. Flamegraphs organize functions alphabetically, so unlike flamecharts there is no deep significance to the horizontal sequence of items - it is only the width and the parent-child relationships that are important.

- **Flamecharts** organize functions chronologically, showing the sequence of operations over time. They're particularly valuable for:
  - Understanding the progression of your application's execution
  - Identifying patterns in memory allocation/deallocation
  - Seeing how different phases of your application behave

For time profiling, flamecharts show when each function executed relative to others. For regular memory profiling, they are less significant because all allocations for a function are shown as at the end of execution of the function, because it is at this point that `thag_profiler` `Profile` object generated for that execution of the function is dropped and its `drop` method writes the function's accumulated allocations to the `-memory.folded` file.
For detailed memory profiling, they are again more significant as they show when the allocations (for `-memory_detail.folded` and deallocations (for `-memory_detail_dealloc.folded`) actually occurred, as they are recorded immediately the allocation or deallocation requests are received and identified by the global allocator.

Choose flamegraphs for a high-level view of resource usage and flamecharts for detailed analysis of execution flow.

## Best Practices

**1. Profile representative workloads**: Make sure your test cases represent real-world usage

**2. Focus on hot paths**: Look for the widest blocks in your flamechart - these are your performance bottlenecks

**3. Compare before/after**: Always compare profiles before and after optimization

**4. Watch for memory bloat**: Use memory profiling to identify excessive allocations

**5. Verify changes**: Always verify automated changes with a diff tool

**6. Don't run with option `both`, as the memory profiling overhead may distort the relative execution times of the functions and sections**

**7.Async Function Profiling**: For accurate callstack representation in async contexts, use the `async_fn` parameter when manually creating profile sections within async functions:

```rust
async fn fetch_data() {
    // Tell the profiler this section is within an async function
    profile!("database_query", async_fn);

    // Async operations...
    let result = query_database().await;

    end!("database_query");
}
```

This ensures the profile correctly associates the section with its async parent function in the profiling output. Without this parameter, the section might be incorrectly duplicated in memory
profiling output.

 ### Memory Profiling Best Practices

When using memory profiling, follow these guidelines for the most accurate results:

```rust
// Use attribute macros for consistent memory tracking
#[enable_profiling]
#[profiled]
fn main() {
    // Run your application...
    memory_intensive_function();
}

#[profiled]
fn memory_intensive_function() {
    // Memory will be automatically tracked for this function
    // and attributed to it in the profiling output

    // Create explicit scope for allocations to ensure
    // they're properly tracked and released
    {
        let data = vec![0u8; 1_000_000];
        process_data(&data);
    } // Memory is released here and recorded
}

For the most accurate memory profiling:

1. Use attribute macros consistently across your codebase
2. Create clear scopes for memory-intensive operations
3. Use thag-analyze's filtering to focus on relevant parts of your application
4. Consider enabling detailed memory profiling for full allocation visibility

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

**1. Missing profile output**: Ensure profiling is enabled and you have write permissions in the current directory.

Ensure your code is compiled with the `debug` option while profiling. E.g. in release mode:

```toml
[profile.release]
debug = true
strip = false
```

Ensure that unbounded section profiles do not go out of scope before the end of the current _function_.

Ensure that bounded section profiles do not go out of scope before the `end!` macro.

**2. Inaccurate profile output**: Ensure you have no nested or overlapping profile sections.

**3. Test failures**: Profiled tests must use serialization

**4. Performance impact**: Memory profiling adds significant overhead.

Consider using SIZE_TRACKING_THRESHOLD=n as discussed above to ignore small allocations of integer `n` bytes or smaller.

**5. File redirect issues**: Never redirect output from the instrumentation tools back to the input file

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
