use crate::cmd_args::ProcFlags;
use crate::errors::BuildRunError;
use crate::manifest::CargoManifest;
use crate::{BuildState, REPL_SUBDIR, TMP_DIR};
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
pub(crate) enum Ast {
    File(syn::File),
    Expr(syn::Expr),
    // None,
}

pub(crate) fn read_file_contents(path: &Path) -> Result<String, BuildRunError> {
    debug!("Reading from {path:?}");
    Ok(fs::read_to_string(path)?)
}

// pub(crate) fn rs_extract_src(rs_contents: &str) -> String {
//     let rs_source = reassemble({
//         rs_contents
//             .lines()
//             .map(str::trim_start)
//             .filter(|&line| !line.starts_with("//!"))
//     });
//     debug!("Rust source string (rs_source) =\n{rs_source}");
//     rs_source
// }

// Inferring dependencies from the abstract syntax tree.
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

fn filter_deps_ast(
    crate_name: &str,
    built_in_crates: &[&str; 6],
    use_renames: &[String],
    dependencies: &mut Vec<String>,
) {
    let crate_name_string = crate_name.to_string();
    if !built_in_crates.contains(&crate_name) && !use_renames.contains(&crate_name_string) {
        // Filter out "crate" entries
        dependencies.push(crate_name_string);
    }
}

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

