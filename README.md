# rs-script

[![Crates.io](https://img.shields.io/crates/v/rs-script.svg)](https://crates.io/crates/rs-script)
[![Documentation](https://docs.rs/rs-script/badge.svg)](https://docs.rs/\n)
[![Build Status](https://github.com/durbanlegend/rs-script/workflows/CI/badge.svg)](https://github.com/durbanlegend/rs-script/actions)

## Intro

`rs-script` is a versatile script runner and REPL for Rust expressions, snippets, and programs. It's a developer tool that allows you to run and test Rust code from the command line for rapid prototyping and exploration. It aims to handle cases that are beyond the scope of the Rust playground or the average script runner, while hopefully being simple and convenient to use.

`rs-script` includes a demo library of over 150 sample scripts. If you've got something good to share, do feel free to offer it, subject to the MIT / Apache 2 licence terms.

## Quick start: ways to run `rs-script`

### * With an expression argument:

```bash
rs_script --expr '"Hello world!"'                                   # Short form: -e
```
```bash
rs_script -e ' {
use jiff::{Zoned, Unit};
Zoned::now().round(Unit::Second)?
}'                                                                  # Long form: --expr
```

### * With a script:

```bash
rs_script demo/iced_tour.rs
```

### * As a REPL:

```bash
rs_script --repl                                                    # Short form: -l
```
![Repl](repl.png)

The REPL has file-backed history and access to graphical and text-based editors such as VS Code, Zed, Helix, Vim, Nano etc. if its `reedline` editor falls short for a particular task.

### * With standard input:

```bash
echo "(1..=10).product::<u32>()" | rs_script --stdin                # Short form: -s
```

### * With a TUI (Terminal User Interface) editor

```bash
rs_script --edit                                                    # Short form: -d
```
![Editor](edit1.png)

![Edit run](edit2.png)

### * With standard input into the TUI editor:

```bash
cat my_file.rs | rs_script --edit                                   # Short form: -d
```

This allows you to edit or append to the stdin input before submitting it to `rs-script`.

### * Getting started:

You have the choice of installing `rs-script` (recommended), or you may prefer to clone it and compile it yourself and run it via `cargo run`.

* Installing gives you speed out of the box and a simpler command-line interface without Cargo. You can download the demo library separately.
* Cloning gives you easy access to the demo scripts library and the opportunity to make local changes or a fork.

## Overview

`rs-script` leverages the reliability and comprehensiveness of Cargo, `syn`, `quote` and `cargo_toml` to build and compile a reliable Rust program from the input code, rather than relying on superficial source code analysis using regular expressions and string parsing.

`rs-script` uses the `syn` crate to parse valid code into an abstract syntax tree (AST). It then uses the `syn` visitor mechanism to traverse the AST to identify dependencies in the code and to determine well-formedness by counting the genuine main methods (as opposed to comments or program code embedded in string literals). These are then filtered to remove duplicates and false positives such as built-in Rust crates, renamed crates and local modules.

(If your code does not successfully parse into an AST because of a coding error, `rs-script` will fall back to using source code analysis to prepare your code for the Rust compiler, which can then show you error messages to help you find the issues.)

You may provide optional metadata in a toml block as described below. `rs-script` uses the `cargo_toml` crate to parse any metadata into a manifest struct, merges in any dependencies inferred from the AST, and then uses the `toml` crate to write out the dedicated Cargo.toml file that Cargo needs to build the script. Finally, in the case of snippets and expressions, it uses the `quote` crate to embed the logic in a well-formed program template, which it then invokes Cargo to build.

All of this is quite fast: the real bottleneck will be the familiar Cargo build process downloading and compiling your dependencies on the initial build. It will be displayed as it happens in the normal way so that there are no mystery delays. If you rerun the compiled script it should be lightning fast.

In this way `rs-script` attempts to handle any valid (or invalid) Rust script, be it a program, snippet or expression. It will try to generate a dedicated Cargo.toml for your script from `use` statements in your code, although for speed and precision I recommend that you embed your own in a toml block:
```/*
[toml]
[dependencies]
...
*/
```
at the start of the script, as you will see done in most of the demos. To assist with this, after each successful Cargo search `rs-script `will generate and print a basic toml block with the crate name and version under a `[dependencies]` header, for you to copy and paste into your script if you want to. It does not print a combined block, so it's up to you to merge all the dependencies into a single toml block. All dependencies can typically go under the single `[dependencies]` header in the toml block, but thanks to `cargo_toml` you can add other Cargo-compliant dependencies sections if you choose to do so.

`rs-script` aims to be as comprehensive as possible without sacrificing speed and simplicity. It uses timestamps to rerun compiled scripts without unnecessary rebuilding, although you can override this behaviour. For example, a precompiled script will calculate the 35,661-digit factorial of 10,000 in under half a second on my M1 MacBook Air.

### Why `rs-script`?
It's the old familiar story: I didn't find what I wanted so I built it myself. Initially I was looking for a hosted version of the Rust playground to allow me to try out new ideas as easily as possible. This soon led me to the various script runners, but I found that what they ran more than anything was "out of steam". Honorable mention to the idiosyncratic but versatile `runner` crate, which I ended up forking with some extensive modifications to bring it up to date and attempt to resolve some tricky dependency issues. However, issues such as conflicting dependencies affecting the `regex` crate made me realise that I could build something closer to my own mission without too much difficulty. `runner`'s premise of alternatives to Cargo was a worthy project in its own right, but all I wanted was a good script runner, and by staying in the Cargo mainstream I could easily overcome the issues I was encountering and leverage Cargo to do most of the hard work. So I started `rs-script` from scratch`.

Quite soon I found that `rs-script` started to "write itself", by allowing me to experiment with promising crates before incorporating them as dependencies.

I don't know what the level of interest may be for a tool like this, but I hope you may find it as useful as I do.

## Installation

### Minimum Supported Rust Version
The minimum supported Rust version (MSRV) for `rs-script` is 1.74.1.

### TODO >>>
You can install `rs-script` using `cargo install`:

```bash
cargo install rs-script
```
### TODO >>> Installing the starter kit (demo directory)


## Usage
Once installed, you can use the `rs_script` command (with underscore) from the command line. `rs-script` uses the clap crate to process command-line arguments including --help.

### TODO >>>
Here are some examples:

### Evaluating an expression
#### Concise fast factorial calculation for numbers up to 34 (it overflows beyond that, but see demos for bigger numbers):
```bash
rs-script -e '(1..=34).product::<u128>()'
```

#### Shoehorn a script into an expression, should the need ever arise!:
```bash
rs-script -e "$(cat demo/fizz_buzz.rs)"
```

#### Run a script in quiet mode but show timings
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
rs-script completed processing script fizz_buzz.rs in 0.15s

```

### Using the REPL
```bash
rs-script -l
```
This will start an interactive REPL session where you can enter or paste in a single- or multi-line Rust expression and press Enter to run it. You can also retrieve and optionally edit an expression from history.
Having evaluated the expression you may choose to edit it, and / or the generated Cargo.toml, in your preferred editor (VS Code, Helix, Zed, nano...) and rerun it. The REPL also offers basic housekeeping functions for the temporary files generated, otherwise being in temporary space they will be cleaned up by the operating system in due course.

#### Revisiting a REPL expression from a previous session
```bash
rs-script -l repl_<nnnnnn>.rs
```
will return to edit and run a named generated script from a previous REPL session.

More informally, you can access the last 25 previous REPL commands or expressions from within the REPL function just by using the up and down arrow keys to navigate history.

#### General notes on REPL
All REPL files are created under the `rs_repl` subdirectory of your temporary directory (e.g. $TMPDIR in *nixes, and referenced as std::env::temp_dir() in Rust) so as not to clog up your system. Until such time as they are harvested by the OS you can display the locations and copy the files if desired.

The REPL feature is not suited to scripts of over about 1K characters, due to the limitations of the underlying line editor. You can overcome these limitations by using the `edit` mode instead, but by this point it's probably more convenient just to use the --stdin / -s feature instead, or save the source in a .rs file and run it from the command line.

## Features

_Rust is primarily an expression language.... In contrast, statements serve mostly to contain and explicitly sequence expression evaluation._\
_— The Rust Reference_

* Runs serious Rust scripts (not just the "Hello, world!" variety) with no need to create a project.
* Aims to be the most capable and reliable script runner for Rust code.
* Crucially, specific features of dependencies may be specified, giving your scripts access to advanced functionality. Local path and git dependencies may also be specified, allowing you to access your unpublished crates.
* A choice of modes - bearing in mind the importance of expressions in Rust:
    * expression mode for small, basic expressions on the fly.
    * REPL mode offers interactivity, and accepts multi-line expressions since it respects matching braces, brackets, parens and quotes.
    * stdin mode accepts larger scripts or programs on the fly, which need not be expressions as such. Being stdin it can be used with piped input.
    * edit mode is stdin mode with the addition of basic TUI (terminal user interface) in-place editing.
    * the classic script mode runs any .rs file consisting of a valid Rust script or program.
* You can use a shebang to write scripts in Rust.
* You can even build your own commands, using the `--executable` (`-x`) option. This will compile a valid script to an executable command in the Cargo bin directory `<home>/.cargo/bin`.
* `rs-script` supports a personal library of code samples for reuse. The downloadable starter set in the demo subdirectory includes numerous examples from popular crates, as well as original examples including fast big-integer factorial and Fibonacci calculation and prototypes of TUI editing and of the adaptive colour palettes described below.
* Automatic support for light or dark backgrounds and a 16- or 256- colour palette for different message types, according to terminal capability. On Windows, `rs-script` defaults to basic ANSI-16 colours and dark mode support for reasons beyond my control, but the dark mode colours it uses have been chosen to work well with most light modes.
* In some cases you may be able to develop a module of a project individually by giving it its own main method and embedded Cargo dependencies and running it from rs-script. Failing that, you can always work on a minimally modified copy in another location. This approach allows you to develop and debug this functionality without having it break your project. For example the demo versions of colors.rs and stdin.rs were both prototypes that were fully developed as scripts before being merged into the main `rs-script` project.

## Platform Support
This crate is designed to be cross-platform and supports:

* MacOS: Tested on MacOS (M1) Sonoma.
* Linux: Tested on Zorin and (WSL2) Ubuntu.
* Windows: Tested on Windows 11:
    - PowerShell 5 and CMD under Windows Terminal and Windows Console
    - WSL2

GitHub actions test each commit on `ubuntu-latest`, `macos-latest` and `windows-latest`.

## Related projects

(With acknowledgements to the author of `rust-script`)

* `evcxr` - Perhaps the most well-known Rust REPL.
* `cargo-script` - Rust script runner (unmaintained project).
* `rust-script` - maintained fork of cargo-script.
* `cargo-eval` - maintained fork of cargo-script.
* `cargo-play` - local Rust playground.
* `irust` - limited Rust REPL.
* `runner` - experimental tool for running Rust snippets without Cargo, exploring dynamic vs static linking for speed. I have an extensively modified fork of this crate on GitHub, but I highly recommend using the current `rs-script` crate rather than that fork.
* `cargo-script-mvs` - RFC demo.

## Contributing

Contributions will be given due consideration if they fit the goals of the project. Please see CONTRIBUTING.md for more details.

## Of possible interest: AI

I made extensive use of free versions of LLMs - mainly ChatGPT and to a lesser extent Gemini - for four aspects of this project:
* problem solving
* suggestions and guidance on best practices
* generation of unit and integration tests
* grunt work of generating "first-approximation" code and boilerplate to spec.

Although these LLMs could be hit-and-miss or clichéd when it comes to specifics and to received wisdom, my experience has been that intensive dialogues with the LLMs have generally either led them to produce worthwhile solutions, or at least led me to see that there were sometimes deeper-seated issues that AI couldn't solve and to dig deeper researching on my own.

I short I found using AI hugely beneficial in terms not only of productivity but of extending the scope of work that I could comfortably take on. I didn't use any licensed or integrated features and at this stage I'm not feeling the lack of same.

## License

SPDX-License-Identifier: Apache-2.0 OR MIT

Licensed under either of

    Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

or

    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you will be dual-licensed as above, without any additional terms or conditions.
