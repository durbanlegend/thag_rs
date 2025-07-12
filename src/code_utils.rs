#![allow(
    clippy::uninlined_format_args,
    clippy::implicit_return,
    clippy::missing_trait_methods
)]

use crate::{
    cvlog_warning, debug_log, re, vlog, Ast, ThagError, ThagResult, DYNAMIC_SUBDIR,
    TEMP_SCRIPT_NAME, TMPDIR, V,
};
use regex::Regex;
use std::{
    fs::{self, remove_dir_all, remove_file, OpenOptions},
    io::{self, BufRead, Write},
    option::Option,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use syn::{
    self,
    visit_mut::{self, VisitMut},
    AttrStyle, Expr, ExprBlock,
};
use thag_profiler::profiled;

#[cfg(debug_assertions)]
use crate::debug_timings;

#[cfg(feature = "build")]
use {
    crate::{BuildState, Cli},
    std::{any::Any, time::SystemTime},
};

#[cfg(target_os = "windows")]
use crate::escape_path_for_windows;

#[cfg(debug_assertions)]
use {crate::cvlog_emphasis, std::time::Instant};

// To move inner attributes out of a syn AST for a snippet.
struct RemoveInnerAttributes {
    found: bool,
}

impl VisitMut for RemoveInnerAttributes {
    #[profiled]
    fn visit_expr_block_mut(&mut self, expr_block: &mut ExprBlock) {
        // Count inner attributes
        self.found = expr_block
            .attrs
            .iter()
            .filter(|attr| attr.style == AttrStyle::Inner(syn::token::Not::default()))
            .count()
            > 0;
        if self.found {
            // Filter out inner attributes
            expr_block
                .attrs
                .retain(|attr| attr.style != AttrStyle::Inner(syn::token::Not::default()));
        }

        // Delegate to the default impl to visit nested expressions.
        visit_mut::visit_expr_block_mut(self, expr_block);
    }
}

/// Remove inner attributes (`#![...]`) from the part of the AST that will be wrapped in
/// `fn main`, as they need to be promoted to the crate level.
#[profiled]
pub fn remove_inner_attributes(expr: &mut syn::ExprBlock) -> bool {
    let remove_inner_attributes = &mut RemoveInnerAttributes { found: false };
    remove_inner_attributes.visit_expr_block_mut(expr);
    remove_inner_attributes.found
}

/// Read the contents of a file. For reading the Rust script.
/// # Errors
/// Will return `Err` if there is any file system error reading from the file path.
#[profiled]
pub fn read_file_contents(path: &Path) -> ThagResult<String> {
    debug_log!("Reading from {path:?}");
    Ok(fs::read_to_string(path)?)
}

/// Parse a Rust expression source string into a syntax tree.
///
/// Although this is primarily intended for incomplete snippets and expressions, if it finds a fully-fledged program that
/// could equally be parsed with `syn::parse_file`, it should succeed anyway by wrapping it in braces. However that is the
/// snippet path and is not encouraged as it is likely to process and wrap the code unnecessarily.
/// # Errors
/// Will return `Err` if there is any error encountered by the `syn` crate trying to parse the source string into an AST.
#[profiled]
pub fn extract_ast_expr(rs_source: &str) -> Result<Expr, syn::Error> {
    let mut expr: Result<Expr, syn::Error> = syn::parse_str::<Expr>(rs_source);
    if expr.is_err() && !(rs_source.starts_with('{') && rs_source.ends_with('}')) {
        // Try putting the expression in braces.
        let string = format!(r"{{{rs_source}}}");
        let str = string.as_str();
        // vlog!(V::N, "str={str}");

        expr = syn::parse_str::<Expr>(str);
    }
    expr
}

/// Convert a Path to a string value, assuming the path contains only valid characters.
/// # Errors
/// Will return `Err` if there is any error caused by invalid characters in the path name.
#[profiled]
pub fn path_to_str(path: &Path) -> ThagResult<String> {
    let string = path
        .to_path_buf()
        .into_os_string()
        .into_string()
        .map_err(ThagError::OsString)?;

    debug_log!("path_to_str={string}");
    Ok(string)
}

#[warn(dead_code)]
/// Display output captured to `std::process::Output`.
/// # Errors
/// Will return `Err` if the stdout or stderr is not found captured as expected.
#[profiled]
pub fn display_output(output: &Output) -> ThagResult<()> {
    // Read the captured output from the pipe
    // let stdout = output.stdout;

    // Print the captured stdout
    vlog!(V::N, "Captured stdout:");
    for result in output.stdout.lines() {
        vlog!(V::N, "{}", result?);
    }

    // Print the captured stderr
    vlog!(V::N, "Captured stderr:");
    for result in output.stderr.lines() {
        vlog!(V::N, "{}", result?);
    }
    Ok(())
}

/// Check if executable is stale, i.e. if raw source script or individual Cargo.toml
/// has a more recent modification date and time
/// # Errors
/// Will return `Err` if either the executable or the Cargo.toml for the script is missing,
/// or if there is a logic error wrapping the path and modified time.
#[cfg(feature = "build")]
#[profiled]
pub fn modified_since_compiled(
    build_state: &BuildState,
) -> ThagResult<Option<(&PathBuf, SystemTime)>> {
    let executable = &build_state.target_path;
    executable.try_exists()?;
    let Ok(metadata) = fs::metadata(executable) else {
        return Ok(None);
    };

    let baseline_modified = metadata.modified()?;

    let files = [&build_state.source_path, &build_state.cargo_toml_path];
    let mut most_recent: Option<(&PathBuf, SystemTime)> = None;
    for &file in &files {
        let Ok(metadata) = fs::metadata(file) else {
            continue;
        };

        let modified_time = metadata.modified()?;

        debug_log!("File: {file:?} modified time is {modified_time:#?}");

        if modified_time < baseline_modified {
            continue;
        }

        if most_recent.is_none()
            || modified_time
                > most_recent
                    .ok_or("Logic error unwrapping what we wrapped ourselves")?
                    .1
        {
            most_recent = Some((file, modified_time));
        }
    }
    if let Some(file) = most_recent {
        vlog!(
            V::V,
            "The most recently modified file compared to {executable:#?} is: {file:#?}"
        );

        debug_log!("Executable modified time is{baseline_modified:#?}");
    } else {
        debug_log!("Neither file was modified more recently than {executable:#?}");
    }
    Ok(most_recent)
}

/// Parse the code into an abstract syntax tree for inspection
/// if possible (should work if the code will compile)
#[must_use]
#[profiled]
pub fn to_ast(sourch_path_string: &str, source_code: &str) -> Option<Ast> {
    #[cfg(debug_assertions)]
    let start_ast = Instant::now();
    #[allow(clippy::option_if_let_else)]
    if let Ok(tree) = { syn::parse_file(source_code) } {
        #[cfg(debug_assertions)]
        {
            cvlog_emphasis!(V::V, "Parsed to syn::File");
            debug_timings(&start_ast, "Completed successful AST parse to syn::File");
        }
        Some(Ast::File(tree))
    } else if let Ok(tree) = { extract_ast_expr(source_code) } {
        #[cfg(debug_assertions)]
        {
            cvlog_emphasis!(V::V, "Parsed to syn::Expr");
            debug_timings(&start_ast, "Completed successful AST parse to syn::Expr");
        }
        Some(Ast::Expr(tree))
    } else {
        cvlog_warning!(V::V,
            "Error parsing syntax tree for `{sourch_path_string}`. Using `rustfmt` to help you debug the script."
        );
        rustfmt(sourch_path_string);

        #[cfg(debug_assertions)]
        debug_timings(&start_ast, "Completed unsuccessful AST parse");
        None
    }
}

type Zipped<'a> = (Vec<Option<&'a str>>, Vec<Option<&'a str>>);

/// Prepare a snippet for wrapping in `fn main` by separating out any inner attributes,
/// as they need to be promoted to crate level.
#[must_use]
#[profiled]
pub fn extract_inner_attribs(rs_source: &str) -> (String, String) {
    use std::fmt::Write;
    let inner_attrib_regex: &Regex = re!(r"(?m)^[\s]*#!\[.+\]");

    debug_log!("rs_source={rs_source}");

    let (inner_attribs, rest): Zipped = rs_source
        .lines()
        .map(|line| -> (Option<&str>, Option<&str>) {
            if inner_attrib_regex.is_match(line) {
                (Some(line), None)
            } else {
                (None, Some(line))
            }
        })
        .unzip();

    // eprintln!("inner_attribs={inner_attribs:#?}\nrest={rest:#?}");
    let inner_attribs = inner_attribs
        .iter()
        .flatten()
        .fold(String::new(), |mut output, &b| {
            let _ = writeln!(output, "{b}");
            output
        });

    let rest = rest.iter().flatten().fold(String::new(), |mut output, &b| {
        let _ = writeln!(output, "    {b}");
        output
    });
    (inner_attribs, rest)
}

/// Convert a Rust code snippet into a program by wrapping it in a main method and other scaffolding.
#[must_use]
#[profiled]
pub fn wrap_snippet(inner_attribs: &str, body: &str) -> String {
    debug_log!("In wrap_snippet");

    debug_log!("In wrap_snippet: inner_attribs={inner_attribs:#?}");
    let wrapped_snippet = format!(
        r#"#![allow(unused_imports,unused_macros,unused_variables,dead_code)]
{inner_attribs}
use std::error::Error;
use std::io;
use std::io::prelude::*;

#[doc = "Wrapped snippet in main method to make it a program."]
#[allow(clippy::unnecessary_wraps)]
fn main() -> Result<(), Box<dyn Error>> {{
{body}
Ok(())
}}
"#
    );

    debug_log!("wrapped_snippet={wrapped_snippet}");
    wrapped_snippet
}

/// Write the source to the destination source-code path.
/// # Errors
/// Will return `Err` if there is any error encountered opening or writing to the file.
#[profiled]
pub fn write_source(to_rs_path: &PathBuf, rs_source: &str) -> ThagResult<fs::File> {
    let mut to_rs_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(to_rs_path)?;

    // debug_log!("Writing out source to {to_rs_path:#?}:\n{}", {
    //     let lines = rs_source.lines();
    //     reassemble(lines)
    // });
    to_rs_file.write_all(rs_source.as_bytes())?;

    debug_log!("Done!");

    Ok(to_rs_file)
}

/// Create empty script file `temp.rs` to hold expression for --expr or --stdin options,
/// and open it for writing.
/// # Errors
/// Will return Err if it can't create the `rs_dyn` directory.
#[profiled]
pub fn create_temp_source_file() -> ThagResult<PathBuf> {
    // Create a directory inside of `std::env::temp_dir()`
    let gen_expr_temp_dir_path = TMPDIR.join(DYNAMIC_SUBDIR);

    // Ensure REPL subdirectory exists
    fs::create_dir_all(&gen_expr_temp_dir_path)?;

    let filename = TEMP_SCRIPT_NAME;
    let path = gen_expr_temp_dir_path.join(filename);
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;

    debug_log!("Created file: {path:#?}");
    Ok(path)
}

/// Combine the elements of a loop filter into a well-formed program.
#[must_use]
#[cfg(feature = "build")]
#[profiled]
pub fn build_loop(args: &Cli, filter: String) -> String {
    use crate::ast::is_unit_return_type;
    let maybe_ast = extract_ast_expr(&filter);
    let returns_unit = maybe_ast.map_or_else(
        |_| {
            let expr_any: &dyn Any = &filter;
            dbg!(&filter);
            !expr_any.is::<()>()
        },
        |expr| is_unit_return_type(&expr),
    );
    let loop_toml = &args.toml;
    let loop_begin = &args.begin;
    let loop_end = &args.end;
    #[allow(clippy::literal_string_with_formatting_args)]
    let filter = if returns_unit {
        filter
    } else {
        format!(r#"let _ = writeln!(io::stdout(), "{{:?}}", {filter});"#)
    };
    // dbg!(&filter);

    format!(
        r"{}
#[allow(unused_imports)]
use std::io::{{self, BufRead, Write as _}};
fn main() -> Result<(), Box<dyn std::error::Error>> {{
    {}
    // Read from stdin and execute main loop for each line
    #[allow(unused_variables)]
    let mut i = 0;
    let stdin = io::stdin();
    for line in stdin.lock().lines() {{
        let line = line?;
        i += 1;
        {filter}
    }}
    {}
    Ok(())
}}
",
        loop_toml.as_ref().map_or_else(String::new, |toml| {
            vlog!(V::V, "toml={toml}");
            format!(
                r"/*[toml]
{toml}
*/"
            )
        }),
        loop_begin.as_ref().map_or("", |prelude| {
            vlog!(V::V, "prelude={prelude}");
            prelude
        }),
        loop_end.as_ref().map_or("", |postlude| {
            vlog!(V::V, "postlude={postlude}");
            postlude
        })
    )
}

/// Clean up temporary files.
/// # Errors
/// Will return `Err` if there is any error deleting the file.
#[profiled]
pub fn clean_up(source_path: &PathBuf, target_dir_path: &PathBuf) -> io::Result<()> {
    // Delete the file
    remove_file(source_path)?;

    // Remove the directory and its contents recursively
    remove_dir_all(target_dir_path)
}

/// Display the contents of a given directory.
/// # Errors
/// Will return `Err` if there is any error reading the directory.
#[profiled]
pub fn display_dir_contents(path: &PathBuf) -> io::Result<()> {
    if path.is_dir() {
        let entries = fs::read_dir(path)?;

        vlog!(V::N, "Directory listing for {path:?}");
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_name = entry.file_name();
            vlog!(
                V::QQ,
                "  {file_name:?} ({})",
                if file_type.is_dir() {
                    "Directory"
                } else {
                    "File"
                }
            );
        }
    }
    Ok(())
}

/// Strip a set of curly braces off a Rust script, if present. This is intended to
/// undo the effect of adding them to create an expression that can be parsed into
/// an abstract syntax tree.
#[must_use]
#[profiled]
pub fn strip_curly_braces(haystack: &str) -> Option<String> {
    // Define the regex pattern
    let re: &Regex = re!(r"(?s)^\s*\{\s*(.*?)\s*\}\s*$");

    // Apply the regex to the input string
    re.captures(haystack)
        .map(|captures| captures[1].to_string())
}

#[must_use]
#[cfg(feature = "build")]
/// Get the source path for the build, either from the original source or the target directory.
/// Returns the appropriately escaped path string for the current platform.
#[profiled]
pub fn get_source_path(build_state: &BuildState) -> String {
    let binding: &PathBuf = if build_state.build_from_orig_source {
        &build_state.source_path
    } else {
        &build_state.target_dir_path.join(&build_state.source_name)
    };

    #[cfg(target_os = "windows")]
    let src_path = escape_path_for_windows(binding.to_string_lossy().as_ref());

    #[cfg(not(target_os = "windows"))]
    let src_path = binding.to_string_lossy().into_owned();
    src_path
}

/// Format a Rust source file in situ using rustfmt. For user diagnostic assistance only.
/// # Panics
/// Will panic if the `rustfmt` failed.
#[profiled]
fn rustfmt(source_path_str: &str) {
    if Command::new("rustfmt").arg("--version").output().is_ok() {
        // Run rustfmt on the source file
        let mut command = Command::new("rustfmt");
        command.arg("--edition");
        command.arg("2021");
        command.arg(source_path_str);
        command
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());
        let output = command.output().expect("Failed to run rustfmt");

        if output.status.success() {
            debug_log!("Successfully formatted {} with rustfmt.", source_path_str);
            debug_log!(
                "{}\n{}",
                source_path_str,
                String::from_utf8_lossy(&output.stdout)
            );
        } else {
            debug_log!(
                "Failed to format {} with rustfmt\n{}",
                source_path_str,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    } else {
        vlog!(
            V::QQ,
            "`rustfmt` not found. Please install it to use this script."
        );
    }
}
