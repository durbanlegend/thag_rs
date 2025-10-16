# thag_profiler

A profiling library for Rust applications that combines time and memory profiling with interactive visualization and cross-platform async support.

thag_profiler is an independent offshoot of the thag(_rs) script runner and REPL.

It aims to lower the barriers to profiling by providing a comprehensive solution that is easy to use and interpret.

[![Hot flamechart](https://durbanlegend.github.io/thag_rs/thag_profiler/assets/flamechart_hot_20250519-155436.png)](https://durbanlegend.github.io/thag_rs/thag_profiler/assets/flamechart_hot_20250519-155436.svg)<br>
*Time profile flamechart with interactive visualization. Click image for full interactive version with clickable bars and search functionality.*

---

[![Memory flamegraph](https://durbanlegend.github.io/thag_rs/thag_profiler/assets/flamegraph_mem_20250518-220050.png)](https://durbanlegend.github.io/thag_rs/thag_profiler/assets/flamegraph_mem_20250518-220050.svg)<br>
*Memory allocation flamegraph with filtered view showing detailed breakdown. Click image for full interactive version.*

---

## thag_profiler Design Goals

`thag_profiler` addresses common profiling challenges with a unified approach:

- **Unified time and memory profiling** - Profile both execution time and memory allocations in a single tool
- **Async-first design** - Full support for async/await code without instrumentation complexity
- **Memory profiling precision** - Allocations are internally tracked by source line number and ring-fenced from profiler code.
- **Interactive visualization** - Intuitive flamegraphs and flamecharts that reveal performance patterns at a glance
- **Cross-platform consistency** - Reliable profiling across macOS, Linux, and Windows
- **Minimal instrumentation overhead** - Simple attributes that don't obscure your code

## Key Features

### Profiling Capabilities
- **Time profiling**: Execution time measurement with low overhead
- **Memory profiling**: Allocation tracking with line-level precision
- **Combined profiling**: Simultaneous time and memory analysis
- **Section profiling**: Profile specific code blocks within functions
- **Async compatibility**: Full support for async/await code

### Ease of Use
- **Zero-overhead when disabled**: No runtime cost when profiling features are disabled
- **Attribute-based instrumentation**: Simple `#[profiled]` annotations
- **Automatic instrumentation tools**: Bulk add/remove profiling with `thag_instrument` and `thag_uninstrument`
- **Runtime configuration**: Enable/disable profiling without recompilation

### Analysis and Visualization
- **Interactive flamegraphs**: Clickable, searchable performance visualization
- **Flamecharts**: Time-ordered execution analysis
- **Function statistics**: Detailed performance metrics
- **Memory allocation analysis**: Size distribution and allocation patterns
- **Filtering capabilities**: Focus on specific code sections

## Installation

Add `thag_profiler` to your `Cargo.toml`:

```toml
[dependencies]
thag_profiler = "0.1.0"
```

Enable profiling features as needed:

```toml
# For time profiling
thag_profiler = { version = "0.1.0", features = ["time_profiling"] }

# For comprehensive profiling (time + memory)
thag_profiler = { version = "0.1.0", features = ["full_profiling"] }
```

Install analysis tools:

```bash
cargo install thag_profiler --features=tools
```

## Quick Start

### Basic Function Profiling

Sync and async functions are annotated exactly the same way:

```rust
use thag_profiler::{enable_profiling, profiled};

#[enable_profiling]
fn main() {
    process_data();
    expensive_calculation();
}

#[profiled]
fn process_data() {
    // Your code here
}

#[profiled]
fn expensive_calculation() -> u64 {
    // CPU-intensive work
    42
}

#[profiled]
async fn fetch_and_process() -> Result<String, Error> {
    let data = fetch_data().await?;
    process_data(data).await
}

#[profiled]
async fn fetch_data() -> Result<Vec<u8>, Error> {
    // Async I/O operations
    Ok(vec![])
}
```

### Memory Profiling

```rust
// Track memory allocations
#[profiled(mem_summary)]
fn allocating_function() {
    let data = vec![0; 1000];
    let more_data = HashMap::new();
    // Memory usage is tracked
}

// Detailed memory tracking
#[profiled(mem_detail)]
fn detailed_memory_analysis() {
    // Individual allocations and deallocations tracked
}
```

### Section Profiling

```rust
use thag_profiler::{profile, end};

#[profiled]
fn complex_operation() {
    // Setup code

    profile!(database_query);
    let results = expensive_database_query();
    end!(database_query);

    profile!(data_processing);
    process_results(results);
    end!(data_processing);
}
```

## Running and Analysis

### Enable Profiling

Configure profiling in your `Cargo.toml`:

```toml
[dependencies]
thag_profiler = { version = "0.1.0", features = ["full_profiling"] }
```

Or enable via command line:

```bash
cargo run --features thag_profiler/full_profiling
```

### Run Your Application

```bash
cargo run
```

This generates `.folded` stack files in the current directory.

### Analyze Results

```bash
thag_profile .
```

This launches an interactive menu to:
- Select profiling data files
- Generate flamegraphs and flamecharts
- View function statistics
- Analyze memory allocation patterns
- Filter and compare results

## Advanced Usage

### Runtime Configuration

Control profiling behavior at runtime:

```rust
#[enable_profiling(runtime)]
fn main() {
    // Profiling controlled by THAG_PROFILER environment variable
}
```

```bash
# Enable both time and memory profiling
THAG_PROFILER=both cargo run

# Memory profiling only
THAG_PROFILER=memory cargo run

# Time profiling with custom output directory
THAG_PROFILER=time,/tmp/profiles cargo run
```

### Profiling Options

Fine-tune profiling behavior:

```rust
#[profiled(time)]           // Time profiling only
#[profiled(mem_summary)]    // Memory allocation summary
#[profiled(mem_detail)]     // Detailed memory tracking
#[profiled(both)]           // Time and memory profiling
```

### Async Best Practices

For async functions with section profiling:

```rust
#[profiled]
async fn async_operation() {
    profile!(database_query, async_fn);
    let result = query_database().await;
    end!(database_query);

    profile!(data_processing, async_fn);
    process_data(result).await;
    end!(data_processing);
}
```

## Memory Profiling Capabilities

### Summary Mode
Tracks allocation counts and total sizes per function - efficient for identifying memory hotspots.

### Detail Mode
Tracks individual allocations and deallocations with full stack traces - comprehensive but with higher overhead.

### Global Profiling
Track all allocations across your entire program:

```rust
#[enable_profiling(memory, function(mem_detail))]
fn main() {
    // All allocations in your program and dependencies are tracked
}
```

## Analysis Tools

### thag_profile - Interactive Analysis
- Select from available profiling data
- Generate flamegraphs and flamecharts
- View function statistics and memory reports
- Filter results to focus on specific areas
- Compare before/after performance

### thag_instrument - Automatic Instrumentation
Bulk add profiling attributes to source files:

```bash
thag_instrument 2021 < input.rs > output.rs
```

### thag_uninstrument - Remove Instrumentation
Clean removal of profiling attributes:

```bash
thag_uninstrument 2021 < input.rs > output.rs
```

## Cross-Platform Support

`thag_profiler` provides consistent profiling across platforms:

- **macOS**: Full time and memory profiling support
- **Linux**: Complete feature set with high precision
- **Windows**: Time profiling and memory profiling with debug symbols

### Windows Configuration

For memory profiling on Windows, ensure debug information is available:

```toml
[profile.release]
debug = true
strip = false
```

## Performance Considerations

- **Minimal overhead**: Time profiling adds low to insignificant overhead in most cases
- **Memory tracking cost**: Summary mode has significant impact; detail mode has substantial overhead
- **Async efficiency**: No special async overhead beyond standard profiling costs
- **Zero cost when disabled**: No runtime impact when profiling features are disabled

## Output and Visualization

### File Naming Convention
Generated files use descriptive names:
- `myapp-20240101-120000-time.folded`
- `myapp-20240101-120000-memory-summary.folded`
- `myapp-20240101-120000-memory-detail.folded`

### Flamegraph Features
- **Interactive navigation**: Click to zoom, search functions
- **Color coding**: Different colors for time vs memory profiles
- **Filtering**: Remove unwanted functions from display
- **Comparison mode**: Side-by-side before/after analysis

## Troubleshooting

### Common Issues

**No profiling output generated:**
- Verify profiling features are enabled in `Cargo.toml`
- Ensure `#[enable_profiling]` is present on main function
- Check that profiling is enabled at runtime

**Missing symbols in output:**
- Enable debug information: `debug = true` in `Cargo.toml`
- Ensure functions aren't optimized away
- Use `strip = false` for release builds

**Memory profiling issues:**
- Use `full_profiling` feature for memory tracking
- Enable debug symbols on Windows
- Consider using summary mode for better performance

### Debug Logging

Enable detailed logging for troubleshooting:

```toml
[dependencies]
thag_profiler = { version = "0.1.0", features = ["full_profiling", "debug_logging"] }
```

## Comparison with Other Tools

While several profiling tools exist for Rust, `thag_profiler` offers unique advantages:

- **Unified approach**: Combines time and memory profiling in one tool
- **Async support**: Full compatibility with async/await code
- **Interactive visualization**: Intuitive flamegraphs that reveal patterns immediately
- **Cross-platform consistency**: Reliable behavior across operating systems
- **Minimal instrumentation**: Simple attributes that don't clutter code

## Contributing

Contributions are welcome! Please ensure:
- All tests pass
- Code follows existing style
- Documentation is updated for new features
- Cross-platform compatibility is maintained

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
