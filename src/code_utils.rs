use crate::builder::gen_build_run;
use crate::cmd_args::{Cli, ProcFlags};
use crate::errors::BuildRunError;
use crate::log;
use crate::logging::Verbosity;
use crate::shared::{debug_timings, Ast, BuildState};
use crate::{debug_log, nu_color_println, nu_resolve_style};
use crate::{DYNAMIC_SUBDIR, REPL_SUBDIR, TEMP_SCRIPT_NAME, TMPDIR};

use cargo_toml::Manifest;
use lazy_static::lazy_static;
use quote::quote;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::{remove_dir_all, remove_file, OpenOptions};
use std::io::{self, BufRead, Write};
use std::option::Option;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
use std::time::{Instant, SystemTime};
use syn::visit::Visit;
use syn::{
    parse_str, Expr, File, Item, ItemExternCrate, ItemMod, ReturnType, Stmt, UsePath, UseRename,
};

/// Read the contents of a file. For reading the Rust script.
pub fn read_file_contents(path: &Path) -> Result<String, BuildRunError> {
    debug_log!("Reading from {path:?}");
    Ok(fs::read_to_string(path)?)
}

/// Infer dependencies from the abstract syntax tree to put in a Cargo.toml.
pub fn infer_deps_from_ast(syntax_tree: &Ast) -> Vec<String> {
    let use_crates = find_use_crates_ast(syntax_tree);
    let extern_crates = find_extern_crates_ast(syntax_tree);
    let use_renames = find_use_renames_ast(syntax_tree);
    let modules = find_modules_ast(syntax_tree);

    let mut dependencies = Vec::new();
    let built_in_crates = ["std", "core", "alloc", "collections", "fmt", "crate"];

    for crate_name in use_crates {
        filter_deps_ast(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &modules,
            &mut dependencies,
        );
    }

    for crate_name in extern_crates {
        filter_deps_ast(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &modules,
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
    modules: &[String],
    dependencies: &mut Vec<String>,
) {
    let crate_name_string = crate_name.to_string();
    // Filter out "crate" entries
    if !built_in_crates.contains(&crate_name)
        && !use_renames.contains(&crate_name_string)
        && !modules.contains(&crate_name_string)
    {
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

    debug_log!("use_renames from ast={:#?}", finder.use_renames);
    finder.use_renames
}

/// Identify modules for filtering use statements from Cargo.toml metadata: abstract syntax tree-based version.
fn find_modules_ast(syntax_tree: &Ast) -> Vec<String> {
    #[derive(Default)]
    struct FindMods {
        modules: Vec<String>,
    }

    impl<'a> Visit<'a> for FindMods {
        fn visit_item_mod(&mut self, node: &'a ItemMod) {
            self.modules.push(node.ident.to_string());
        }
    }

    let mut finder = FindMods::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }

    debug_log!("modules from ast={:#?}", finder.modules);
    finder.modules
}

/// Identify use crate statements for inclusion in Cargo.toml metadata: abstract syntax tree-based version.
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

    debug_log!("use_crates from ast={:#?}", finder.use_crates);
    finder.use_crates
}

/// Identify extern crate stxatements for inclusion in Cargo.toml metadata: abstract syntax tree-based version.
fn find_extern_crates_ast(syntax_tree: &Ast) -> Vec<String> {
    #[derive(Default)]
    struct FindCrates {
        extern_crates: Vec<String>,
    }

    impl<'a> Visit<'a> for FindCrates {
        fn visit_item_extern_crate(&mut self, node: &'a ItemExternCrate) {
            self.extern_crates.push(node.ident.to_string());
        }
    }

    let mut finder = FindCrates::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }

    debug_log!("extern_crates from ast={:#?}", finder.extern_crates);
    finder.extern_crates
}

