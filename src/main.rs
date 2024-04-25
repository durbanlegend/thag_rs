#![allow(clippy::uninlined_format_args)]
use crate::cmd_args::{get_opt, get_proc_flags, ProcFlags};
use crate::code_utils::{
    clean_up, debug_timings, display_dir_contents, display_timings, rustfmt, wrap_snippet,
};
use crate::code_utils::{modified_since_compiled, parse_source};
use crate::errors::BuildRunError;
use crate::manifest::{default_manifest, CargoManifest};

use clap::Parser;
use clap_repl::ClapEditor;
use convert_case::{Case, Casing};
use core::str;
use env_logger::{fmt::WriteStyle, Builder, Env};
use homedir::get_my_home;
use lazy_static::lazy_static;
use log::{debug, log_enabled, Level::Debug};
use owo_colors::colors::{BrightWhite, Red};
use owo_colors::{OwoColorize, Stream};
use quote::quote;
use rustyline::config::Configurer;
use rustyline::DefaultEditor;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::{fs, io::Write as OtherWrite}; // Use PathBuf for paths
use strum::{EnumIter, EnumProperty, IntoEnumIterator, IntoStaticStr};
use syn::{self, Expr};

mod cmd_args;
mod code_utils;
mod errors;
mod manifest;
mod owo_styles;
mod tui_editor;

// const PACKAGE_DIR: &str = env!("CARGO_MANIFEST_DIR");
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const REPL_SUBDIR: &str = "rs_repl";
const RS_SUFFIX: &str = ".rs";
pub(crate) const TOML_NAME: &str = "Cargo.toml";

lazy_static! {
    // #[derive(Debug)]
    // static ref HOME_DIR: &'static Path = get_my_home().unwrap().unwrap().as_path();
    static ref TMP_DIR: PathBuf = env::temp_dir();
}

#[derive(Debug)]
pub(crate) enum ScriptState {
    /// Repl with no script name provided by user
    #[allow(dead_code)]
    Anonymous,
    /// Repl with script name
    NamedEmpty { script: String },
    /// Script name provided by user
    Named { script: String },
}

