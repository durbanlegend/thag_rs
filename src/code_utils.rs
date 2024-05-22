use crate::cmd_args::{Opt, ProcFlags};
use crate::errors::BuildRunError;
use crate::manifest::CargoManifest;
use crate::{gen_build_run, BuildState, EXPR_SUBDIR, REPL_SUBDIR, TMPDIR};
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use std::fs::{remove_dir_all, remove_file, OpenOptions};
use strum::Display;
use syn::{Expr, UsePath};

use std::error::Error;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::option::Option;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
use std::str::FromStr;
use std::time::{Instant, SystemTime};
use syn::{visit::Visit, UseRename};

#[derive(Clone, Debug, Display)]
/// Abstract syntax tree wrapper for use with syn.
pub(crate) enum Ast {
    File(syn::File),
    Expr(syn::Expr),
    // None,
}

/// Read the contents of a file. For reading the Rust script.
pub(crate) fn read_file_contents(path: &Path) -> Result<String, BuildRunError> {
    debug!("Reading from {path:?}");
    Ok(fs::read_to_string(path)?)
}

/// Infer dependencies from the abstract syntax tree to put in a Cargo.toml.
pub(crate) fn infer_deps_from_ast(syntax_tree: &Ast) -> Vec<String> {
    let use_crates = find_use_crates_ast(syntax_tree);
    let extern_crates = find_extern_crates_ast(syntax_tree);
    let use_renames = find_use_renames_ast(syntax_tree);

    // match syntax_tree {
    //     Ast::File(ast) => debug!("&&&&&&&& Ast={ast:#?}"),
    //     Ast::Expr(ast) => debug!("&&&&&&&& Ast={ast:#?}"),
    // }

    let mut dependencies = Vec::new();
    let built_in_crates = ["std", "core", "alloc", "collections", "fmt", "crate"];

    for crate_name in use_crates {
        filter_deps_ast(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &mut dependencies,
        );
    }

    for crate_name in extern_crates {
        filter_deps_ast(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &mut dependencies,
        );
    }

    // Deduplicate the list of dependencies
    dependencies.sort();
    dependencies.dedup();

    dependencies
}

/// Filter out crates that don't need to be added as dependencies: abstract syntax tree-based version.
fn filter_deps_ast(
    crate_name: &str,
    built_in_crates: &[&str; 6],
    use_renames: &[String],
    dependencies: &mut Vec<String>,
) {
    let crate_name_string = crate_name.to_string();
    // Filter out "crate" entries
    if !built_in_crates.contains(&crate_name) && !use_renames.contains(&crate_name_string) {
        dependencies.push(crate_name_string);
    }
}

/// Identify use ... as statements for exclusion from Cargo.toml metadata: abstract syntax tree-based version.
fn find_use_renames_ast(syntax_tree: &Ast) -> Vec<String> {
    #[derive(Default)]
    struct FindCrates {
        use_renames: Vec<String>,
    }

    impl<'a> Visit<'a> for FindCrates {
        fn visit_use_rename(&mut self, node: &'a UseRename) {
            self.use_renames.push(node.rename.to_string());
        }
    }

    let mut finder = FindCrates::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }

    debug!("use_renames from ast={:#?}", finder.use_renames);
    finder.use_renames
}

/// Identify use crate statements for exclusion from Cargo.toml metadata: abstract syntax tree-based version.
fn find_use_crates_ast(syntax_tree: &Ast) -> Vec<String> {
    #[derive(Default)]
    struct FindCrates {
        use_crates: Vec<String>,
    }

    impl<'a> Visit<'a> for FindCrates {
        fn visit_use_path(&mut self, node: &'a UsePath) {
            self.use_crates.push(node.ident.to_string());
        }
    }

    let mut finder = FindCrates::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }

    debug!("use_crates from ast={:#?}", finder.use_crates);
    finder.use_crates
}

/// Identify extern crate statements for inclusion in Cargo.toml metadata: abstract syntax tree-based version.
fn find_extern_crates_ast(syntax_tree: &Ast) -> Vec<String> {
    #[derive(Default)]
    struct FindCrates {
        extern_crates: Vec<String>,
    }

    impl<'a> Visit<'a> for FindCrates {
        fn visit_use_path(&mut self, node: &'a UsePath) {
            self.extern_crates.push(node.ident.to_string());
        }
    }

    let mut finder = FindCrates::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }

    debug!("extern_crates from ast={:#?}", finder.extern_crates);
    finder.extern_crates
}

