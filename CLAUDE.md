# thag_rs Development Guide

## Build Commands
- Build: `cargo build`
- Run: `cargo run -- [args]`
- Test all: `cargo test`
- Test single: `cargo test test_name -- --nocapture`
- Integration test: `cargo test --test integration_test`
- Lint: `cargo clippy --all-targets --all-features`
- Format: `cargo fmt`
- Flamegraph: `cargo flamegraph`
- Profile: `cargo run --features profiling -- [args]`

## Dependency versioning

Always use only `major.minor` unless there is a good reason to use the `patch` part.

## Demo and sample scripts
- For the main thag_rs crate, example code should be implemented as scripts in the demo subdirectory.
Run with `cargo run demo/script_name.rs`. Any valid Cargo.toml info can be placed in the normal format in a toml block at the top of the program, like this:
```Rust
/*[toml]
 [dependencies]
 ...
 [features]
 ...
  */
 ```
The [toml] marker must be adjacent to the '/*' comment opener, not on the next line.
Usually thag will infer dependencies, so unless special features of dependencies are required, normally the only toml information is for the thag_rs crate or its subcrates if used, in the following format (pick crate/subcrate/s as needed):
```Rust
/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" } # features if needed
thag_profiler = { version = "0.1, thag-auto", features = ["full_profiling"] } # features if needed
thag_rs = { version = "0.2, thag-auto", features = [...] }  # features if needed
thag_styling = { version = "0.2, thag-auto", features = [...] }  # features if needed
*/
 ```
The thag-auto is used by thag to decide whether to use crates.io, git or a local path. Generally as we are testing new thag functionality, any script with a thag-auto dependency should be run with the env var THAG_DEV_PATH=$PWD from the project dir. The thag-auto must be specified exactly as shown inside the quotes, and not as thag-auto = true.

These scripts need full doc comments (/// or //:, not //!), a `//# Purpose:` one-liner and a `//# Categories: xxx, xxx, xxx, ...` one-liner where the categories are the lower-case versions listed in lines 82ff of thag_proc_macros/src/category_enum.rs, unquoted. See existing scripts for the format. Recommend new categories by all means, but do not make up your own. Please follow these instructions exactly or ask clarification.
I repeat, do not use `//!` for doc comments in demo scripts, as they have cause unwanted behaviour in the past (I can't recall specifics) due to their semantics.

Scripts that are evolved into particularly useful generic tools may be promoted to thag_rs/src/bin for inclusion as binaries in the main project. This should be done only in consultation with me. Their .toml blocks should normally be left in place, but they will need entries in Cargo.toml - see existing tools.

- **The bank/ subdirectory** is an unofficial scratchpad and scrapyard for code that doesn't merit being in demo/ but may be of some future use, even if just as a historical reference. Some of the examples are still valid and working, but by and large code in bank/ does not need to be kept up to date.

- **Verbosity**: To control the verbosity of output, use a verbosity-gating macro like `thag::common::vprtln!` in conjunction with the Verbosity enum (alias V) and setting the global verbosity, e.g. `thag::common::init_verbosity(V::V)?;` or `thag::common::set_verbosity!(debug)`, all re-exported by the other thag crates like `thag_styling`. E.g.:
```
/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/
use thag_styling::{init_verbosity, set_verbosity, vprtln, V};

// Macro method
set_verbosity!(verbose);
vprtln!(V::V, "Verbose mode");

// Alternatively, function method:
init_verbosity(V::D)?;
vprtln!(V::D, "Debug mode");
```

## Code Style Guidelines
- **Imports**: Group std imports first, then external crates, then internal modules
- **Conditional imports**: Use `#[cfg(feature = "feature_name")]` for feature-gated imports
- **Error handling**: Use `ThagResult<T>` for thag_rs, `StylingResult<T>` for thag_styling, `ProfileResult<T>`for thag_profiler and `ThagCommonResult<T>` for thag_common, and wrap errors with appropriate `From` implementations
- **Naming**: CamelCase for types, snake_case for functions and variables
- **Documentation**: Document all public items, especially interfaces and non-obvious behavior
- **Profiling**: Use `#[profiled]` attribute on functions that should be profiled
- **Features**: Clearly mark feature-dependent code with `#[cfg(feature = "feature_name")]`
- **Testing**: Write unit tests for modules, with integration tests for full workflows. Unit test function names should start with `test_<module_name>_` to facilitate confining testing to a specific module.
- **Formatting**: Follow rustfmt conventions; run `cargo fmt` before committing
- **Modules**: Prefer modules in their own programs named <module_name>.rs rather than in directories with a mod.rs if possible.
- **Redundancy**: Don't generate identical functions for different variants of scripts or programs, such as the `thag_styling::exporters` variants, if the common code can reasonably by code once in a lib.rs.
- **Coding**: Try to be clippy::pedantic compliant. Generate for 2021 edition and Rust version in package.rust-version of Cargo.toml.
- **Coloring and styling**: Use thag_styling and not the `colored` crate!

## thag_styling file formats
- **thag_styling themes**: Filenames and `name` field should be (lower) kebab case. The files should be in TOML format with the .toml suffix, and the file name stem should start with `thag-`, end in `-light` or `-dark` as appropriate.
- **generated terminal themes**: The file name stem should consist of or start with the stem of the source thag_styling theme. Since Alacritty and WezTerm both have the same .toml extension, _alacritty or _wezterm should be appended to the stem as appropriate for these 2 formats to distinguish between them. These should all be exported to the appropriate subdirectories of ./exported_themes

## Timeouts
You can run `function timeout() { perl -e 'alarm shift; exec @ARGV' "$@"; }` which will allow you to use the `timeout` command in scripts. This statement is now at the end of /Users/donf/.zshrc and /Users/donf/.bashrc on my Mac so that I always have the timeout command available when running in zsh or bash.

## Markdown files
Each bullet point line (`- Lorem ipsum ...`) should be preceded by a blank line, other MacDown (for one) runs them together into one line. This is not necessary if the lines are bolded, e.g. `- **Lorem ipsum ...**`.
Similarly, line consisting of ``` to mark the start of an example must also be preceded by a blank line.
Always verify that functions exist before including them in examples.
