## Running the scripts in `demo` and `tools`

`thag_rs` uses `clap` for a standard command-line interface. Try `thag --help` (or -h) if
you get stuck.

### In its simplest form:


    thag <path to script>

###### E.g.:

    thag demo/hello.rs

### Passing options and arguments to a script:

Use `--` to separate options and arguments meant for the script from those meant for `thag` itself.

###### E.g.:

demo/fib_dashu_snippet.rs expects to be passed an integer _n_ and will compute the _nth_ number in the
Fibonacci sequence.

     thag demo/fib_dashu_snippet.rs -- 100

### Full syntax:

    thag [THAG OPTIONS] <path to script> [-- [SCRIPT OPTIONS] <script args>]

###### E.g.:

`demo/clap_tut_builder_01.rs` is a published example from the `clap` crate.
Its command-line signature looks like this:

    clap_tut_builder_01 [OPTIONS] [name] [COMMAND]

The arguments in their short form are:

    `-c <config_file>`      an optional configuration file
    `-d` / `-dd` / `ddd`    debug, at increasing levels of verbosity
    [name]                  an optional filename
    [COMMAND]               a command (e.g. test) to run

If we were to compile `clap_tut_builder_01` as an executable (`-x` option) and then run it, we might pass
it some parameters like this:

    clap_tut_builder_01 -dd -c my.cfg my_file test -l

and get output like this:

    Value for name: my_file
    Value for config: my.cfg
    Debug mode is on
    Printing testing lists...

Running the source from `thag` looks similar, we just replace `clap_tut_builder_01` by `thag demo/clap_tut_builder_01.rs --`:

*thag demo/clap_tut_builder_01.rs --* -dd -c my.cfg my_file test -l

Any parameters for `thag` should go before the `--`, e.g. we may choose use -qq to suppress `thag` messages:

    thag demo/clap_tut_builder_01.rs -qq -- -dd -c my.cfg my_file test -l

which will give identical output to the above.



##### Remember to use `--` to separate options and arguments that are intended for `thag` from those intended for the target script.

### TODO: check:
For detailed documentation on the `category_enum` procedural macro, see [category_enum](proc_macros/docs/category_enum.md).

***
## Detailed script listing

### Script: dethagomizer.rs

