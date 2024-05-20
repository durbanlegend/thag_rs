#![allow(clippy::uninlined_format_args)]
use crate::cmd_args::{get_opt, get_proc_flags, Opt, ProcFlags};
use crate::code_utils::{
    debug_timings, display_timings, rustfmt, strip_curly_braces, wrap_snippet,
};
use crate::code_utils::{modified_since_compiled, parse_source_file, write_source};
use crate::errors::BuildRunError;
use crate::manifest::CargoManifest;
use code_utils::Ast;
use env_logger::{fmt::WriteStyle, Builder, Env};
use home::home_dir;
use lazy_static::lazy_static;
use log::{debug, log_enabled, Level::Debug};
use term_colors::MessageLevel;

use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::{fs, io::Write as OtherWrite};

mod cmd_args;
mod code_utils;
mod errors;
mod manifest;
mod repl;
mod term_colors;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const RS_SUFFIX: &str = ".rs";
pub(crate) const FLOWER_BOX_LEN: usize = 70;
pub(crate) const REPL_SUBDIR: &str = "rs_repl";
pub(crate) const TOML_NAME: &str = "Cargo.toml";

lazy_static! {
    static ref TMP_DIR: PathBuf = env::temp_dir();
}

#[derive(Debug)]
pub(crate) enum ScriptState {
    /// Repl with no script name provided by user
    #[allow(dead_code)]
    Anonymous,
    /// Repl with script name.
    // TODO: phase out script string? Or replace by path? Can maybe phase out whole enum ScriptState.
    NamedEmpty { script: String, repl_path: PathBuf },
    /// Script name provided by user
    Named {
        script: String,
        script_dir_path: PathBuf,
    },
}

