# thag_demo

[![Crates.io](https://img.shields.io/crates/v/thag_demo.svg)](https://crates.io/crates/thag_demo)
[![Documentation](https://docs.rs/thag_demo/badge.svg)](https://docs.rs/thag_demo)

Interactive demos for `thag_rs` and `thag_profiler` - run Rust profiling examples without installing thag!

## Quick Start

**One-line demo experience:**

```bash
curl -sL https://raw.githubusercontent.com/durbanlegend/thag_rs/main/thag_demo/install_and_demo.sh | bash
```

**Or install manually:**

```bash
cargo install thag_demo
thag_demo --list
```

## What is thag_demo?

`thag_demo` is a lightweight facade over `thag_rs` that provides curated profiling examples and demonstrations. It's designed to let you explore the capabilities of `thag_profiler` without needing to install the full `thag_rs` toolkit.

## Available Demos

### 🔥 Basic Profiling
```bash
thag_demo basic-profiling
```
Learn the fundamentals of function timing and profiling with the `#[profiled]` attribute.

### 🧠 Memory Profiling
```bash
thag_demo memory-profiling
```
Explore memory allocation tracking, heap analysis, and memory flamegraphs.

### 🚀 Async Profiling
```bash
thag_demo async-profiling
```
Discover how to profile async functions, futures, and Tokio runtime integration.

### ⚖️ Performance Comparison
```bash
thag_demo comparison
```
See before/after performance analysis with differential profiling techniques.

### 📊 Interactive Flamegraphs
```bash
thag_demo flamegraph
```
Generate and understand interactive flamegraphs for visual performance analysis.

### 🏆 Comprehensive Benchmark
```bash
thag_demo benchmark
```
Run a full-featured benchmark showcasing all profiling capabilities.

### 📝 Custom Scripts
```bash
thag_demo script <script_name>
```
Run any script from the thag_rs demo collection.

## Features

- **Zero installation friction** - One command to install and run
- **Interactive examples** - Each demo explains what it's doing
- **Progressive complexity** - Start simple, work up to advanced features
- **Visual output** - Generates flamegraphs and performance visualizations
- **Educational** - Learn profiling techniques through practical examples
- **Comprehensive** - Covers time, memory, async, and differential profiling

## What You'll Learn

### Time Profiling
- Function-level timing with `#[profiled]`
- Flamegraph generation and interpretation
- Hotspot identification and analysis

### Memory Profiling
- Allocation tracking and visualization
- Memory leak detection
- Heap analysis and optimization

### Async Profiling
- Profiling async functions and futures
- Understanding async execution patterns
- Tokio runtime integration

### Advanced Features
- Differential profiling for before/after comparisons
- Custom profiling annotations
- Performance optimization techniques

## How It Works

`thag_demo` is a thin wrapper around `thag_rs` that:

1. **Bundles curated examples** - High-quality profiling demonstrations
2. **Configures thag_rs** - Pre-configured for optimal demo experience  
3. **Provides guidance** - Explains what each demo teaches
4. **Generates artifacts** - Creates flamegraphs and profile data you can explore

## Requirements

- Rust toolchain (stable)
- Internet connection (for initial install)

## Output Files

Each demo generates several files you can explore:

- `*.svg` - Interactive flamegraphs (open in browser)
- `*.folded` - Raw profile data for analysis
- Console output with explanations and tips

## Integration with thag_profiler

All demos use `thag_profiler` annotations:

```rust
use thag_profiler::{enable_profiling, profiled};

#[profiled]
fn my_function() {
    // Your code here
}

#[enable_profiling(time)]
fn main() {
    my_function();
}
```

## Next Steps

After running the demos:

1. **Explore the flamegraphs** - Open the `.svg` files in your browser
2. **Try thag_profiler** - Add profiling to your own projects
3. **Use thag_rs** - Install the full toolkit for script development
4. **Read the docs** - Check out the full documentation

## Resources

- [thag_profiler documentation](https://docs.rs/thag_profiler)
- [thag_rs repository](https://github.com/durbanlegend/thag_rs)
- [Profiling guide](https://github.com/durbanlegend/thag_rs/blob/main/thag_profiler/README.md)
- [More examples](https://github.com/durbanlegend/thag_rs/tree/main/demo)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.