/// Infer dependencies from source code to put in a Cargo.toml.
/// Fallback version for when an abstract syntax tree cannot be parsed.
pub(crate) fn infer_deps_from_source(code: &str) -> Vec<String> {
    lazy_static! {
        static ref USE_REGEX: Regex = Regex::new(r"(?m)^[\s]*use\s+([^;{]+)").unwrap();
        static ref MACRO_USE_REGEX: Regex = Regex::new(r"(?m)^[\s]*#\[macro_use\((\w+)\)").unwrap();
        static ref EXTERN_CRATE_REGEX: Regex =
            Regex::new(r"(?m)^[\s]*extern\s+crate\s+([^;{]+)").unwrap();
    }

    debug!("######## In code_utils::infer_deps_from_source");
    let use_renames = find_use_renames_source(code);

    let mut dependencies = Vec::new();

    let built_in_crates = ["std", "core", "alloc", "collections", "fmt", "crate"];

    for cap in USE_REGEX.captures_iter(code) {
        let crate_name = cap[1].to_string();
        debug!("@@@@@@@@ dependency={crate_name}");
        filter_deps_source(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &mut dependencies,
        );
    }

    // Similar checks for other regex patterns

    for cap in MACRO_USE_REGEX.captures_iter(code) {
        let crate_name = cap[1].to_string();
        filter_deps_source(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &mut dependencies,
        );
    }

    for cap in EXTERN_CRATE_REGEX.captures_iter(code) {
        let crate_name = cap[1].to_string();
        filter_deps_source(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &mut dependencies,
        );
    }
    // Deduplicate the list of dependencies
    dependencies.sort();
    dependencies.dedup();

    debug!("dependencies from source={dependencies:#?}");
    dependencies
}

/// Filter out crates that don't need to be added as dependencies: fallback version using regex on source code.
fn filter_deps_source(
    crate_name: &str,
    built_in_crates: &[&str; 6],
    use_renames: &[String],
    dependencies: &mut Vec<String>,
) {
    if let Some((dep, _)) = crate_name.split_once(':') {
        let dep_string = dep.to_owned();
        if !built_in_crates.contains(&dep) && !use_renames.contains(&dep_string) {
            dependencies.push(dep_string);
        }
    }
}

/// Identify use ... as statements for exclusion from Cargo.toml metadata.
/// Fallback version for when an abstract syntax tree cannot be parsed.
pub(crate) fn find_use_renames_source(code: &str) -> Vec<String> {
    lazy_static! {
        static ref USE_AS_REGEX: Regex = Regex::new(r"(?m)^\s*use\s+.+as\s+(\w+)").unwrap();
    }

    debug!("######## In code_utils::find_use_renames_source");

    let mut use_renames: Vec<String> = vec![];

    for cap in USE_AS_REGEX.captures_iter(code) {
        let use_rename = cap[1].to_string();
        debug!("@@@@@@@@ use_rename={use_rename}");
        use_renames.push(use_rename);
    }

    debug!("use_renames from source={use_renames:#?}");
    use_renames
}

/// Extract embedded Cargo.toml metadata from a Rust source string.
pub(crate) fn extract_manifest(
    rs_full_source: &str,
    start_parsing_rs: Instant,
) -> Result<CargoManifest, Box<dyn Error>> {
    let maybe_rs_toml = extract_toml_block(rs_full_source);

    let rs_manifest = if let Some(rs_toml_str) = maybe_rs_toml {
        CargoManifest::from_str(&rs_toml_str)?
    } else {
        CargoManifest::from_str("")?
    };

    debug_timings(&start_parsing_rs, "Parsed source");
    Ok(rs_manifest)
}

