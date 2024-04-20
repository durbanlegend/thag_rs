#![allow(clippy::uninlined_format_args)]
use crate::cmd_args::{get_opt, get_proc_flags, ProcFlags};
use crate::code_utils::{
    clean_up, debug_timings, display_dir_contents, display_timings, wrap_snippet,
};
use crate::code_utils::{modified_since_compiled, parse_source};
use crate::errors::BuildRunError;
use crate::manifest::CargoManifest;
use crate::tui_editor::Editor;

use clap::command;
use clap::Parser;
use clap_repl::ClapEditor;
use core::str;
use env_logger::{fmt::WriteStyle, Builder, Env};
use log::debug;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::time::Instant;
use std::{fs, io::Write as OtherWrite}; // Use PathBuf for paths

mod cmd_args;
mod code_utils;
mod errors;
mod manifest;
mod tui_editor;

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
    pub(crate) target_dir_path: PathBuf,
    pub(crate) target_dir_str: String,
    pub(crate) target_path: PathBuf,
    pub(crate) cargo_toml_path: PathBuf,
    pub(crate) cargo_manifest: Option<CargoManifest>,
    pub(crate) must_gen: bool,
    pub(crate) must_build: bool,
}

impl BuildState {
    pub(crate) fn pre_configure(
        options: &cmd_args::Opt,
        proc_flags: &ProcFlags,
        empty: bool,
    ) -> Result<Self, Box<dyn Error>> {
        if options.script.is_none() {
            return Err(Box::new(BuildRunError::NoneOption(
                "No script specified".to_string(),
            )));
        }
        let script = (options.script).clone().unwrap();
        let path = Path::new(&script);
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
        let script_path = current_dir_path.join(PathBuf::from(script.clone()));
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
        let target_path_clone = target_path.clone();

        let cargo_toml_path = target_dir_path.join(TOML_NAME).clone();

        let mut build_state = Self {
            source_stem,
            source_name,
            source_path,
            target_dir_path,
            target_dir_str,
            target_path,
            cargo_toml_path,
            ..Default::default()
        };

        // let (must_gen, must_build) = get_must_gen_build(empty, &mut build_state, &proc_flags);
        let stale_executable = if !empty && target_path_clone.exists() {
            modified_since_compiled(&build_state).is_some()
        } else {
            true
        };
        let force = proc_flags.contains(ProcFlags::FORCE);
        let gen_requested = proc_flags.contains(ProcFlags::GENERATE);
        let build_requested = proc_flags.contains(ProcFlags::BUILD);
        let repl = proc_flags.contains(ProcFlags::REPL);
        build_state.must_gen = force || repl || (gen_requested && stale_executable);
        build_state.must_build = force || repl || (build_requested && stale_executable);

        debug!("build_state={build_state:#?}");

        Ok(build_state)
    }
}

#[derive(Debug, Parser)]
#[command(name = "", arg_required_else_help(true))] // This name will show up in clap's error messages, so it is important to set it to "".
enum LoopCommand {
    /// Enter, paste or modify your code and optionally edit your generated Cargo.toml .
    Continue,
    /// Delete generated files
    Delete,
    /// List generated files
    List,
    /// Exit REPL
    Exit,
}

#[derive(Debug, Parser)]
#[command(name = "", arg_required_else_help(true))] // This name will show up in clap's error messages, so it is important to set it to "".
enum ProcessCommand {
    /// Cancel and discard this code, restart REPL
    Cancel,
    /// Return to editor for another try
    Retry,
    /// Attempt to build and run your Rust code
    Submit,
    // Exit REPL
    Exit,
}

