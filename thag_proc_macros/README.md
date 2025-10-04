# thag_proc_macros

[![Crates.io](https://img.shields.io/crates/v/thag_proc_macros.svg)](https://crates.io/crates/thag_proc_macros)
[![Documentation](https://docs.rs/thag_proc_macros/badge.svg)](https://docs.rs/thag_proc_macros)

Procedural macros for `thag_rs` and `thag_profiler`.

This is a proc macro crate that provides the macros used by `thag_profiler` for code profiling, along with utility macros used internally by other `thag_rs` subcrates. If you're using `thag_profiler` or `thag_rs`, you're already benefiting from these macros.

## What's Inside

`thag_proc_macros` provides:

- **Profiling Macros** - `#[profiled]`, `#[enable_profiling]`, `profile!`, and `end!` for performance profiling

- **Safe Print Macros** - Thread-safe printing macros for concurrent output

- **File Navigation** - `file_navigator!` macro for interactive directory traversal with `inquire`

- **Styling Macros** - Theme generation, palette methods, ANSI styling support, and compile-time theme loading

- **Category System** - `category_enum!` for organizing demo scripts

- **Internal Utilities** - Build-time code generation and utility macros

## Who This Is For

**Most users won't import this crate directly.** If you're using `thag_profiler`, you're already using these macros through that crate. If you're using `thag_rs`, the macros you need are re-exported.

You might want to use `thag_proc_macros` directly if you're:

- Performing directory navigation with `inquire`

- Building custom profiling integrations

- Developing new `thag_rs` subcrates that need the macro infrastructure

- Contributing to the thag ecosystem

## Usage

Add `thag_proc_macros` to your `Cargo.toml`:

```toml
[dependencies]
thag_proc_macros = "0.2"
```

### Example: Profiling Macros

The profiling macros are the most commonly used. They're typically imported through `thag_profiler`:

```rust
use thag_proc_macros::{enable_profiling, profiled, profile, end};

#[enable_profiling(time)]
fn main() {
    expensive_calculation();

    profile!(section_name);
    // Critical section to profile
    end!(section_name);
}

#[profiled]
fn expensive_calculation() -> u64 {
    // Function is automatically profiled
    42
}
```

See the [thag_profiler documentation](https://docs.rs/thag_profiler) for comprehensive profiling examples.

### Example: Safe Print Macros

Thread-safe printing for concurrent environments:

```rust
use thag_proc_macros::{safe_println, safe_eprintln};

fn main() {
    // Safe for concurrent use
    safe_println!("This prints safely from multiple threads");
    safe_eprintln!("Errors also print safely");
}
```

### Example: Category Enum

Generate enums from configuration files:

```rust
use thag_proc_macros::category_enum;

// Generates Category enum from config file
category_enum!("path/to/categories.toml");
```

### Example: File Navigator

The `file_navigator!` macro generates an interactive file browser that works with `inquire`. It's used extensively by thag's own tools for navigating directories to select files or save-locations:

```rust
use inquire;
use thag_proc_macros::file_navigator;

fn main() {
    // Generate the FileNavigator struct and implementation
    file_navigator!();

    // Navigate to select a .rs file
    let result = select_file(
        &mut FileNavigator::new(),
        Some("rs"),     // File extension filter
        false,          // Don't show hidden files
    );

    match result {
        Ok(path) => println!("Selected: {}", path.display()),
        Err(e) => eprintln!("\nError: {}", e),
    }
}
```

Output shows an interactive menu with the current directory structure, e.g.:

```
> *TYPE PATH TO NAVIGATE*
  ..
  📁 assets
  📁 bank
  📁 benches
  📁 built_in
v 📁 claude
[Press Enter to navigate, select a file to load]
```

Navigate with arrow keys, select with Enter. The macro handles the entire navigation UI.

## Features

### Default Features

None - include only what you need.

### Available Features

- **`time_profiling`** - Enable time-based profiling macros

- **`full_profiling`** - Enable comprehensive profiling (time + memory)

- **`debug_logging`** - Enable debug logging in profiling code

- **`tui`** - Enable TUI-related macros

- **`internal_docs`** - Show internal API documentation

Example with profiling features:

```toml
[dependencies]
thag_proc_macros = { version = "0.2", features = ["full_profiling"] }
```

## How It Works

As a proc macro crate, `thag_proc_macros` operates at compile time. The macros analyze and transform your code during compilation, generating optimized implementations that have zero runtime overhead when profiling features are disabled.

**Key Design Principles**:

- **Zero-cost abstractions** - No overhead when features are disabled

- **Compile-time safety** - Errors caught during compilation, not at runtime

- **Feature-gated** - Include only what you need

## Documentation

For detailed API documentation, see [docs.rs/thag_proc_macros](https://docs.rs/thag_proc_macros).

For profiling usage, see the [thag_profiler documentation](https://docs.rs/thag_profiler).

## Part of the thag Ecosystem

`thag_proc_macros` is one component of the larger `thag_rs` toolkit:

- **[thag_rs](https://crates.io/crates/thag_rs)** - The main script runner and REPL

- **[thag_profiler](https://crates.io/crates/thag_profiler)** - Lightweight code profiling (main user of these macros)

- **[thag_styling](https://crates.io/crates/thag_styling)** - Terminal styling with theme support

- **[thag_common](https://crates.io/crates/thag_common)** - Shared types and utilities

- **[thag_demo](https://crates.io/crates/thag_demo)** - Interactive demos

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions will be considered (under MIT/Apache 2 license) if they align with the aims of the project.

Rust code should pass clippy::pedantic checks.