/// Infer dependencies from source code to put in a Cargo.toml.
/// Fallback version for when an abstract syntax tree cannot be parsed.
pub fn infer_deps_from_source(code: &str) -> Vec<String> {
    lazy_static! {
        static ref USE_REGEX: Regex = Regex::new(r"(?m)^[\s]*use\s+([^;{]+)").unwrap();
        static ref MACRO_USE_REGEX: Regex = Regex::new(r"(?m)^[\s]*#\[macro_use\((\w+)\)").unwrap();
        static ref EXTERN_CRATE_REGEX: Regex =
            Regex::new(r"(?m)^[\s]*extern\s+crate\s+([^;{]+)").unwrap();
    }

    debug_log!("In code_utils::infer_deps_from_source");
    let use_renames = find_use_renames_source(code);
    let modules = find_modules_source(code);

    let mut dependencies = Vec::new();

    let built_in_crates = ["std", "core", "alloc", "collections", "fmt", "crate"];

    for cap in USE_REGEX.captures_iter(code) {
        let crate_name = cap[1].to_string();
        debug_log!("dependency={crate_name}");
        filter_deps_source(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &modules,
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
            &modules,
            &mut dependencies,
        );
    }

    for cap in EXTERN_CRATE_REGEX.captures_iter(code) {
        let crate_name = cap[1].to_string();
        filter_deps_source(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &modules,
            &mut dependencies,
        );
    }
    // Deduplicate the list of dependencies
    dependencies.sort();
    dependencies.dedup();

    debug_log!("dependencies from source={dependencies:#?}");
    dependencies
}

/// Filter out crates that don't need to be added as dependencies: fallback version using regex on source code.
fn filter_deps_source(
    crate_name: &str,
    built_in_crates: &[&str; 6],
    use_renames: &[String],
    modules: &[String],
    dependencies: &mut Vec<String>,
) {
    debug_log!("crate_name={crate_name}");
    let dep = if crate_name.contains(':') {
        crate_name.split_once(':').unwrap().0
    } else {
        crate_name
    };

    let dep_string = dep.to_owned();
    debug_log!("dep_string={dep_string}, built_in_crates={built_in_crates:#?}, use_renames={use_renames:#?}, modules={modules:#?}");
    if !built_in_crates.contains(&dep)
        && !use_renames.contains(&dep_string)
        && !modules.contains(&dep_string)
    {
        dependencies.push(dep_string);
    }
}

/// Identify use ... as statements for exclusion from Cargo.toml metadata.
/// Fallback version for when an abstract syntax tree cannot be parsed.
pub fn find_use_renames_source(code: &str) -> Vec<String> {
    lazy_static! {
        static ref USE_AS_REGEX: Regex = Regex::new(r"(?m)^\s*use\s+.+as\s+(\w+)").unwrap();
    }

    debug_log!("In code_utils::find_use_renames_source");

    let mut use_renames: Vec<String> = vec![];

    for cap in USE_AS_REGEX.captures_iter(code) {
        let use_rename = cap[1].to_string();
        debug_log!("use_rename={use_rename}");
        use_renames.push(use_rename);
    }

    debug_log!("use_renames from source={use_renames:#?}");
    use_renames
}

/// Identify mod statements for exclusion from Cargo.toml metadata.
/// Fallback version for when an abstract syntax tree cannot be parsed.
pub fn find_modules_source(code: &str) -> Vec<String> {
    lazy_static! {
        static ref MODULE_REGEX: Regex = Regex::new(r"(?m)^[\s]*mod\s+([^;{\s]+)").unwrap();
    }

    debug_log!("In code_utils::find_use_renames_source");

    let mut modules: Vec<String> = vec![];

    for cap in MODULE_REGEX.captures_iter(code) {
        let module = cap[1].to_string();
        debug_log!("module={module}");
        modules.push(module);
    }

    debug_log!("modules from source={modules:#?}");
    modules
}

