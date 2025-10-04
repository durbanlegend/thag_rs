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

### Script: thag_alacritty_add_theme.rs

**Description:**  Install generated themes for Alacritty terminal emulator

 This tool installs Alacritty themes into Alacritty's configuration directory
 and updates the Alacritty configuration file with appropriate import statements.
 The themes will typically have been created by `thag_gen_terminal_themes.rs`.
 Get Alacritty configuration directories and files
 Select themes to install using file navigator
 Find theme files in a directory
 Update Alacritty configuration with theme imports
 Update configuration file with specific theme
 Show manual configuration instructions
 Show installation summary
 Show verification steps

**Purpose:** Install and configure thag themes for Alacritty terminal

**Crates:** `dirs`, `inquire`, `thag_proc_macros`, `thag_styling`, `toml_edit`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [thag_alacritty_add_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_alacritty_add_theme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_alacritty_add_theme.rs
```

---

### Script: thag_ast.rs

**Description:**  Tries to convert input to a `syn` abstract syntax tree (`syn::File` or `syn::Expr`).

**Purpose:** Debugging

**Crates:** `quote`, `syn`, `thag_common`

**Type:** Program

**Categories:** AST, crates, technique, tools

**Link:** [thag_ast.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_ast.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_ast.rs
```

---

### Script: thag_cargo.rs

**Description:**  `thag` prompted front-end command to run Cargo commands on scripts.

 Prompts the user to select a Rust script and a cargo command to run against the
 script's generated project, and invokes `thag` with the --cargo option to run it.

**Purpose:** A user-friendly interface to the `thag` `--cargo` option.

**Crates:** `atty`, `inquire`, `thag_common`, `thag_proc_macros`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_cargo.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_cargo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_cargo.rs
```

---

### Script: thag_clippy.rs

**Description:**  `thag` prompted front-end command to run `clippy` on scripts.

 Prompts the user to select a Rust script and one or more Clippy lints to run against the
 script's generated project, and invokes `thag` with the --cargo option to run it.

**Purpose:** A user-friendly interface to the `thag` `--cargo` option specifically for running `cargo clippy` on a script.

**Crates:** `atty`, `inquire`, `thag_styling`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_clippy.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_clippy.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_clippy.rs
```

---

### Script: thag_convert_themes.rs

**Description:**  Converts `base16` and `base24` themes to `thag` `toml` format. Tested on `tinted-theming` crate to date.

 ## Usage examples:

 ### Convert a single theme

 ```Rust
 thag_convert_themes -i themes/wezterm/atelier_seaside_light.yaml -o themes/converted
 ```

 ### Convert a directory of themes (verbosely)

 ```Rust
 thag_convert_themes -i themes/wezterm -o themes/converted -v
 ```

 ### Convert and also generate 256-color versions (verbosely)

 ```Rust
 thag_convert_themes -i themes/wezterm -o themes/converted -c -v
 ```

 ### Force overwrite existing themes

 ```Rust
 thag_convert_themes -i themes/wezterm -o themes/converted -f
 ```


**Purpose:** Theme generation.

**Crates:** `clap`, `serde`, `serde_yaml_ok`, `thag_styling`, `toml`

**Type:** Program

**Categories:** tools

**Link:** [thag_convert_themes.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_convert_themes.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_convert_themes.rs
```

---

### Script: thag_convert_themes_alt.rs

**Description:**  Converts `base16` and `base24` themes to `thag` `toml` format. Tested on `tinted-theming` crate to date.

 Alternative version with headings assigned according to prominence.

 ## Usage examples:

 ### Convert a single theme

 ```Rust
 thag_convert_themes -i themes/wezterm/atelier_seaside_light.yaml -o themes/converted
 ```

 ### Convert a directory of themes (verbosely)

 ```Rust
 thag_convert_themes -i themes/wezterm -o themes/converted -v
 ```

 ### Convert and also generate 256-color versions (verbosely)

 ```Rust
 thag_convert_themes -i themes/wezterm -o themes/converted -c -v
 ```

 ### Force overwrite existing themes

 ```Rust
 thag_convert_themes -i themes/wezterm -o themes/converted -f
 ```


**Purpose:** Theme generation.

**Crates:** `clap`, `serde`, `serde_yaml_ok`, `thag_styling`, `toml`

**Type:** Program

**Categories:** tools

**Link:** [thag_convert_themes_alt.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_convert_themes_alt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_convert_themes_alt.rs
```

---

### Script: thag_copy.rs

**Description:**  Copies text input from `stdin` to the system clipboard.

 A cross-platform equivalent to macOS's `pbcopy`. See also `src/bin/thag_paste.rs`.
 May not work in Linux environments owing to the X11 and Wayland requiring the copying
 app to be open when pasting from the system clipboard. See `arboard` Readme.

**Purpose:** Utility

**Crates:** `arboard`, `thag_common`

**Type:** Program

**Categories:** tools

**Link:** [thag_copy.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_copy.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_copy.rs
```

---

### Script: thag_demo_help.rs

**Description:**  Demo tool to showcase the lightweight help system.

 This tool demonstrates how to use the built-in help system for thag tools.
 It shows how help information is extracted from comments and displayed consistently.

**Purpose:** Demonstrate the lightweight help system for thag tools

**Crates:** `thag_common`

**Type:** Program

**Categories:** demo, tools

**Link:** [thag_demo_help.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_demo_help.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_demo_help.rs
```

---

### Script: thag_detect_term.rs

**Description:**  A basic tool I cobbled together that uses different crates to a) test terminal
 types on different platforms, b) determine and cross-check if a light or dark
 theme is in use and c) determine the level of colour supported reported by
 the terminal.

