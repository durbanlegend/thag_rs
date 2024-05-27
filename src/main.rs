#![allow(clippy::uninlined_format_args)]
use crate::cmd_args::{get_opt, get_proc_flags, Cli, ProcFlags};
use crate::code_utils::{
    debug_timings, display_timings, extract_ast, extract_manifest, modified_since_compiled,
    read_file_contents, rustfmt, strip_curly_braces, wrap_snippet, write_source,
};
use crate::errors::BuildRunError;
use crate::manifest::CargoManifest;
use crate::term_colors::nu_resolve_style;
use code_utils::Ast;
use env_logger::Builder;
use env_logger::Env;
use env_logger::WriteStyle;
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
mod stdin;
mod term_colors;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const RS_SUFFIX: &str = ".rs";
pub(crate) const FLOWER_BOX_LEN: usize = 70;
pub(crate) const REPL_SUBDIR: &str = "rs_repl";
pub(crate) const DYNAMIC_SUBDIR: &str = "rs_dyn";
pub(crate) const TEMP_SCRIPT_NAME: &str = "temp.rs";
pub(crate) const TOML_NAME: &str = "Cargo.toml";

lazy_static! {
    pub(crate) static ref TMPDIR: PathBuf = env::temp_dir();
}

#[derive(Debug)]
pub(crate) enum ScriptState {
    /// Repl with no script name provided by user
    #[allow(dead_code)]
    Anonymous,
    /// Repl with script name.
    NamedEmpty {
        script: String,
        script_dir_path: PathBuf,
    },
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
    pub(crate) fn get_script_dir_path(&self) -> Option<PathBuf> {
        match self {
            ScriptState::Anonymous => None,
            ScriptState::Named {
                script_dir_path, ..
            } => Some(script_dir_path.clone()),
            ScriptState::NamedEmpty {
                script_dir_path: script_path,
                ..
            } => Some(script_path.clone()),
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
        options: &Cli,
        script_state: &ScriptState,
    ) -> Result<Self, Box<dyn Error>> {
        let is_repl = proc_flags.contains(ProcFlags::REPL);
        let is_expr = options.expression.is_some();
        let is_stdin = proc_flags.contains(ProcFlags::STDIN);
        let is_dynamic = is_expr | is_stdin;
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
            TMPDIR.join(REPL_SUBDIR)
        } else {
            std::env::current_dir()?.canonicalize()?
        };

        let script_path = if is_repl {
            script_state
                .get_script_dir_path()
                .expect("Missing script path")
                .join(source_name.clone())
        } else if is_dynamic {
            script_state
                .get_script_dir_path()
                .expect("Missing script path")
                .join(TEMP_SCRIPT_NAME)
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
                .get_script_dir_path()
                .expect("Missing ScriptState::NamedEmpty.repl_path")
                .join(TEMP_SCRIPT_NAME)
        } else if is_dynamic {
            TMPDIR.join(DYNAMIC_SUBDIR)
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

//      TODO:
//       1.  Consider supporting alternative TOML embedding keywords so we can run examples/regex_capture_toml.rs.
//       2.  Consider history support for stdin.
//       3.  Paste event in Windows slow or not happening?
//       4.  TUI editor as an option on stdin.
//       5.  How to navigate reedline history entry by entry instead of line by line.
//       6.  How to insert line feed from keyboard to split line in reedline. (Supposedly shift+enter)
//       8.  Cat files before delete.
//       9.  DONE Consider making script name optional, with -s/stdin parm as per my runner changes?
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

#[allow(clippy::too_many_lines)]
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
        TMPDIR.join(REPL_SUBDIR)
    } else {
        std::env::current_dir()?.canonicalize()?
    };

    validate_options(&options, &proc_flags)?;

    // Normal REPL with no named script
    let repl_source_path = if is_repl && options.script.is_none() {
        Some(code_utils::create_next_repl_file())
    } else {
        None
    };

    let is_expr = proc_flags.contains(ProcFlags::EXPR);
    let is_stdin = proc_flags.contains(ProcFlags::STDIN);
    let is_dynamic = is_expr | is_stdin;

    if is_dynamic {
        code_utils::create_temp_source_file();
    }

    // Reusable source path for expressions and stdin evaluation
    // let temp_source_path = if is_expr {
    //     Some(code_utils::create_temp_source_file())
    // } else {
    //     None
    // };

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
    } else if is_dynamic {
        debug!("^^^^^^^^ is_dynamic={is_dynamic}");
        <std::path::PathBuf as std::convert::AsRef<Path>>::as_ref(&TMPDIR)
            .join(DYNAMIC_SUBDIR)
            .clone()
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
            script,
            script_dir_path,
        }
    } else if is_repl {
        let script = repl_source_path
            .expect("Missing newly created REPL source path")
            .display()
            .to_string();
        ScriptState::NamedEmpty {
            script,
            script_dir_path,
        }
    } else {
        assert!(is_dynamic);
        ScriptState::NamedEmpty {
            script: String::from(TEMP_SCRIPT_NAME),
            script_dir_path,
        }
    };

    // debug!("script_state={script_state:?}");

    let mut build_state = BuildState::pre_configure(&proc_flags, &options, &script_state)?;
    if is_repl {
        debug!("build_state.source_path={:?}", build_state.source_path);
    }

    if is_repl {
        repl::run_repl(&mut options, &proc_flags, &mut build_state, start)?;
    } else if is_dynamic {
        let rs_source = if is_expr {
            let Some(rs_source) = options.expression.clone() else {
                return Err(Box::new(BuildRunError::Command(
                    "Missing expression for --expr option".to_string(),
                )));
            };
            rs_source
        } else {
            assert!(is_stdin);
            debug!("About to call read_stdin()");
            let vec = stdin::read_stdin()?;
            debug!("vec={vec:#?}");
            vec.join("\n")
        };
        let rs_manifest = extract_manifest(&rs_source, Instant::now())
            .map_err(|_err| BuildRunError::FromStr("Error parsing rs_source".to_string()))?;
        build_state.rs_manifest = Some(rs_manifest);

        let maybe_ast = extract_ast(&rs_source);

        if let Ok(expr_ast) = maybe_ast {
            code_utils::process_expr(
                &expr_ast,
                &mut build_state,
                &rs_source,
                &mut options,
                &proc_flags,
                &start,
            )?;
        } else {
            nu_color_println!(
                nu_resolve_style(MessageLevel::Error),
                "Error parsing code: {:#?}",
                maybe_ast
            );
        }
    } else {
        gen_build_run(
            &mut options,
            &proc_flags,
            &mut build_state,
            None::<Ast>,
            &start,
        )?;
    }

    Ok(())
}

