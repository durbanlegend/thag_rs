#![allow(clippy::uninlined_format_args)]
use crate::code_utils::{debug_timings, display_output, display_timings, get_proc_flags};
use crate::errors::BuildRunError;
use core::str;
use log::{debug, info};
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

use crate::cmd_args::ProcFlags;
use crate::code_utils::configure_build_state;
use crate::toml_utils::CargoManifest;

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
    // pub(crate) target_dir_str: String,
    pub(crate) cargo_toml_path: PathBuf,
    pub(crate) cargo_manifest: CargoManifest,
}

//      TODO:
//       1.  Rethink flags: -r just to run instead of requiring -n or running by default
//       2.  Move generate method to code_utils? etc.
//       3.  snippets
//       4.  features
//       5.  bool -> 2-value enums?
//       8.  Print a warning if no options chosen.
//       9.  --quiet option?.

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    configure_log();

    debug!("PACKAGE_DIR={PACKAGE_DIR}");
    debug!("PACKAGE_NAME={PACKAGE_NAME}");
    debug!("VERSION={VERSION}");

    let options = cmd_args::get_opt();
    let proc_flags = get_proc_flags(&options)?;
    debug!("flags={proc_flags:#?}");

    debug_timings(start, "Set up processing flags");

    debug!(
        "options.script={}; options.script.ends_with(RS_SUFFIX)? {})",
        options.script,
        options.script.ends_with(RS_SUFFIX)
    );

    if !options.script.ends_with(RS_SUFFIX) {
        return Err(Box::new(BuildRunError::Command(format!(
            "Script name must end in {RS_SUFFIX}"
        ))));
    }

    let (mut build_state, rs_source) = configure_build_state(&options, &proc_flags)?;

    let cargo_toml_path = &build_state.target_dir_path.join(TOML_NAME).clone();
    build_state.cargo_toml_path = cargo_toml_path.clone();
    debug!("3. build_state={build_state:#?}");
    if proc_flags.contains(ProcFlags::GENERATE) {
        generate(&build_state, &rs_source, &proc_flags)?;
    }

    if proc_flags.contains(ProcFlags::BUILD) {
        build(&proc_flags, &build_state)?;
    }

    if proc_flags.contains(ProcFlags::RUN) {
        run(&proc_flags, &build_state)?;
    }

    let process = &format!(
        "{PACKAGE_NAME} completed script {}",
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

    info!("In generate, proc_flags={proc_flags}");

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

    debug!("Writing out source {}", {
        let lines = rs_source.lines();
        code_utils::reassemble(lines)
    });

    target_rs_file.write_all(rs_source.as_bytes())?;

    // let relative_path = source_path;
    // let mut absolute_path = PathBuf::from(PACKAGE_DIR);
    // absolute_path.push(relative_path);
    // debug!("Absolute path of generated program: {absolute_path:?}");
    // info!("##### Source code generation succeeded!");

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
    info!("##### Cargo.toml generation succeeded!");

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

    debug!("BBBBBBBB In build!");

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

    // Redirect stdout to a pipe
    let mut child = build_command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn command");

    if verbose {
        display_output(&mut child);
    }

    // Wait for the child process to finish
    child.wait().expect("failed to wait for child");
    if build_command.status()?.success() {
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
    // source_stem: &str,
    build_state: &BuildState,
) -> Result<(), BuildRunError> {
    let start_run = Instant::now();
    // let verbose = proc_flags.contains(ProcFlags::VERBOSE);

    // debug!("BuildState={build_state:#?}");
    let mut absolute_path = build_state.target_dir_path.clone();
    absolute_path.push(format!("./target/debug/{}", build_state.source_stem));
    // debug!("Absolute path of generated program: {absolute_path:?}");

    let mut run_command = Command::new(format!("{}", absolute_path.display()));
    debug!("Run command is {run_command:?}");

    let exit_status = run_command.spawn()?.wait()?;

    debug!("Exit status={exit_status:#?}");

    // handle_outcome(exit_status, false, false, &run_output, "Run")?;

    // let output = String::from_utf8_lossy(&run_output.stdout);
    // println!("Run output:");
    // output.lines().for_each(|line| debug!("{line}"));

    display_timings(&start_run, "Completed run", proc_flags);

    Ok(())
}