/// Extract embedded Cargo.toml metadata from a Rust source string.
pub fn extract_manifest(
    rs_full_source: &str,
    start_parsing_rs: Instant,
) -> Result<Manifest, Box<dyn Error>> {
    let maybe_rs_toml = extract_toml_block(rs_full_source);

    let rs_manifest = if let Some(rs_toml_str) = maybe_rs_toml {
        // debug_log!("rs_toml_str={rs_toml_str}");
        Manifest::from_str(&rs_toml_str)?
    } else {
        Manifest::from_str("")?
    };

    // debug_log!("rs_manifest={rs_manifest:#?}");

    debug_timings(&start_parsing_rs, "Parsed source");
    Ok(rs_manifest)
}

fn extract_toml_block(input: &str) -> Option<String> {
    let re = Regex::new(r"(?s)/\*\[toml\](.*?)\*/").unwrap();
    re.captures(input)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Parse a Rust expression source string into a syntax tree.
/// We are not primarily catering for programs with a main method (`syn::File`),
pub fn extract_ast(rs_source: &str) -> Result<Expr, syn::Error> {
    let mut expr: Result<Expr, syn::Error> = syn::parse_str::<Expr>(rs_source);
    if expr.is_err() && !(rs_source.starts_with('{') && rs_source.ends_with('}')) {
        // Try putting the expression in braces.
        let string = format!(r"{{{rs_source}}}");
        let str = string.as_str();
        // log!(Verbosity::Normal, "str={str}");

        expr = syn::parse_str::<Expr>(str);
    }
    expr
}

/// Process a Rust expression
pub fn process_expr(
    expr_ast: &Expr,
    build_state: &mut BuildState,
    rs_source: &str,
    options: &mut Cli,
    proc_flags: &ProcFlags,
    start: &Instant,
) -> Result<(), Box<dyn Error>> {
    let syntax_tree = Some(Ast::Expr(expr_ast.clone()));
    write_source(&build_state.source_path, rs_source)?;
    let result = gen_build_run(options, proc_flags, build_state, syntax_tree, start);
    log!(Verbosity::Normal, "{result:?}");
    Ok(())
}

/// Convert a Path to a string value, assuming the path contains only valid characters.
pub fn path_to_str(path: &Path) -> Result<String, Box<dyn Error>> {
    let string = path
        .to_path_buf()
        .clone()
        .into_os_string()
        .into_string()
        .map_err(BuildRunError::OsString)?;
    debug_log!("path_to_str={string}");
    Ok(string)
}

/// Reassemble an Iterator of lines from the disentangle function to a string of text.
#[inline]
pub fn reassemble<'a>(map: impl Iterator<Item = &'a str>) -> String {
    use std::fmt::Write;
    map.fold(String::new(), |mut output, b| {
        let _ = writeln!(output, "{b}");
        output
    })
}

/// Unescape \n markers to convert a string of raw text to readable lines.
#[inline]
pub fn disentangle(text_wall: &str) -> String {
    reassemble(text_wall.lines())
}

#[allow(dead_code)]
pub fn re_disentangle(text_wall: &str) -> String {
    use std::fmt::Write;
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?m)(?P<line>.*?)(?:[\\]n|$)").unwrap();
    }

    // We extract the non-greedy capturing group named "line" from each capture of the multi-line mode regex..
    RE.captures_iter(text_wall)
        .map(|c| c.name("line").unwrap().as_str())
        .fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "{b}");
            output
        })
}

#[allow(dead_code)]
/// Display output captured to `std::process::Output`.
pub fn display_output(output: &Output) -> Result<(), Box<dyn Error>> {
    // Read the captured output from the pipe
    // let stdout = output.stdout;

    // Print the captured stdout
    log!(Verbosity::Normal, "Captured stdout:");
    for result in output.stdout.lines() {
        log!(Verbosity::Normal, "{}", result?);
    }

    // Print the captured stderr
    log!(Verbosity::Normal, "Captured stderr:");
    for result in output.stderr.lines() {
        log!(Verbosity::Normal, "{}", result?);
    }
    Ok(())
}