fn validate_options(options: &Cli, proc_flags: &ProcFlags) -> Result<(), Box<dyn Error>> {
    if let Some(ref script) = options.script {
        if !script.ends_with(RS_SUFFIX) {
            return Err(Box::new(BuildRunError::Command(format!(
                "Script name must end in {RS_SUFFIX}"
            ))));
        }
        if proc_flags.contains(ProcFlags::EXPR) {
            return Err(Box::new(BuildRunError::Command(
                "Incompatible options: --expr option and script name".to_string(),
            )));
        }
    } else if !proc_flags.contains(ProcFlags::EXPR)
        && !proc_flags.contains(ProcFlags::REPL)
        && !proc_flags.contains(ProcFlags::STDIN)
    {
        return Err(Box::new(BuildRunError::Command(
            "Missing script name".to_string(),
        )));
    }
    Ok(())
}

fn debug_print_config() {
    debug!("PACKAGE_NAME={PACKAGE_NAME}");
    debug!("VERSION={VERSION}");
    debug!("REPL_SUBDIR={REPL_SUBDIR}");
}

fn gen_build_run(
    options: &mut Cli,
    proc_flags: &ProcFlags,
    build_state: &mut BuildState,
    syntax_tree: Option<Ast>,
    start: &Instant,
) -> Result<(), Box<dyn Error>> {
    // let verbose = proc_flags.contains(ProcFlags::VERBOSE);
    let proc_flags = &proc_flags;
    let options = &options;

    if build_state.must_gen {
        let source_path: &Path = &build_state.source_path;
        let start_parsing_rs = Instant::now();
        let mut rs_source = read_file_contents(source_path)?;
        let rs_manifest: CargoManifest = {
            let result = extract_manifest(&rs_source, start_parsing_rs);
            debug_timings(&start_parsing_rs, "Parsed source");
            result
        }?;
        // debug!("&&&&&&&& rs_manifest={rs_manifest:#?}");
        // debug!("&&&&&&&& rs_source={rs_source}");
        if build_state.rs_manifest.is_none() {
            build_state.rs_manifest = Some(rs_manifest);
        }
        // let mut rs_source = read_file_contents(&build_state.source_path)?;
        let syntax_tree: Option<Ast> = if syntax_tree.is_none() {
            code_utils::to_ast(&rs_source)
        } else {
            syntax_tree
        };

        // debug!("syntax_tree={syntax_tree:#?}");

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
            // let start_quote = Instant::now();
            let rust_code = if let Some(ref syntax_tree_ref) = syntax_tree {
                quote::quote!(
                    // fn type_of<T>(_: T) -> &'static str {
                    //     std::any::type_name::<T>()
                    // }
                    //     println!("Type of expression is {}", type_of(#syntax_tree_ref));)
                    // .to_string()
                    // println!("Expression returned {:#?}", #syntax_tree_ref);)
                    // .to_string()
                    dbg!(#syntax_tree_ref);
                )
                .to_string()
            } else {
                format!(r#"println!("Expression returned {{}}", {rs_source});"#)
            };
            // display_timings(&start_quote, "Completed quote", proc_flags);
            wrap_snippet(&rust_code.to_string())
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
    let quiet = proc_flags.contains(ProcFlags::QUIET);

    debug!("BBBBBBBB In build");

    let Ok(cargo_toml_path_str) = code_utils::path_to_str(&build_state.cargo_toml_path) else {
        return Err(BuildRunError::OsString(
            build_state.cargo_toml_path.clone().into_os_string(),
        ));
    };
    let mut build_command = Command::new("cargo");
    // Rustc writes to std
    let mut args = vec!["build", "--manifest-path", &cargo_toml_path_str];
    // if verbose {
    //     args.push("--verbose");
    // };
    if quiet {
        args.push("--quiet");
    }

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
