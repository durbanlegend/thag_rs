use crate::cmd_args::ProcFlags;
use crate::errors::BuildRunError;
use crate::manifest::CargoManifest;
use crate::{BuildState, REPL_SUBDIR, TMP_DIR};
use log::debug;
use regex::Regex;
use std::fs::{remove_dir_all, remove_file};
use std::io::{self, BufRead, Read, Write};
use std::option::Option;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output};
use std::str::FromStr;
use std::time::{Instant, SystemTime};
use std::{collections::HashSet, error::Error, fs, path::Path};
use syn::{File, Item};

#[allow(dead_code, clippy::uninlined_format_args)]
fn main() {
    let code_snippet = r#"
  use std::io;

  #[macro_use]
  extern crate serde_derive;

  fn main() {
    println!("Hello, world!");
  }
  "#;

    let dependencies = infer_dependencies(code_snippet);
    println!("Potential dependencies: {dependencies:?}");
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

// Make a best effort to help the user by inferring dependencies from the source code.
pub(crate) fn infer_dependencies(code: &str) -> HashSet<String> {
    // debug!("######## In code_utils::infer_dependencies");

    let mut dependencies = HashSet::new();

    let use_regex = Regex::new(r"(?m)^[\s]*use\s+([^;{]+)").unwrap();
    let macro_use_regex = Regex::new(r"(?m)^[\s]*#\[macro_use\]\s+::\s+([^;{]+)").unwrap();
    let extern_crate_regex = Regex::new(r"(?m)^[\s]*extern\s+crate\s+([^;{]+)").unwrap();

    let built_in_crates = &["std", "core", "alloc", "collections", "fmt"];

    for cap in use_regex.captures_iter(code) {
        let dependency = cap[1].to_string();
        debug!("@@@@@@@@ dependency={dependency}");
        if !built_in_crates
            .iter()
            .any(|builtin| dependency.starts_with(builtin))
        {
            if let Some((dep, _)) = dependency.split_once(':') {
                dependencies.insert(dep.to_owned());
            }
        }
    }

    // Similar checks for other regex patterns

    for cap in macro_use_regex.captures_iter(code) {
        let dependency = cap[1].to_string();
        if !built_in_crates
            .iter()
            .any(|builtin| dependency.starts_with(builtin))
        {
            dependencies.insert(dependency);
        }
    }

    for cap in extern_crate_regex.captures_iter(code) {
        let dependency = cap[1].to_string();
        if !built_in_crates
            .iter()
            .any(|builtin| dependency.starts_with(builtin))
        {
            dependencies.insert(dependency);
        }
    }

    dependencies
}

pub(crate) fn parse_source(source_path: &Path) -> Result<(CargoManifest, String), Box<dyn Error>> {
    let start_parsing_rs = Instant::now();

    let rs_full_source = read_file_contents(source_path)?;

    let (rs_source, rs_toml_str) = separate_rust_and_toml(&rs_full_source);

    let rs_manifest = CargoManifest::from_str(&rs_toml_str)?;
    //     let rs_manifest = rs_extract_manifest(&rs_full_source)?;
    //     // debug!("@@@@ rs_manifest (before deps, showing features)={rs_manifest:#?}");

    //     let rs_source = rs_extract_src(&rs_full_source);

    debug_timings(start_parsing_rs, "Parsed source");
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

    for line in source_code.lines() {
        // Check if the line contains the start of the metadata block
        if line.trim().starts_with("/*[toml]") {
            is_metadata_block = true;
            continue;
        }

        // Check if the line contains the end of the metadata block
        if line.trim() == "*/" {
            is_metadata_block = false;
            continue;
        }

        // Check if the line is a TOML comment
        if line.trim().starts_with("//!") {
            toml_metadata.push_str(line.trim_start_matches("//!"));
            toml_metadata.push('\n');
            continue;
        }

        // Add the line to the appropriate string based on the metadata block status
        if is_metadata_block {
            toml_metadata.push_str(line);
            toml_metadata.push('\n');
        } else {
            rust_code.push_str(line);
            rust_code.push('\n');
        }
    }

    // Trim trailing whitespace from both strings
    rust_code = rust_code.trim().to_string();
    toml_metadata = toml_metadata.trim().to_string();

    (rust_code, toml_metadata)
}

// fn rs_extract_toml(rs_contents: &str) -> String {
//     let rs_toml_str = {
//         let str_iter = rs_contents
//             .lines()
//             .map(str::trim_start)
//             .filter(|&line| line.starts_with("//!"))
//             .map(|line| line.trim_start_matches('/').trim_start_matches('!'));
//         reassemble(str_iter)
//     };
//     debug!("Rust source manifest info (rs_toml_str) = {rs_toml_str}");
//     rs_toml_str
// }

pub(crate) fn path_to_str(path: &Path) -> Result<String, Box<dyn Error>> {
    let string = path
        .to_path_buf()
        .clone()
        .into_os_string()
        .into_string()
        .map_err(BuildRunError::OsString)?;
    debug!("current_dir_str={string}");
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
pub(crate) fn debug_timings(start: Instant, process: &str) {
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

        if modified_time < baseline_modified {
            continue;
        }

        if most_recent.is_none() || modified_time > most_recent.unwrap().1 {
            most_recent = Some((file, modified_time));
        }
    }
    // if let Some((file, _mod_time)) = most_recent {
    if let Some(file) = most_recent {
        println!("The most recently modified file compared to {executable:#?} is: {file:#?}");
        debug!("Executable modified time is{baseline_modified:#?}");
    } else {
        debug!("Neither file was modified more recently than {executable:#?}");
    }
    most_recent
}

pub(crate) fn has_main(source: &str) -> bool {
    let main_methods = count_main_methods(source);
    debug!("main_methods={main_methods}");
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

pub(crate) fn count_main_methods(source_code: &str) -> usize {
    // Parse the source code into a syntax tree
    let syntax_tree: File = match syn::parse_str(source_code) {
        Ok(tree) => tree,
        Err(_) => return 0, // Return 0 if parsing fails
    };

    // Count the number of main() methods
    let mut main_method_count = 0;
    for item in syntax_tree.items {
        if let Item::Fn(item_fn) = item {
            // Check if the function is named "main" and has no arguments
            if item_fn.sig.ident == "main" && item_fn.sig.inputs.is_empty() {
                main_method_count += 1;
            }
        }
    }
    main_method_count
}

pub(crate) fn wrap_snippet(rs_source: &str) -> String {
    use std::fmt::Write;
    let use_regex = Regex::new(r"(?i)^[\s]*use\s+([^;{]+)").unwrap();
    let macro_use_regex = Regex::new(r"(?i)^[\s]*#\[macro_use\]\s+::\s+([^;{]+)").unwrap();
    let extern_crate_regex = Regex::new(r"(?i)^[\s]*extern\s+crate\s+([^;{]+)").unwrap();

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

    format!(
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
    )
}

pub(crate) fn create_next_repl_file() -> PathBuf {
    // let repl_temp_dir = Path::new(&TMP_DIR);
    let repl_temp_dir = TMP_DIR.join(REPL_SUBDIR);
    // Create a directory inside of `std::env::temp_dir()`
    debug!("repl_temp_dir = std::env::temp_dir() = {repl_temp_dir:?}");

    // Ensure REPL subdirectory exists
    fs::create_dir_all(repl_temp_dir.clone()).expect("Failed to create REPL directory");

    // Find existing files with the pattern repl_<nnnnnn>.rs
    let existing_files: Vec<_> = fs::read_dir(repl_temp_dir.clone())
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            // println!("path={path:?}, path.is_file()={}, path.extension()?.to_str()={:?}, path.file_stem()?.to_str()={:?}", path.is_file(), path.extension()?.to_str(), path.file_stem()?.to_str());
            if path.is_file()
                && path.extension()?.to_str() == Some("rs")
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

    let next_file_num = match existing_files.as_slice() {
        [] => 0, // No existing files, start with 000000
        _ if existing_files.contains(&999_999) => {
            // Wrap around and find the first gap
            for i in 0..999_999 {
                if !existing_files.contains(&i) {
                    return create_repl_file(&repl_temp_dir, i);
                }
            }
            panic!("Cannot create new file: all possible filenames already exist in the REPL directory.");
        }
        _ => existing_files.iter().max().unwrap() + 1, // Increment from highest existing number
    };

    create_repl_file(&repl_temp_dir, next_file_num)
}

pub(crate) fn create_repl_file(repl_dir: &Path, num: u32) -> PathBuf {
    let padded_num = format!("{:06}", num);
    let filename = format!("repl_{}.rs", padded_num);
    let path = repl_dir.join(&filename);
    fs::File::create(path.clone()).expect("Failed to create file");
    println!("Created file: {}", filename);
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

pub(crate) fn rustfmt(build_state: &BuildState) -> Result<(), Box<dyn Error>> {
    let source_path_buf = build_state.source_path.clone();
    let source_path_str = source_path_buf
        .to_str()
        .ok_or("Error accessing path to source file")?;

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
