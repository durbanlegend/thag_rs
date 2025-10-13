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

### ⏱️ Time Profiling
```bash
thag_demo time-profiling
```
Execution time profiling of nested functions with automatic flamegraph generation and browser visualization.

#### Demo

[![asciicast](https://asciinema.org/a/yafJCMFioVa6vur4Pdk52ZG2A.svg)](https://asciinema.org/a/yafJCMFioVa6vur4Pdk52ZG2A)

*Click to watch: The demo shows building and running an instrumented program, then generating an interactive flamegraph*

[![Flamegraph from thag_demo](https://durbanlegend.github.io/thag_rs/thag_demo/assets/time_profiling_demo.png)](https://durbanlegend.github.io/thag_rs/thag_demo/assets/time_profiling_demo.svg)<br>
*Interactive flamegraph showing execution time across nested function calls. Click image for interactive version with clickable bars and search.*

### 🧠 Memory Profiling
```bash
thag_demo memory-profiling
```
Explore memory allocation tracking, heap analysis, and memory flamegraphs.

### 🤹‍♂️ Async Profiling
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

### 🏁 Comprehensive Benchmark
```bash
thag_demo benchmark
```
Run a full-featured benchmark showcasing all profiling capabilities.

### 🧭 Interactive Demo Browser
```bash
thag_demo browse
```
Browse and run demo scripts interactively with filtering and search.

#### Demo

[![asciicast](https://asciinema.org/a/3TgTf3w3O57Zr7G6GYUuwlq4y.svg)](https://asciinema.org/a/3TgTf3w3O57Zr7G6GYUuwlq4y)

*Click to watch: The demo shows running `thag_demo browse` to view all the `thag` scripts in $THAG_DEMO_DIR, and selecting and running the `ratatui` showcase demo. It also highlights the use of `thag_styling` integrations to automatically theme both the `inquire` selection list and the `ratatui` showcase screen with the current `catppuccin-mocha` theme.*

##### Detailed steps

 - Invoke thag_demo with the browse option.

 - Inquire crate displays themed scrollable list of thag demo scripts.

 - Type rata to narrow the search on the fly.

 - Arrow down to select ratatui_theming_showcase script.

 - Enter to invoke run options for the script.

 - Final Enter builds script if needed and runs it.

 - Demo shows `ratatui` showcase app, themed by thag_styling with the terminal - ’s current catppuccin-mocha theme, as specified by $THAG_THEME.

 - Show 1st panel of app with progress bar responding to keys or mouse.  - Dashed border effect in some areas is an artefact of the video player.

 - Show pop-up help.

 - Show remaining 3 panels in turn.

 - Enter q to return to thag_demo browse.

 - Esc to end thag_demo browse.

### 📋 List All Scripts
```bash
thag_demo list-scripts
```
See all available demo scripts with descriptions and categories.

### ⚙️ Demo Directory Management
```bash
thag_demo manage
```
Download, update, or manage the demo script collection.

### 📝 Custom Scripts
```bash
thag_demo script <script_name>
```
Run any script from the thag_rs demo collection.

## Features

- **Zero installation friction** - One command to install and run
- **Interactive demo browser** - Browse 330+ demo scripts with filtering and search
- **Automatic demo directory management** - Downloads demo collection as needed
- **Interactive examples** - Each demo explains what it's doing
- **Progressive complexity** - Start simple, work up to advanced features
- **Visual output** - Generates flamegraphs and performance visualizations
- **Educational** - Learn profiling techniques through practical examples
- **Comprehensive** - Covers time, memory, async, and differential profiling
- **Smart discovery** - Finds demo directory in multiple standard locations

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
2. **Manages demo collection** - Automatically downloads 330+ demo scripts
3. **Provides interactive browsing** - Filter and search through demos by name/category
4. **Configures thag_rs** - Pre-configured for optimal demo experience
5. **Provides guidance** - Explains what each demo teaches
6. **Generates artifacts** - Creates flamegraphs and profile data you can explore

### Demo Directory Discovery

The tool automatically searches for demo scripts in multiple locations:

- Sibling to thag_demo installation

- `~/.thag/demo` (standard user location)

- `$THAG_DEMO_DIR` environment variable

- `./demo` in current directory

If not found, it offers to download the demo collection using `thag_get_demo_dir`.

## Interactive Commands

### Browse Demos Interactively
```bash
thag_demo browse
```
- Filter demos by typing part of the name
- Navigate with arrow keys
- See descriptions and categories inline
- Run demos directly from the browser

### Manage Demo Directory
```bash
thag_demo manage
```
- Download the demo collection if not present
- Update existing demo collection
- View directory information and statistics
- Browse demos interactively

### List All Available Scripts
```bash
thag_demo list-scripts
```
- Shows both built-in demos and script demos
- Displays descriptions and categories
- Provides usage examples

## Requirements

- Rust toolchain (stable)
- Internet connection (for demo directory download)
- Git (for downloading demo collection)

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

Contributions will be considered (under MIT/Apache 2 license) if they align with the aims of the project.

Rust code should pass clippy::pedantic checks.
