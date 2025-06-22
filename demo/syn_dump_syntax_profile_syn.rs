/*[toml]
[dependencies]
colored = "2.1.0"
quote = "1.0.37"
syn = { version = "2", features = ["extra-traits", "full", "parsing"] }
# syn = { path = "/Users/donf/projects/syn", features = ["extra-traits", "full", "parsing"] }
proc-macro2 = { version = "1", features = ["span-locations"] }
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["full_profiling", "debug_logging"] }
# thag_profiler = { version = "0.1", features = ["full_profiling", "debug_logging"  ] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling", "debug_logging"] }
*/

/// A version of the published example from the `syn` crate used to demonstrate profiling a dependency with `thag_profiler`.
/// Description "Parse a Rust source file into a `syn::File` and print out a debug representation of the syntax tree."
///
/// Pass it the absolute or relative path of any Rust PROGRAM source file, e.g. its own
/// path that you passed to the script runner to invoke it.
///
/// NB: Pick a script that is a valid program (containing `fn main()` as opposed to a snippet).
///
/// E.g.:
///
/// ```
/// THAG_PROFILER=both,,announce,true thag demo/syn_dump_syntax_profile_syn.rs -tf -- demo/hello_main.rs
/// ```
///
/// See the `README.md` for the explanation of the `THAG_PROFILER` arguments
//# Purpose: demonstrate profiling a dependency with `thag_profiler`.
//# Categories: AST, crates, technique
//# Sample arguments: `-- demo/hello_main.rs`

// Parse a Rust source file into a `syn::File` and print out a debug
// representation of the syntax tree.
//
// Use the following command from this directory to test this program by
// running it on its own source code:
//
//     cargo run -- src/main.rs
//
// The output will begin with:
//
//     File {
//         shebang: None,
//         attrs: [
//             Attribute {
//                 pound_token: Pound,
//                 style: AttrStyle::Inner(
//         ...
//     }
use colored::Colorize;
use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;
use std::fmt::{self, Display};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use thag_profiler::{self, enable_profiling, mem_tracking};

enum Error {
    IncorrectUsage,
    ReadFile(io::Error),
    ParseFile {
        error: syn::Error,
        filepath: PathBuf,
        source_code: String,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IncorrectUsage => write!(f, "Usage: dump-syntax path/to/filename.rs"),
            Error::ReadFile(error) => write!(f, "Unable to read file: {}", error),
            Error::ParseFile {
                error,
                filepath,
                source_code,
            } => render_location(f, error, filepath, source_code),
        }
    }
}

#[enable_profiling(runtime)]
fn main() {
    eprintln!(
        "is_profiling_enabled()? {}, get_global_profile_type(): {:?}",
        thag_profiler::is_profiling_enabled(),
        thag_profiler::get_global_profile_type()
    );
    if let Err(error) = try_main() {
        let _ = writeln!(io::stderr(), "{}", error);
        process::exit(1);
    }
}

fn try_main() -> Result<(), Error> {
    let mut args = env::args_os();
    let _ = args.next(); // executable name

    let filepath = match (args.next(), args.next()) {
        (Some(arg), None) => PathBuf::from(arg),
        _ => return Err(Error::IncorrectUsage),
    };

    let code = fs::read_to_string(&filepath).map_err(Error::ReadFile)?;
    let syntax = syn::parse_file(&code).map_err({
        |error| Error::ParseFile {
            error,
            filepath,
            source_code: code,
        }
    })?;
    println!("{:#?}", syntax);

    Ok(())
}

// Render a rustc-style error message, including colors.
//
//     error: Syn unable to parse file
//       --> main.rs:40:17
//        |
//     40 |     fn fmt(&self formatter: &mut fmt::Formatter) -> fmt::Result {
//        |                  ^^^^^^^^^ expected `,`
//
fn render_location(
    formatter: &mut fmt::Formatter,
    err: &syn::Error,
    filepath: &Path,
    code: &str,
) -> fmt::Result {
    let start = err.span().start();
    let mut end = err.span().end();

    let code_line = match start.line.checked_sub(1).and_then(|n| code.lines().nth(n)) {
        Some(line) => line,
        None => return render_fallback(formatter, err),
    };

    if end.line > start.line {
        end.line = start.line;
        end.column = code_line.len();
    }

    let filename = filepath
        .file_name()
        .map(OsStr::to_string_lossy)
        .unwrap_or(Cow::Borrowed("main.rs"));

    write!(
        formatter,
        "\n\
         {error}{header}\n\
         {indent}{arrow} {filename}:{linenum}:{colnum}\n\
         {indent} {pipe}\n\
         {label} {pipe} {code}\n\
         {indent} {pipe} {offset}{underline} {message}\n\
         ",
        error = "error".red().bold(),
        header = ": Syn unable to parse file".bold(),
        indent = " ".repeat(start.line.to_string().len()),
        arrow = "-->".blue().bold(),
        filename = filename,
        linenum = start.line,
        colnum = start.column,
        pipe = "|".blue().bold(),
        label = start.line.to_string().blue().bold(),
        code = code_line.trim_end(),
        offset = " ".repeat(start.column),
        underline = "^"
            .repeat(end.column.saturating_sub(start.column).max(1))
            .red()
            .bold(),
        message = err.to_string().red(),
    )
}

fn render_fallback(formatter: &mut fmt::Formatter, err: &syn::Error) -> fmt::Result {
    write!(formatter, "Unable to parse file: {}", err)
}
