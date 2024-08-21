#![allow(clippy::uninlined_format_args)]
use crate::cmd_args::{Cli, ProcFlags};
use crate::debug_log;
use crate::errors::BuildRunError;
use crate::logging::Verbosity;
use crate::modified_since_compiled;
use crate::DYNAMIC_SUBDIR;
use crate::REPL_SUBDIR;
use crate::RS_SUFFIX;
use crate::TEMP_SCRIPT_NAME;
use crate::TMPDIR;
use crate::TOML_NAME;
use crate::{log, PACKAGE_NAME};

use cargo_toml::Manifest;
use home::home_dir;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::error::Error;
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
use strum::Display;

/// Reset the display by moving the cursor to the first column and showing it.
/// Crates like `termbg` and `supports-color` send an operating system command (OSC)
/// to interrogate the screen but with side effects which we attempt(ed) to undo here.
/// Unfortunately this appends the `MoveToColumn` and Show command sequences to the
/// program's output, which prevents it being used as a filter in a pipeline. On
/// Windows we resort to defaults and configuration; on other platforms any lingering
/// effects of disabling this remain to be seen.
/// # Panics
/// Will panic if a crossterm execute command fails.
#[deprecated(
    since = "0.1.0",
    note = "Redundant and pollutes piped output. Alternatives already in place."
)]
pub fn clear_screen() {
    // Commented out because this turns up at the end of the output
    // let mut out = stdout();
    // out.execute(MoveToColumn(0)).unwrap();
    // out.execute(Show).unwrap();
    // out.flush().unwrap();
}

/// An abstract syntax tree wrapper for use with syn.
#[derive(Clone, Debug, Display)]
pub enum Ast {
    File(syn::File),
    Expr(syn::Expr),
    // None,
}

/// Required to use quote! macro to generate code to resolve expression.
impl ToTokens for Ast {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Ast::File(file) => file.to_tokens(tokens),
            Ast::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}

/// A struct to encapsulate the attributes of the current build as needed by the various
/// functions co-operating in the generation, build and execution of the code.
#[derive(Clone, Debug, Default)]
pub struct BuildState {
    #[allow(dead_code)]
    pub working_dir_path: PathBuf,
    pub source_stem: String,
    pub source_name: String,
    #[allow(dead_code)]
    pub source_dir_path: PathBuf,
    pub source_path: PathBuf,
    pub cargo_home: PathBuf,
    pub target_dir_path: PathBuf,
    pub target_path: PathBuf,
    pub cargo_toml_path: PathBuf,
    pub rs_manifest: Option<Manifest>,
    pub cargo_manifest: Option<Manifest>,
    pub must_gen: bool,
    pub must_build: bool,
}