//      TODO:
//       0.  Relocate target directory to ~./cargo.
//       1.  Rename tui_textarea_editor to tui_editor.
//       2.  Simple repl option for snippets?
//       3.  Replace //! by //: or something else that doesn't conflict with intra-doc links.
//       4.  Debug tui_textarea_editor.rs not saving Cargo.toml.
//       5.  Don't infer dependencies from use statements that refer back to something already
//              defined, like KeyCode and Constraint in tui_scrollview.rs.
//       6.  bool -> 2-value enums?
//       7.  Find a way to print out a nice prompt before loop
//       8.  Cat files before delete.
//       9.  --quiet option?.
//      10.  Delete generated files by default on exit.
//      11.  Support calculator-like snippets.
//      12.  Consider supporting vi editor family or
//
#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    configure_log();

    debug!("PACKAGE_DIR={PACKAGE_DIR}");
    debug!("PACKAGE_NAME={PACKAGE_NAME}");
    debug!("VERSION={VERSION}");

    let mut options = get_opt();
    let proc_flags = get_proc_flags(&options)?;
    debug!("proc_flags={proc_flags:#?}");

    // let m = clap::command!().get_matches();
    // debug!("matches={m:?}");

    debug_timings(start, "Set up processing flags");

    if !&options.args.is_empty() {
        debug!("... args:");
        for arg in &options.args {
            debug!("{arg}");
        }
    }

    let repl = proc_flags.contains(ProcFlags::REPL);

    let (script, empty) = if let Some(ref script) = options.script {
        if !script.ends_with(RS_SUFFIX) {
            return Err(Box::new(BuildRunError::Command(format!(
                "Script name must end in {RS_SUFFIX}"
            ))));
        }
        (script.to_owned(), false)
    } else {
        assert!(repl);
        let path = code_utils::create_next_repl_file();

        let script = path.file_name().unwrap().to_str().unwrap().to_string();
        (script, true)
    };

    // TODO Distinguish two meanings of empty.
    //     1. No script provided for repl, one has been provided by this point, so this
    //         meaning is redundant at this point in the code.
    //     2. The script in 1. is supposed to have been filled in but may still be empty,

    if empty {
        // Backfill script name into CLI options before calling pre_config_build_state.
        options.script = Some(format!("examples/{script}"));
    }

    // let mut build_state = pre_config_build_state(&options)?;
    let mut build_state = BuildState::pre_configure(&options, &proc_flags, empty)?;

    // let (must_gen, must_build) = get_must_gen_build(empty, &mut build_state, &proc_flags);

    if build_state.must_gen {
        let (mut rs_manifest, rs_source): (CargoManifest, String) =
            parse_source(&build_state.source_path)?;

        build_state.cargo_manifest = Some(manifest::merge_manifest(
            &build_state,
            &rs_source,
            &mut rs_manifest,
        )?);
    }

    if repl {
        let dash_line = "-".repeat(50);
        println!("{dash_line}");
        // println!("Enter continue, exit or help");
        let mut loop_editor = ClapEditor::<LoopCommand>::new();
        let mut loop_command = loop_editor.read_command();
        'level2: loop {
            let Some(ref command) = loop_command else {
                loop_command = loop_editor.read_command();
                continue 'level2;
            };
            match command {
                LoopCommand::Exit => return Ok(()),
                LoopCommand::Delete => {
                    clean_up(&build_state.source_path, &build_state.target_dir_path)?;
                    println!("Deleted");
                }
                LoopCommand::List => {
                    // Display file listing
                    if build_state.source_path.exists() {
                        println!("File: {:?}", &build_state.source_path);
                    }

                    // Display directory contents
                    display_dir_contents(&build_state.target_dir_path)?;

                    // Check if neither file nor directory exist
                    if !&build_state.source_path.exists() && !&build_state.target_dir_path.exists()
                    {
                        println!("No temporary files found");
                    }
                }
                LoopCommand::Continue => {
                    let files = [
                        format!("{}", build_state.source_path.display()),
                        format!("{}/Cargo.toml", build_state.target_dir_str),
                    ]
                    .into_iter();
                    debug!("files={files:#?}");
                    Editor::new(files)?.run()?;

                    println!("Enter cancel, retry, submit, exit or help");
                    let mut process_editor = ClapEditor::<ProcessCommand>::new();
                    'level3: loop {
                        let Some(command) = process_editor.read_command() else {
                            continue 'level3;
                        };
                        match command {
                            ProcessCommand::Exit => return Ok(()),
                            ProcessCommand::Submit => {
                                let result = gen_build_run(
                                    // empty,
                                    &mut options,
                                    &proc_flags,
                                    // &script,
                                    &mut build_state,
                                    &start,
                                );
                                if result.is_err() {
                                    println!("{result:?}");
                                }

                                break 'level3;
                            }
                            ProcessCommand::Cancel => {
                                loop_command = loop_editor.read_command();
                                continue 'level2;
                            }
                            ProcessCommand::Retry => {
                                loop_command = Some(LoopCommand::Continue);
                                continue 'level2;
                            }
                        }
                    }
                }
            }
            loop_command = loop_editor.read_command();
        }
    } else {
        gen_build_run(
            // empty,
            &mut options,
            &proc_flags,
            // &script,
            &mut build_state,
            &start,
        )?;
    }

    Ok(())
}