impl ScriptState {
    pub(crate) fn get_script(&self) -> Option<String> {
        match self {
            ScriptState::Anonymous => None,
            ScriptState::NamedEmpty { script, .. } | ScriptState::Named { script, .. } => {
                Some(script.to_string())
            }
        }
    }
    pub(crate) fn get_repl_path(&self) -> Option<PathBuf> {
        match self {
            ScriptState::Anonymous => None,
            ScriptState::Named {
                script_dir_path, ..
            } => Some(script_dir_path.clone()),
            ScriptState::NamedEmpty { repl_path, .. } => Some(repl_path.clone()),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BuildState {
    #[allow(dead_code)]
    pub(crate) working_dir_path: PathBuf,
    pub(crate) source_stem: String,
    pub(crate) source_name: String,
    #[allow(dead_code)]
    pub(crate) source_dir_path: PathBuf,
    pub(crate) source_path: PathBuf,
    pub(crate) cargo_home: PathBuf,
    pub(crate) target_dir_path: PathBuf,
    pub(crate) target_path: PathBuf,
    pub(crate) cargo_toml_path: PathBuf,
    pub(crate) rs_manifest: Option<CargoManifest>,
    pub(crate) cargo_manifest: Option<CargoManifest>,
    pub(crate) must_gen: bool,
    pub(crate) must_build: bool,
}

impl BuildState {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn pre_configure(
        proc_flags: &ProcFlags,
        script_state: &ScriptState,
    ) -> Result<Self, Box<dyn Error>> {
        let is_repl = proc_flags.contains(ProcFlags::REPL);
        let maybe_script = script_state.get_script();
        if maybe_script.is_none() {
            return Err(Box::new(BuildRunError::NoneOption(
                "No script specified".to_string(),
            )));
        }
        let script = (maybe_script).clone().unwrap();
        debug!("script={script}");
        let path = Path::new(&script);
        debug!("path={path:#?}");
        let source_name: String = path.file_name().unwrap().to_str().unwrap().to_string();
        debug!("source_name={source_name}");
        let source_stem = {
            let Some(stem) = source_name.strip_suffix(RS_SUFFIX) else {
                return Err(Box::new(BuildRunError::Command(format!(
                    "Error stripping suffix from {}",
                    source_name
                ))));
            };
            stem.to_string()
        };

        let working_dir_path = if is_repl {
            TMP_DIR.join(REPL_SUBDIR)
        } else {
            std::env::current_dir()?.canonicalize()?
        };

        let script_path = if is_repl {
            script_state
                .get_repl_path()
                .expect("Missing script path")
                .join(source_name.clone())
        } else {
            working_dir_path.join(PathBuf::from(script.clone()))
        };

        debug!("script_path={script_path:#?}");
        let source_path = script_path.canonicalize()?;
        debug!("source_dir_path={source_path:#?}");
        if !source_path.exists() {
            return Err(Box::new(BuildRunError::Command(format!(
                "No script named {} or {} in path {source_path:?}",
                source_stem, source_name
            ))));
        }

        let source_dir_path = source_path
            .parent()
            .expect("Problem resolving to parent directory")
            .to_path_buf();
        let cargo_home = if is_repl {
            working_dir_path.clone()
        } else {
            PathBuf::from(match std::env::var("CARGO_HOME") {
                Ok(string) if string != String::new() => string,
                _ => {
                    let home_dir = home_dir().ok_or("Can't resolve home directory")?;
                    debug!("home_dir={}", home_dir.display());
                    home_dir.join(".cargo").display().to_string()
                }
            })
        };
        debug!("cargo_home={}", cargo_home.display());

        let target_dir_path = if is_repl {
            script_state
                .get_repl_path()
                .expect("Missing ScriptState::NamedEmpty.repl_path")
        } else {
            cargo_home.join(&source_stem)
        };

        debug!("target_dir_path={}", target_dir_path.display());
        let mut target_path = target_dir_path.join("target").join("debug");
        target_path = if cfg!(windows) {
            target_path.join(source_stem.clone() + ".exe")
        } else {
            target_path.join(&source_stem)
        };

        let target_path_clone = target_path.clone();

        let cargo_toml_path = target_dir_path.join(TOML_NAME).clone();

        let mut build_state = Self {
            working_dir_path,
            source_stem,
            source_name,
            source_dir_path,
            source_path,
            cargo_home,
            target_dir_path,
            target_path,
            cargo_toml_path,
            ..Default::default()
        };

        let force = proc_flags.contains(ProcFlags::FORCE);
        (build_state.must_gen, build_state.must_build) = if force {
            (true, true)
        } else {
            let stale_executable = matches!(script_state, ScriptState::NamedEmpty { .. })
                || !target_path_clone.exists()
                || modified_since_compiled(&build_state).is_some();
            let gen_requested = proc_flags.contains(ProcFlags::GENERATE);
            let build_requested = proc_flags.contains(ProcFlags::BUILD);
            let must_gen = force || is_repl || (gen_requested && stale_executable);
            let must_build = force || is_repl || (build_requested && stale_executable);
            (must_gen, must_build)
        };

        debug!("build_state={build_state:#?}");

        Ok(build_state)
    }
}

#[derive(Debug)]
struct Context<'a> {
    options: &'a mut Opt,
    proc_flags: &'a ProcFlags,
    // cmd_list: String,
    build_state: &'a mut BuildState,
    start: &'a Instant,
}

//      TODO:
//       2.  Discontinue //! support?
//       3.  Only check terminal once up front? Avoid test string showing up unexpectedly.
//       5.  How to navigate reedline history entry by entry instead of line by line.
//       6.  How to insert line feed from keyboard to split line in reedline. (Supposedly shift+enter)
//       8.  Cat files before delete.
//       9.  Consider making script name optional, with -n/stdin parm as per my runner changes?
//      10.  Decide if it's worth passing the wrapped syntax tree to gen_build_run from eval just to avoid
//           re-parsing it for that specific use case.
//      11.  Clean up debugging
//      12.  "edit" crate - how to reconfigure editors dynamically - instructions unclear.
//      13.  Clap aliases not working in REPL.
//      14.  Get rid of date and time in RHS of REPL? - doesn't seem to be an option.
//      15.  Help command in eval, same as quit and q
//      16.  Work on examples/reedline_clap_repl_gemini.rs
//      17.
//      18.  How to set editor in Windows.
//

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    configure_log();

    let mut options = get_opt();
    let proc_flags = get_proc_flags(&options)?;

    if log_enabled!(Debug) {
        debug_print_config();
        debug!("proc_flags={proc_flags:#?}");
        debug_timings(&start, "Set up processing flags");

        if !&options.args.is_empty() {
            debug!("... args:");
            for arg in &options.args {
                debug!("{arg}");
            }
        }
    }

    // Access TMP_DIR
    // println!("Temporary directory: {:?}", *TMP_DIR);

    let is_repl = proc_flags.contains(ProcFlags::REPL);

    let working_dir_path = if is_repl {
        TMP_DIR.join(REPL_SUBDIR)
    } else {
        std::env::current_dir()?.canonicalize()?
    };

    if let Some(ref script) = options.script {
        if !script.ends_with(RS_SUFFIX) {
            return Err(Box::new(BuildRunError::Command(format!(
                "Script name must end in {RS_SUFFIX}"
            ))));
        }
    }

    // Normal REPL with no named script
    let repl_source_path = if is_repl && options.script.is_none() {
        Some(code_utils::create_next_repl_file())
    } else {
        None
    };

    let script_dir_path = if is_repl {
        if let Some(ref script) = options.script {
            // REPL with repeat of named script
            let source_stem = script
                .strip_suffix(RS_SUFFIX)
                .expect("Failed to strip extension off the script name");
            working_dir_path.join(source_stem)
        } else {
            // Normal REPL with no script name
            repl_source_path
                .as_ref()
                .expect("Missing path of newly created REPL souece file")
                .parent()
                .expect("Could not find parent directory of repl source file")
                .to_path_buf()
        }
    } else {
        // Normal script file prepared beforehand
        let script = options
            .script
            .clone()
            .expect("Problem resolving script path");
        let script_path = PathBuf::from(script);
        let script_dir_path = script_path
            .parent()
            .expect("Problem resolving script parent path");
        // working_dir_path.join(PathBuf::from(script.clone()))
        script_dir_path
            .canonicalize()
            .expect("Problem resolving script dir path")
    };

    let script_state = if let Some(ref script) = options.script {
        let script = script.to_owned();
        ScriptState::Named {
            script_dir_path,
            script,
        }
    } else {
        assert!(is_repl);
        // let repl_source_path = code_utils::create_next_repl_file();
        // let repl_path = repl_source_path
        //     .parent()
        //     .expect("Could not find parent directory of repl source file")
        //     .to_path_buf();
        let script = repl_source_path
            .expect("Missing newly created REPL source path")
            .display()
            .to_string();
        ScriptState::NamedEmpty {
            repl_path: script_dir_path,
            script,
        }
    };

    println!("script_state={script_state:?}");

    let mut build_state = BuildState::pre_configure(&proc_flags, &script_state)?;
    if is_repl {
        debug!("build_state.source_path={:?}", build_state.source_path);
    }

    if is_repl {
        repl::run_repl(&mut options, &proc_flags, &mut build_state, start)?;
    } else {
        gen_build_run(
            &&mut options,
            &proc_flags,
            &mut build_state,
            None::<Ast>,
            &start,
        )?;
    }

    Ok(())
}

fn debug_print_config() {
    debug!("PACKAGE_NAME={PACKAGE_NAME}");
    debug!("VERSION={VERSION}");
    debug!("REPL_SUBDIR={REPL_SUBDIR}");
}

fn gen_build_run(
    options: &&mut cmd_args::Opt,
    proc_flags: &ProcFlags,
    build_state: &mut BuildState,
    syntax_tree: Option<Ast>,
    start: &Instant,
) -> Result<(), Box<dyn Error>> {
    // let verbose = proc_flags.contains(ProcFlags::VERBOSE);
    let proc_flags = &proc_flags;
    let options = &options;

    if build_state.must_gen {
        let (rs_manifest, mut rs_source): (CargoManifest, String) =
            parse_source_file(&build_state.source_path)?;
        println!("&&&&&&&& rs_manifest={rs_manifest:#?}");
        println!("&&&&&&&& rs_source={rs_source}");
        if build_state.rs_manifest.is_none() {
            build_state.rs_manifest = Some(rs_manifest);
        }
        // let mut rs_source = read_file_contents(&build_state.source_path)?;
        let syntax_tree: Option<Ast> = if syntax_tree.is_none() {
            code_utils::to_ast(&rs_source)
        } else {
            syntax_tree
        };

        if build_state.rs_manifest.is_some() {
            build_state.cargo_manifest = Some(manifest::merge_manifest(
                build_state,
                &rs_source,
                &syntax_tree,
            )?);
        }

        let has_main = if let Some(ref syntax_tree_ref) = syntax_tree {
            code_utils::has_main(syntax_tree_ref)
        } else {
            code_utils::has_main_alt(&rs_source)
        };

        // println!("######## build_state={build_state:#?}");
        rs_source = if has_main {
            // Strip off any enclosing braces we may have added
            if rs_source.starts_with('{') {
                strip_curly_braces(&rs_source).unwrap_or(rs_source)
            } else {
                rs_source
            }
        } else {
            wrap_snippet(&rs_source)
            // build_state.syntax_tree = Some(Ast::File(syn::parse_file(&rs_source)?));
        };
        generate(build_state, &rs_source, proc_flags)?;
    } else {
        println!(
            "{}",
            nu_ansi_term::Color::Yellow
                // .bold()
                .paint("Skipping unnecessary generation step.  Use --force (-f) to override.")
        );
        // build_state.cargo_manifest = Some(default_manifest(build_state)?);
        build_state.cargo_manifest = None; // Don't need it in memory, build will find it on disk
    }
    if build_state.must_build {
        build(proc_flags, build_state)?;
    } else {
        println!(
            "{}",
            nu_ansi_term::Color::Yellow
                // .bold()
                .paint("Skipping unnecessary cargo build step. Use --force (-f) to override.")
        );
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
    rs_source: &str,
    proc_flags: &ProcFlags,
) -> Result<(), Box<dyn Error>> {
    let start_gen = Instant::now();
    let verbose = proc_flags.contains(ProcFlags::VERBOSE);

    debug!("In generate, proc_flags={proc_flags}");

    debug!(
        "build_state.target_dir_path={:#?}",
        build_state.target_dir_path
    );

    if !build_state.target_dir_path.exists() {
        fs::create_dir_all(&build_state.target_dir_path)?;
    }

    let target_rs_path = build_state.target_dir_path.clone();
    let target_rs_path = target_rs_path.join(&build_state.source_name);
    // let is_repl = proc_flags.contains(ProcFlags::REPL);
    if verbose {
        println!("GGGGGGGG Creating source file: {target_rs_path:?}");
    }
    write_source(&target_rs_path, rs_source)?;
    rustfmt(build_state)?;

    // debug!("cargo_toml_path will be {:?}", &build_state.cargo_toml_path);
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
    // debug!("cargo_toml_path={:?}", &build_state.cargo_toml_path);
    // debug!("##### Cargo.toml generation succeeded!");

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

    let dash_line = "-".repeat(FLOWER_BOX_LEN);
    println!("{}", nu_ansi_term::Color::Yellow.paint(dash_line.clone()));

    let _exit_status = run_command.spawn()?.wait()?;

    println!("{}", nu_ansi_term::Color::Yellow.paint(dash_line.clone()));

    // debug!("Exit status={exit_status:#?}");

    display_timings(&start_run, "Completed run", proc_flags);

    Ok(())
}