impl BuildState {
    #[allow(clippy::too_many_lines)]
    /// Sets up the `BuildState` instance from the constants, the environment, the processing
    /// flags and in some cases directly from the command-line arguments.
    /// # Errors
    /// Any errors in the pre-configuration
    /// # Panics
    pub fn pre_configure(
        proc_flags: &ProcFlags,
        options: &Cli,
        script_state: &ScriptState,
    ) -> Result<Self, Box<dyn Error>> {
        let is_repl = proc_flags.contains(ProcFlags::REPL);
        let is_expr = options.expression.is_some();
        let is_stdin = proc_flags.contains(ProcFlags::STDIN);
        let is_edit = proc_flags.contains(ProcFlags::EDIT);
        let is_loop = proc_flags.contains(ProcFlags::LOOP);
        let is_dynamic = is_expr | is_stdin | is_edit | is_loop;
        let is_check = proc_flags.contains(ProcFlags::CHECK);
        let build_exe = proc_flags.contains(ProcFlags::EXECUTABLE);
        let maybe_script = script_state.get_script();
        let Some(script) = maybe_script.clone() else {
            return Err(Box::new(BuildRunError::NoneOption(
                "No script specified".to_string(),
            )));
        };
        debug_log!("script={script}");
        let path = Path::new(&script);
        debug_log!("path={path:#?}");
        let Some(filename) = path.file_name() else {
            return Err(Box::new(BuildRunError::NoneOption(
                "No filename specified".to_string(),
            )));
        };
        let Some(source_name) = filename.to_str() else {
            return Err(Box::new(BuildRunError::NoneOption(
                "Error converting filename to a string".to_string(),
            )));
        };

        let source_name = source_name.to_string();
        debug_log!("source_name={source_name}");
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

        debug_log!("script_path={script_path:#?}");
        let source_path = script_path.canonicalize()?;
        debug_log!("source_path={source_path:#?}");
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
        let cargo_home = PathBuf::from(match std::env::var("CARGO_HOME") {
            Ok(string) if string != String::new() => string,
            _ => {
                let home_dir = home_dir().ok_or("Can't resolve home directory")?;
                debug_log!("home_dir={}", home_dir.display());
                home_dir.join(".cargo").display().to_string()
            }
        });
        debug_log!("cargo_home={}", cargo_home.display());

        let target_dir_path = if is_repl {
            script_state
                .get_script_dir_path()
                .expect("Missing ScriptState::NamedEmpty.repl_path")
                .join(TEMP_SCRIPT_NAME)
        } else if is_dynamic {
            TMPDIR.join(DYNAMIC_SUBDIR)
        } else {
            TMPDIR.join(PACKAGE_NAME).join(&source_stem)
        };

        debug_log!("target_dir_path={}", target_dir_path.display());
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
            let build_requested = proc_flags.intersects(ProcFlags::BUILD | ProcFlags::CHECK);
            //            debug_log!(
            //                "proc_flags={proc_flags:?}, build_requested={build_requested}, target_path={:?},
            // try_exists: {:?}, exists: {}, proc_flags.contains(ProcFlags::BUILD)?: {},
            // proc_flags.contains(ProcFlags::CHECK)? :{}, proc_flags.intersects(ProcFlags::BUILD | ProcFlags::CHECK)?: {}",
            //                target_path_clone, &target_path_clone.try_exists(), &target_path_clone.exists(), proc_flags.contains(ProcFlags::BUILD), proc_flags.contains(ProcFlags::CHECK), proc_flags.intersects(ProcFlags::BUILD | ProcFlags::CHECK)
            //            );
            let must_gen =
                force || is_repl || is_loop || is_check || (gen_requested && stale_executable);
            let must_build = force
                || is_repl
                || is_loop
                || build_exe
                || is_check
                || (build_requested && stale_executable);
            (must_gen, must_build)
        };

        debug_log!("build_state={build_state:#?}");

        Ok(build_state)
    }
}

/// An enum to encapsulate the type of script in play.
#[derive(Debug)]
pub enum ScriptState {
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
    /// Return the script name wrapped in an Option.
    #[must_use]
    pub fn get_script(&self) -> Option<String> {
        match self {
            ScriptState::Anonymous => None,
            ScriptState::NamedEmpty { script, .. } | ScriptState::Named { script, .. } => {
                Some(script.to_string())
            }
        }
    }
    /// Return the script's directory path wrapped in an Option.
    #[must_use]
    pub fn get_script_dir_path(&self) -> Option<PathBuf> {
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

#[inline]
/// Developer method to log method timings.
pub fn debug_timings(start: &Instant, process: &str) {
    let dur = start.elapsed();
    debug_log!("{} in {}.{}s", process, dur.as_secs(), dur.subsec_millis());
}

#[inline]
/// Display method timings when either the --verbose or --timings option is chosen.
pub fn display_timings(start: &Instant, process: &str, proc_flags: &ProcFlags) {
    let dur = start.elapsed();
    let msg = format!("{process} in {}.{}s", dur.as_secs(), dur.subsec_millis());

    debug_log!("{msg}");
    if proc_flags.intersects(ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        log!(Verbosity::Quieter, "{msg}");
    }
}

// Helper function to sort out the issues caused by Windows using the escape character as
// the file separator.
#[must_use]
pub fn escape_path_for_windows(path: &str) -> String {
    if cfg!(windows) {
        path.replace('\\', "/")
    } else {
        path.to_string()
    }
}

/// Control debug logging
#[macro_export]
macro_rules! debug_log {
    // When the feature is enabled, pass everything to log::debug!
    ($($arg:tt)*) => {
        #[cfg(feature = "debug-logs")]
        {
            log::debug!($($arg)*);
        }

        #[cfg(not(feature = "debug-logs"))]
        {
            // Drop the arguments to avoid unused variable warnings
            let _ = format_args!($($arg)*);
        }
    };
}