impl ScriptState {
    pub(crate) fn get_script(&self) -> Option<String> {
        match self {
            ScriptState::Anonymous => None,
            ScriptState::NamedEmpty { script } | ScriptState::Named { script } => {
                Some(script.to_string())
            }
        }
    }
}

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
        proc_flags: &ProcFlags,
        script_state: &ScriptState,
    ) -> Result<Self, Box<dyn Error>> {
        let maybe_script = script_state.get_script();
        if maybe_script.is_none() {
            return Err(Box::new(BuildRunError::NoneOption(
                "No script specified".to_string(),
            )));
        }
        let script = (maybe_script).clone().unwrap();
        let path = Path::new(&script);
        let source_name: String = path.file_name().unwrap().to_str().unwrap().to_string();
        // debug!("source_name={source_name}");
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
        // debug!("script_path={script_path:#?}");
        let source_path = script_path.canonicalize()?;
        // debug!("source_dir_path={source_path:#?}");
        if !source_path.exists() {
            return Err(Box::new(BuildRunError::Command(format!(
                "No script named {} or {} in path {source_path:?}",
                source_stem, source_name
            ))));
        }

        // let gen_build_dir = format!("{PACKAGE_DIR}/.cargo/{source_stem}");

        // // debug!("gen_build_dir={gen_build_dir:?}");
        // let target_dir_str = gen_build_dir.clone();
        // let target_dir_path = PathBuf::from_str(&target_dir_str)?;
        // let mut target_path = target_dir_path.clone();
        // target_path.push(format!("./target/debug/{}", source_stem));

        let home_dir = get_my_home()?.ok_or("Can't resolve home directory")?;
        debug!("home_dir={}", home_dir.display());
        let target_dir_path = home_dir.join(format!(".cargo/{source_stem}"));
        debug!("target_dir_path={}", target_dir_path.display());
        // let gen_build_dir = format!("/.cargo/{source_stem}", home_dir.display().to_str());
        // debug!("gen_build_dir={gen_build_dir:?}");
        // let target_dir_str = gen_build_dir.clone();
        // let target_dir_path = PathBuf::from_str(&target_dir_str)?;
        let target_dir_str = target_dir_path.display().to_string();
        let target_path = target_dir_path
            // .clone()
            .join(format!("./target/debug/{}", source_stem));
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

        let stale_executable = matches!(script_state, ScriptState::NamedEmpty { .. })
            || !target_path_clone.exists()
            || modified_since_compiled(&build_state).is_some();
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

#[derive(Debug, Parser, EnumIter, EnumProperty, IntoStaticStr)]
#[command(name = "")] // This name will show up in clap's error messages, so it is important to set it to "".
enum LoopCommand {
    /// Enter, paste or modify your code and optionally edit your generated Cargo.toml
    #[clap(visible_alias = "c")]
    Continue,
    /// Delete generated files
    #[clap(visible_alias = "d")]
    Delete,
    /// Evaluate an expression. Enclose complex expressions in braces {}.
    #[clap(visible_alias = "e")]
    Eval,
    /// List generated files
    #[clap(visible_alias = "l")]
    List,
    /// Exit REPL
    #[clap(visible_alias = "q")]
    Quit,
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
    Quit,
}

//      TODO:
//       1.
//       2.  tui-editor auto-save or check for unsaved changes on quit.
//       3.  Replace //! by //: or something else that doesn't conflict with intra-doc links.
//       4.  Consider adding braces around repl if not an expression.
//       5.  Don't infer dependencies from use statements that refer back to something already
//              defined, like KeyCode and Constraint in tui_scrollview.rs.
//       6.  bool -> 2-value enums?
//       7.  Find a way to print out a nice prompt before loop
//       8.  Cat files before delete.
//       9.  --quiet option?.
//      10.  Consider making script name optional, with -n/stdin parm as per my runner changes?
//      11.  Clean up debugging
//      12.  Consider supporting vi editor family, nvim/Helix for rust-analyzer support or editor crate
//
#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    configure_log();

    let mut options = get_opt();
    let proc_flags = get_proc_flags(&options)?;

    if log_enabled!(Debug) {
        debug_print_config();
        debug!("proc_flags={proc_flags:#?}");
        debug_timings(start, "Set up processing flags");

        if !&options.args.is_empty() {
            debug!("... args:");
            for arg in &options.args {
                debug!("{arg}");
            }
        }
    }

    // Access TMP_DIR
    println!("Temporary directory: {:?}", *TMP_DIR);

    let repl = proc_flags.contains(ProcFlags::REPL);

    let script_state = if let Some(ref script) = options.script {
        if !script.ends_with(RS_SUFFIX) {
            return Err(Box::new(BuildRunError::Command(format!(
                "Script name must end in {RS_SUFFIX}"
            ))));
        }
        let script = script.to_owned();
        ScriptState::Named { script }
    } else {
        assert!(repl);
        let path = code_utils::create_next_repl_file();
        let script = path.to_str().unwrap().to_string();
        ScriptState::NamedEmpty { script }
    };

    if repl {
        debug!("script_state={script_state:?}");
    }
    let mut build_state = BuildState::pre_configure(&proc_flags, &script_state)?;
    if repl {
        debug!("build_state.source_path={:?}", build_state.source_path);
    }

    build_state.cargo_manifest = if build_state.must_gen {
        if matches!(script_state, ScriptState::Named { .. }) {
            let (mut rs_manifest, rs_source): (CargoManifest, String) =
                parse_source(&build_state.source_path)?;

            Some(manifest::merge_manifest(
                &build_state,
                &rs_source,
                &mut rs_manifest,
            )?)
        } else {
            Some(default_manifest(&build_state)?)
        }
    } else {
        None
    };

    if repl {
        let dash_line = "-".repeat(50);

        // Using strum and convert_case, but be careful that the latter's kebab case
        // doesn't match serde's version when it comes to numbers :(
        let cmd_vec = LoopCommand::iter()
            .map(|v| <LoopCommand as Into<&'static str>>::into(v).to_case(Case::Kebab))
            .collect::<Vec<String>>();
        let cmd_list = cmd_vec.join(", ") + " or help";

        println!("{dash_line}");
        let disp_cmd_list = || {
            println!(
                "Enter one of: {}",
                cmd_list.if_supports_color(Stream::Stdout, |text| text.blue().on_cyan())
            );
        };
        disp_cmd_list();
        let mut loop_editor = ClapEditor::<LoopCommand>::new();
        let mut loop_command = loop_editor.read_command();
        'level2: loop {
            let Some(ref command) = loop_command else {
                loop_command = loop_editor.read_command();
                continue 'level2;
            };
            match command {
                // LoopCommand::Quit => {
                //     // Closing it manually to catch any error
                //     match script_state {
                //         ScriptState::NamedEmpty { temp_path, .. } => temp_path.close(),
                //         _ => Ok(()),
                //     }?;

                //     return Ok(());
                // }
                LoopCommand::Quit => return Ok(()),
                LoopCommand::Delete => {
                    let clean_up = clean_up(&build_state.source_path, &build_state.target_dir_path);
                    if clean_up.is_ok()
                        || (!&build_state.source_path.exists()
                            && !&build_state.target_dir_path.exists())
                    {
                        println!("Deleted");
                    } else {
                        println!(
                            "Failed to delete all files - enter l(ist) to list remaining files"
                        );
                    }
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
                    // let editor = &mut Editor::new(files)?;
                    // editor.run()?;
                    edit::edit_file(&build_state.source_path)?;

                    println!("Enter cancel, retry, submit, quit or help");
                    let mut process_editor = ClapEditor::<ProcessCommand>::new();
                    'level3: loop {
                        let Some(command) = process_editor.read_command() else {
                            continue 'level3;
                        };
                        match command {
                            ProcessCommand::Quit => return Ok(()),
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
                                disp_cmd_list();
                                continue 'level2;
                            }
                            ProcessCommand::Retry => {
                                loop_command = Some(LoopCommand::Continue);
                                disp_cmd_list();
                                continue 'level2;
                            }
                        }
                    }
                }
                LoopCommand::Eval => {
                    let mut rl = DefaultEditor::new().unwrap();
                    rl.set_auto_add_history(true);
                    loop {
                        println!("Enter an expression (e.g., 2 + 3), or q to quit:");

                        let input = rl.readline(">> ").expect("Failed to read input");
                        // Process user input (line)
                        // rl.add_history_entry(&line); // Add current line to history
                        // Parse the expression string into a syntax tree
                        let str = &input.trim();
                        if str.to_lowercase() == "q" {
                            disp_cmd_list();
                            break;
                        }
                        let expr: Result<Expr, syn::Error> = syn::parse_str::<Expr>(str);

                        match expr {
                            Ok(expr) => {
                                // Generate Rust code for the expression
                                let rust_code = quote!(println!("result={:?}", #expr););

                                let rs_source = format!("{rust_code}");
                                debug!("rs_source={rs_source}"); // Careful, needs to impl Display

                                write_source(build_state.source_path.clone(), &rs_source)?;

                                rustfmt(&build_state)?;

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
                                disp_cmd_list();
                                break;
                            }
                            Err(err) => {
                                println!(
                                    "{}",
                                    format!("Error parsing expression: {}", err)
                                        .fg::<Red>()
                                        .bg::<BrightWhite>()
                                );
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

// fn color_stream(var_name: String, fg: dyn OwoColorize::color, bg: dyn OwoColorize::on_color) {
//     let _ = OwoColorize::if_supports_color(&var_name, Stream::Stdout, |text| text.fg().bg());
// }

fn debug_print_config() {
    debug!("PACKAGE_NAME={PACKAGE_NAME}");
    debug!("VERSION={VERSION}");
    debug!("REPL_SUBDIR={REPL_SUBDIR}");
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
        let rs_source = if has_main {
            rs_source
        } else {
            if verbose {
                println!("Source does not contain fn main(), thus a snippet");
            }
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
    let _target_rs_file = write_source(target_rs_path, rs_source)?;

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

fn write_source(to_rs_path: PathBuf, rs_source: &String) -> Result<fs::File, Box<dyn Error>> {
    let mut to_rs_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(to_rs_path)?;
    debug!("Writing out source:\n{}", {
        let lines = rs_source.lines();
        code_utils::reassemble(lines)
    });
    to_rs_file.write_all(rs_source.as_bytes())?;
    debug!("Done!");

    Ok(to_rs_file)
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
