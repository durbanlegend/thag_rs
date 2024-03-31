#![allow(clippy::uninlined_format_args)]
use crate::cmd_args::{get_proc_flags, ProcFlags};
use crate::code_utils::{debug_timings, display_output, display_timings};
use crate::code_utils::{modified_since_compiled, parse_source, pre_config_build_state};
use crate::errors::BuildRunError;
use crate::toml_utils::{default_manifest, CargoManifest};

use core::str;
use log::debug;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
use std::{fs, io::Write as OtherWrite}; // Use PathBuf for paths

mod cmd_args;
mod code_utils;
mod errors;
mod toml_utils;

const PACKAGE_DIR: &str = env!("CARGO_MANIFEST_DIR");
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const RS_SUFFIX: &str = ".rs";
pub(crate) const TOML_NAME: &str = "Cargo.toml";

#[derive(Debug, Default)]
pub(crate) struct BuildState {
    pub(crate) source_stem: String,
    pub(crate) source_name: String,
    pub(crate) source_path: PathBuf,
    // pub(crate) source_str: String,
    pub(crate) target_dir_path: PathBuf,
    pub(crate) target_dir_str: String,
    pub(crate) target_path: PathBuf,
    pub(crate) cargo_toml_path: PathBuf,
    pub(crate) cargo_manifest: CargoManifest,
}

//      TODO:
//       2.  Move generate method to code_utils? etc.
//       3.  snippets
//       4.  features
//       5.  bool -> 2-value enums?
//       9.  --quiet option?.

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    configure_log();

    debug!("PACKAGE_DIR={PACKAGE_DIR}");
    debug!("PACKAGE_NAME={PACKAGE_NAME}");
    debug!("VERSION={VERSION}");

    let options = cmd_args::get_opt();
    let proc_flags = get_proc_flags(&options)?;
    debug!("proc_flags={proc_flags:#?}");

    debug_timings(start, "Set up processing flags");

    debug!(
        "options.script={}; options.script.ends_with(RS_SUFFIX)? {})",
        options.script,
        options.script.ends_with(RS_SUFFIX)
    );

    if !&options.args.is_empty() {
        debug!("... args:");
        for arg in &options.args {
            debug!("{arg}");
        }
    }

    if !options.script.ends_with(RS_SUFFIX) {
        return Err(Box::new(BuildRunError::Command(format!(
            "Script name must end in {RS_SUFFIX}"
        ))));
    }

    let mut build_state = pre_config_build_state(&options)?;

    let stale_executable = if build_state.target_path.exists() {
        modified_since_compiled(&build_state).is_some()
    } else {
        true
    };

    let force = proc_flags.contains(ProcFlags::FORCE);
    let gen_requested = proc_flags.contains(ProcFlags::GENERATE);

    if force || (gen_requested && stale_executable) {
        let (mut rs_manifest, rs_source): (CargoManifest, String) =
            parse_source(&build_state.source_path)?;
        let source_path = build_state.source_path.clone();
        if !source_path.exists() {
            return Err(Box::new(BuildRunError::Command(format!(
                "No script named {} or {} in path {source_path:?}",
                &build_state.source_stem, &build_state.source_name
            ))));
        }
        build_state.cargo_manifest =
            toml_utils::resolve_deps(&build_state, &rs_source, &mut rs_manifest)?;

        generate(&build_state, &rs_source, &proc_flags)?;
    } else {
        println!("Skipping unnecessary generation step");
        build_state.cargo_manifest = default_manifest(&build_state)?;
    }

    let build_requested = proc_flags.contains(ProcFlags::BUILD);
    if force || (build_requested && stale_executable) {
        build(&proc_flags, &build_state)?;
    } else {
        println!("Skipping unnecessary build step");
    }

    if proc_flags.contains(ProcFlags::RUN) {
        run(&proc_flags, &options.args, &build_state)?;
    }

    let process = &format!(
        "{PACKAGE_NAME} completed processing of script {}",
        &build_state.source_name
    );
    display_timings(&start, process, &proc_flags);

    Ok(())
}