fn extract_toml_block(input: &str) -> Option<String> {
    let re = Regex::new(r"(?s)/\*\[toml\](.*?)\*/").unwrap();
    re.captures(input)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

#[allow(dead_code)]
fn extract_rust_and_toml(source_code: &str) -> (String, String) {
    let mut rust_code = String::new();
    let mut toml_metadata = String::new();
    let mut is_metadata_block = false;
    let metadata_block_finished = false;

    for line in source_code.lines() {
        // Check if the line contains the start of the metadata block
        // debug!("line={line}");
        if !metadata_block_finished && !is_metadata_block {
            let toml_flag = "/*[toml]";
            let index = line.find(toml_flag);
            // debug!("index={index:#?}");
            if let Some(i) = index {
                // Save anything before the toml flag.
                if i > 0 {
                    let (rust, _toml_flag) = line.split_at(i);
                    rust_code.push_str(rust);
                    rust_code.push('\n');
                    // debug!("Saved rust portion: {rust}");
                }
                is_metadata_block = true;
                continue;
            };
        }

        // Check if the line contains the end of the metadata block
        let end_toml_flag = "*/";
        let index = line.find(end_toml_flag);
        // debug!("index={index:#?}");
        if let Some(i) = index {
            // Save anything before the toml flag as toml.
            if i > 0 {
                let (toml, _remnant) = line.split_at(i);
                toml_metadata.push_str(toml);
                toml_metadata.push('\n');
                // debug!("Saved toml portion: {toml}");
            }

            // Save the line as Rust code.
            rust_code.push_str(line);
            rust_code.push('\n');
            // debug!("Saved line to rustn: {line}");
            is_metadata_block = false;
            continue;
        };

        // Check if the line is a TOML comment
        let line_trim = line.trim();
        if line_trim.starts_with("//!") {
            let toml = line_trim.trim_start_matches("//!").trim();
            if !toml.is_empty() {
                toml_metadata.push_str(toml);
                toml_metadata.push('\n');
                // debug!("Pushed old-style toml comment {toml}");
            }
            continue;
        }
        if line_trim.starts_with("{//!") {
            // Save the curly brace.
            let (_rust, toml) = line.split_at(4);
            rust_code.push('{');
            rust_code.push('\n');
            // debug!("Saved opening brace as Rust: {{");
            if !toml.is_empty() {
                toml_metadata.push_str(toml.trim());
                toml_metadata.push('\n');
                // debug!("Pushed old-style toml comment {toml}");
            }
            continue;
        }

        // Add the line to the appropriate string based on the metadata block status
        if is_metadata_block {
            let toml = line.trim();
            if !toml.is_empty() {
                toml_metadata.push_str(toml);
                toml_metadata.push('\n');
                // debug!("Saved toml line: {line}");
            }
        } else {
            if line.starts_with("#!") {
                debug!("Ignoring shebang {line}");
                continue;
            }
            rust_code.push_str(line);
            rust_code.push('\n');
            // debug!("Saved rust line: {line}");
        }
    }

    // Trim trailing whitespace from both strings
    toml_metadata = toml_metadata.trim_end().to_string();
    rust_code = rust_code.trim_end().to_string();

    (toml_metadata, rust_code)
}

/// Parse a Rust expression source string into a syntax tree.
/// We are not primarily catering for programs with a main method (`syn::File`),
pub(crate) fn extract_ast(rs_source: &str) -> Result<Expr, syn::Error> {
    let mut expr: Result<Expr, syn::Error> = syn::parse_str::<Expr>(rs_source);
    if expr.is_err() && !(rs_source.starts_with('{') && rs_source.ends_with('}')) {
        // Try putting the expression in braces.
        let string = format!(r"{{{rs_source}}}");
        let str = string.as_str();
        println!("str={str}");

        expr = syn::parse_str::<Expr>(str);
    }
    expr
}

// pub(crate) fn process_expression(
//     ast: &Result<Expr, syn::Error>,
//     build_state: &mut BuildState,
//     rs_source: &str,
//     options: &mut Opt,
//     proc_flags: &ProcFlags,
//     start: &Instant,
// ) -> Result<(), BuildRunError> {
//     match ast {
//         Ok(ref expr_ast) => {
//             process_expr(expr_ast, build_state, rs_source, options, proc_flags, start)?;
//         }
//         Err(err) => {
//             nu_color_println!(
//                 nu_resolve_style(MessageLevel::Error),
//                 "Error parsing code: {}",
//                 err
//             );
//         }
//     };
//     Ok(())
// }

/// Process a Rust expression
pub(crate) fn process_expr(
    expr_ast: &Expr,
    build_state: &mut BuildState,
    rs_source: &str,
    options: &mut Opt,
    proc_flags: &ProcFlags,
    start: &Instant,
) -> Result<(), BuildRunError> {
    let syntax_tree = Some(Ast::Expr(expr_ast.clone()));
    write_source(&build_state.source_path, rs_source)?;
    let result = gen_build_run(options, proc_flags, build_state, syntax_tree, start);
    println!("{result:?}");
    Ok(())
}

/// Convert a Path to a string value, assuming the path contains only valid characters.
pub(crate) fn path_to_str(path: &Path) -> Result<String, Box<dyn Error>> {
    let string = path
        .to_path_buf()
        .clone()
        .into_os_string()
        .into_string()
        .map_err(BuildRunError::OsString)?;
    debug!("path_to_str={string}");
    Ok(string)
}

/// Reassemble an Iterator of lines from the disentangle function to a string of text.
#[inline]
pub(crate) fn reassemble<'a>(map: impl Iterator<Item = &'a str>) -> String {
    use std::fmt::Write;
    map.fold(String::new(), |mut output, b| {
        let _ = writeln!(output, "{b}");
        output
    })
}