// TODO wait to see if redundant and get rid of it.
/// Handle the outcome of a process and optionally display its stdout and/or stderr
#[allow(dead_code)]
pub fn handle_outcome(
    exit_status: ExitStatus,
    display_stdout: bool,
    display_stderr: bool,
    output: &std::process::Output,
    process: &str,
) -> Result<(), BuildRunError> {
    if exit_status.success() {
        if display_stdout {
            let stdout = String::from_utf8_lossy(&output.stdout);
            debug_log!("{} succeeded!", process);
            stdout.lines().for_each(|line| {
                debug_log!("{}", line);
            });
        }
    } else if display_stderr {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error_msg.lines().for_each(|line| {
            debug_log!("{line}");
        });
        return Err(BuildRunError::Command(format!("{process} failed")));
    };
    Ok(())
}

/// Check if executable is stale, i.e. if raw source script or individual Cargo.toml
/// has a more recent modification date and time
pub fn modified_since_compiled(build_state: &BuildState) -> Option<(&PathBuf, SystemTime)> {
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
        debug_log!("File: {file:?} modified time is {modified_time:#?}");

        if modified_time < baseline_modified {
            continue;
        }

        if most_recent.is_none() || modified_time > most_recent.unwrap().1 {
            most_recent = Some((file, modified_time));
        }
    }
    if let Some(file) = most_recent {
        log!(
            Verbosity::Verbose,
            "The most recently modified file compared to {executable:#?} is: {file:#?}"
        );
        debug_log!("Executable modified time is{baseline_modified:#?}");
    } else {
        debug_log!("Neither file was modified more recently than {executable:#?}");
    }
    most_recent
}

/// Count the number of `main()` methods in an abstract syntax tree.
pub fn count_main_methods(syntax_tree: &Ast) -> usize {
    #[derive(Default)]
    struct FindMainFns {
        main_method_count: usize,
    }

    impl<'a> Visit<'a> for FindMainFns {
        fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
            if node.sig.ident == "main" && node.sig.inputs.is_empty() {
                self.main_method_count += 1;
            }
        }
    }

    let mut finder = FindMainFns::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }

    finder.main_method_count
}

/// Parse the code into an abstract syntax tree for inspection
/// if possible (should work if the code will compike)
pub fn to_ast(source_code: &str) -> Option<Ast> {
    let start_ast = Instant::now();
    if let Ok(tree) = extract_ast(source_code) {
        debug_timings(&start_ast, "Completed successful AST parse to syn::Expr");
        Some(Ast::Expr(tree))
    } else if let Ok(tree) = syn::parse_file(source_code) {
        // Temp: highlight the unexpected
        nu_color_println!(
            nu_resolve_style(crate::MessageLevel::Warning),
            "Parsed to syn::File"
        );
        debug_timings(&start_ast, "Completed successful AST parse to syn::File");
        Some(Ast::File(tree))
    } else {
        log!(
            Verbosity::Quiet,
            "Error parsing syntax tree. Using regex to help you debug the script"
        );
        // debug_log!("Error parsing syntax tree, using regex instead");
        debug_timings(&start_ast, "Completed unsuccessful AST parse");
        None
    }
}

/// Convert a Rust code snippet into a program by wrapping it in a main method and other scaffolding.
pub fn wrap_snippet(rs_source: &str) -> String {
    use std::fmt::Write;

    lazy_static! {
        static ref USE_REGEX: Regex = Regex::new(r"(?i)^[\s]*use\s+([^;{]+)").unwrap();
        static ref MACRO_USE_REGEX: Regex =
            Regex::new(r"(?i)^[\s]*#\[macro_use\]\s+::\s+([^;{]+)").unwrap();
        static ref EXTERN_CRATE_REGEX: Regex =
            Regex::new(r"(?i)^[\s]*extern\s+crate\s+([^;{]+)").unwrap();
    }

    debug_log!("In wrap_snippet");

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

    debug_log!("prelude={prelude:#?}\nbody={body:#?}");
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
    debug_log!("wrapped_snippet={wrapped_snippet}");
    wrapped_snippet
}