**Description:**  Unescape \n and \\" markers in a string to convert the wall of text to readable lines.
 This is trickier than it seems because in a compile-time literal, \n compiles to the
 true line feed character 10 (x0A), whereas a \n generated or captured as a literal
 at run time is encoded as ('\', 'n'() = (92, 110) = 0x5c6e. Not surprisingly, the two
 representations, while they look identical to the programmer, don't always behave
 the same.

 See `demo/dethagomizer.rs` for a Regex version.

**Purpose:** Useful script for converting a wall of text such as some TOML errors back into legible formatted messages.

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [dethagomizer.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/dethagomizer.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/dethagomizer.rs
```

---

### Script: download_demo_dir.rs

**Description:**  Downloader for the `demo` directory. Basics courtesy of GPT.

**Purpose:** Download the demo directory from Github main.

**Crates:** `reqwest`, `rfd`, `serde`

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [download_demo_dir.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/download_demo_dir.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/download_demo_dir.rs
```

---

### Script: error_builder.rs

**Description:**  Quick and easy prompted generator for new custom error types and new variants required
 by existing custom error types. Prompts for the new or existing custom error type, the
 new variants, any types wrapped by the new variants, and any special display messages.
 The output can be saved to a new error module in the case of a new custom error type,
 or simply copied and pasted in sections from the output into an existing error module
 in the case of an existing custom error type.

 Strategy and grunt work thanks to ChatGPT.

**Purpose:** Facilitate generation and enhancement of custom error modules.

**Crates:** `heck`, `inquire`

**Type:** Program

**Categories:** technique, tools

**Link:** [error_builder.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/error_builder.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/error_builder.rs
```

---

### Script: filter_demos.rs

**Description:**  Select demo scripts and generate and serve HTML report.

 Strategy and grunt work thanks to ChatGPT.

**Purpose:** Allow user to select scripts by category.

**Crates:** `edit`, `inquire`, `thag_proc_macros`, `thag_rs`, `tokio`, `warp`

**Type:** Program

**Categories:** technique, tools

**Link:** [filter_demos.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/filter_demos.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/filter_demos.rs
```

---

### Script: gen_readme.rs

**Description:**  This is the script used to collect script metadata for the `demo` and `tools` directories and generate
 local `README.md` files documenting those directories.

 Strategy and grunt work thanks to ChatGPT.

**Purpose:** Document demo scripts in a demo/README.md as a guide for the user, and the same for tools scripts.

**Crates:** `heck`, `inquire`, `thag_proc_macros`, `thag_rs`

**Type:** Program

**Categories:** technique, tools

**Link:** [gen_readme.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/gen_readme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/gen_readme.rs
```

---

### Script: input_expr_to_ast.rs

**Description:**  Tries to convert input to a `syn` abstract syntax tree (syn::Expr).

**Purpose:** Debugging

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, technique, tools

**Link:** [input_expr_to_ast.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/input_expr_to_ast.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/input_expr_to_ast.rs
```

---

### Script: input_file_to_ast.rs

**Description:**  Tries to convert input to a `syn` abstract syntax tree (syn::File).

**Purpose:** Debugging

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, technique, tools

**Link:** [input_file_to_ast.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/input_file_to_ast.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/input_file_to_ast.rs
```

---

### Script: profile_instr.rs

**Description:**  A stand-alone convenience tool to instrument a Rust source program for `thag_rs` profiling.
 It accepts the source code on stdin and outputs instrumented code to stdout.
 The instrumentation consists of adding the #[enable_profiling] attribute to `fn main` if
 present, and the #[profiled] attribute to all other functions and methods, as well as import
 statements for the `thag_rs` profiling.
 module and proc macro library. It is intended to be lossless, using the `rust-analyzer` crate
 to preserve the original source code intact with its comments and formatting. However, by using
 it you accept responsibility for all consequences of instrumentation and profiling.
 It's recommended to use profiling only in development environments and thoroughly test the
 instrumented code before deploying it.
 It's also recommended to do a side-by-side comparison of the original and instrumented code
 to ensure that the instrumentation did not introduce any unintended changes.
 Free tools for this purpose include `diff`, `sdiff` git diff, GitHub desktop and BBEdit.
 This tool attempts to position the injected code sensibly and to avoid duplication of existing
 `thag_rs` profiling code. It implements default profiling which currently includes both execution
 time and memory usage, but this is easily tweaked manually by modifying the instrumented code by
 adding the keyword `profile_type = ["time" | "memory"])` to the `#[enable_profiling]` attribute,
 e.g.: `#[enable_profiling(profile_type = "time")]`.

 This tool is intended for use with the `thag_rs` command-line tool or compiled into a binary.
 Run it with the `-qq` flag to suppress unwanted output. It requires a positive integer argument
 being a Rust edition number (2015, 2018, 2021). 2024 can't yet be supported.

 E.g.

 1. As a script:

 ```
 thag tools/profile_instr.rs -qq -- 2021 < demo/colors.rs > demo/colors_instrumented.rs
 ```

 2. As a command (compiled with `thag tools/profile_instr.rs -x`)

 ```
 profile_instr < demo/colors.rs -- 2018 > demo/colors_instrumented.rs
 ```

 Doc comment

**Purpose:** Stand-alone tool to instrument any Rust source code for `thag` profiling.

**Crates:** `env`, `ra_ap_syntax`

**Type:** Program

**Categories:** profiling, tools

**Link:** [profile_instr.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/profile_instr.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/profile_instr.rs
```

---

### Script: profile_remove.rs

**Description:**  A stand-alone convenience tool to remove `thag_rs` profiling instrumentation from a Rust source
 program.
 It accepts the instrumented source code on stdin and outputs uninstrumented code to stdout.
 The process consists of removing any and all attribute macro and other ("legacy" / prototype)
 macro invocations of `thag_rs` profiling. It is intended to be lossless, using the `rust-analyzer`
 crate to preserve the original source code intact with its comments and formatting. However, by
 using it you accept responsibility for all consequences.
 It's recommended to use profiling only in development environments and thoroughly test or
 remove the instrumented code before deploying it.
 It's also recommended to do a side-by-side comparison of the original and de-instrumented code
 to ensure that the removal of instrumentation did not introduce any unintended changes.
 Free tools for this purpose include `diff`, `sdiff` git diff, GitHub desktop and BBEdit.

 This tool is intended for use with the `thag_rs` command-line tool or compiled into a binary.
 Run it with the `-qq` flag to suppress unwanted output. It requires a positive integer argument
 being a Rust edition number (2015, 2018, 2021). 2024 can't yet be supported.

 E.g.

 1. As a script:

 ```
 thag tools/profile_uninstr.rs -qq -- 2021 < demo/colors_instrumented.rs > demo/colors.rs
 ```

 2. As a command (compiled with `thag tools/profile_uninstr.rs -x`)

 ```
 profile_uninstr < demo/colors_instrumented.rs -- 2018 > demo/colors.rs
 ```


**Purpose:** Stand-alone tool to remove any and all `thag_rs` profiling instrumentation from any Rust source code.

**Crates:** `env`, `ra_ap_syntax`

**Type:** Program

**Categories:** profiling, tools

**Link:** [profile_remove.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/profile_remove.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/profile_remove.rs
```

---

### Script: term_detection_pack.rs

**Description:**  A basic tool I cobbled together that uses different crates to a) test terminal
 types on different platforms, b) determine and cross-check if a light or dark
 theme is in use and c) determine the level of colour supported reported by
 the terminal.

**Purpose:** Allow checking of terminals on platforms to be supported, also test reliability of different crates.

**Crates:** `crossterm`, `log`, `simplelog`, `supports_color`, `termbg`, `terminal_light`

**Type:** Snippet

**Categories:** crates, tools

**Link:** [term_detection_pack.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/term_detection_pack.rs)

**Not suitable to be run from a URL.**


---

### Script: thag_cargo.rs

**Description:**  `thag` prompted front-end command to run Cargo commands on scripts. It is recommended to compile this to an executable with -x.
 Prompts the user to select a Rust script and a cargo command to run against the script's generated project, and
 and invokes `thag` with the --cargo option to run it.

**Purpose:** A user-friendly interface to the `thag` `--cargo` option.

**Crates:** `atty`, `inquire`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_cargo.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_cargo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_cargo.rs
```

---

### Script: thag_clippy.rs

**Description:**  `thag` prompted front-end command to run `clippy` on scripts. It is recommended to compile this to an executable with -x.
 Prompts the user to select a Rust script and one or more Clippy lints to run against the script's generated project, and
 and invokes `thag` with the --cargo option to run it.

**Purpose:** A user-friendly interface to the `thag` `--cargo` option specifically for running `cargo clippy` on a script.

**Crates:** `atty`, `colored`, `inquire`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_clippy.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_clippy.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_clippy.rs
```

---

### Script: thag_config_builder.rs

**Description:**  Prompted config file builder for `thag`, intended to be saved as a command with `-x`.
 Makes a modified copy of a user-selected `config.toml` file. Some fields such as
 RGB values in decimal and hex are not prompted for as they are more easily entered
 using a text editor.

**Purpose:** Handy configuration file builder.

**Crates:** `colored`, `convert_case`, `dirs`, `documented`, `inquire`, `strum`, `syn`, `thag_proc_macros`, `thag_rs`, `toml`

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [thag_config_builder.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_config_builder.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_config_builder.rs
```

---

### Script: thag_from_rust_script.rs

**Description:**  Converts embedded manifest format from `rust-script` to `thag`.

 E.g. `cat <path_to_rust_script_file> | thag -qq demo/thag_from_rust_script.rs | thag -s [-- [options] [args] ...]`

 Place any command-line options and/or arguments for the script at the end after a -- as shown.


**Purpose:** Convenience for any `rust-script` user who wants to try out `thag`.

**Type:** Program

**Categories:** crates, tools

**Link:** [thag_from_rust_script.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_from_rust_script.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_from_rust_script.rs
```

---

### Script: thag_profile.rs

**Description:**  Profile graph/chart generator for the `thag` internal profiler.

 E.g.:

```
 thag demo/thag_profile.rs -x    # Compile this script as a command

 cargo run --features thag/profile <path>/demo/time_cookbook.rs -f   # Profile a demo script

 thag_profile    # Generate a flamechart or show stats for the new profile
```

**Purpose:** Low-footprint profiling.

**Crates:** `chrono`, `dirs`, `inferno`, `inquire`, `serde`, `serde_json`, `strum`, `thag_rs`

**Type:** Program

**Categories:** tools

**Link:** [thag_profile.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_profile.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_profile.rs
```

---

### Script: thag_to_rust_script.rs

**Description:**  Converts embedded manifest format from `thag` to `rust-script`.

**Purpose:** Convenience for any `thag` user who wants to try out `rust-script`.

**Type:** Program

**Categories:** crates, tools

**Link:** [thag_to_rust_script.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_to_rust_script.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_to_rust_script.rs
```

---

### Script: thag_url.rs

**Description:**  `thag` front-end command to run scripts from URLs. It is recommended to compile this with -x.

**Purpose:** A front-end to allow thag to run scripts from URLs while offloading network dependencies from `thag` itself.

**Crates:** `syn`, `tempfile`, `tinyget`, `url`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_url.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_url.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_url.rs
```

---

### Script: theme_converter.rs

**Description:**  Converts `base16` and `base24` themes to `thag` `toml` format. Tested on `tinted-theming` crate to date.

**Purpose:** Theme generation.

**Crates:** `clap`, `serde`, `serde_yaml_ok`, `thag_rs`, `toml`

**Type:** Program

**Categories:** tools

**Link:** [theme_converter.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/theme_converter.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/theme_converter.rs
```

---

### Script: theme_helper.rs

**Description:**  AI-generated prototype tool to demonstrate themes and help implement their background
 colouring on terminal emulators.

**Purpose:** Help get best use out of styling with popular themes.

**Type:** Program

**Categories:** reference, technique, tools

**Link:** [theme_helper.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/theme_helper.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/theme_helper.rs
```

---