/// Unescape \n markers to convert a string of raw text to readable lines.
#[inline]
pub(crate) fn disentangle(text_wall: &str) -> String {
    reassemble(text_wall.lines())
}

#[allow(dead_code)]
/// Display output captured to `std::process::Output`.
pub(crate) fn display_output(output: &Output) -> Result<(), Box<dyn Error>> {
    // Read the captured output from the pipe
    // let stdout = output.stdout;

    // Print the captured stdout
    println!("Captured stdout:");
    for result in output.stdout.lines() {
        println!("{}", result?);
    }

    // Print the captured stderr
    println!("Captured stderr:");
    for result in output.stderr.lines() {
        println!("{}", result?);
    }
    Ok(())
}

#[inline]
/// Display method timings when either the --verbose or --timings option is chosen.
pub(crate) fn display_timings(start: &Instant, process: &str, proc_flags: &ProcFlags) {
    let dur = start.elapsed();
    let msg = format!("{process} in {}.{}s", dur.as_secs(), dur.subsec_millis());

    debug!("{msg}");
    if proc_flags.intersects(ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        println!("{msg}");
    }
}

#[inline]
/// Developer method to log method timings.
pub(crate) fn debug_timings(start: &Instant, process: &str) {
    let dur = start.elapsed();
    debug!("{} in {}.{}s", process, dur.as_secs(), dur.subsec_millis());
}

// TODO wait to see if redundant and get rid of it.
/// Handle the outcome of a process and optionally display its stdout and/or stderr
#[allow(dead_code)]
pub(crate) fn handle_outcome(
    exit_status: ExitStatus,
    display_stdout: bool,
    display_stderr: bool,
    output: &std::process::Output,
    process: &str,
) -> Result<(), BuildRunError> {
    if exit_status.success() {
        if display_stdout {
            let stdout = String::from_utf8_lossy(&output.stdout);
            debug!("{} succeeded!", process);
            stdout.lines().for_each(|line| {
                debug!("{line}");
            });
        }
    } else if display_stderr {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error_msg.lines().for_each(|line| {
            debug!("{line}");
        });
        return Err(BuildRunError::Command(format!("{} failed", process)));
    };
    Ok(())
}

/// Check if executable is stale, i.e. if raw source script or individual Cargo.toml
/// has a more recent modification date and time
pub(crate) fn modified_since_compiled(build_state: &BuildState) -> Option<(&PathBuf, SystemTime)> {
    let executable = &build_state.target_path;
    assert!(executable.exists(), "Missing executable");
    let Ok(metadata) = fs::metadata(executable) else {
        return None;
    };

    // Panic because this shouldn't happen
    let baseline_modified = metadata
        .modified()
        .expect("Missing metadata for executable file {executable:#?}");

    let files = [&build_state.source_path, &build_state.cargo_toml_path];
    let mut most_recent: Option<(&PathBuf, SystemTime)> = None;
    for &file in &files {
        let Ok(metadata) = fs::metadata(file) else {
            continue;
        };

        let modified_time = metadata
            .modified()
            .expect("Missing metadata for file {file:#?}"); // Handle potential errors
        debug!("File: {file:?} modified time is {modified_time:#?}");

        if modified_time < baseline_modified {
            continue;
        }

        if most_recent.is_none() || modified_time > most_recent.unwrap().1 {
            most_recent = Some((file, modified_time));
        }
    }
    if let Some(file) = most_recent {
        println!("The most recently modified file compared to {executable:#?} is: {file:#?}");
        debug!("Executable modified time is{baseline_modified:#?}");
    } else {
        debug!("Neither file was modified more recently than {executable:#?}");
    }
    most_recent
}

/// Determine if a Rust script already has a main method: abstract syntax tree version.
pub(crate) fn has_main(syntax_tree: &Ast) -> bool {
    let main_methods = count_main_methods(syntax_tree);
    debug!("main_methods={main_methods}");
    has_one_main(main_methods)
}

/// Count the number of `main()` methods in an abstract syntax tree.
fn count_main_methods(syntax_tree: &Ast) -> usize {
    #[derive(Default)]
    struct FindCrates {
        main_method_count: usize,
    }

    impl<'a> Visit<'a> for FindCrates {
        fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
            if node.sig.ident == "main" && node.sig.inputs.is_empty() {
                self.main_method_count += 1;
            }
        }
    }

    let mut finder = FindCrates::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }

    finder.main_method_count
}

/// Determine if a Rust script already has a main method.
/// Fallback version for when an abstract syntax tree cannot be parsed.
pub(crate) fn has_main_alt(rs_source: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?m)^\s*fn\s* main\(\s*\)").unwrap();
    }
    let main_methods = RE.find_iter(rs_source).count();
    debug!("main_methods={main_methods}");
    has_one_main(main_methods)
}

