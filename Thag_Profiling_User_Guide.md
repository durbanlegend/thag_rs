# Profiling with thag_rs: A User Guide

## Introduction

  Profiling is an essential part of optimizing your Rust applications. With thag_rs, profiling becomes straightforward and consistent across all platforms. No more wrestling with platform-specific
  profiling tools or complex setup procedures.

  This guide explains how to use thag_rs's built-in profiling capabilities to identify performance bottlenecks, analyze memory usage, and optimize your code.

## Getting Started

### Enabling Profiling

  There are two ways to enable profiling in thag_rs:

  1. Via Cargo feature flag - Use this for occasional profiling during development:
  cargo run --features profiling -- your_script.rs
  2. Via #[enable_profiling] attribute - Use this to include profiling in release builds or for permanent instrumentation:

``` Rust
#[enable_profiling]
fn main() {
    // Your code here
  }
```

### Profiling Types

  thag_rs supports two types of profiling, separately or combined:

  - Time Profiling: Measures wall-clock execution time of functions
  - Memory Profiling: Tracks memory allocation and usage
  - Both: Combines time and memory profiling

  You can specify the profiling type when enabling profiling:

``` Rust
  // In your code
  profiling::enable_profiling(true, ProfileType::Both)?;
```

### Instrumenting Your Code

#### Automatic Instrumentation

  For easier profiling, thag_rs provides tools to automatically instrument your code:

##### Using the profile_instrument tool

  For existing source files, you can use the profile_instrument tool to automatically add profiling attributes:

  cargo run --bin profile_instrument -- path/to/your/source.rs

  This will add #[profile] attributes to functions and methods, and #[enable_profiling] to main() if present.

##### Removing Instrumentation

  When you're done profiling, you can remove the instrumentation:

  cargo run --bin profile_remove -- path/to/your/source.rs

#### Manual Instrumentation

##### Using the #[profile] Attribute

  You can add the #[profile] attribute to any function to profile it with a meaningful function or method name:

``` Rust
  use thag_proc_macros::profile;

  #[profile]
  fn expensive_calculation() -> u64 {
      // Your code here
  }

  // Also works with async functions!
  #[profile]
  async fn fetch_data() -> Result<String, Error> {
      // Async operations
  }
```

  For fine-grained control, you can manually profile specific functions or code sections:

  fn my_function() {
      // Profile the whole function
      profile_fn!("my_function");

      // Do some work...

      // Profile a specific section
      profile_section!("expensive_operation");
      for i in 0..1000 {
          // Expensive operation here
      }

      // Profile memory usage specifically
      profile_memory!("allocate_large_buffer");
      let buffer = vec![0; 1_000_000];

      // Profile both time and memory
      profile_both!("complex_operation");
      // Operation that's both CPU and memory intensive
  }

### Analyzing Profile Results

#### Profile Output

  Profiling generates folded stack files with timestamps in your current directory:

  - {executable_name}-{timestamp}.folded - For time profiling
  - {executable_name}-{timestamp}-memory.folded - For memory profiling

#### Using the Profiling Analyzer

  thag_rs includes a powerful analysis tool:

  cargo run --bin profile_analyze -- [options]

  Options include:
  - --file <path> - Specify the profile file to analyze
  - --filter <pattern> - Filter functions by pattern
  - --compare <path> - Compare with another profile file
  - --output <path> - Specify output path for flamechart

##### Interpreting Results

  The analyzer produces:

  1. Statistical Summary: Shows function calls, total time, average time
  2. Interactive Flamechart: Visual representation of performance data

##### Flamecharts

  Flamecharts provide an intuitive visualization of your profiling data.
  The wider a function appears, the more time it takes relative to the total execution.
  Flamecharts are interactive SVGs that allow you to zoom in on specific functions,
  hover over functions to see detailed information, search for specific functions,
  and compare before/after optimizations.

  You may be more familiar with flamegraphs than flamecharts. Flamecharts are distinguished by laying out data on the horizontal axis sequentially instead of alphabetically.
  Thag profiling uses flamecharts to reflect the sequence of events, in particular for the execution timeline. For memory profiling the sequence will be the sequence of `drop` events,
  since this is the point at which thag profiling records the allocation and deallocation.

`thag` uses the `inferno` crate to generate flamecharts. For the execution timeline, the analysis tool allows you to choose the `inferno` color scheme to use. For the memory flamechart,
it uses `inferno`'s memory-optimized color scheme.

 [Image: assets/timeline_flamechart.png](assets/timeline_flamechart.png)

  [Link: assets/flamechart.svg](assets/flamegraph.svg)

###### Interactive features of the SVG flamecharts include:

  - Zoom: Click on a function to zoom in
  - Details: Hover over a function to see detailed information
  - Search: Search for specific functions
  - Differential View: Compare before/after optimizations

##### Profiling Best Practices

  1. Profile Representative Workloads: Make sure your test cases represent real-world usage.
  2. Focus on Hot Paths: Look for the widest blocks in your flamechart - these are your performance bottlenecks.
  3. Compare Before/After: Always compare profiles before and after optimization to ensure you've made an improvement.
  4. Watch for Memory Bloat: Use memory profiling to identify excessive allocations.
  5. Before running or committing source code changes made by the automated tools, be sure to verify the changes with a diff tool.
  6. Use Serial Testing: When writing tests that use profiled functions, use the serial_test crate:

``` Rust
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_profiled_function() {
      // Tests using profiled functions
  }
```

  This is important because thag_rs profiling maintains global state that isn't thread-safe.

### Advanced Features

#### Profiling Async Code

  The #[profile] attribute works seamlessly with async functions:

``` Rust
  #[profile]
  async fn process_data() -> Result<Data, Error> {
      // Async operations
  }
```

  The profiler will correctly track time spent in the future, including time between .await points.

#### Ending Profile Sections Early

  Sometimes you may want to end profiling before a section's scope ends:

``` Rust
  fn complex_operation() {
      profile_section!("initialization");
      // Initialization code...

      if skip_rest {
          end_profile_section("initialization"); // End early
      }

      // More code...
  }
```

#### Custom Profile Names

  You can provide custom names for profiled methods:

``` Rust
  impl MyStruct {
      #[profile]
      fn process(&self) {
          // This will be profiled as "fn::process"
      }

      // With custom name
      fn calculate(&self) {
          profile_method!("MyStruct::custom_calculate");
          // This will be profiled as "MyStruct::custom_calculate"
      }
  }
```

### Troubleshooting

#### Common Issues

  1. Missing Profile Output: Ensure profiling is enabled and you have write permissions in the current directory.
  2. Test Failures: Profiled tests must use the #[serial] attribute from the serial_test crate to prevent concurrent access to profiling data.
  3. Overhead: Profiling adds some overhead. For extremely performance-sensitive code, be aware that the measurements include this overhead.

#### Inspecting Profile Files

  The folded stack files are human-readable. You can inspect them directly:

```bash
head your-executable-timestamp.folded
```
## Conclusion

  `thag_rs` profiling aims to provide a simple but effective cross-platform solution for understanding and optimizing your Rust code
  by combining easy instrumentation, detailed analysis, and interactive visualizations to helpmake your code faster and more efficient.

  You can get started quickly by running `thag`'s `profile_instrument` tool to auto-instrument one or more source files of interest with #[profile]
  attributes, and (in the case of `fn main`), #[enable_profiling] attributes. Then run your code as normal with the #[enable_profiling] attribute,
  or if running a script from `thag` you can use `features=profiling`, and on termination run the `profile_analyze` tool to select and analyze the profile data in the current directory.