/// When no AST, make a best effort to help the user by inferring dependencies from the source code.
pub(crate) fn infer_deps_from_source(code: &str) -> Vec<String> {
    debug!("######## In code_utils::infer_deps_from_source");
    let use_renames = find_use_renames_source(code);

    let mut dependencies = Vec::new();

    let use_regex = Regex::new(r"(?m)^[\s]*use\s+([^;{]+)").unwrap();
    let macro_use_regex = Regex::new(r"(?m)^[\s]*#\[macro_use\((\w+)\)").unwrap();
    let extern_crate_regex = Regex::new(r"(?m)^[\s]*extern\s+crate\s+([^;{]+)").unwrap();

    let built_in_crates = ["std", "core", "alloc", "collections", "fmt", "crate"];

    for cap in use_regex.captures_iter(code) {
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

    for cap in macro_use_regex.captures_iter(code) {
        let crate_name = cap[1].to_string();
        filter_deps_source(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &mut dependencies,
        );
    }

    for cap in extern_crate_regex.captures_iter(code) {
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

/// When no AST, find redefined crates (that would cause Cargo search false positives) from the source code.
pub(crate) fn find_use_renames_source(code: &str) -> Vec<String> {
    debug!("######## In code_utils::find_use_renames_source");

    let use_as_regex = Regex::new(r"(?m)^\s*use\s+.+as\s+(\w+)").unwrap();

    let mut use_renames: Vec<String> = vec![];

    for cap in use_as_regex.captures_iter(code) {
        let use_rename = cap[1].to_string();
        debug!("@@@@@@@@ use_rename={use_rename}");
        use_renames.push(use_rename);
    }

    debug!("use_renames from source={use_renames:#?}");
    use_renames
}

pub(crate) fn parse_source_file(
    source_path: &Path,
) -> Result<(CargoManifest, String), Box<dyn Error>> {
    let start_parsing_rs = Instant::now();

    let rs_full_source = read_file_contents(source_path)?;

    let result = parse_source_str(&rs_full_source, start_parsing_rs);
    debug_timings(&start_parsing_rs, "Parsed source");
    result
}

pub(crate) fn parse_source_str(
    rs_full_source: &str,
    start_parsing_rs: Instant,
) -> Result<(CargoManifest, String), Box<dyn Error>> {
    let (rs_toml_str, rs_source) = separate_rust_and_toml(rs_full_source);

    let rs_manifest = CargoManifest::from_str(&rs_toml_str)?;
    //     let rs_manifest = rs_extract_manifest(&rs_full_source)?;
    //     // debug!("@@@@ rs_manifest (before deps, showing features)={rs_manifest:#?}");

    //     let rs_source = rs_extract_src(&rs_full_source);

    debug_timings(&start_parsing_rs, "Parsed source");
    Ok((rs_manifest, rs_source))
}

// fn rs_extract_manifest(rs_contents: &str) -> Result<CargoManifest, BuildRunError> {
//     let rs_toml_str = rs_extract_toml(rs_contents);
//     CargoManifest::from_str(&rs_toml_str)
// }

fn separate_rust_and_toml(source_code: &str) -> (String, String) {
    let mut rust_code = String::new();
    let mut toml_metadata = String::new();
    let mut is_metadata_block = false;
    let mut metadata_block_finished = false;

    for line in source_code.lines() {
        // Check if the line contains the start of the metadata block
        let line = line.trim();
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
        if line == "*/" {
            is_metadata_block = false;
            metadata_block_finished = true;
            // debug!("End of metadata block");
            continue;
        }

        // Check if the line is a TOML comment
        if line.starts_with("//!") {
            toml_metadata.push_str(line.trim_start_matches("//!"));
            toml_metadata.push('\n');
            // debug!("Pushed old-style toml comment");
            continue;
        }

        // Add the line to the appropriate string based on the metadata block status
        if is_metadata_block {
            toml_metadata.push_str(line);
            toml_metadata.push('\n');
            // debug!("Saved toml line: {line}");
        } else {
            rust_code.push_str(line);
            rust_code.push('\n');
            // debug!("Saved rust line: {line}");
        }
    }

    // Trim trailing whitespace from both strings
    toml_metadata = toml_metadata.trim().to_string();
    rust_code = rust_code.trim().to_string();

    (toml_metadata, rust_code)
}

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

/// Unescape \n markers in a string to convert the wall of text to readable lines.
#[inline]
pub(crate) fn reassemble<'a>(map: impl Iterator<Item = &'a str>) -> String {
    use std::fmt::Write;
    map.fold(String::new(), |mut output, b| {
        let _ = writeln!(output, "{b}");
        output
    })
}

/// Unescape \n markers in a string to convert the wall of text to readable lines.
#[inline]
pub(crate) fn disentangle(text_wall: &str) -> String {
    reassemble(text_wall.lines())
}

#[allow(dead_code)]
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
pub(crate) fn display_timings(start: &Instant, process: &str, proc_flags: &ProcFlags) {
    let dur = start.elapsed();
    let msg = format!("{process} in {}.{}s", dur.as_secs(), dur.subsec_millis());

    debug!("{msg}");
    if proc_flags.intersects(ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        println!("{msg}");
    }
}

#[inline]
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

pub(crate) fn has_main(syntax_tree: &Ast) -> bool {
    let main_methods = count_main_methods(syntax_tree);
    debug!("main_methods={main_methods}");
    has_one_main(main_methods)
}

pub(crate) fn has_main_alt(rs_source: &str) -> bool {
    let re = Regex::new(r"(?m)^\s*fn\s* main\(\s*\)").unwrap();
    let main_methods = re.find_iter(rs_source).count();
    debug!("main_methods={main_methods}");
    has_one_main(main_methods)
}

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

/// Count the number of `main()` methods
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

pub(crate) fn wrap_snippet(rs_source: &str) -> String {
    use std::fmt::Write;
    let use_regex = Regex::new(r"(?i)^[\s]*use\s+([^;{]+)").unwrap();
    let macro_use_regex = Regex::new(r"(?i)^[\s]*#\[macro_use\]\s+::\s+([^;{]+)").unwrap();
    let extern_crate_regex = Regex::new(r"(?i)^[\s]*extern\s+crate\s+([^;{]+)").unwrap();

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
            if use_regex.is_match(line)
                || macro_use_regex.is_match(line)
                || extern_crate_regex.is_match(line)
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
        r"
#![allow(unused_imports,unused_macros,unused_variables,dead_code)]
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

pub(crate) fn create_next_repl_file() -> PathBuf {
    // let repl_temp_dir = Path::new(&TMP_DIR);
    let gen_repl_temp_dir_path = TMP_DIR.join(REPL_SUBDIR);
    // Create a directory inside of `std::env::temp_dir()`
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

#[allow(dead_code)]
pub(crate) fn read_stdin() -> Result<String, io::Error> {
    println!("Enter or paste Rust source code at the prompt and hit Ctrl-D when done");
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

pub(crate) fn clean_up(source_path: &PathBuf, target_dir_path: &PathBuf) -> io::Result<()> {
    // Delete the file
    remove_file(source_path)?;

    // Remove the directory and its contents recursively
    remove_dir_all(target_dir_path)
}

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

pub(crate) fn rustfmt(build_state: &BuildState) -> Result<(), BuildRunError> {
    let source_path_buf = build_state.source_path.clone();
    let source_path_str = source_path_buf
        .to_str()
        .ok_or(String::from("Error accessing path to source file"))?;

    if Command::new("rustfmt").arg("--version").output().is_ok() {
        // Run rustfmt on the source file
        let mut command = Command::new("rustfmt");
        command.arg(source_path_str);
        let output = command.output().expect("Failed to run rustfmt");

        if output.status.success() {
            println!("Successfully formatted {} with rustfmt.", source_path_str);
        } else {
            eprintln!(
                "Failed to format {} with rustfmt:\n{}",
                source_path_str,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    } else {
        eprintln!("`rustfmt` not found. Please install it to use this script.");
    }
    Ok(())
}
