# rs-script

[![Crates.io](https://img.shields.io/crates/v/rs-script.svg)](https://crates.io/crates/rs-script)
[![Documentation](https://docs.rs/rs-script/badge.svg)](https://docs.rs/\n)
[![Build Status](https://github.com/durbanlegend/rs-script/workflows/CI/badge.svg)](https://github.com/durbanlegend/rs-script/actions)

## Overview

`rs-script` is a script runner and REPL for Rust expressions, snippets, and programs. This tool allows you to quickly run and test Rust code from the command line, making it an excellent choice for rapid prototyping and learning.

`rs-script` is Cargo-based. It attempts to handle any valid program, snippet or expression. It will usually manage to generate a dedicated Cargo.toml for this script from use statements in your code, or you can embed your own in a /*[toml] */ block at the start of the script.
`rs-script` aims to be as comprehensive as possible without sacrificing speed and simplicity. It uses timestamps to rerun compiled scripts without unnecesssary rebuilding, although this behaviour may be overridden. Thus a precompiled script will calculate the 35,661-digit factorial of 10,000 in under half a second on my M1 Macbook Air.

## Installation

You can install `rs-script` using `cargo install`:

```bash
cargo install rs-script
```

## Usage
Once installed, you can use rs-script from the command line. rs-script uses the clap crate to process command-line arguments including --help.
Here are some examples:

### Running a script
```bash
rs-script [OPTIONS] path/to/your_script.rs
```

### Using the REPL
```bash
rs-script -l
```
This will start an interactive REPL session where you can enter, paste, or modify from history a single- or multi-line Rust expression and press Enter to run it. You can then edit the expression or the generated Cargo.toml in your preferred editor and rerun, among other useful functions.

```bash
rs-script -l repl_nnnnnn.rs
```
will return to edit and run a named generated script from a previous REPL session.

All REPL files are created on $TMP_DIR so as not to clog up your system, but before they are harvested you can display the locations and copy them if desired.

## Features
* TODO Evaluate Rust expressions on the fly.
* Run entire Rust scripts without creating a project.
* Generally faster and more versatile than the Rust playground
* Crucially, dependency features may be specified, allowing advanced functionality in scripts.
* Interactive REPL mode for rapid experimentation.
* Use a shebang to write scripts in Rust
* Support a personal library of code samples for reuse. Starter set includes multiple examples from popular crates, as well as original examples including fast factorial and Fibonacci calculation with big-integer support, light-dark theme detection, TUI editing and colour support.

## Platform Support
This crate is designed to be cross-platform and supports:

* MacOS: Tested on MacOS Sonoma.
* Linux: Tested on Zorin and (WSL2) Ubuntu.
* Windows: Tested on Windows 11:
    - PowerShell 5 and CMD under Windows Terminal and Windows Console
    - WSL2

## Related projects

(With acknowledgements to rust-script)

* evcxr - Rust REPL
* rust-script - Rust script runner
* cargo-script - the unmaintained project that rust-script was forked from.
*cargo-eval - maintained fork of cargo-script.
* cargo-play - local Rust playground.
* runner - tool for running Rust snippets without Cargo. I have an updated fork of this.
* cargo-script-mvs - RFC demo

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for more details.

## License

Licensed under either of

Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual-licensed as above, without any additional terms or conditions.
