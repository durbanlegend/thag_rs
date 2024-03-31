use crate::cmd_args::ProcFlags;
use crate::errors::BuildRunError;
use crate::toml_utils::CargoManifest;
use crate::PACKAGE_DIR;
use crate::RS_SUFFIX;
use crate::{cmd_args, toml_utils::rs_extract_manifest};
use crate::{BuildState, TOML_NAME};
use log::debug;
use regex::Regex;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::{ExitStatus, Output};
use std::str::FromStr;
use std::time::Instant;
use std::time::SystemTime;
use std::{collections::HashSet, error::Error, fs, path::Path};

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

pub(crate) fn rs_extract_src(rs_contents: &str) -> String {
    let rs_source = reassemble({
        rs_contents
            .lines()
            .map(str::trim_start)
            .filter(|&line| !line.starts_with("//!"))
    });
    debug!("Rust source string (rs_source) =\n{rs_source}");
    rs_source
}

// Make a best effort to help the user by inferring dependencies from the source code.
pub(crate) fn infer_dependencies(code: &str) -> HashSet<String> {
    // debug!("######## In code_utils::infer_dependencies");

    let mut dependencies = HashSet::new();

    let use_regex = Regex::new(r"(?i)^[\s]*use\s+([^;{]+)").unwrap();
    let macro_use_regex = Regex::new(r"(?i)^[\s]*#\[macro_use\]\s+::\s+([^;{]+)").unwrap();
    let extern_crate_regex = Regex::new(r"(?i)^[\s]*extern\s+crate\s+([^;{]+)").unwrap();

    let built_in_crates = &["std", "core", "alloc", "collections", "fmt"];

    for cap in use_regex.captures_iter(code) {
        let dependency = cap[1].to_string();
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
    let rs_manifest = rs_extract_manifest(&rs_full_source)?;
    let rs_source = rs_extract_src(&rs_full_source);

    debug_timings(start_parsing_rs, "Parsed source");
    Ok((rs_manifest, rs_source))
}

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

pub(crate) fn pre_config_build_state(
    options: &cmd_args::Opt,
) -> Result<BuildState, Box<dyn Error>> {
    let path = Path::new(&options.script);
    let source_name: String = path.file_name().unwrap().to_str().unwrap().to_string();
    debug!("source_name = {source_name}");
    let source_stem = {
        let Some(stem) = source_name.strip_suffix(RS_SUFFIX) else {
            return Err(Box::new(BuildRunError::Command(format!(
                "Error stripping suffix from {}",
                source_name
            ))));
        };
        stem.to_string()
    };
    let source_name = source_name.to_string();
    let current_dir_path = std::env::current_dir()?.canonicalize()?;
    let script_path = current_dir_path.join(PathBuf::from(options.script.clone()));
    debug!("script_path={script_path:#?}");
    let source_path = script_path.canonicalize()?;
    debug!("source_dir_path={source_path:#?}");
    if !source_path.exists() {
        return Err(Box::new(BuildRunError::Command(format!(
            "No script named {} or {} in path {source_path:?}",
            source_stem, source_name
        ))));
    }

    let gen_build_dir = format!("{PACKAGE_DIR}/.cargo/{source_stem}");
    debug!("gen_build_dir={gen_build_dir:?}");
    let target_dir_str = gen_build_dir.clone();
    let target_dir_path = PathBuf::from_str(&target_dir_str)?;
    let mut target_path = target_dir_path.clone();
    target_path.push(format!("./target/debug/{}", source_stem));

    let cargo_toml_path = target_dir_path.join(TOML_NAME).clone();
    let build_state = BuildState {
        source_stem,
        source_name,
        source_path,
        // source_str: source_dir_str,
        target_dir_path,
        target_dir_str,
        target_path,
        cargo_toml_path,
        ..Default::default()
    };
    // debug!("build_state={build_state:#?}");

    Ok(build_state)
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
    let re = Regex::new(r"(?x)\bfn\s* main\(\s*\)").unwrap();
    let matches = re.find_iter(source).count();
    debug!("matches={matches}");
    match matches {
        0 => false,
        1 => true,
        _ => {
            writeln!(
                &mut std::io::stderr(),
                "Invalid source, contains {matches} occurrences of fn main(), at most 1 is allowed"
            )
            .unwrap();
            std::process::exit(1);
        }
    }
}

pub(crate) fn wrap_snippet(rs_source: &str) -> String {
    use std::fmt::Write;
    // let prelude = String.new();
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
        r"{prelude}
fn main() {{
{body}}}
"
    )
}