/// Determine if a Rust script has a single main method or none.
/// Terminate processing if there is more than one main method.
fn has_one_main(main_methods: usize) -> bool {
    match main_methods {
        0 => false,
        1 => true,
        _ => {
            writeln!(
                &mut std::io::stderr(),
                "Invalid source, contains {main_methods} occurrences of fn main(), at most 1 is allowed"
            )
            .unwrap();
            std::process::exit(1);
        }
    }
}

/// Parse the code into an abstract syntax tree for inspection
/// if possible. Otherwise don't give up - it may yet compile.
pub(crate) fn to_ast(source_code: &str) -> Option<Ast> {
    let start_ast = Instant::now();
    if let Ok(tree) = syn::parse_file(source_code) {
        debug_timings(&start_ast, "Completed successful AST parse to syn::File");
        Some(Ast::File(tree))
    } else if let Ok(tree) = syn::parse_str::<Expr>(source_code) {
        debug_timings(&start_ast, "Completed successful AST parse to syn::Expr");
        Some(Ast::Expr(tree))
    } else {
        debug!("Error parsing syntax tree, using regex instead");
        debug_timings(&start_ast, "Completed unsuccessful AST parse");
        None
    }
}

/// Convert a Rust code snippet into a program by wrapping it in a main method and other scaffolding.
pub(crate) fn wrap_snippet(rs_source: &str) -> String {
    use std::fmt::Write;

    lazy_static! {
        static ref USE_REGEX: Regex = Regex::new(r"(?i)^[\s]*use\s+([^;{]+)").unwrap();
        static ref MACRO_USE_REGEX: Regex =
            Regex::new(r"(?i)^[\s]*#\[macro_use\]\s+::\s+([^;{]+)").unwrap();
        static ref EXTERN_CRATE_REGEX: Regex =
            Regex::new(r"(?i)^[\s]*extern\s+crate\s+([^;{]+)").unwrap();
    }

    debug!("In wrap_snippet");

    // // Workaround: strip off any enclosing braces.
    // let rs_source = if rs_source.starts_with('{') && rs_source.ends_with('}') {
    //     let rs_source = rs_source.trim_start_matches('{');
    //     let rs_source = rs_source.trim_end_matches('}');
    //     rs_source
    // } else {
    //     rs_source
    // };

    let (prelude, body): (Vec<Option<&str>>, Vec<Option<&str>>) = rs_source
        .lines()
        .map(|line| -> (Option<&str>, Option<&str>) {
            if USE_REGEX.is_match(line)
                || MACRO_USE_REGEX.is_match(line)
                || EXTERN_CRATE_REGEX.is_match(line)
            {
                (Some(line), None)
            } else {
                (None, Some(line))
            }
        })
        .unzip();

    // debug!("prelude={prelude:#?}\nbody={body:#?}");
    let prelude = prelude
        .iter()
        .flatten()
        .fold(String::new(), |mut output, &b| {
            let _ = writeln!(output, "{b}");
            output
        });

    let body = body.iter().flatten().fold(String::new(), |mut output, &b| {
        let _ = writeln!(output, "    {b}");
        output
    });

    let wrapped_snippet = format!(
        r"#![allow(unused_imports,unused_macros,unused_variables,dead_code)]
use std::error::Error;
use std::io;
use std::io::prelude::*;

{prelude}
fn main() -> Result<(), Box<dyn Error>> {{
{body}
Ok(())
}}
"
    );
    debug!("wrapped_snippet={wrapped_snippet}");
    wrapped_snippet
}

pub(crate) fn write_source(
    to_rs_path: &PathBuf,
    rs_source: &str,
) -> Result<fs::File, BuildRunError> {
    let mut to_rs_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(to_rs_path)?;
    debug!("Writing out source to {to_rs_path:#?}:\n{}", {
        let lines = rs_source.lines();
        reassemble(lines)
    });
    to_rs_file.write_all(rs_source.as_bytes())?;
    debug!("Done!");

    Ok(to_rs_file)
}

