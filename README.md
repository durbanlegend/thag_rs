# rs-script

[![Crates.io](https://img.shields.io/crates/v/rs-script.svg)](https://crates.io/crates/rs-script)
[![Documentation](https://docs.rs/rs-script/badge.svg)](https://docs.rs/\n)
[![Build Status](https://github.com/durbanlegend/rs-script/workflows/CI/badge.svg)](https://github.com/durbanlegend/rs-script/actions)

## Overview

`rs-script` is a simple and serious script runner and REPL for Rust expressions, snippets, and programs. This tool allows you to quickly run and test Rust code from the command line for rapid prototyping and learning. It is intended to handle cases that are beyond the scope of the Rust playground.

`rs-script` is Cargo-based. It attempts to handle any valid Rust program, snippet or expression. It will usually manage to generate a dedicated Cargo.toml for this script from "use" statements in your code, or for speed and precision you can embed your own in a /*[toml] */ block at the start of the script, as shown in numerous examples.

`rs-script` aims to be as comprehensive as possible without sacrificing speed and simplicity. It uses timestamps to rerun compiled scripts without unnecessary rebuilding, although this behaviour may be overridden. For example, a precompiled script will calculate the 35,661-digit factorial of 10,000 in under half a second on my M1 Macbook Air.

### Why `rs-script`?
As so often happens, this project arose out of need. Initially I was looking for a hosted version of the Rust playground to allow me to try out new ideas quickly. This soon led me to the various script runners, but I found that what they ran more than anything was "out of steam". I even went so far as to fork the idiosyncratic but versatile `runner` crate with extensive modification to bring it up to date and attempt to resolve some tricky dependency issues, but I saw that by staying in the Cargo mainstream I could easily overcome these issues and leverage Cargo to do most of the hard work, so I started `rs-script` from scratch.

I also found that `rs-script` started to "write itself", by allowing me to experiment wiht promising crates before incorporationg them as dependencies. I don't know what the market is for a tool like this, but I hope you may find it as useful as I do.

## Installation

### Minimum Supported Rust Version
The minimum supported Rust version (MSRV) for rs-script is 1.74.1.

You can install `rs-script` using `cargo install`:

```bash
cargo install rs-script
```

## Usage
Once installed, you can use rs-script from the command line. rs-script uses the clap crate to process command-line arguments including --help.

### TODO >>>
Here are some examples:

### Running a script
```bash
rs-script [OPTIONS] path/to/your_script.rs
```

### Using the REPL
```bash
rs-script -l
```
This will start an interactive REPL session where you can enter, paste, or modify from history, a single- or multi-line Rust expression and press Enter to run it. You can then edit the expression and / or the generated Cargo.toml in your preferred editor (VS Code, Helix, Zed, nano...) and rerun it. The REPL also offers a few houskeeping functions for the temporary files generated, otherwise being in temporary space they will be housekept by the operating system in due course.

#### Revisiting a REPL expression
```bash
rs-script -l repl_nnnnnn.rs
```
will return to edit and run a named generated script from a previous REPL session.

More informally, the last 25 previous REPL expressions can be accessed from within the REPL function just by using the up and down arrow keys to navigate history from the `eval` command.

#### General notes on REPL
All REPL files are created under the rs_repl subdirectory of your temporary directory (e.g. $TMPDIR in *nixes, and referenced as std::env::temp_dir() in Rust) so as not to clog up your system, but before they are harvested you can display the locations and copy them if desired.

The REPL feature, in particular the most convenient `eval` mode, is not suited to scripts of over about 1K characters, due to the limitations of the underlying line editor. These limitations can be overcome by using the `edit` mode instead, but by this point it is probably more convenient just to use the --stdin/-s feature instead or save the source in a .rs file and run it from the command line.

## Features
* Runs serious Rust scripts (not just the "Hello, world!" variety) with no need to create a project.
* Aims to be the most capable and reliable script runner.
* Crucially, specific features of dependencies may be specified, giving your scripts access to advanced functionality.
* A choice of modes:
    * expression mode for the smallest most basic expressions on the fly
    * REPL adds interactivity and a more convenient multi-line mode
    * stdin mode accepts larger scripts on the fly and provides basic TUI (terminal user interface) editing in place.
    * script mode runs any valid script or program in a .rs file.
* You may develop a module of a project individually by giving it its own main method and embedded Cargo dependencies and running it from rs-script.
* You can use a shebang to write scripts in Rust.
* Supports a personal library of code samples for reuse. The starter set provided includes multiple examples from popular crates, as well as original examples including fast factorial and Fibonacci calculation with big-integer support, light-dark theme detection, TUI editing and colour support.
* Automatic support for light or dark backgrounds and a 16- or 256- colour palette for different message types, according to terminal capability. Defaults to basic ANSI-16 colours and dark mode support on Windows for reasons beyond my control, but it defaults to dark mode colours that will also work well with light modes.

## Platform Support
This crate is designed to be cross-platform and supports:

* MacOS: Tested on MacOS (M1) Sonoma.
* Linux: Tested on Zorin and (WSL2) Ubuntu.
* Windows: Tested on Windows 11:
    - PowerShell 5 and CMD under Windows Terminal and Windows Console
    - WSL2

## Related projects

(With acknowledgements to the author of rust-script)

* evcxr - Perhaps the most well-known Rust REPL.
* cargo-script - Rust script runner (unmaintained project).
* rust-script - maintained fork of cargo-script.
* cargo-eval - maintained fork of cargo-script.
* cargo-play - local Rust playground.
* irust - limited Rust REPL.
* runner - experimental tool for running Rust snippets without Cargo, exploring dynamic vs static linking for speed. I have an extensively modified fork of this crate on Github, but I highly recommend using `rs-script` crate rather than that fork.
* cargo-script-mvs - RFC demo.

## Contributing

Contributions are welcome. Please see CONTRIBUTING.md for more details.

## License

Licensed under either of

Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual-licensed as above, without any additional terms or conditions.
