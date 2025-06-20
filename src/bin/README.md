## Running the built-in tools

`thag_rs` includes several built-in tools that are compiled as separate binaries. These tools are available after installing `thag_rs` with the `tools` feature enabled.

### Installation with tools

```bash
cargo install thag_rs --features tools
```

### Basic usage

Each tool can be run directly by name:

```bash
thag_convert_themes --help
thag_clippy
thag_gen_readme
# ... etc
```

### Getting help

Most tools support `--help` or `-h` for usage information:

```bash
thag_convert_themes --help
thag_clippy --help
```

### Tool categories

The tools are organized into several categories:

- **Development tools**: Code analysis, formatting, and development utilities
- **Theme tools**: Theme conversion and management utilities  
- **Documentation tools**: README generation and documentation utilities
- **Analysis tools**: AST analysis, profiling, and debugging tools
- **Utility tools**: General-purpose utilities and helpers

### Building from source

If you're building from source, you can build all tools with:

```bash
cargo build --features tools
```

Or build individual tools:

```bash
cargo build --bin thag_convert_themes --features tools
```

### Integration with thag

Many of these tools integrate with the main `thag` command and can be used as part of your Rust scripting workflow.

***
## Detailed script listing

### Script: thag_ast.rs

**Description:**  Tries to convert input to a `syn` abstract syntax tree (`syn::File` or `syn::Expr`).

**Purpose:** Debugging

**Crates:** `quote`, `syn`, `thag_rs`

**Type:** Program

**Categories:** AST, crates, technique, tools