/// Create the next sequential REPL file according to the `repl_nnnnnn.rs` standard used by this crate.
pub(crate) fn create_next_repl_file() -> PathBuf {
    // Create a directory inside of `std::env::temp_dir()`
    let gen_repl_temp_dir_path = TMPDIR.join(REPL_SUBDIR);

    debug!("repl_temp_dir = std::env::temp_dir() = {gen_repl_temp_dir_path:?}");

    // Ensure REPL subdirectory exists
    fs::create_dir_all(gen_repl_temp_dir_path.clone()).expect("Failed to create REPL directory");

    // Find existing dirs with the pattern repl_<nnnnnn>
    let existing_dirs: Vec<_> = fs::read_dir(gen_repl_temp_dir_path.clone())
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            // println!("path={path:?}, path.is_file()={}, path.extension()?.to_str()={:?}, path.file_stem()?.to_str()={:?}", path.is_file(), path.extension()?.to_str(), path.file_stem()?.to_str());
            if path.is_dir()
                // && path.extension()?.to_str() == Some("rs")
                && path.file_stem()?.to_str()?.starts_with("repl_")
            {
                let stem = path.file_stem().unwrap();
                let num_str = stem.to_str().unwrap().trim_start_matches("repl_");
                // println!("stem={stem:?}; num_str={num_str}");
                if num_str.len() == 6 && num_str.chars().all(char::is_numeric) {
                    Some(num_str.parse::<u32>().unwrap())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // debug!("existing_files={existing_files:?}");

    let next_file_num = match existing_dirs.as_slice() {
        [] => 0, // No existing files, start with 000000
        _ if existing_dirs.contains(&999_999) => {
            // Wrap around and find the first gap
            for i in 0..999_999 {
                if !existing_dirs.contains(&i) {
                    return create_repl_file(&gen_repl_temp_dir_path, i);
                }
            }
            panic!("Cannot create new file: all possible filenames already exist in the REPL directory.");
        }
        _ => existing_dirs.iter().max().unwrap() + 1, // Increment from highest existing number
    };

    create_repl_file(&gen_repl_temp_dir_path, next_file_num)
}

// Create a REPL file on disk, given the path and sequence number.
pub(crate) fn create_repl_file(gen_repl_temp_dir_path: &Path, num: u32) -> PathBuf {
    let padded_num = format!("{:06}", num);
    let dir_name = format!("repl_{padded_num}");
    let target_dir_path = gen_repl_temp_dir_path.join(dir_name);
    fs::create_dir_all(target_dir_path.clone()).expect("Failed to create REPL directory");

    let filename = format!("repl_{padded_num}.rs");
    let path = target_dir_path.join(filename);
    fs::File::create(path.clone()).expect("Failed to create file");
    println!("Created file: {path:#?}");
    path
}

/// Create empty script file `temp.rs` to hold expression for --expr or --stdin options,
/// and open it for writing.
pub(crate) fn create_temp_source_path() -> PathBuf {
    // Create a directory inside of `std::env::temp_dir()`
    let gen_expr_temp_dir_path = TMPDIR.join(EXPR_SUBDIR);

    // Ensure EXPR subdirectory exists
    fs::create_dir_all(gen_expr_temp_dir_path.clone()).expect("Failed to create REPL directory");

    let filename = "temp.rs";
    let path = gen_expr_temp_dir_path.join(filename);
    // fs::File::create(path.clone()).expect("Failed to create file");
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path.clone())
        .expect("Failed to create file");
    println!("Created file: {path:#?}");
    path
}

#[allow(dead_code)]
/// Prompt for and read Rust source code from stdin.
pub(crate) fn read_stdin() -> Result<String, io::Error> {
    println!("Enter or paste lines of Rust source code at the prompt and press Ctrl-{} on a new line when done",
        if cfg!(windows) { 'Z' } else { 'D' }
    );
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Clean up temporary files.
pub(crate) fn clean_up(source_path: &PathBuf, target_dir_path: &PathBuf) -> io::Result<()> {
    // Delete the file
    remove_file(source_path)?;

    // Remove the directory and its contents recursively
    remove_dir_all(target_dir_path)
}

/// Display the contents of a given directory.
pub(crate) fn display_dir_contents(path: &PathBuf) -> io::Result<()> {
    if path.is_dir() {
        let entries = fs::read_dir(path)?;

        println!("Directory listing for {:?}", path);
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_name = entry.file_name();
            println!(
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

/// Format a Rust source file in situ using rustfmt.
pub(crate) fn rustfmt(build_state: &BuildState) -> Result<(), BuildRunError> {
    let source_path_buf = build_state.source_path.clone();
    let source_path_str = source_path_buf
        .to_str()
        .ok_or(String::from("Error accessing path to source file"))?;

    if Command::new("rustfmt").arg("--version").output().is_ok() {
        // Run rustfmt on the source file
        let mut command = Command::new("rustfmt");
        command.arg("--verbose");
        command.arg("--edition");
        command.arg("2021");
        command.arg(source_path_str);
        let output = command.output().expect("Failed to run rustfmt");

        if output.status.success() {
            debug!("Successfully formatted {} with rustfmt.", source_path_str);
            debug!(
                "{}\n{}",
                source_path_str,
                String::from_utf8_lossy(&output.stdout)
            );
        } else {
            debug!(
                "Failed to format {} with rustfmt\n{}",
                source_path_str,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    } else {
        eprintln!("`rustfmt` not found. Please install it to use this script.");
    }
    Ok(())
}

/// Strip a set of curly braces off a Rust script, if present. This is intended to
/// undo the effect of adding them to create an expression that can be parsed into
/// an abstract syntax tree.
pub(crate) fn strip_curly_braces(haystack: &str) -> Option<String> {
    // Define the regex pattern
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?s)^\s*\{\s*(.*?)\s*\}\s*$").unwrap();
    }

    // Apply the regex to the input string
    RE.captures(haystack)
        .map(|captures| captures[1].to_string())
}

#[cfg(test)]
fn get_mismatched_lines(expected_rust_code: &str, rust_code: &str) -> Vec<(String, String)> {
    let mut mismatched_lines = Vec::new();
    for (expected_line, actual_line) in expected_rust_code.lines().zip(rust_code.lines()) {
        if expected_line != actual_line {
            println!("expected:{expected_line}\n  actual:{actual_line}");
            mismatched_lines.push((expected_line.to_string(), actual_line.to_string()));
        }
    }
    mismatched_lines
}

#[cfg(test)]
fn compare(mismatched_lines: &[(String, String)], expected_rust_code: &str, rust_code: &str) {
    if !mismatched_lines.is_empty() {
        println!(
            r"Found mismatched lines between expected and actual code:
                expected:{}
                  actual:{}",
            // mismatched_lines
            expected_rust_code,
            rust_code
        );
    }
}

#[test]
fn test_separate_rust_and_toml_empty() {
    let source_code = "";

    let (toml_metadata, rust_code) = extract_rust_and_toml(source_code);

    // Assert TOML metadata
    assert_eq!("", toml_metadata);

    // Assert Rust code (ignoring potential whitespace differences)
    let expected_rust_code = "";

    let mismatched_lines = get_mismatched_lines(expected_rust_code, &rust_code);

    println!("mismatched_lines={mismatched_lines:#?}");

    compare(&mismatched_lines, expected_rust_code, &rust_code);
    assert!(expected_rust_code.trim() == rust_code.trim());
}

#[test]
fn test_separate_rust_and_toml_but_toml_only() {
    let source_code = r#"{/*[toml]
[dependencies]
itertools = "0.12.1"
*/}
    "#;

    let (toml_metadata, rust_code) = extract_rust_and_toml(source_code);

    // Assert TOML metadata
    let expected_toml_metadata = r#"[dependencies]
itertools = "0.12.1""#;
    assert_eq!(expected_toml_metadata, toml_metadata);

    // Assert Rust code (ignoring potential whitespace differences)
    let expected_rust_code = r#"{
}"#;

    let mismatched_lines = get_mismatched_lines(expected_rust_code, &rust_code);

    println!("mismatched_lines={mismatched_lines:#?}");

    compare(&mismatched_lines, expected_rust_code, &rust_code);
    assert!(expected_rust_code.trim() == rust_code.trim());
}

#[test]
fn test_separate_rust_and_toml_but_rust_only() {
    let source_code = r#"use rug::Integer;
let other = "world";
println!("Hello {other}!");
"#;

    let (toml_metadata, rust_code) = extract_rust_and_toml(source_code);

    // Assert TOML metadata
    let expected_toml_metadata = "";
    assert_eq!(expected_toml_metadata, toml_metadata);

    // Assert Rust code (ignoring potential whitespace differences)
    let expected_rust_code = source_code;

    let mismatched_lines = get_mismatched_lines(expected_rust_code, &rust_code);

    println!("mismatched_lines={mismatched_lines:#?}");

    compare(&mismatched_lines, expected_rust_code, &rust_code);
    assert!(expected_rust_code.trim() == rust_code.trim());
}

#[test]
fn test_separate_rust_and_toml_both_ways() {
    let source_code = r#"/*[toml]
[dependencies]
crossterm = "0.27.0"
*/

//! log = "0.4.21"

use std::io::stdout;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};

fn main() -> std::io::Result<()> {
    // using the macro
    execute!(
        stdout(),
        SetForegroundColor(Color::DarkBlue),
        SetBackgroundColor(Color::Yellow),
        Print("Styled text here."),
        ResetColor
    )?;

    println!();

    Ok(())
}
"#;

    let (toml_metadata, rust_code) = extract_rust_and_toml(source_code);

    // Assert TOML metadata
    let expected_toml_metadata = r#"[dependencies]
crossterm = "0.27.0"
log = "0.4.21""#;
    assert_eq!(expected_toml_metadata, toml_metadata);

    // Assert Rust code (ignoring potential whitespace differences)
    let expected_rust_code = r#"

use std::io::stdout;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};

fn main() -> std::io::Result<()> {
    // using the macro
    execute!(
        stdout(),
        SetForegroundColor(Color::DarkBlue),
        SetBackgroundColor(Color::Yellow),
        Print("Styled text here."),
        ResetColor
    )?;

    println!();

    Ok(())
}
"#;

    let mismatched_lines = get_mismatched_lines(expected_rust_code, &rust_code);

    println!("mismatched_lines={mismatched_lines:#?}");

    compare(&mismatched_lines, expected_rust_code, &rust_code);
    assert!(expected_rust_code.trim() == rust_code.trim());
}

#[test]
fn test_separate_rust_and_toml_whitespace() {
    let source_code = r#"/*[toml]

  [dependencies]

   crossterm = "0.27.0"
*/

//!    log = "0.4.21"

use std::io::stdout;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};

fn main() -> std::io::Result<()> {
    // using the macro
    execute!(
        stdout(),
        SetForegroundColor(Color::DarkBlue),
        SetBackgroundColor(Color::Yellow),
        Print("Styled text here."),
        ResetColor
    )?;

    println!();

    Ok(())
}
"#;

    let (toml_metadata, rust_code) = extract_rust_and_toml(source_code);

    // Assert TOML metadata
    let expected_toml_metadata = r#"[dependencies]
crossterm = "0.27.0"
log = "0.4.21""#;
    assert_eq!(expected_toml_metadata, toml_metadata);

    // Assert Rust code (ignoring potential whitespace differences)
    let expected_rust_code = r#"

use std::io::stdout;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};