fn generate(
    build_state: &BuildState,
    rs_source: &String,
    proc_flags: &ProcFlags,
) -> Result<(), Box<dyn Error>> {
    let start_gen = Instant::now();
    let verbose = proc_flags.contains(ProcFlags::VERBOSE);

    debug!("In generate, proc_flags={proc_flags}");

    fs::create_dir_all(&build_state.target_dir_path)?;

    let target_rs_path = build_state.target_dir_path.clone();
    let target_rs_path = target_rs_path.join(&build_state.source_name);
    if verbose {
        println!("GGGGGGGG Creating source file: {target_rs_path:?}");
    }
    let mut target_rs_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(target_rs_path)?;
    debug!("GGGGGGGG Done!");

    debug!("Writing out source:\n{}", {
        let lines = rs_source.lines();
        code_utils::reassemble(lines)
    });

    target_rs_file.write_all(rs_source.as_bytes())?;

    debug!("cargo_toml_path will be {:?}", &build_state.cargo_toml_path);
    if !Path::try_exists(&build_state.cargo_toml_path)? {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&build_state.cargo_toml_path)?;
    }
    // debug!("cargo_toml: {cargo_toml:?}");

    let cargo_manifest_str: &str = &build_state.cargo_manifest.to_string();

    debug!(
        "cargo_manifest_str: {}",
        code_utils::disentangle(cargo_manifest_str)
    );

    let mut toml_file = fs::File::create(&build_state.cargo_toml_path)?;
    toml_file.write_all(cargo_manifest_str.as_bytes())?;
    debug!("cargo_toml_path={:?}", &build_state.cargo_toml_path);
    debug!("##### Cargo.toml generation succeeded!");

    display_timings(&start_gen, "Completed generation", proc_flags);

    Ok(())
}

// Configure log level
fn configure_log() {
    use env_logger::fmt::WriteStyle;
    use env_logger::Builder;
    use env_logger::Env;

    let env = Env::new().filter("RUST_LOG"); //.default_write_style_or("auto");
    let mut binding = Builder::new();
    let builder = binding.parse_env(env);
    builder.write_style(WriteStyle::Always);
    builder.init();

    // Builder::new().filter_level(log::LevelFilter::Debug).init();
}

/// Build the Rust program using Cargo (with manifest path)
fn build(proc_flags: &ProcFlags, build_state: &BuildState) -> Result<(), BuildRunError> {
    let start_build = Instant::now();
    let verbose = proc_flags.contains(ProcFlags::VERBOSE);

    debug!("BBBBBBBB In build");

    let Ok(cargo_toml_path_str) = code_utils::path_to_str(&build_state.cargo_toml_path) else {
        return Err(BuildRunError::OsString(
            build_state.cargo_toml_path.clone().into_os_string(),
        ));
    };
    let mut build_command = Command::new("cargo");
    // Rustc writes to std
    let mut args = vec!["build", "--manifest-path", &cargo_toml_path_str];
    if verbose {
        args.push("--verbose");
    };

    build_command.args(&args); // .current_dir(build_dir);

    // Show sign of life in case build takes a while
    eprintln!("Building...");

    // Redirect stdout and stderr to pipes
    let mut child = build_command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let exit_status = child.wait()?;

    // Wait for the child process to finish with output collected
    let output = child.wait_with_output()?;

    if verbose {
        let _ = display_output(&output);
    }

    if exit_status.success() {
        debug!("Build succeeded");
    } else {
        return Err(BuildRunError::Command(String::from("Build failed")));
    };

    display_timings(&start_build, "Completed build", proc_flags);

    Ok(())
}

// Run the built program
fn run(
    proc_flags: &ProcFlags,
    args: &[String],
    build_state: &BuildState,
) -> Result<(), BuildRunError> {
    let start_run = Instant::now();
    debug!("RRRRRRRR In run");

    // debug!("BuildState={build_state:#?}");
    let target_path = build_state.target_path.clone();
    // debug!("Absolute path of generated program: {absolute_path:?}");

    let mut run_command = Command::new(format!("{}", target_path.display()));
    run_command.args(args);

    debug!("Run command is {run_command:?}");

    // Sandwich command between two lines of dashes in the terminal
    let dash_line = "-".repeat(50);
    println!("{dash_line}");

    let _exit_status = run_command.spawn()?.wait()?;

    let dash_line = "-".repeat(50);
    println!("{dash_line}");

    // debug!("Exit status={exit_status:#?}");

    display_timings(&start_run, "Completed run", proc_flags);

    Ok(())
}