pub fn write_source(to_rs_path: &PathBuf, rs_source: &str) -> Result<fs::File, BuildRunError> {
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

/// Create the next sequential REPL file according to the `repl_nnnnnn.rs` standard used by this crate.
pub fn create_next_repl_file() -> PathBuf {
    // Create a directory inside of `std::env::temp_dir()`
    let gen_repl_temp_dir_path = TMPDIR.join(REPL_SUBDIR);

    debug_log!("repl_temp_dir = std::env::temp_dir() = {gen_repl_temp_dir_path:?}");

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
pub fn create_repl_file(gen_repl_temp_dir_path: &Path, num: u32) -> PathBuf {
    let padded_num = format!("{:06}", num);
    let dir_name = format!("repl_{padded_num}");
    let target_dir_path = gen_repl_temp_dir_path.join(dir_name);
    fs::create_dir_all(target_dir_path.clone()).expect("Failed to create REPL directory");

    let filename = format!("repl_{padded_num}.rs");
    let path = target_dir_path.join(filename);
    fs::File::create(path.clone()).expect("Failed to create file");
    debug_log!("Created file: {path:#?}");
    path
}

/// Create empty script file `temp.rs` to hold expression for --expr or --stdin options,
/// and open it for writing.
pub fn create_temp_source_file() -> PathBuf {
    // Create a directory inside of `std::env::temp_dir()`
    let gen_expr_temp_dir_path = TMPDIR.join(DYNAMIC_SUBDIR);

    // Ensure EXPR subdirectory exists
    fs::create_dir_all(gen_expr_temp_dir_path.clone()).expect("Failed to create REPL directory");

    let filename = TEMP_SCRIPT_NAME;
    let path = gen_expr_temp_dir_path.join(filename);
    // fs::File::create(path.clone()).expect("Failed to create file");
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path.clone())
        .expect("Failed to create file");
    debug_log!("Created file: {path:#?}");
    path
}

/// Clean up temporary files.
pub fn clean_up(source_path: &PathBuf, target_dir_path: &PathBuf) -> io::Result<()> {
    // Delete the file
    remove_file(source_path)?;

    // Remove the directory and its contents recursively
    remove_dir_all(target_dir_path)
}

/// Display the contents of a given directory.
pub fn display_dir_contents(path: &PathBuf) -> io::Result<()> {
    if path.is_dir() {
        let entries = fs::read_dir(path)?;

        log!(Verbosity::Normal, "Directory listing for {:?}", path);
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_name = entry.file_name();
            log!(
                Verbosity::Quiet,
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
pub fn rustfmt(build_state: &BuildState) -> Result<(), BuildRunError> {
    let target_rs_path = build_state.target_dir_path.clone();
    let target_rs_path = target_rs_path.join(&build_state.source_name);
    let source_path_str = target_rs_path
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
        log!(
            Verbosity::Quiet,
            "`rustfmt` not found. Please install it to use this script."
        );
    }
    Ok(())
}

/// Strip a set of curly braces off a Rust script, if present. This is intended to
/// undo the effect of adding them to create an expression that can be parsed into
/// an abstract syntax tree.
pub fn strip_curly_braces(haystack: &str) -> Option<String> {
    // Define the regex pattern
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?s)^\s*\{\s*(.*?)\s*\}\s*$").unwrap();
    }

    // Apply the regex to the input string
    RE.captures(haystack)
        .map(|captures| captures[1].to_string())
}

#[cfg(test)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// Cache any functions that we may find in a snippet expression in a Hashmap, so
/// that if the last statement in the expression is a function call, we can look
/// up its return type and determine whether to wrap it in a println! statement.
fn extract_functions(expr: &syn::Expr) -> HashMap<String, ReturnType> {
    #[derive(Default)]
    struct FindFns {
        function_map: HashMap<String, ReturnType>,
    }

    impl<'ast> Visit<'ast> for FindFns {
        fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
            println!("Node={:#?}", node);
            println!("Ident={}", node.sig.ident);
            println!("Output={:#?}", node.sig.output.clone());
            self.function_map
                .insert(node.sig.ident.to_string(), node.sig.output.clone());
        }
    }

    let mut finder = FindFns::default();
    finder.visit_expr(expr);

    finder.function_map
}

