# thag_profiler

A profiling library for Rust applications that provides time and memory profiling with minimal instrumentation overhead.

## Features

- **Zero-overhead when disabled**: No runtime cost when profiling features are disabled
- **Time profiling**: Execution time measurement with low overhead
- **Memory profiling**: Allocation tracking at line-level precision
- **Section profiling**: Profile specific code blocks within functions
- **Async support**: Works with async/await code
- **Automatic instrumentation**: Tools for bulk adding/removing profiling annotations
- **Interactive visualization**: Flamegraphs and flamecharts via `inferno`
- **Cross-platform**: Supports macOS, Linux, and Windows
- **Proc macro based**: Simple attribute-based interface

## Installation

Add `thag_profiler` to your `Cargo.toml`:

```toml
[dependencies]
thag_profiler = "0.1.0"
```

For time profiling, enable the `time_profiling` feature:

```toml
[dependencies]
thag_profiler = { version = "0.1.0", features = ["time_profiling"] }
```

For memory profiling, enable the `full_profiling` feature:

```toml
[dependencies]
thag_profiler = { version = "0.1.0", features = ["full_profiling"] }
```

Install analysis tools:

```bash
cargo install thag_profiler --no-default-features --features=tools
```

## Quick Start

### 1. Basic Usage

Add profiling to your code:

```rust
use thag_profiler::{enable_profiling, profiled};

#[enable_profiling]
fn main() {
    expensive_calculation();
    process_data();
}

#[profiled]
fn expensive_calculation() -> u64 {
    // Your code here
    42
}

#[profiled]
fn process_data() {
    // Your code here
}
```

### 2. Enable Profiling Features

Configure your `Cargo.toml` to enable profiling:

```toml
[dependencies]
thag_profiler = { version = "0.1.0", features = ["time_profiling"] }
```

Or enable via command line:

```bash
cargo run --features thag_profiler/time_profiling
```

### 3. Run Your Application

```bash
cargo run
```

This generates `.folded` files in the current directory.

### 4. Analyze Results

```bash
thag_profile .
```

This opens an interactive menu to explore profiling data and generate flamegraphs.

## Detailed Usage

### Function Profiling

Add the `#[profiled]` attribute to functions you want to profile:

```rust
#[profiled]
fn expensive_function() -> u64 {
    // Function implementation
}

#[profiled]
async fn async_function() -> Result<String, Error> {
    // Async implementation
}
```

### Section Profiling

Profile specific code sections within functions:

```rust
use thag_profiler::{profile, end};

#[profiled]
fn complex_operation() {
    // Setup code

    profile!(expensive_part);
    // Expensive operation
    expensive_computation();
    end!(expensive_part);

    // Cleanup code
}
```

### Profiling Options

The `#[profiled]` attribute accepts several options:

```rust
#[profiled(time)]           // Time profiling only
#[profiled(mem_summary)]    // Memory summary
#[profiled(mem_detail)]     // Detailed memory tracking
#[profiled(both)]           // Time and memory
```

### Enable Profiling Options

The `#[enable_profiling]` attribute configures program-wide profiling:

```rust
#[enable_profiling]         // Default: follows feature configuration
#[enable_profiling(time)]   // Time profiling only
#[enable_profiling(memory)] // Memory profiling only
#[enable_profiling(both)]   // Time and memory profiling
#[enable_profiling(no)]     // Disable profiling
```

### Runtime Configuration

Use `#[enable_profiling(runtime)]` for runtime control via environment variables:

```rust
#[enable_profiling(runtime)]
fn main() {
    // Your code
}
```

Set the `THAG_PROFILER` environment variable:

```bash
THAG_PROFILER=both cargo run           # Time and memory
THAG_PROFILER=time cargo run           # Time only
THAG_PROFILER=memory cargo run         # Memory only
THAG_PROFILER=none cargo run           # Disabled
```

## Memory Profiling

### Summary Mode

Tracks allocation counts and sizes per function:

```rust
#[profiled(mem_summary)]
fn allocating_function() {
    let data = vec![0; 1000];
    // Function implementation
}
```

### Detailed Mode

Tracks individual allocations and deallocations:

```rust
#[profiled(mem_detail)]
fn detailed_function() {
    let data = vec![0; 1000];
    // Function implementation
}
```

### Global Memory Profiling

Track all allocations across the entire program:

```rust
#[enable_profiling(memory, function(mem_detail))]
fn main() {
    // All allocations will be tracked
}
```

## Analysis Tools

### thag_profile

Interactive analysis tool for exploring profiling data:

```bash
thag_profile <output_directory>
```

Features:
- Interactive file selection
- Flamegraph and flamechart generation
- Function statistics
- Memory allocation analysis
- Graph filtering

### thag_instrument

Automatically add profiling attributes to source files:

```bash
thag_instrument 2021 < input.rs > output.rs
```

### thag_uninstrument

Remove profiling attributes from source files:

```bash
thag_uninstrument 2021 < input.rs > output.rs
```

## Advanced Configuration

### Custom Features

Create custom profiling features in your `Cargo.toml`:

```toml
[dependencies]
thag_profiler = "0.1.0"

[features]
my_profiling = ["thag_profiler/full_profiling"]
default = ["my_profiling"]
```

### Release Build Profiling

Enable debug information in release builds:

```toml
[profile.release]
debug = true
strip = false
```

### Async Best Practices

For async functions with section profiling, use the `async_fn` flag:

```rust
#[profiled]
async fn async_operation() {
    profile!(database_query, async_fn);
    let result = query_database().await;
    end!(database_query);
}
```

## Output Files

Profiling generates `.folded` files with naming convention:

- `<program>-<timestamp>-<type>.folded`
- `<program>-<timestamp>-<type>-<mode>.folded`

Examples:
- `myapp-20240101-120000-time.folded`
- `myapp-20240101-120000-memory-summary.folded`
- `myapp-20240101-120000-memory-detail.folded`

## Troubleshooting

### Common Issues

**No profiling output generated:**
- Ensure profiling features are enabled
- Check that `#[enable_profiling]` is present
- Verify feature configuration in `Cargo.toml`

**Missing symbols in flamegraphs:**
- Enable debug information in release builds
- Check that functions are not inlined

**Memory profiling not working on Windows:**
- Ensure debug information is enabled
- Use the `full_profiling` feature

### Debug Logging

Enable debug logging with the `debug_logging` feature:

```toml
[dependencies]
thag_profiler = { version = "0.1.0", features = ["full_profiling", "debug_logging"] }
```

## Performance Considerations

- Time profiling adds minimal overhead when enabled
- Memory profiling has higher overhead than time profiling
- Detailed memory profiling can be slow with many allocations
- Use summary mode for initial analysis, detail mode for specific hotspots

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

## Contributing

Contributions are welcome. Please ensure all tests pass and follow the existing code style.