**Purpose:** Allow checking of terminals on platforms to be supported, also test reliability of different crates.

**Crates:** `log`, `simplelog`, `termbg`, `terminal_light`, `thag_common`

**Type:** Program

**Categories:** crates, tools

**Link:** [thag_detect_term.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_detect_term.rs)

**Not suitable to be run from a URL.**


---

### Script: thag_edit_theme.rs

**Description:** / Interactive theme editor for manual color role adjustments

 This tool allows you to interactively edit theme color assignments,
 particularly useful when automatic conversion doesn't quite match your preferences.

 A color candidate with provenance information
 Main theme editor state

**Purpose:** Edit and customize theme color role assignments interactively

**Crates:** `clap`, `inquire`, `thag_styling`, `toml`

**Type:** Program

**Categories:** color, interactive, styling, theming, tools

**Link:** [thag_edit_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_edit_theme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_edit_theme.rs
```

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

**Crates:** `anyhow`, `atty`, `crossterm`, `inquire`, `side_by_side_diff`, `tempfile`, `thag_proc_macros`, `thag_styling`

**Type:** Program

**Categories:** diagnosis, technique, thag_front_ends, tools

**Link:** [thag_expand.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_expand.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_expand.rs
```

---

### Script: thag_find_demos.rs

**Description:**  Select demo scripts and generate and serve HTML report.

 Strategy and grunt work thanks to `ChatGPT`.

**Purpose:** Allow user to select scripts by category.

**Crates:** `edit`, `inquire`, `thag_proc_macros`, `thag_rs`, `tokio`, `warp`

**Type:** Program

**Categories:** technique, tools

**Link:** [thag_find_demos.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_find_demos.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_find_demos.rs
```

---

### Script: thag_from_rust_script.rs

**Description:**  Converts embedded manifest format from `rust-script` to `thag`.

 E.g. `cat <path_to_rust_script_file> | thag -qq demo/thag_from_rust_script.rs | thag -s [-- [options] [args] ...]`

 Place any command-line options and/or arguments for the script at the end after a -- as shown.


**Purpose:** Convenience for any `rust-script` user who wants to try out `thag`.

**Type:** Program

**Categories:** crates, tools

**Link:** [thag_from_rust_script.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_from_rust_script.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_from_rust_script.rs
```

---

### Script: thag_gen_config.rs

**Description:**  Prompted config file builder for `thag`.

 Makes a modified copy of a user-selected `config.toml` file.

 Some fields, such as RGB values in decimal and hex, are not prompted for as they are more easily entered
 using a text editor.

**Purpose:** Handy configuration file builder.

**Crates:** `convert_case`, `dirs`, `documented`, `inquire`, `strum`, `syn`, `thag_common`, `thag_styling`, `toml`

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [thag_gen_config.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_gen_config.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_gen_config.rs
```

---

### Script: thag_gen_errors.rs

**Description:**  Quick and easy prompted generator for new custom error types and new variants required
 by existing custom error types.

 Prompts for the new or existing custom error type, the
 new variants, any types wrapped by the new variants, and any special display messages.

 The output can be saved to a new error module in the case of a new custom error type,
 or simply copied and pasted in sections from the output into an existing error module
 in the case of an existing custom error type.

 Strategy and grunt work thanks to `ChatGPT`.

**Purpose:** Facilitate generation and enhancement of custom error modules.

**Crates:** `heck`, `inquire`, `thag_styling`

**Type:** Program

**Categories:** technique, tools

**Link:** [thag_gen_errors.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_gen_errors.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_gen_errors.rs
```

---

### Script: thag_gen_proc_macro_readme.rs

**Description:**  This script generates documentation for proc macros defined in `demo/proc_macros/lib.rs`
 and creates a comprehensive README.md file for the proc macros directory.

 It extracts proc macro definitions, their documentation, and links them to their
 corresponding example files in the demo/ directory.

**Purpose:** Generate README.md documentation for proc macros with examples and usage

**Crates:** `syn`, `thag_styling`

**Type:** Program

**Categories:** proc_macros, technique, tools

**Link:** [thag_gen_proc_macro_readme.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_gen_proc_macro_readme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_gen_proc_macro_readme.rs
```

---

### Script: thag_gen_readme.rs

**Description:**  This is the script used to collect script metadata for the `demo` and `tools` directories and generate
 local `README.md` files documenting those directories.

 Strategy and grunt work thanks to `ChatGPT`.

**Purpose:** Document demo scripts in a demo/README.md as a guide for the user, and the same for tools/ scripts.

**Crates:** `heck`, `inquire`, `pathdiff`, `regex`, `strum`, `thag_proc_macros`, `thag_rs`

**Type:** Program

**Categories:** technique, tools

**Link:** [thag_gen_readme.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_gen_readme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_gen_readme.rs
```

---

### Script: thag_gen_terminal_themes.rs

**Description:**  Export `thag_styling` themes to multiple terminal emulator formats

 This tool exports `thag_styling` theme files to the following terminal emulator formats:
 `Alacritty`, `iTerm2`, `Kitty`, `Konsole`, `Mintty`, `WezTerm` and `Windows Terminal`.
 Themes are exported to organized subdirectories in ./`exported_themes`/. It also
 optionally displays instructions for installing them into the respective emulators.
 Select theme files using file navigator
 Find all theme files in a directory
 Interactive theme browser similar to `thag_show_themes`
 Get export configuration from user
 Process a single theme file
 Show installation instructions for selected formats with actual theme names

**Purpose:** Export thag themes to multiple terminal emulator formats and display further instructions.

**Crates:** `inquire`, `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [thag_gen_terminal_themes.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_gen_terminal_themes.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_gen_terminal_themes.rs
```

---

### Script: thag_get_demo_dir.rs

**Description:**  Fast download of `thag_rs` demo directory (starter kit) with subdirectories.
 Git `sparse-checkout` approach suggested and written by ChatGPT, local directory handling assisted by Claude.

 `thag_styling` included

**Purpose:** Prototype for `thag_get_demo_dir`.

**Crates:** `inquire`, `thag_styling`

**Type:** Program

**Categories:** crates, prototype, technique, tools

**Link:** [thag_get_demo_dir.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_get_demo_dir.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_get_demo_dir.rs
```

---

### Script: thag_image_to_theme.rs

**Description:**  Generate `thag_styling` themes from image files using file navigator.

 This tool analyzes image files to extract dominant colors and generates
 `thag_styling`-compatible theme files. Supports auto-detection of theme type
 (light/dark) and customizable color extraction parameters.
 Get theme configuration from user input
 Display comprehensive theme information
 Display color palette with visual preview
 Extract RGB information from a style for display
 Save theme to a TOML file using file navigator

**Purpose:** Generate custom `thag_styling` themes from images

**Crates:** `inquire`, `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [thag_image_to_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_image_to_theme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_image_to_theme.rs
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

**Crates:** `thag_common`

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [thag_legible.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_legible.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_legible.rs
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
    thag_markdown <path_to_markdown_file>
    ```

 3. Open `http://localhost:8080` in your browser to view the rendered HTML.

**Purpose:** Useful tool and demo.

**Crates:** `reqwest`, `serde_json`, `thag_common`, `tokio`, `warp`

**Type:** Program

**Categories:** demo, tools

**Link:** [thag_markdown.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_markdown.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_markdown.rs
```

---

### Script: thag_migrate_tool.rs

**Description:**  Tool to help migrate existing tools from tools/ to src/bin/ with auto-help integration.

 This utility helps migrate tools by:
 - Moving files from tools/ to src/bin/
 - Adding the `auto_help!` macro integration
 - Updating Cargo.toml entries if needed
 - Preserving all existing functionality

**Purpose:** Migrate tools from tools/ directory to src/bin/ with auto-help integration

**Crates:** `inquire`, `thag_styling`

**Type:** Program

**Categories:** tools

**Link:** [thag_migrate_tool.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_migrate_tool.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_migrate_tool.rs
```

---

### Script: thag_mintty_add_theme.rs

**Description:**  Mintty theme installer for Git Bash on Windows

 This Windows-only tool installs thag themes into Mintty by copying theme files
 to the Mintty themes directory and optionally updating the ~/.minttyrc configuration.
 Supports selecting individual themes or entire directories of themes.
 Check if we have write permission to a directory
 Select themes to install using file navigator
 Find mintty theme files in a directory (files with no extension)
 Check if a file appears to be a mintty theme file
 Copy a mintty theme file to the themes directory

**Purpose:** Install thag themes for Mintty (Git Bash)

**Crates:** `dirs`, `inquire`, `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools, windows

**Link:** [thag_mintty_add_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_mintty_add_theme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_mintty_add_theme.rs
```

---

### Script: thag_palette.rs

**Description:**  Terminal Palette Display Tool

 This tool displays the current terminal's color palette, including:
 - All 16 ANSI colors (0-15)
 - Extended 256-color palette samples
 - True color capability test
 - Terminal background detection
 - Current thag theme colors for comparison
 Display basic terminal information
 Display the 16 basic ANSI colors
 Display a row of colors with their indices and names
 Display samples from the 256-color palette
 Test true color capability with a gradient
 Display current thag theme colors
 Extract RGB information from a style for display

**Purpose:** Show terminal palette colors

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [thag_palette.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_palette.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_palette.rs
```

---

### Script: thag_palette_vs_theme.rs

**Description:**  Terminal palette comparison tool with theme selection

 This tool displays the current terminal's color palette alongside
 a selected thag theme for direct comparison. Helps identify color
 mapping issues and verify theme installation.
 RGB color representation
 Error types for palette querying
 Cached palette query results
 Production-ready palette color query using crossterm threading

 # Errors

 Will bubble up any terminal errors encountered.
 Parse OSC 4 response from accumulated buffer
 Parse hex component (2 or 4 digits)
 Get terminal identifier for caching
 Production-ready palette detection with caching
 Select a theme using file navigator or built-in themes
 Display basic terminal information
 Attempt to detect terminal emulator
 Display the 16 basic ANSI colors
 Display a row of colors with their indices and names
 Display theme colors with visual preview
 Display recommendations based on comparison
 Detect potential issues with theme/terminal compatibility
 Calculate contrast ratio between two RGB colors
 Extract RGB information from a style for display
 Brighten a color by increasing its components

**Purpose:** Compare terminal palette with thag theme colors

**Crates:** `char`, `crossterm`, `inquire`, `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [thag_palette_vs_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_palette_vs_theme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_palette_vs_theme.rs
```

---

### Script: thag_paste.rs

**Description:**  Writes text from the system clipboard to `stdout`.

 A cross-platform equivalent to macOS's `pbpaste`. See also `src/bin/thag_copy.rs`.
 May not work with `pbcopy` in Linux environments owing to the X11 and Wayland requiring the copying
 app to be open when pasting from the system clipboard. See `arboard` Readme.

**Purpose:** Utility

**Crates:** `arboard`, `thag_common`

**Type:** Program

**Categories:** tools

**Link:** [thag_paste.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_paste.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_paste.rs
```

---

### Script: thag_prompt.rs

**Description:**  Basic prompted front-end to build and run a `thag` command.
 Copy text to clipboard using arboard (cross-platform)
 Expand environment variables in a string (e.g., $PWD, ${HOME})

**Purpose:** Simplify running `thag`.

**Crates:** `arboard`, `clap`, `inquire`, `regex`, `thag_rs`, `thag_styling`

**Type:** Program

**Categories:** cli, interactive, thag_front_ends, tools

**Link:** [thag_prompt.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_prompt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_prompt.rs
```

---

### Script: thag_rs.rs

**Description:** 
**Purpose:** 

**Crates:** `thag_profiler`, `thag_rs`, `thag_styling`

**Type:** Program

**Categories:** missing

**Link:** [thag_rs.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_rs.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_rs.rs
```

---

### Script: thag_show_themes.rs

**Description:**  Display built-in themes and their styling with terminal setup instructions

 This tool helps you explore the available built-in themes and provides
 terminal setup instructions for optimal display.

**Purpose:** Help get best use out of styling with built-in themes.

**Crates:** `inquire`, `thag_styling`

**Type:** Program

**Categories:** reference, technique, tools

**Link:** [thag_show_themes.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_show_themes.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_show_themes.rs
```

---

### Script: thag_sync_palette.rs

**Description:**  Terminal palette synchronization using OSC sequences,

 This binary provides command-line access to `thag_styling`'s palette synchronization
 functionality, allowing you to apply theme colors directly to your terminal's palette for convenience.

 Note that it does not support `KDE Konsole` or `Mintty` terminal types because they do not support
 the required OSC 4 ANSI escape sequences.

**Purpose:** Try out custom terminal themes or apply them dynamically, including for a session if incorporated in a terminal profile file.

**Crates:** `thag_styling`

**Type:** Program

**Categories:** ansi, color, customization, interactive, styling, terminal, theming, tools, windows, xterm

**Link:** [thag_sync_palette.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_sync_palette.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_sync_palette.rs
```

---

### Script: thag_theme.rs

**Description:**  Displays the current theme palette and attributes.

 E.g. `thag_theme` or `thag src/bin/thag_theme.rs`

**Purpose:** Show current theme.

**Crates:** `thag_styling`

**Type:** Program

**Categories:** ansi, color, styling, terminal, theming, tools, xterm

**Link:** [thag_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_theme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_theme.rs
```

---

### Script: thag_to_rust_script.rs

**Description:**  Converts embedded manifest format from `thag` to `rust-script`.

**Purpose:** Convenience for any `thag` user who wants to try out `rust-script`.

**Crates:** `thag_rs`

**Type:** Program

**Categories:** crates, tools

**Link:** [thag_to_rust_script.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_to_rust_script.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_to_rust_script.rs
```

---

### Script: thag_url.rs

**Description:**  `thag` front-end command to run Rust scripts from URLs provided that `thag` can figure out the dependencies.

 It supports raw files as well as GitHub, GitLab, Bitbucket and the Rust Playground.

 This relies on `thag`'s dependency inference to resolve dependencies and even features (since default
 features for a given crate can be configured via `thag -C`). Failing that, you can of course paste the raw
 source into the thag playground (`thag -d`) and edit / run / save it there, or directly into a `.rs` file
 and run it with `thag /path/to/file`.

 Usage:

 ```bash
 thag_url <url> [additional_thag_args] [-- <script_args>]
 ```

 Example:

 ```bash
 thag_url https://github.com/clap-rs/clap/blob/master/examples/demo.rs -- --name "is this the Krusty Krab?"
 ```

 Extracts the syn AST expression for a Rust code section.

 # Errors

 This function will bubble up any `syn` parse errors encountered.

**Purpose:** A front-end to allow `thag` to run scripts from URLs while keeping `thag` itself free of network dependencies.

**Crates:** `syn`, `tempfile`, `thag_common`, `tinyget`, `url`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_url.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_url.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_url.rs
```

---

### Script: thag_winterm_add_theme.rs

**Description:**  Windows Terminal theme installer with file navigator

 This Windows-only tool installs thag themes into Windows Terminal by
 adding theme schemes to the settings.json configuration file. Supports
 selecting individual themes or entire directories of themes.

**Purpose:** Install thag themes for Windows Terminal

**Crates:** `colored`, `dirs`, `inquire`, `serde_json`, `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools, windows

**Link:** [thag_winterm_add_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_winterm_add_theme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/src/bin/thag_winterm_add_theme.rs
```

---