/// Determine if the return type of the expression is unit (the empty tuple `()`),
/// otherwise we wrap it in a println! statement.
pub fn is_unit_return_type(expr: &Expr) -> bool {
    let function_map = extract_functions(expr);
    println!("function_map={function_map:#?}");

    is_last_stmt_unit(expr, &function_map)
}

/// Recursively alternate with function `is_stmt_unit_type` until we drill down through
/// all the blocks, loops and if-conditions to find the last executable statement and
/// determine if it returns a unit type or a value worth printing.
///
/// This function finds the last statement in a given expression and determines if it
/// returns a unit type.
fn is_last_stmt_unit(expr: &Expr, function_map: &HashMap<String, ReturnType>) -> bool {
    debug_log!("%%%%%%%% expr={expr:#?}");
    match expr {
        Expr::ForLoop(for_loop) => {
            debug_log!("%%%%%%%% Expr::ForLoop(for_loop))");
            if let Some(last_stmt) = for_loop.body.stmts.last() {
                is_stmt_unit_type(last_stmt, function_map)
            } else {
                debug_log!("%%%%%%%% Not if let Some(last_stmt) = for_loop.body.stmts.last()");
                false
            }
        }
        Expr::While(_) => {
            debug_log!("%%%%%%%% Expr::While(_))");
            true
        }
        Expr::Loop(_) => {
            debug_log!("%%%%%%%% Expr::While(_))");
            true
        }
        Expr::If(expr_if) => {
            if expr_if.else_branch.is_none() {
                return true;
            };
            if let Some(last_stmt) = expr_if.then_branch.stmts.last() {
                let mut then_is_unit_type = is_stmt_unit_type(last_stmt, function_map);
                while then_is_unit_type {
                    if let Some(ref stmt) = expr_if.else_branch {
                        let expr1 = &*stmt.1;
                        // The else branch expression may only be an If or Block expression,
                        // not any of the other types of expression.
                        match expr1 {
                            Expr::Block(expr_block) => {
                                then_is_unit_type = if let Some(last_stmt) =
                                    expr_block.block.stmts.last()
                                {
                                    is_stmt_unit_type(last_stmt, function_map)
                                } else {
                                    debug_log!("%%%%%%%% Not if let Some(last_stmt) = expr_block.block.stmts.last()");
                                    false
                                };
                            }
                            Expr::If(expr_if) => {
                                if let Some(last_stmt) = expr_if.then_branch.stmts.last() {
                                    then_is_unit_type = is_stmt_unit_type(last_stmt, function_map);
                                }
                            }
                            _ => panic!("Expected else branch expression to be If or Block"),
                        }
                    }
                }
                then_is_unit_type
            } else {
                debug_log!(
                    "%%%%%%%% Not if let Some(last_stmt) = expr_if.then_branch.stmts.last()"
                );
                false
            }
        }
        Expr::Block(expr_block) => {
            if let Some(last_stmt) = expr_block.block.stmts.last() {
                is_stmt_unit_type(last_stmt, function_map)
            } else {
                debug_log!("%%%%%%%% Not if let Some(last_stmt) = expr_block.block.stmts.last()");
                false
            }
        }
        Expr::Match(expr_match) => {
            for arm in &expr_match.arms {
                // println!("arm.body={:#?}", arm.body);
                let expr = &*arm.body;
                if is_last_stmt_unit(expr, function_map) {
                    continue;
                } else {
                    debug_log!("%%%%%%%% Match arm returns unit type");
                    return false;
                }
            }
            debug_log!("%%%%%%%% Match arm returns non-unit type");
            true
        }
        Expr::Call(expr_call) => {
            if let Expr::Path(path) = &*expr_call.func {
                if let Some(ident) = path.path.get_ident() {
                    if let Some(return_type) = function_map.get(&ident.to_string()) {
                        return match return_type {
                            ReturnType::Default => {
                                debug_log!("%%%%%%%% ReturnType::Default");
                                true
                            }
                            ReturnType::Type(_, ty) => {
                                if let syn::Type::Tuple(tuple) = &**ty {
                                    debug_log!("%%%%%%%% Tuple ReturnType");
                                    tuple.elems.is_empty()
                                } else {
                                    debug_log!("%%%%%%%% Non-unit return type");
                                    false
                                }
                            }
                        };
                    }
                }
            }

            false
        }
        Expr::Closure(expr_closure) => match &expr_closure.output {
            ReturnType::Default => true,
            ReturnType::Type(_, ty) => {
                if let syn::Type::Tuple(tuple) = &**ty {
                    tuple.elems.is_empty()
                } else {
                    false
                }
            }
        },
        _ => {
            panic!("%%%%%%%% Expression not catered for: {expr:#?}");
            // false
        }
    }
}

