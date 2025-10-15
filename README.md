# thag_rs

[![Crates.io](https://img.shields.io/crates/v/thag_rs.svg)](https://crates.io/crates/thag_rs)
[![Crates.io size](https://img.shields.io/crates/size/thag_rs)](https://img.shields.io/crates/size/thag_rs)
[![Build Status](https://github.com/durbanlegend/thag_rs/actions/workflows/ci.yml/badge.svg)](https://github.com/durbanlegend/thag_rs/actions/workflows/ci.yml/badge.svg)
<img src="https://img.shields.io/badge/rustc-stable+-green.svg" alt="supported rustc stable" />
<a href="https://deps.rs/repo/github/durbanlegend/thag_rs"><img src="https://deps.rs/repo/github/durbanlegend/thag_rs/status.svg" alt="dependency status"/></a>

## Intro

***thag_rs*** (command ***thag***) is a robust toolkit and playground designed to make your Rust development experience smoother and more rewarding.

v0.2 brings many new features and a sleek look with themes to match your terminal preferences.

***thag*** includes:

  - a script runner

  - an expression evaluator

  - a REPL that also lets you save your code as a script

  - playground capability without limitations on dependencies.

  - an option to build fast commands from your scripts

<details>
<summary>📋 an improved demo proc macro starter kit   [demo/proc_macros](demo/proc_macros/README.md)</summary>

- a ***select set of 12 useful and educational examples***

- full documentation

- demo scripts

- a learning path

- compile-time macro expansion support for proc macro debugging
</details>

### New in `thag` v0.2:



<details>
<summary>📋 a command ***thag_url*** to intelligently run example scripts directly from URLs</summary>

  - Automatically resolves straightforward dependencies.

      <summary>📋 Supports popular repo, playground and other URLs.</summary>

      - GitHub

      - GitLab

      - BitBucket

      - Rust Playground

      - Straightforward URLs.
      </details>

</details>

- 34 other independently installable command-line tools for further analysing your scripts, such as showing macro expansions and cargo trees, and running clippy or tests against them. Details here: [thag tools](src/bin/README.md).

- A **massive improvement in space management** of compiled scripts through a shared build target architecture. All scripts now share a single build cache instead of maintaining separate `target` directories. This reduces bad-case disk usage by **97%** (from 6 GB+ to ~225 MB for ~340 scripts) in testing.

- It also provides **10-15x faster rebuilds** and a big speed-up when building scripts with shared dependencies, as dependencies are compiled once and reused across all scripts.

- The ***proc macro starter kit*** goes from being a rough sketchpad to a ***select set of useful and educational examples*** with individual demo scripts and a debugging expansion option. See its README file here: [demo/proc_macros](demo/proc_macros/README.md).

#### Companion Crates Released with v0.2:

We're also releasing two independent crates that complement thag_rs:

  - ***thag_profiler*** - A capable, easy-to-use graphical cross-platform profiler packaged as an independent lightweight library and tools. Features async support, time and memory profiling, interactive flamegraphs, and zero-cost abstraction. [Learn more →](thag_profiler/README.md)

  - ***thag_styling*** - A terminal styling library supporting 290+ popular themes across all major terminal emulators. Automatically adapts to your terminal's color scheme. Used by thag_rs and available for your own projects. [Learn more →](thag_styling/README.md)

The ***core purpose of `thag`*** is straightforward: to make it easy and fun to test ideas, explore software, or debug issues in Rust. It provides the tools you need to quickly experiment without the overhead of creating new projects or writing boilerplate.

`thag` is largely TOML-free thanks to its ***dependency inference***, which automatically detects and configures the crates your code needs.

When you do need TOML, `thag` supports a good subset - dependencies, features, profiles and lints - as well as intelligently generating TOML for you to embed if you choose.

Whether you're:

- Prototyping a concept

- Running crate examples without the need to install

- Using dependencies not supported by the Rust playground

- Debugging proc macros

- Running quick calculations

- Profiling performance of your project

- Converting your scripts into lightning-fast commands

`thag_rs` helps you get answers faster—from simple one-liners to complex multi-file programs.

___

🚀 **The basics:**

- Run Rust code straight from the command line or a URL

- Evaluate expressions on the fly

- Interactive REPL mode for rapid prototyping

- Uses AST analysis to understand your code

- "toml block" comments use `cargo_toml` to support not only dependencies but other valid manifest entries such as features, profiles and lints.

- Shebang support for true scripting (but `thag`'s command building is much better still!)

- Loop-filter mode for data processing

💡 **Smart Features:**

- Toml-free by default: dependency inference from imports and Rust paths (`x::y::z`).

  `thag` can run many crate examples without needing TOML information. The `thag_url` tool can run them directly URLs including GitHub, GitLab, BitBucket and Rust Playground.

  ```bash
  thag_url https://github.com/clap-rs/clap/blob/master/examples/demo.rs -- --name "is this the Krusty Krab?"

  thag_url https://github.com/mikaelmello/inquire/blob/main/inquire/examples/render_config.rs --features=inquire/date
  ```

  **New in 0.2.0:** Since dependency inference has proved fast and reliable in extensive use and testing, toml block content that is no longer necessary (that is, most of it) has been removed from most demo scripts with no noticeable loss of speed. This mostly toml-free scripting is easier and more productive and removes the burden of updating dependencies, at the cost of slightly greater exposure to incompatibilities between newer versions of dependencies.

- You're still in control: dependency inference (max/min/config/none) and/or toml block.

- Beyond shebangs: build instant commands from your snippets and programs.

- Playground features with no limit on dependencies:

  - REPL edit / run commands using your favourite editor.

  - REPL tui command using built-in TUI editor with Ctrl-D submit.

  - TUI edit-submit from command line with `thag -d`, which also accepts piped input.

- An evolution path for your code from REPL to edit-submit loop to saved scripts.

- Edit-submit standard input.

- Integrate your favourite editor (VS Code, Helix, Zed, vim, nano etc.).

- Run any Cargo command (clippy, tree, test) against your scripts.
(Yes, you can include unit tests in your scripts).

- View macro expansions side-by-side with your base script.

- Proc macro development support, including an improved documented [starter kit](demo/proc_macros/README.md) and an option to show an expanded view of your proc macro at compile time.

- Configurable automated inclusion of `derive` or other features in named dependencies.

- **New in 0.2.0:** Gorgeous coloration and styling across the board thanks to our companion crate, [thag_styling](thag_styling/README.md). `thag_styling` supports 290+ popular themes and automatically adapts based on a THAG_THEME environment variable or your terminal's background color. Also available for use in your own projects, with integrations for `console`, `crossterm`, `nu-ansi-term`, `owo-colors`, and `ratatui`.

![Catppuccin Mocha](assets/theme_catp_mocha.png)
![Gruvbox light, hard (base16)](assets/theme_gbox_lh.png)

- **New in 0.2.0:** An optional set of 35 [lightweight commands](src/bin/README.md) is now included, to handle such diverse tasks as downloading the demo directory, displaying ASTs, expanding macros in user code, generating error modules and types, verifying GitHub-compatible markdown, detecting terminal attributes, running remote scripts from sources like GitHub repos, managing thag themes and more.

💡 **Getting Started:**

Jump into `thag`'s collection of 330+ sample scripts in [demo/README.md](https://github.com/durbanlegend/thag_rs/blob/main/demo/README.md) to see what's possible. Suggestions and contributions welcome (under MIT/Apache 2 license) if they fit the goals of the project

## Installation

### Minimum supported Rust version
The minimum supported Rust version (MSRV) for the current version of `thag_rs` is 1.82.

### Installation options

#### Cargo install
You can install `thag_rs` using `cargo install`:

Full `thag` binary install with additional tools (recommended):

```bash
cargo install thag_rs --features tools
```

[The additional tools](src/bin/README.md)

`thag` binary install without tools:

```bash
cargo install thag_rs
```

On *nix systems you may need to install `OpenSSL` and/or `pkg-conf` for the `openssl-sys` transitive dependency. Refer to the [openssl crate](https://docs.rs/openssl/latest/openssl/) documentation.

For example, for Debian and Ubuntu:

```bash
$ sudo apt-get install pkg-config libssl-dev
```

#### Downloading the starter kit (demo directory)

As of 0.2.0, the `thag` tool `thag_get_demo_dir` does a fast download of the demo directory to a location of your choice. `thag` documentation assumes that demo scripts are located in a `demo` subdirectory of your current working directory, so this is recommended as the most convenient option.

As from `v0.1.1` you can download `demo.zip` from `https://github.com/durbanlegend/thag_rs/releases`.

Note that you can also link to individual demo files via their links in `demo/README.md` and manually download the file from the download icon provided.

## Quick start: ways to run the `thag` command

### * With an expression argument:

```bash
thag --expr '"Hello world!"'                                    # Short form: -e
```
![Expr](assets/hellot.png)

```bash
thag -qe u64::MAX                                               # Long form: --quiet --expr
```

Invoking quiet mode (`--quiet / -q`) suppresses most feedback except for the flowerbox to highlight the output.

Quickly calculating a percentage:

![Percentage](assets/percentage.png)

No quotes, no spaces, escape asterisks and parens, and remember Rust decimal arithmetic requires decimal points or floating point notation on all numbers.

Invoking `--quiet` twice (`-qq` for short) suppresses all non-error feedback including the flowerbox, allowing the
output to be used as a filter.

By default, `thag` and Cargo feed back to you:

```bash
thag -e ' {
use jiff::{Zoned, Unit};
Zoned::now().round(Unit::Second)?
}'                                                              # Long form: --expr
```
![Expr](assets/jifft.png)

### * With a script:

Here's a sample interactive script for discovering Pythagorean triangles with integer sides:

```bash
thag demo/py_thag.rs
```
![Script](assets/script1t.png)

```bash
thag demo/iced_tour.rs
```
This builds and runs the tour of the `iced` cross-platform GUI library. The script is taken from the published examples in the `iced` crate.

#### Using a shebang

To run a script directly from the command line, you can add a shebang of the form `#! /usr/bin/env thag` on line 1. You need to give the script execute permissions in order to run it. For example (*nix):

```bash
chmod ug+x demo/fib_basic.rs
demo/fib_basic.rs -- 10
```

This is useful, but a shebang still means building the script each time you use it.
Instead, you can simply compile it to a fast Rust command with the --executable (-x) option. See `As an executable` below.

### * As a REPL (Read-Evaluate-Print Loop):

```bash
thag --repl                                                     # Short form: -r
```

[![asciicast](https://asciinema.org/a/Ydpea3nitN5hzsnbByF7iqICT.svg)](https://asciinema.org/a/Ydpea3nitN5hzsnbByF7iqICT)

*Click to watch: Interactive REPL session showing expression evaluation, multi-line snippets, TUI integration, and editing workflow (2:17)*

(The orange flashes on the command line as I paste in input are caused by the `reedline` hinter helpfully matching the pasted input against history to offer completion hints.)
The REPL has file-backed searchable history and access to graphical and text-based editors such as VS Code, Zed, Helix, Vim, nano etc. via the VISUAL or EDITOR environment variables, in case its `reedline` editor falls short for a particular task. The key bindings in the latter depend on your terminal settings and you should probably not expect too much in the way of navigation shortcuts.

### * With standard input:

```bash
echo '(1..=10).product::<u32>()' | thag --stdin                 # Short form: -s
```

Place any arguments for the script after `--` to separate them from `thag` arguments:

```bash
echo 'println!("Hello {}", std::env::args().nth(1).ok_or_else(|| "No name supplied")?);' | thag -s -- Ferris
```

This is equivalent to:

```bash
thag -e 'println!("Hello {}", std::env::args().nth(1).ok_or_else(|| "No name supplied")?);' -- Ferris
```

### * With the built-in TUI (Terminal User Interface) editor

```bash
thag --edit                                                     # Short form: -d
```
![TUI Editor](assets/edit1t.png)
*Built-in TUI editor with syntax highlighting and themed interface*

![TUI Output](assets/edit2t.png)
*Running code directly from the editor with Ctrl-R*

#### TUI Editor Demos

**Demo 1: Edit & Run Workflow (30 seconds)**

[![asciicast](https://asciinema.org/a/nB3lFb6LgaHOF1s3dm5srjwyY.svg)](https://asciinema.org/a/nB3lFb6LgaHOF1s3dm5srjwyY)

*Click to watch: Load code from stdin, make quick edits, discover key bindings (Ctrl-L), and submit for execution*

**Demo 2: Data Composition (1:14)**

[![asciicast](https://asciinema.org/a/LvSHLiZPC6lfCgSN4Q0sUJjpG.svg)](https://asciinema.org/a/LvSHLiZPC6lfCgSN4Q0sUJjpG)

*Click to watch: Copy lines with `thag_copy`, retrieve skeleton from history, paste clipboard contents, rearrange code with TextArea buffer (Ctrl-X/Ctrl-Y), and run*

### * With standard input into the TUI editor:

```bash
cat my_file.rs | thag --edit                                    # Short form: -d
```

This allows you to edit or append to the stdin input before submitting it to `thag_rs`. It has file-backed history so you don't lose your edits.

#### Key bindings in the TUI editor
`thag_rs` tries to define all key bindings explicitly. However, the terminal emulator you use is bound to intercept some of these keys, rendering them unavailable for editing.
If specific key bindings don't work for you, you may have to adjust your terminal settings. For example:

1. In order for the Option key on Mac to behave correctly in `iterm2`, you may need to choose `iterm2` Settings | Profiles | Keys to specify how to handle the left and right Option keys.
You can choose one of: Normal, Meta or Esc+. The `iterm2` recommended setting of Esc+ is what works in testing.

2. In order for the Shift-Up and Shift-Down key combinations to work on Apple Terminal, you may need to add the following to your Apple Terminal Settings | Profiles | Keyboard settings:
Shift-Up: `\033;[2A` and `Shift-Down`: `\033;[2B`. Use the Esc key to generate \033. This is not necessary on Iterm2 or WezTerm.

If all else fails, try another terminal emulator.

The TUI editor is also used in the promote-to-TUI (`tui`) and edit history (`history`) functions in the REPL, so the
above also applies there.

Similar considerations apply to the basic REPL mode (--repl / -r). Note that the key bindings there are not identical to the TUI because the basic REPL uses mostly standard `reedline` emacs
key bindings and the TUI uses mostly standard `tui-textarea` key bindings.

### * As a filter on standard input (loop mode):

At a minimum, this loops though `stdin` running the `--loop` expression against every line. The line number and content are made available to the expression as `i` and `line` respectively.

```bash
thag --loop 'println!("{i:>3}.  {line}")' < demo/iter.rs       # Short form: -l
```
![Loop](assets/loopt.png)

`thag` prints returned values automatically. Strings are now quoted:

```bash
thag -ql 'format!("{i:>3}.  {line}")' < demo/hello.rs            # Long form: --quiet --loop

```
For a true filter that you can safely pipe to another process, use `-qq` (or `--quiet --quiet`) to suppress all non-error output:

```bash
bash-3.2$ thag -qql 'println!("{i:>3}.  {line}")' < demo/hello.rs | grep Categories
  3.  //# Categories: basic
bash-3.2$
```
Loop mode also accepts the following optional arguments supplying surrounding code, along the lines of AWK:

```bash
--cargo (-M)    for specifying dependencies etc. in Cargo.toml format. Seldom needed, thanks to dependency inference.
--begin (-B)    for specifying any imports, functions/closures, declarations etc. to be run before the loop.
--end   (-E)    for specifying any summary or final logic to run after the loop.
```

#### Further examples:

```bash
thag --loop 'let gt = if line.len() > 3 { count += 1; true } else { false }; let _ = println!("{gt}");' --begin 'let mut count = 0;' --end 'println!("Total: {}", count);' --toml '[dependencies]
regex = "1.0"' < demo/hello_main.rs
```

The following example shows short forms, multi-line input and debug mode:

```zsh
thag -vv -B 'let mut min = usize::MAX;
    let mut shortest = String::new();' \
-l '{
            let l = line.len();
            let is_shortest = if l < min {
                min = l; shortest = line.to_string(); true
            } else { false };
            is_shortest
        }' \
-E '
    println!("shortest line is: {shortest} of length {min}");' < demo/hello.rs
```

Note: in general if you are planning to **pipe Rust output**, it's probably a good idea to use `writeln!(io::stdout(), "{...}")`,
rather than `println!`, since (as at edition 2021) `println!` panics if it encounters an error, and this
includes the broken pipe error from a head command. **This is a Rust issue not a `thag_rs` issue.**
See `https://github.com/BurntSushi/advent-of-code/issues/17`.
For an example of tolerating a broken pipe, see
`demo/thag_from_rust_script.rs`.

#### Using `writeln!` with a pipe:

```bash
bash-3.2$ thag -qql 'let _ = writeln!(io::stdout(), "{i:>3}.  {line}");' < demo/hello.rs | grep Categories    # long form: --quiet --quiet --loop
  3.  //# Categories: basic
bash-3.2$
```

### * As an executable:
The --executable (-x) option builds your script in **release mode** and moves it to ~/.cargo/bin/, which is highly recommended to be in your path as `thag` and its tools are installed there.

```bash
thag demo/smol_chat_server.rs -x                                        # Long form: --executable
```
![XBuild](assets/xbuildt.png)

You can of course use an OS command to rename the executable if you so desire.

![XBuild](assets/xrenamet.png)

However, it's probably best to rename your source in the first place so you don't lose track of where the command came from if you want to update it.

I **recommend building an executable over using a shebang** because it performs much better on several counts:

- It cuts out the middleman (`thag`)
- You only incur the overhead and latency of the build process once at build time, instead of every time you use the script.
- It builds in release mode, making it execute faster as well
- You can use a tool like `llvm-strip` to strip sections from the executable to make it smaller

An executable is also more convenient on one count: it dispenses with the need for the `--` argument separator because `thag` is no longer being invoked first, so there's no need to separate two sets of arguments.

On the downside, the compiled executable may typically take up between 0.5 MB and 10 MB of disk space (somewhat less if stripping).

Putting it to use:

![XBuild](assets/xuset.png)

### * Run `cargo test` in place:
The --test-only (-T) is for scripts (not snippets) with embedded unit tests. (For snippets there's the --cargo (-A) option that accepts cargo subcommands including `test`).
This option leaves the source script untouched but generates a temporary Cargo.toml for it in the usual way and invokes `cargo test` on this to run the internal unit tests.

As usual, `thag` must rely on dependency inference and/or a toml block in the script for the manifest information.
In the following example, `demo/config_with_tests.rs` demonstrates testing a copy of `thag`'s own `config` module with its unit tests embedded and minor adjustments to
import statements and a small toml block for the `thag_rs` crate itself. In this case we pass the absolute path of a file via an environment variable to the unit test that needs it.

```bash
env TEST_CONFIG_PATH=/absolute/path/to/config.toml thag demo/config_with_tests.rs -Tvf -- --show-output --test-threads=3  # Long form: --test-only
```

### * Command-line options

Hopefully the help screen is self-explanatory:

![Help](assets/helpt.png)

`thag_rs` uses a standard `clap` CLI, so it follows the common `clap` conventions, in a very similar way to `cargo`. You can enter `thag` arguments and options in any order. If your script or dynamic run accepts arguments of its own, they must come after the `thag` arguments and separated from them by a double dash (`--`).

## Overview

### Features

_Rust is primarily an expression language.... In contrast, statements serve mostly to contain and explicitly sequence expression evaluation._
_— The Rust Reference_

* Runs serious Rust scripts (not just the "Hello, world!" variety) with no need to create a project.
* Aims to be the most capable and reliable script runner for Rust code.
* A choice of modes:
    * **Expression mode** for small individual expressions on the fly.
    * **REPL mode** offers interactivity, and accepts multi-line expressions since it uses bracket matching to wait for closing braces, brackets, parens and quotes.
    If REPL mode becomes too limiting, you have two alternative ways to promote your expression to a full-fledged script from the REPL editor.
    * **Stdin mode** accepts larger scripts and programs on the fly, as typed, pasted or piped input or as URLs (via `thag_url`).
    * **Edit mode** via a basic TUI (terminal user interface) editor, with optional `thag_url` or other piped input.
    * The classic **script mode** runs an .rs file consisting of a valid Rust snippet or program.


* **Dependency inference** from code, with generation of minimal and maximal toml blocks for pasting into your Rust source script.
* **Any valid Cargo.toml** input may be specified in the toml block, e.g.:
  - Specific features of dependencies for advanced functionality
  - Local path and git dependencies
  - A [profile.dev] with a non-default optimisation level
  - A [[bin]] to rename the executable output.
* You can use a **shebang** to write scripts in Rust, or better yet...
* For **more speed** and a seamless experience you can build your own commands, using the `--executable` (`-x`) option. This compiles a valid script to a release-optimised executable command in the Cargo bin directory `<home>/.cargo/bin`.
* `thag_rs` supports a personal library of code samples for reuse. The downloadable **starter set** in the demo subdirectory contains over 330 demo scripts, including numerous examples from popular crates. It also has many original examples ranging from the trivial to the complex, including many prototypes of concepts used in building the project, such as TUI editing, `syn` AST manipulation, terminal theme detection and color handling, and command-line tool and REPL building. There are also demos of compile-time and run-time Rust type detection strategies, informal testing scripts, the script that generates the README for the demos, and a range of fast big-integer factorial and Fibonacci calculation scripts
* Automatic support for **light or dark backgrounds** and a **16- or 256- color palette** for different message types, according to terminal capability. Alternatively, you can specify your terminal preferences in a `config.toml` file. On Windows prior to the Windows Terminal 1.22 Preview of August 2024, interrogating the terminal is not supported and tends to cause interference, so in the absence of a `config.toml` file, `thag_rs` currently defaults to basic Ansi-16 colors and dark mode support. However, the dark mode colors it uses have been chosen to work well with most light modes.
* In some cases you may be able to develop a module of a project individually by giving it its own main method and embedded Cargo dependencies and running it from `thag_rs`. Failing that, you can always work on a minimally modified copy in another location. This approach allows you to develop and debug a new module without having it break your project. For example the demo versions of colors.rs and stdin.rs were both prototypes that were fully developed as scripts before being merged into the main `thag_rs` project.

### * Getting started:

You have the choice of installing `thag_rs` (recommended), or you may prefer to clone it and compile it yourself and run it via `cargo run`.

* Installing gives you speed out of the box and a simpler command-line interface without invoking Cargo yourself. You have a choice:
```bash
cargo install thag_rs
```
or choose an appropriate installer for your environment from the Github releases page `https://github.com/durbanlegend/thag_rs/releases`, as from `v0.1.1`.

You can also download the starter kit of demo scripts as `demo.zip`
from the same page.

* Cloning gives you immediate access to the demo scripts library and the opportunity to make local changes or a fork.

`thag_rs` uses Cargo, `syn`, `quote` and `cargo_toml` to analyse and wrap well-formed snippets and expressions into working programs. Well-formed input programs are identified by having a valid `fn main` (or more than one - see below) and are passed unchanged to `cargo build`.

`thag_rs` uses `syn` to parse valid code into an abstract syntax tree (AST). Among other benefits this prevents it from being fooled by code embedded in comments or string literals, which is the curse of regular expressions and string parsing. `thag_rs` then uses the `syn` visitor mechanism to traverse the AST to identify dependencies in the code so as to generate a `Cargo.toml`. It filters these to remove duplicates and false positives such as built-in Rust crates, renamed crates and local modules.

Well-formedness is determined by counting occurrences of a `main` function in the AST. The absence of a `fn main` signifies a snippet or expression, whereas more than one `fn main` may be valid but must be actively confirmed as such by the user with the `--multimain (-m)` option.

If your code does not successfully parse into an AST because of a coding error, `thag_rs` falls back to using source code analysis to prepare your code for the Rust compiler, which can then show you error messages to help you find the issues.

You may provide optional valid (Cargo.toml) metadata in a toml block as described below. `thag_rs` uses `cargo_toml` to parse any metadata into a manifest struct, merges in any dependencies, features or patches inferred from the AST, and then uses `toml` to write out the dedicated Cargo.toml file that Cargo needs to build the script. Finally, in the case of snippets and expressions, it uses `quote` to embed the logic in a well-formed program template and `prettyplease` to format it, and finally invokes Cargo to build it.

All of this happens quite fast: the real bottleneck is the familiar Cargo build process downloading and compiling your dependencies on the initial build. Cargo build output is displayed in real time by default so that there are no mystery delays. If you rerun the compiled script it should be almost immediate.

In this way `thag_rs` attempts to handle any valid (or invalid) Rust script, be it a program, snippet or expression. It tries to generate a dedicated Cargo.toml for your script from `use` statements in your code, although for precision and (marginally?) better speed you may need or wish to embed your own in a toml block:
```/*
[toml]
[dependencies]
...
*/
```
at the start of the script, as you can see done in some of the demos. To help with this, after each successful Cargo lookup `thag_rs` generates and prints a basic toml block with the crate name and version under a `[dependencies]` header, for you to copy and paste into your script if you want to. (As in the second `--expr` example above.) It does not print a combined block, so it's up to you to merge all the dependencies into a single toml block. All dependencies can typically go under the single `[dependencies]` header in the toml block, but thanks to `cargo_toml` there is no specific limit on what valid Cargo code you can place in the toml block.

`thag_rs` aims to be as comprehensive as possible without sacrificing speed and transparency. It uses timestamps to rerun compiled scripts without unnecessary rebuilding, although you can override this behaviour. For example, a precompiled script calculated the 35,661-digit factorial of 10,000 in under half a second on my M1 MacBook Air.

### Example of using a toml block (demo/prettyplease.rs)

    /*[toml]
    [dependencies]
    prettyplease = "0.2.32"
    syn = { version = "2", default-features = false, features = ["full", "parsing"] }
    */

    /// Published example from `prettyplease` Readme.
    //# Purpose: Demo featured crate.
    const INPUT: &str = stringify! {
        use crate::{
              lazy::{Lazy, SyncLazy, SyncOnceCell}, panic,
            sync::{ atomic::{AtomicUsize, Ordering::SeqCst},
                mpsc::channel, Mutex, },
          thread,
        };
        impl<T, U> Into<U> for T where U: From<T> {
            fn into(self) -> U { U::from(self) }
        }
    };

    fn main() {
        let syntax_tree = syn::parse_file(INPUT).unwrap();
        let formatted = prettyplease::unparse(&syntax_tree);
        print!("{}", formatted);
    }

## Usage

Once installed, you can use the `thag` command from the command line. `thag` uses `clap` to process command-line arguments including `--help`.

See the `Quick start` section for a comprehensive introduction. Here are some further examples:

### Evaluating an expression
#### Concise fast factorial calculation

```bash
thag -e '(1..=34).product::<u128>()'
```
This panics beyond `34!` due to using Rust primitives, but see `demo/factorial_dashu_product.rs` for arbitrarily big numbers:

#### Make a command from an expression

Three `thag` features that make expressions powerful are:

1. The ability to compile them with `-x` (caution: `-ex` ❌, `-xe` ✅ to make a release build.

2. Dependency inference means no need to provide a Cargo.toml or other form of dependency metadata.

3. You may use the shorthand `?` to unwrap results in your expression. See the `Ferris` example earlier.

Here's how to make a useful command `myip` to retrieve your external IP address. Be sure that `~/.cargo/bin` is in your path, or move the command to a directory that is:

```bash
thag -u true -xe 'ureq::get("https://api.ipify.org").call()?.into_body().read_to_string()?' && mv ~/.cargo/bin/temp ~/.cargo/bin/myip && echo Success

$ myip
***.***.***.*** [redacted]
```

#### Shoehorn a script into an expression, just because!
```bash
thag -e "$(cat demo/fizz_buzz_gpt.rs)"
```
The `--expr` flag not only evaluatsz an expression, it also accepts a valid Rust program or set of statements.
The different ways `thag` accepts code are as far as possible "orthagonal" to the common way it processes them.

### Running a script in quiet mode but with timings
```bash
bash-3.2$ thag -tq demo/fizz_buzz_gpt.rs

──────────────────────────────────────────────────────────────────────
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
16
...
89
FizzBuzz
91
92
Fizz
94
Buzz
Fizz
97
98
Fizz
Buzz
──────────────────────────────────────────────────────────────────────
Completed run in 0.21s
thag_rs completed processing script fizz_buzz_gpt.rs in 0.59s
bash-3.2$
```

### Using the REPL
```bash
thag -r
```
This starts an interactive REPL session where you can enter or paste in a single- or multi-line Rust expression and press Enter to run it. You can also retrieve and optionally edit an expression from history.
Having evaluated the expression you may choose to edit it, and / or the generated Cargo.toml, in your preferred editor (VS Code, Helix, Zed, nano, etc.) and rerun it. The REPL also offers basic housekeeping functions for the temporary files generated, otherwise being in temporary space they will be cleaned up by the operating system in due course.

You can access the last 25 REPL commands or expressions from within the REPL function just by using the up and down arrow keys to navigate history.

#### General notes on REPL
The REPL temporary files are created under the `rs_repl` subdirectory of your temporary directory (for example $TMPDIR in *nixes, and referenced as std::env::temp_dir() in Rust). The generated script is called `repl_script.rs`.

The REPL feature is not suited to scripts of over about 1K characters, due to the limitations of the underlying line editor. If you're in REPL mode and it starts cramping your style, you can clear the REPL command line with `Ctrl-u` and promote the current expression to a full-blown script using either the built-in TUI editor with history support, or the editor of your choice without history support. Both of these options mean that your Rust script no longer has to be a smallish single expression, and both allow you to save your script to a .rs file or your choice and run it from the command line after you exit the REPL session.

1. _The TUI editor._ The `tui` REPL command opens your current REPL expression in the built-in TUI editor. When you've finished editing, you can run it with `Ctrl-d` and/or save it to a .rs file of your choice. The `tui` command also places the expression in the TUI editor's history, which by the way is kept separate from the REPL's history because it is not subject to the same limitations. `Ctrl-d` autosaves your changes so that you can keep repeating the cycle of `tui` and `Ctrl-d` until you're satisfied with the outcome. The TUI editor is pretty basic but has the advantage of file-backed history support so you can come back to your code later.

2. _Your preferred editor._ You can enter the `edit` command on the REPL command line to edit your script in your preferred editor as configured via the $VISUAL or $EDITOR environment variables, and save it from there. Get back to the REPL session by closing the edit session or tabbing back in the operating system, and run the updated code with the REPL's `run` command. Alternatively you can save the source to a .rs file. As with `tui` and `Ctrl-d`, you can keep repeating the cycle of `edit` and `run` until you're satisfied. However, since the `edit` command has no history support, the only way to see it or preserve the final code in history is to switch to the `tui` command once you've finished, which adds it to the TUI history.

## Usage notes

### Testing changes to dependencies

As mentioned earlier, ``thag_rs`` uses timestamps to rerun compiled scripts without unnecessary rebuilding. You can override this behaviour with the `--force (-f)` option.

This is important to note if you are using the script to test changes to a dependency specified in the toml block, typically as a path dependency but possibly a git or even a version dependency.

When using a git dependency that resides on GitHub, you may need to specify the `rev` parameter to pick up the latest version. This is a feature of GitHub, not of `thag_rs`.

If you make changes to the dependency but not to the script, you need to specify -f to rebuild the script so that it picks up the changed dependency.

### Other dependency tips

Missing toml entries are not pulled in for crates that are only referenced by qualified paths instead of being imported with `use`. In this case you need to add either a toml block entry or at least one "redundant" `use <crate>...;` statement for the crate.

If you want to ensure that a dependency in a TOML block is up to date, you can get the latest version in copyable format by issuing `cargo search <crate> [--limit 1]` from the command line. Alternatively, comment it out temporarily with a hash (`#`) and let `thag -c` do the search for you.

## Platform Support
This crate is designed to be cross-platform, and supports MacOS, Linux and Windows.

### Currently tested on:
* MacOS (M1) Sonoma, Sequoia, Tahoe.
* Linux: Zorin and (WSL2) Ubuntu
* Windows 11:
  - PowerShell 5 and 7
  - CMD
  - Windows Console
  - Windows Terminal up to 1.22 Preview with OSC query support.
  - WSL2
* GitHub Actions test each commit and PR on `ubuntu-latest`, `macos-latest` and `windows-latest`.

### Using `thag_rs` as a library

If you choose to use `thag-rs` library functions in your code, you can significantly reduce its footprint
by choosing only the features you need

#### Feature dependency tree:

```
default
├── simplelog
└── full
    ├── repl
    │   └── tui
    │    |  ├── build
    │    |  │   ├── ast
    │    |  │   │   ├── core  ★                # Fundamental feature set
    │    |  │   │   │   ├── error_handling     # Error types and handling
    │    |  │   │   │   ├── log_impl           # Basic logging infrastructure
    │    |  │   │   │   │   └── (simplelog | env_logger)
    │    |  │   │   │   └── styling            # Basic terminal styling
    │    |  │   │   ├── quote
    │    |  │   │   └── syn
    │    |  │   ├── config
    │    |  │   │   ├── core  ★ (shared)       # Core features required here too
    │    |  │   │   ├── mockall
    │    |  │   │   ├── serde_with
    │    |  │   │   └── toml_edit
    │    |  │   └── ratatui                    # TUI framework (shared with color_detect)
    │    |  │
    │    |  ├── tui-textarea                   # Text editing widget
    │    |  ├── serde_json                     # JSON support
    │    |  └── scopeguard                     # Resource cleanup (shared with color_detect)
    │    |
    │    └── nu-ansi-term
    │    └── reedline
    │
    └── color_detect     # Optional terminal detection, only included in full
        ├── config
        ├── ratatui      # TUI framework (shared with build)
        ├── scopeguard   # (shared with tui)
        ├── supports-color
        └── termbg
Core Feature Set (★):
- Basic logging and error handling
- Essential macros: sprtln, debug_log, lazy_static_var, vlog, regex
- Styling system and macros: svprtln, style_for_role
- Fundamental types and traits
Optional features:
- profiling     # Enables profiling via thag_profiler (for internal use)
- debug-logs
- nightly
- no_format_snippet
Common Usage Patterns:
1. Just core functionality:
   features = ["core", "simplelog"]
2. Core with profiling enabled:
   features = ["core", "simplelog", "profiling"]
3. Core with color detection:
   features = ["core", "color_detect", "simplelog"]
4. Full functionality with profiling:
   features = ["full", "simplelog", "profiling"]
Optional features can be added at any level:
- debug-logs
- nightly
- no_format_snippet
- profiling
Note: when using without default features, must specify a logging implementation:
cargo add thag_rs --no-default-features --features="repl,simplelog"
or
cargo add thag_rs --no-default-features --features="repl,env_logger"
```

## Why "thag"?

After the late Thag Simmons. A stone-age power tool for the [grug brained developer](https://grugbrain.dev/) to beat Rust code into submission. Why type long name when short sharp name do trick?

## Related projects

(Hat-tip to the author of `rust-script`)

* `cargo-script` - The Rust RFC Book `https://rust-lang.github.io/rfcs/3424-cargo-script.html`
* `evcxr` - Perhaps the most well-known Rust REPL.
* `cargo-script` - (Unrelated to the Rust RFC one). Rust script runner (unmaintained project).
* `rust-script` - maintained fork of the preceding cargo-script.
* `cargo-eval` - maintained fork of the preceding cargo-script.
* `cargo-play` - local Rust playground.
* `irust` - limited Rust REPL.
* `runner` - experimental tool for running Rust snippets without Cargo, exploring dynamic vs static linking for speed. I have an extensively modified fork of this crate on GitHub, but I highly recommend using the current `thag_rs` crate rather than that fork.
* `cargo-script-mvs` - RFC demo.

There is more discussion of prior art at the Rust RFC link.

## License

SPDX-License-Identifier: Apache-2.0 OR MIT

Licensed under either of

    Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

or

    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

as you prefer.

## Contributing

Contributions will be considered (under MIT/Apache 2 license) if they align with the aims of the project.

Rust code should pass clippy::pedantic checks.
