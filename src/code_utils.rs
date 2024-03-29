use crate::cmd_args::ProcFlags;
use crate::errors::BuildRunError;
use crate::toml_utils::CargoManifest;
use crate::{cmd_args, toml_utils::rs_extract_manifest};
use log::debug;
use regex::Regex;
use std::io::Read;
use std::process::ExitStatus;
use std::time::Instant;
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
    debug!("Rust source string (rs_source) = {rs_source}");
    rs_source
}

// Make a best effort to help the user by inferring dependencies from the source code.
pub(crate) fn infer_dependencies(code: &str) -> HashSet<String> {
    let mut dependencies = HashSet::new();

    let use_regex = Regex::new(r"(?i)use\s+([^;{]+)").unwrap();
    let macro_use_regex = Regex::new(r"(?i)#\[macro_use\]\s+::\s+([^;{]+)").unwrap();
    let extern_crate_regex = Regex::new(r"(?i)extern\s+crate\s+([^;{]+)").unwrap();

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

// pub(crate) fn build_code_path(source_stem: &str) -> Result<PathBuf, Box<dyn Error>> {
//     let source_name = format!("{source_stem}.rs");
//     let project_dir = env::var("PWD")?;
//     let project_path = PathBuf::from(project_dir);
//     let mut code_path: PathBuf = project_path; // .join("examples");
//                                                // let default_toml_path = code_path.join("default_cargo.toml");
//     code_path.push(source_name);
//     Ok(code_path)
// }

/// Set up the processing flags from the command line arguments and pass them back.
pub(crate) fn get_proc_flags(options: &cmd_args::Opt) -> Result<ProcFlags, Box<dyn Error>> {
    let flags = {
        if options.all && options.no_run {
            // println!(
            //     "Conflicting options {} and {} specified",
            //     options.all, options.no_run
            // );
            return Err(Box::new(BuildRunError::Command(format!(
                "Conflicting options {} and {} specified",
                options.all, options.no_run
            ))));
        }
        let mut flags = ProcFlags::empty();
        flags.set(ProcFlags::GENERATE, options.generate | options.all);
        flags.set(ProcFlags::BUILD, options.build | options.all);
        flags.set(ProcFlags::VERBOSE, options.verbose);
        flags.set(ProcFlags::TIMINGS, options.timings);
        flags.set(ProcFlags::RUN, !options.no_run);
        flags.set(ProcFlags::ALL, options.all);
        if !(flags.contains(ProcFlags::ALL)) {
            flags.set(
                ProcFlags::ALL,
                options.generate & options.build && !options.no_run,
            );
        }

        let formatted = flags.to_string();
        let parsed = formatted
            .parse::<ProcFlags>()
            .map_err(|e| BuildRunError::FromStr(e.to_string()))?;

        assert_eq!(flags, parsed);

        Ok::<cmd_args::ProcFlags, BuildRunError>(flags)
    }?;
    Ok(flags)
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

pub(crate) fn display_output(child: &mut std::process::Child) {
    // Read the captured output from the pipe
    let mut stdout = child.stdout.take().expect("failed to get stdout");
    let mut output = String::new();
    stdout
        .read_to_string(&mut output)
        .expect("failed to read stdout");

    // Print the captured stdout
    println!("Captured stdout:\n{}", output);

    let mut stderr = child.stderr.take().expect("failed to get stdout");
    stderr
        .read_to_string(&mut output)
        .expect("failed to read stderr");

    // Print the captured stderr
    println!("Captured stderr:\n{}", output);
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