fn gen_build_run(
    // empty: bool,
    options: &mut cmd_args::Opt,
    proc_flags: &ProcFlags,
    // script: &str,
    build_state: &mut BuildState,
    start: &Instant,
) -> Result<(), Box<dyn Error>> {
    let verbose = proc_flags.contains(ProcFlags::VERBOSE);
    let proc_flags = &proc_flags;
    let options = &options;
    // let build_state: &mut BuildState = build_state;

    if build_state.must_gen {
        let (mut rs_manifest, rs_source): (CargoManifest, String) =
            parse_source(&build_state.source_path)?;

        build_state.cargo_manifest = Some(manifest::merge_manifest(
            build_state,
            &rs_source,
            &mut rs_manifest,
        )?);

        let has_main = code_utils::has_main(&rs_source);
        if verbose {
            println!("Source does not contain fn main(), thus a snippet");
        }
        let rs_source = if has_main {
            rs_source
        } else {
            wrap_snippet(&rs_source)
        };

        generate(build_state, &rs_source, proc_flags)?;
    } else {
        println!("Skipping unnecessary generation step. Use --force (-f) to override.");
        // build_state.cargo_manifest = Some(default_manifest(build_state)?);
        build_state.cargo_manifest = None; // Don't need it in memory, build will find it on disk
    }
    if build_state.must_build {
        build(proc_flags, build_state)?;
    } else {
        println!("Skipping unnecessary build step. Use --force (-f) to override.");
    }
    if proc_flags.contains(ProcFlags::RUN) {
        run(proc_flags, &options.args, build_state)?;
    }
    let process = &format!(
        "{PACKAGE_NAME} completed processing script {}",
        build_state.source_name
    );
    display_timings(start, process, proc_flags);
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

    let cargo_manifest_str: &str = &build_state.cargo_manifest.as_ref().unwrap().to_string();

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
    // let verbose = proc_flags.contains(ProcFlags::VERBOSE);

    debug!("BBBBBBBB In build");

    let Ok(cargo_toml_path_str) = code_utils::path_to_str(&build_state.cargo_toml_path) else {
        return Err(BuildRunError::OsString(
            build_state.cargo_toml_path.clone().into_os_string(),
        ));
    };
    let mut build_command = Command::new("cargo");
    // Rustc writes to std
    let args = vec!["build", "--manifest-path", &cargo_toml_path_str];
    // if verbose {
    //     args.push("--verbose");
    // };

    build_command.args(&args); // .current_dir(build_dir);

    // Show sign of life in case build takes a while
    eprintln!("Building...");

    // Redirect stdout and stderr to inherit from the parent process (terminal)
    build_command
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    // Execute the command and handle the result
    let output = build_command
        .spawn()
        .expect("failed to spawn cargo build process");

    // Wait for the process to finish
    let exit_status = output
        .wait_with_output()
        .expect("failed to wait on cargo build");

    if exit_status.status.success() {
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