fn main() -> std::io::Result<()> {
    // using the macro
    execute!(
        stdout(),
        SetForegroundColor(Color::DarkBlue),
        SetBackgroundColor(Color::Yellow),
        Print("Styled text here."),
        ResetColor
    )?;

    println!();

    Ok(())
}
"#;

    let mismatched_lines = get_mismatched_lines(expected_rust_code, &rust_code);

    println!("mismatched_lines={mismatched_lines:#?}");

    compare(&mismatched_lines, expected_rust_code, &rust_code);
    assert!(expected_rust_code.trim() == rust_code.trim());
}

#[test]
fn test_separate_rust_and_toml_full() {
    let source_code = r#"{/*[toml]
[dependencies]
itertools = "0.12.1"
*/

use itertools::Itertools;

let fib = |n: usize| -> usize {
  itertools::iterate((0, 1), |&(a, b)| (b, a + b))
    .take(n + 1)
    .last()
    .unwrap()
    .0
};

println!("Enter a number from 0 to 91");
println!("Type lines of text at the prompt and hit Ctrl-{} on a new line when done", if cfg!(windows) {'Z'} else {'D'});

let mut buffer = String::new();
io::stdin().lock().read_to_string(&mut buffer)?;

let n: usize = buffer.trim_end()
  .parse()
  .expect("Can't parse input into a positive integer");

let f = fib(n);
println!("Number {n} in the Fibonacci sequence is {f}");
}"#;

    let (toml_metadata, rust_code) = extract_rust_and_toml(source_code);

    // Assert TOML metadata
    assert_eq!(
        toml_metadata,
        r#"[dependencies]
itertools = "0.12.1""#
    );

    // Assert Rust code (ignoring potential whitespace differences)
    let expected_rust_code = r#"{

use itertools::Itertools;

let fib = |n: usize| -> usize {
  itertools::iterate((0, 1), |&(a, b)| (b, a + b))
    .take(n + 1)
    .last()
    .unwrap()
    .0
};

println!("Enter a number from 0 to 91");
println!("Type lines of text at the prompt and hit Ctrl-{} on a new line when done", if cfg!(windows) {'Z'} else {'D'});

let mut buffer = String::new();
io::stdin().lock().read_to_string(&mut buffer)?;

let n: usize = buffer.trim_end()
  .parse()
  .expect("Can't parse input into a positive integer");

let f = fib(n);
println!("Number {n} in the Fibonacci sequence is {f}");
}"#;

    let mismatched_lines = get_mismatched_lines(expected_rust_code, &rust_code);

    println!("mismatched_lines={mismatched_lines:#?}");

    compare(&mismatched_lines, expected_rust_code, &rust_code);
    assert!(expected_rust_code.trim() == rust_code.trim());
}