**Link:** [thag_ast.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_ast.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_ast.rs
```

---

### Script: thag_cargo.rs

**Description:**  `thag` prompted front-end command to run Cargo commands on scripts.

 Prompts the user to select a Rust script and a cargo command to run against the
 script's generated project, and invokes `thag` with the --cargo option to run it.

**Purpose:** A user-friendly interface to the `thag` `--cargo` option.

**Crates:** `atty`, `thag_proc_macros`, `thag_rs`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_cargo.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_cargo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_cargo.rs
```

---

### Script: thag_clippy.rs

**Description:**  `thag` prompted front-end command to run `clippy` on scripts.

 Prompts the user to select a Rust script and one or more Clippy lints to run against the
 script's generated project, and invokes `thag` with the --cargo option to run it.

**Purpose:** A user-friendly interface to the `thag` `--cargo` option specifically for running `cargo clippy` on a script.

**Crates:** `atty`, `colored`, `inquire`, `thag_rs`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_clippy.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_clippy.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_clippy.rs
```

---

### Script: thag_convert_themes.rs

**Description:**  Converts `base16` and `base24` themes to `thag` `toml` format. Tested on `tinted-theming` crate to date.

**Purpose:** Theme generation.

**Crates:** `clap`, `serde`, `serde_yaml_ok`, `thag_rs`, `toml`

**Type:** Program

**Categories:** tools

**Link:** [thag_convert_themes.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_convert_themes.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_convert_themes.rs
```

---

### Script: thag_demo_help.rs

**Description:**  Demo tool to showcase the lightweight help system.

 This tool demonstrates how to use the built-in help system for thag tools.
 It shows how help information is extracted from comments and displayed consistently.

**Purpose:** Demonstrate the lightweight help system for thag tools

**Crates:** `thag_rs`

**Type:** Program

**Categories:** demo, tools

**Link:** [thag_demo_help.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_demo_help.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_demo_help.rs
```

---

### Script: thag_detect_term.rs

**Description:**  A basic tool I cobbled together that uses different crates to a) test terminal
 types on different platforms, b) determine and cross-check if a light or dark
 theme is in use and c) determine the level of colour supported reported by
 the terminal.

**Purpose:** Allow checking of terminals on platforms to be supported, also test reliability of different crates.

**Crates:** `log`, `simplelog`, `supports_color`, `termbg`, `terminal_light`, `thag_rs`

**Type:** Program

**Categories:** crates, tools

**Link:** [thag_detect_term.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_detect_term.rs)

**Not suitable to be run from a URL.**


---

### Script: thag_expand.rs

**Description:**  Useful front-end for `thag --cargo <script> --expand`, which in turn uses `cargo-expand` to show the macro expansion
 of a user script. This tool provides a user-friendly interface to select the script to analyse and to view the expanded code,
 either on its own or side-by-side with the original script using a choice of diff tools.

 Available viewing options for expanded code
 Run cargo-expand on the input file and return the expanded output
 Display original and expanded code side by side
 Detect terminal width to optimize side-by-side display

**Purpose:** Display the expanded code of a user script on its own or side-by-side with the original script using a choice of diff tools.

**Crates:** `anyhow`, `atty`, `crossterm`, `side_by_side_diff`, `tempfile`, `thag_proc_macros`, `thag_rs`

**Type:** Program

**Categories:** diagnosis, technique, thag_front_ends, tools

**Link:** [thag_expand.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_expand.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_expand.rs
```

---

### Script: thag_find_demos.rs

**Description:**  Select demo scripts and generate and serve HTML report.

 Strategy and grunt work thanks to `ChatGPT`.

**Purpose:** Allow user to select scripts by category.

**Crates:** `edit`, `inquire`, `thag_proc_macros`, `thag_rs`, `tokio`, `warp`

**Type:** Program

**Categories:** technique, tools

**Link:** [thag_find_demos.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_find_demos.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_find_demos.rs
```

---

### Script: thag_gen_config.rs

**Description:**  Prompted config file builder for `thag`.

 Makes a modified copy of a user-selected `config.toml` file.

 Some fields, such as RGB values in decimal and hex, are not prompted for as they are more easily entered
 using a text editor.

**Purpose:** Handy configuration file builder.

**Crates:** `colored`, `convert_case`, `dirs`, `documented`, `inquire`, `strum`, `syn`, `thag_proc_macros`, `thag_rs`, `toml`

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [thag_gen_config.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_gen_config.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_gen_config.rs
```

---

### Script: thag_gen_errors.rs

**Description:**  Quick and easy prompted generator for new custom error types and new variants required
 by existing custom error types. Prompts for the new or existing custom error type, the
 new variants, any types wrapped by the new variants, and any special display messages.
 The output can be saved to a new error module in the case of a new custom error type,
 or simply copied and pasted in sections from the output into an existing error module
 in the case of an existing custom error type.

 Strategy and grunt work thanks to `ChatGPT`.

**Purpose:** Facilitate generation and enhancement of custom error modules.

**Crates:** `heck`, `inquire`, `thag_rs`

**Type:** Program

**Categories:** technique, tools

**Link:** [thag_gen_errors.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_gen_errors.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_gen_errors.rs
```

---

### Script: thag_gen_readme.rs

**Description:**  This is the script used to collect script metadata for the `demo` and `tools` directories and generate
 local `README.md` files documenting those directories.

 Strategy and grunt work thanks to `ChatGPT`.

**Purpose:** Document demo scripts in a demo/README.md as a guide for the user, and the same for tools/ scripts.

**Crates:** `heck`, `pathdiff`, `thag_proc_macros`, `thag_rs`

**Type:** Program

**Categories:** technique, tools

**Link:** [thag_gen_readme.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_gen_readme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_gen_readme.rs
```

---

### Script: thag_get_demo.rs

**Description:**  Downloader for the `demo` directory. Basics courtesy of GPT.

**Purpose:** Download the demo directory from Github main.

**Crates:** `reqwest`, `rfd`, `serde`, `thag_rs`

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [thag_get_demo.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_get_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_get_demo.rs
```

---

### Script: thag_legible.rs

**Description:**  Unescape \n and \\" markers in a string to convert the wall of text to readable lines.
 This is trickier than it seems because in a compile-time literal, \n compiles to the
 true line feed character 10 (x0A), whereas a \n generated or captured as a literal
 at run time is encoded as ('\', 'n'() = (92, 110) = 0x5c6e. Not surprisingly, the two
 representations, while they look identical to the programmer, don't always behave
 the same.

 See `demo/dethagomizer.rs` for a Regex version.

**Purpose:** Useful script for converting a wall of text such as some TOML errors back into legible formatted messages.

**Crates:** `thag_rs`

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [thag_legible.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_legible.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_legible.rs
```

---

### Script: thag_markdown.rs

**Description:**  Quick markdown viewer checks your markdown for GitHub compatibility.

 Script generated by GitHub Copilot.

 Uses the GitHub REST API endpoint `/markdown` to convert Markdown to HTML and serves the HTML using the `warp` web framework. Make sure your Markdown file is less than 400 KB, as per the API's limitations.

 ### Instructions:
 1. Set the `GITHUB_TOKEN` environment variable with your GitHub token (classic or fine-grained).
 2. Run the script:
    ```bash
    thag -q tools/github_markdown_viewer.rs -- <path_to_markdown_file>
    ```
 3. Open `http://localhost:8080` in your browser to view the rendered HTML.

**Purpose:** Useful tool and demo.

**Crates:** `reqwest`, `serde_json`, `thag_rs`, `tokio`, `warp`

**Type:** Program

**Categories:** demo, tools

**Link:** [thag_markdown.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_markdown.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_markdown.rs
```

---

### Script: thag_migrate_tool.rs

**Description:**  Tool to help migrate existing tools from tools/ to src/bin/ with auto-help integration.

 This utility helps migrate tools by:
 - Moving files from tools/ to src/bin/
 - Adding the auto_help! macro integration
 - Updating Cargo.toml entries if needed
 - Preserving all existing functionality

**Purpose:** Migrate tools from tools/ directory to src/bin/ with auto-help integration

**Crates:** `inquire`, `thag_rs`

**Type:** Program

**Categories:** tools

**Link:** [thag_migrate_tool.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_migrate_tool.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_migrate_tool.rs
```

---

### Script: thag_rs.rs

**Description:** 
**Purpose:** 

**Crates:** `thag_profiler`, `thag_rs`

**Type:** Program

**Categories:** missing

**Link:** [thag_rs.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_rs.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_rs.rs
```

---

### Script: thag_show_themes.rs

**Description:**  AI-generated prototype tool to demonstrate themes and help implement their background
 colouring on terminal emulators.

**Purpose:** Help get best use out of styling with popular themes.

**Crates:** `thag_rs`

**Type:** Program

**Categories:** reference, technique, tools

**Link:** [thag_show_themes.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_show_themes.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_show_themes.rs
```

---

### Script: thag_to_rust_script.rs

**Description:**  Converts embedded manifest format from `thag` to `rust-script`.

**Purpose:** Convenience for any `thag` user who wants to try out `rust-script`.

**Crates:** `thag_rs`

**Type:** Program

**Categories:** crates, tools

**Link:** [thag_to_rust_script.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_to_rust_script.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_to_rust_script.rs
```

---

### Script: thag_url.rs

**Description:**  `thag` front-end command to run scripts from URLs.

 Extracts the syn AST expression for a Rust code section.

 # Errors

 This function will bubble up any `syn` parse errors encountered.

**Purpose:** A front-end to allow `thag` to run scripts from URLs while offloading network dependencies from `thag` itself.

**Crates:** `syn`, `tempfile`, `thag_rs`, `tinyget`, `url`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_url.rs](https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_url.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/src/bin/thag_url.rs
```

---

