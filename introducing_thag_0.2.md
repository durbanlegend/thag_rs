# Announcing thag 0.2: A versatile Rust REPL/script runner with dependency inference and TUI editing

As a veteran experimenter, it's always struck me as unfortunate to have to make a new Rust project for every little thing. I'm a fan of the [cargo-script initiative](https://internals.rust-lang.org/t/pre-rfc-cargo-script-for-everyone/18639), but in the mean time I want to slice and dice Rust, while making the most of the great Rust tooling that already exists. I threw my hat in the ring with thag 0.1 in 2024, and now, for those who might be interested in this kind of thing, I offer for your consideration the enhanced [thag 0.2](https://github.com/durbanlegend/thag_rs/blob/main/README.md) with theming goodness and a companion [profiler](https://github.com/durbanlegend/thag_rs/blob/main/thag_profiler/README.md) for good measure.

## What is thag?

thag is a Rust playground and REPL that aims to reduce the ceremony of running quick Rust experiments while still supporting full project complexity when needed.

## Why use lot word when demo do trick?

[Demo here (7 min)](https://asciinema.org/a/cIqUWnYBygGz2ZiEhGOfmSPMX)
*Recommended to watch in full-screen mode*

## Core features
- Run Rust programs, snippets and expressions without explicit Cargo.toml boilerplate.
- Automatic dependency inference, with the ability to configure default-feature overrides for any crate in your user preferences.
- Comprehensive, authentic Cargo.toml support for dependencies, features, profiles and lints when needed, via an embedded /*[toml] ... */ block.
- A built-in REPL with multi-line editing, history, TUI-editor and preferred-editor support, and ability to save expressions as functional thag scripts.
- The ability to execute scripts from URLs (useful for sharing examples).
- Common processing engine with multiple UIs - command-line expression, REPL expression, stdin, paste-and-run TUI editor or script.
- An option to build commands as release builds from scripts and even expressions.
- Suitable as a filter in *nix pipe / filter chains.
- An integrated TUI editor for quick modifications, integration with user's preferred editor
- 30+ command-line tools to execute scripts from URLs, expand scripts, run clippy or other cargo commands on them, copy and paste between the clipboard and standard input/output, convert embedded '\n' characters to line feeds, display terminal characteristics, palette and thag theming, apply thag theme to the terminal, generate custom error types, etc.
- Full support for proc macros and complex dependencies
- 330+ demo scripts and a proc macro starter kit.
- 290+ terminal themes (automated conversion from popular theme collections)

## Motivation

I wanted to be able to try out a piece of Rust logic quickly or build a small proof of concept with the minimum of fuss, and save it in a library for later, without having to build a Rust project for each one. Prior script runners and the Rust Playground solve part of this, but I wanted:
  - A better REPL experience.
  - Support for any and all depencencies.
  - Able to run crate examples without cloning the crates
  - A tool that would be reliable, flexible, fast and frictionless.
  - A minimum of manual dependency management - let the runner infer and build the Cargo.toml from the `use` statements etc. in the `syn` AST.
  - AST- and cargo_toml-based so as to be reliable and not tripped up by code in comments.
  - Cross-platform capability and minimal restrictions on the development environment, so it could be useful to others.
  - A development path from idea to expression to snippet to script to module.

## Example Usage

```bash
# Command-line expression
thag -e '(1..=34).product::<u128>()'

# Simple script
thag script.rs

# REPL mode
thag -r

# From URL
thag_url https://gist.github.com/user/abc123

# Paste-edit-submit cycle
thag -d

# Edit in TUI
thag -d < script.rs

# Run or evaluate from standard input
cat script.rs | thag -s

# From the system clipboard
thag_paste | thag -s

# Shebang support
./script.rs

# Command creation (release build to ~/.cargo/bin)
thag -x some_tool.rs
some_tool
```

[Include screenshot/demo of REPL or TUI here]

## Technical Details
- Uses standard tooling: cargo, syn, quote, serde etc.
- Uses a shared compilation cache to speed up rebuilds and builds with common dependencies.
- Infers dependencies by analyzing AST for imports and qualifiers.
- Supports embedded Cargo.toml via cargo_toml in script comments for complex cases
- Cross-platform: macOS, Linux, and Windows
  - (Contributed fixes to termbg and expander crates for Windows compatibility)

The project also includes two subcrates that may be independently useful:
- **thag_styling**: Terminal theme library and tools with 290+ themes converted from popular collections, for a beautiful cohesive appearance that automatically matches the user's terminal theme with guaranteed legibility and minimal effort. The developer does not need to specify colors and styles directly and worry about ensuring legibility on different light or dark backgrounds. Instead, messages are styled according to purpose (heading1, error, warning etc.) and theming is then automatic according to the end user's preferred theme. Verbosity gating is also built in.
- **thag_profiler**: Zero-overhead time and memory profiling toolkit with auto-instrumentation, async support and `inferno` flamegraph generation.

## Installation

```bash
cargo install thag_rs
```

Repository: https://github.com/durbanlegend/thag_rs

## Current Status

The tool is functional and I use it daily. Areas where feedback would be valuable:
1. Cases where dependency inference fails or guesses wrong
2. REPL UX suggestions (what's missing from your workflow?)
3. Integration experiences with existing tools
4. Feature requests or use cases I haven't considered

The project is MIT/Apache-2.0 licensed and contributions are welcome.

I built this primarily to solve my own workflow issues, but I hope others find it useful. Happy to answer questions about design decisions or implementation details.
```
