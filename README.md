# rs-script

[![Crates.io](https://img.shields.io/crates/v/rs-script.svg)](https://crates.io/crates/rs-script)
[![Documentation](https://docs.rs/rs-script/badge.svg)](https://docs.rs/\n)
[![Build Status](https://github.com/durbanlegend/rs-script/workflows/CI/badge.svg)](https://github.com/durbanlegend/rs-script/actions)

## Overview

`rs-script` is a simple and serious script runner and REPL for Rust expressions, snippets, and programs. This tool allows you to quickly run and test Rust code from the command line for rapid prototyping and exploration. It is intended to handle cases that are beyond the scope of the Rust playground or the average script runner.

`rs-script` is Cargo-based and will prefer tools like syn, quote and toml over string operations where possible, in order to ensure correctness. It attempts to handle any valid Rust program, snippet or expression. It will usually manage to generate a dedicated Cargo.toml for your script from "use" statements in your code, although for speed and precision it is recommended that you embed your own in a /*[toml] */ block at the start of the script, as in most of the demos. The Cargo search will helpfully print the equivalent block for you to copy and paste if you want to.

`rs-script` aims to be as comprehensive as possible without sacrificing speed and simplicity. It uses timestamps to rerun compiled scripts without unnecessary rebuilding, although you can override this behaviour. For example, a precompiled script will calculate the 35,661-digit factorial of 10,000 in under half a second on my M1 Macbook Air.

### Why `rs-script`?
As so often happens, this project arose out of need. Initially I was looking for a hosted version of the Rust playground to allow me to try out new ideas quickly. This soon led me to the various script runners, but I found that what they ran more than anything was "out of steam". I even went so far as to fork the idiosyncratic but versatile `runner` crate with extensive modification to bring it up to date and attempt to resolve some tricky dependency issues, but I saw that by staying in the Cargo mainstream I could easily overcome these issues and leverage Cargo to do most of the hard work, so I started `rs-script` from scratch.

I also found that `rs-script` started to "write itself", by allowing me to experiment wiht promising crates before incorporationg them as dependencies. I don't know what the level of interest may be for a tool like this, but I hope you may find it as useful as I do.

## Installation

### Minimum Supported Rust Version
The minimum supported Rust version (MSRV) for rs-script is 1.74.1.

### TODO >>>
You can install `rs-script` using `cargo install`:

```bash
cargo install rs-script
```
### TODO >>> Installing the starter kit (demo directory)


## Usage
Once installed, you can use rs-script from the command line. rs-script uses the clap crate to process command-line arguments including --help.

### TODO >>>
Here are some examples:

### Evaluating an expression
#### Quickly calculate the factorial of a number up to 34 (overflows beyond that, but see demos for bigger numbers):
```bash
rs-script -e '(1..=34).product::<u128>()'
```

#### Shoehorn a script into an expression, should the need ever arise!:
```bash
rs-script -e "$(cat demo/fizz_buzz.rs)"
```

#### Run a script quietly except for timings
```bash
rs-script -tq demo/fizz_buzz.rs
1
2
fizz
4
buzz
fizz
7
8
fizz
buzz
...
fizz
94
buzz
fizz
97
98
fizz
Completed run in 0.14s
rs_script completed processing script fizz_buzz.rs in 0.15s

```

### Using the REPL
```bash
rs-script -l
```
This will start an interactive REPL session where you can enter or paste in a single- or multi-line Rust expression and press Enter to run it. You can also retrieve and optionally modify and expression from history.
Having evaluated the expression you may choose to edit it, and / or the generated Cargo.toml, in your preferred editor (VS Code, Helix, Zed, nano...) and rerun it. The REPL also offers a few houskeeping functions for the temporary files generated, otherwise being in temporary space they will be housekept by the operating system in due course.

#### Revisiting a REPL expression from a previous session
```bash
rs-script -l repl_nnnnnn.rs
```
will return to edit and run a named generated script from a previous REPL session.

More informally, the last 25 previous REPL expressions can be accessed from within the REPL function just by using the up and down arrow keys to navigate history from the `eval` command.

#### General notes on REPL
All REPL files are created under the rs_repl subdirectory of your temporary directory (e.g. $TMPDIR in *nixes, and referenced as std::env::temp_dir() in Rust) so as not to clog up your system. Before they are harvested by the OS you can display the locations and copy the files if desired.

The REPL feature, in particular the most convenient `eval` mode, is not suited to scripts of over about 1K characters, due to the limitations of the underlying line editor. These limitations can be overcome by using the `edit` mode instead, but by this point it is probably more convenient just to use the --stdin / -s feature instead, or save the source in a .rs file and run it from the command line.

## Features

_Rust is primarily an expression language.... In contrast, statements serve mostly to contain and explicitly sequence expression evaluation._

_— The Rust Reference_

* Runs serious Rust scripts (not just the "Hello, world!" variety) with no need to create a project.
* Aims to be the most capable and reliable script runner.
* Crucially, specific features of dependencies may be specified, giving your scripts access to advanced functionality. Local path and git dependencies may also be specified, allowing you to access your unpublished crates.
* A choice of modes - bearing in mind the importance of expressions in Rust:
    * expression mode for small, basic expressions on the fly.
    * REPL mode offers interactivity, and accepts multi-line expressions since it respects matching braces, brackets, parens and quotes.
    * stdin mode accepts larger scripts or programs on the fly, which need not be expressions as such. Being stdin it can be used with piped input.
    * edit mode adds basic TUI (terminal user interface) editing-in-place to stdin mode.
    * the classic script mode runs any .rs file consisting of a valid Rust script or program.
* You may develop a module of a project individually by giving it its own main method and embedded Cargo dependencies and running it from rs-script.
* You can use a shebang to write scripts in Rust.
* Supports a personal library of code samples for reuse. The starter set provided includes multiple examples from popular crates, as well as original examples including fast factorial and Fibonacci calculation with big-integer support, light-dark theme detection, TUI editing and colour support.
* Automatic support for light or dark backgrounds and a 16- or 256- colour palette for different message types, according to terminal capability. `rs-script` defaults to basic ANSI-16 colours and dark mode support on Windows for reasons beyond my control, but the dark mode colours it uses should also work well with most light modes.

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

Contributions will be given due consideration if they fit the goals of the project. Please see CONTRIBUTING.md for more details.

## Of possible interest: AI

I made extensive use of free versions of LLMs - mostly ChatGPT and to a lesser extent Gemini - for three aspects of this project:
* problem solving
* guidance on best practices
* grunt work of generating "first-approximation" code and boilerplate to spec.

Although these LLMs could be very hit-and-miss or clichéd when it comes to specifics and to received wisdom, my experience has been that intensive dialogues with the LLMs have generally either led them to produce worthwhile solutions, or at least led me to see that there were sometimes deeper-seated issues that AI couldn't solve and to dig deeper researching on my own.

I short I found using AI hugely beneficial in terms not only of productivity but of extending the scope of work that I could comfortably take on. I didn't use any licensed or integrated features and at this stage I'm not feeling the lack of same.

## License

Licensed under either of

Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you will be dual-licensed as above, without any additional terms or conditions.