/// Recursively alternate with function `is_last_stmt_unit` until we drill down through
/// all the blocks, loops and if-conditions to find the last executable statement and
/// determine if it returns a unit type or a value worth printing.
fn is_stmt_unit_type(last_stmt: &Stmt, function_map: &HashMap<String, ReturnType>) -> bool {
    debug_log!("%%%%%%%% last_stmt={last_stmt:#?}");
    match last_stmt {
        Stmt::Expr(expr, None) => {
            debug_log!("%%%%%%%% expr={expr:#?}");
            debug_log!("%%%%%%%% Stmt::Expr(_, None)");
            is_last_stmt_unit(expr, function_map)
        } // Expression without semicolon
        Stmt::Expr(_, Some(_)) => {
            debug_log!("%%%%%%%% Stmt::Expr(_, Some(_))");
            true
        } // Expression with semicolon returns unit
        Stmt::Macro(m) => {
            let is_some = m.semi_token.is_some();
            debug_log!("%%%%%%%% Stmt::Macro({m:#?}), m.semi_token.is_some()={is_some}");
            is_some
        } // Macro with a semicolon returns unit
        _ => {
            panic!("%%%%%%%% Statement not catered for: {last_stmt:#?}");
            // false
        }
    }
}

pub fn returns_unit(expr: &Expr) -> bool {
    // Check if the expression returns a unit value
    let is_unit_type = matches!(expr, Expr::Tuple(tuple) if tuple.elems.is_empty());
    nu_color_println!(
        nu_resolve_style(crate::MessageLevel::Emphasis),
        "is_unit_type={is_unit_type}"
    );
    is_unit_type
}

// I don't trust this from GPT
pub fn extract_expr_from_file(file: &File) -> Option<Expr> {
    // Traverse the file to find the main function and extract expressions from it
    for item in &file.items {
        if let Item::Fn(func) = item {
            if func.sig.ident == "main" {
                let stmts = &func.block.stmts;
                // Collect expressions from the statements
                let exprs: Vec<Expr> = stmts
                    .iter()
                    .filter_map(|stmt| match stmt {
                        Stmt::Expr(expr, _) => Some(expr.clone()),
                        Stmt::Macro(macro_stmt) => {
                            let mac = &macro_stmt.mac;
                            let macro_expr = quote! {
                                #mac
                            };
                            Some(
                                parse_str(&macro_expr.to_string())
                                    .expect("Unable to parse macro expression"),
                            )
                        }
                        _ => None,
                    })
                    .collect();

                // Combine the expressions into a single expression if needed
                if !exprs.is_empty() {
                    let combined_expr = quote! {
                        { #(#exprs);* }
                    };
                    return Some(
                        parse_str(&combined_expr.to_string())
                            .expect("Unable to parse combined expression"),
                    );
                }
            }
        }
    }
    None
}
