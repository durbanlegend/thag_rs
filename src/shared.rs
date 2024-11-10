#![allow(clippy::uninlined_format_args)]
use crate::{
    debug_log, modified_since_compiled, vlog, DYNAMIC_SUBDIR, PACKAGE_NAME, REPL_SUBDIR, RS_SUFFIX,
    TEMP_DIR_NAME, TEMP_SCRIPT_NAME, TMPDIR, TOML_NAME, V,
};
use crate::{Cli, ProcFlags};
use crate::{ThagError, ThagResult};
use cargo_toml::Manifest;
use crossterm::event::Event;
use firestorm::profile_fn;
use home::home_dir;
use mockall::automock;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::convert::Into;
use std::time::Duration;
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
use strum::Display;

/// An abstract syntax tree wrapper for use with syn.
#[derive(Clone, Debug, Display)]
pub enum Ast {
    File(syn::File),
    Expr(syn::Expr),
    // None,
}

impl Ast {
    #[must_use]
    pub const fn is_file(&self) -> bool {
        match self {
            Self::File(_) => true,
            Self::Expr(_) => false,
        }
    }
}

/// Required to use quote! macro to generate code to resolve expression.
impl ToTokens for Ast {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        profile_fn!(to_tokens);
        match self {
            Self::File(file) => file.to_tokens(tokens),
            Self::Expr(expr) => expr.to_tokens(tokens),
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
    pub build_from_orig_source: bool,
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
        args: &Cli,
        script_state: &ScriptState,
    ) -> ThagResult<Self> {
        profile_fn!(pre_configure);
        let is_repl = proc_flags.contains(ProcFlags::REPL);
        let is_expr = args.expression.is_some();
        let is_stdin = proc_flags.contains(ProcFlags::STDIN);
        let is_edit = proc_flags.contains(ProcFlags::EDIT);
        let is_loop = proc_flags.contains(ProcFlags::LOOP);
        let is_dynamic = is_expr | is_stdin | is_edit | is_loop;
        let is_check = proc_flags.contains(ProcFlags::CHECK);
        let build_exe = proc_flags.contains(ProcFlags::EXECUTABLE);
        let maybe_script = script_state.get_script();
        let Some(ref script) = maybe_script else {
            return Err(ThagError::NoneOption("No script specified"));
        };
        #[cfg(debug_assertions)]
        debug_log!("script={script}");
        let path = Path::new(script);
        debug_log!("path={path:#?}");
        let Some(filename) = path.file_name() else {
            return Err(ThagError::NoneOption("No filename specified"));
        };
        let Some(source_name) = filename.to_str() else {
            return Err(ThagError::NoneOption(
                "Error converting filename to a string",
            ));
        };

        debug_log!("source_name={source_name}");
        let Some(source_stem) = source_name.strip_suffix(RS_SUFFIX) else {
            return Err(format!("Error stripping suffix from {source_name}").into());
        };
        let working_dir_path = if is_repl {
            TMPDIR.join(REPL_SUBDIR)
        } else {
            std::env::current_dir()?.canonicalize()?
        };

        let script_path = if is_repl {
            script_state
                .get_script_dir_path()
                .ok_or("Missing script path")?
                .join(source_name)
        } else if is_dynamic {
            script_state
                .get_script_dir_path()
                .ok_or("Missing script path")?
                .join(TEMP_SCRIPT_NAME)
        } else {
            working_dir_path.join(PathBuf::from(script))
        };

        debug_log!("script_path={script_path:#?}");
        let source_path = script_path.canonicalize()?;
        debug_log!("source_path={source_path:#?}");
        if !source_path.exists() {
            return Err(format!(
                "No script named {source_stem} or {source_name} in path {source_path:?}"
            )
            .into());
        }

        let source_dir_path = source_path
            .parent()
            .ok_or("Problem resolving to parent directory")?
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
                .ok_or("Missing ScriptState::NamedEmpty.repl_path")?
                .join(TEMP_DIR_NAME)
        } else if is_dynamic {
            TMPDIR.join(DYNAMIC_SUBDIR)
        } else {
            TMPDIR.join(PACKAGE_NAME).join(source_stem)
        };

        debug_log!("target_dir_path={}", target_dir_path.display());
        let mut target_path = target_dir_path.join("target").join("debug");

        #[cfg(target_os = "windows")]
        {
            target_path = target_path.join(format!("{source_stem}.exe"));
        }
        #[cfg(not(target_os = "windows"))]
        {
            target_path = target_path.join(source_stem);
        }

        let target_path_exists = target_path.exists();

        let cargo_toml_path = target_dir_path.join(TOML_NAME);
        let source_stem = { source_stem.to_string() };
        let source_name = source_name.to_string();

        let mut build_state = Self {
            working_dir_path,
            source_stem,
            source_name,
            source_dir_path,
            source_path,
            cargo_home,
            target_dir_path,
            target_path,
            cargo_toml_path: cargo_toml_path.clone(),
            ..Default::default()
        };

        let force = proc_flags.contains(ProcFlags::FORCE);
        (build_state.must_gen, build_state.must_build) = if force {
            (true, true)
        } else {
            let stale_executable = matches!(script_state, ScriptState::NamedEmpty { .. })
                || !target_path_exists
                || modified_since_compiled(&build_state)?.is_some();
            let must_gen = force
                || is_repl
                || is_loop
                || is_check
                || stale_executable
                || !cargo_toml_path.exists();
            let must_build =
                must_gen || is_repl || is_loop || build_exe || is_check || stale_executable;
            (must_gen, must_build)
        };

        #[cfg(debug_assertions)]
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
            Self::Anonymous => None,
            Self::NamedEmpty { script, .. } | Self::Named { script, .. } => {
                Some(script.to_string())
            }
        }
    }
    /// Return the script's directory path wrapped in an Option.
    #[must_use]
    pub fn get_script_dir_path(&self) -> Option<PathBuf> {
        match self {
            Self::Anonymous => None,
            Self::Named {
                script_dir_path, ..
            } => Some(script_dir_path.clone()),
            Self::NamedEmpty {
                script_dir_path: script_path,
                ..
            } => Some(script_path.clone()),
        }
    }
}

#[inline]
/// Developer method to log method timings.
pub fn debug_timings(start: &Instant, process: &str) {
    profile_fn!(debug_timings);
    let dur = start.elapsed();
    debug_log!("{} in {}.{}s", process, dur.as_secs(), dur.subsec_millis());
}

#[inline]
/// Display method timings when either the --verbose or --timings option is chosen.
pub fn display_timings(start: &Instant, process: &str, proc_flags: &ProcFlags) {
    profile_fn!(display_timings);
    let dur = start.elapsed();
    let msg = format!("{process} in {}.{}s", dur.as_secs(), dur.subsec_millis());

    debug_log!("{msg}");
    if proc_flags.intersects(ProcFlags::DEBUG | ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        vlog!(V::QQ, "{msg}");
    }
}

// Helper function to sort out the issues caused by Windows using the escape character as
// the file separator.
#[must_use]
#[inline]
#[cfg(target_os = "windows")]
pub fn escape_path_for_windows(path_str: &str) -> String {
    path_str.replace('\\', "/")
}

#[must_use]
#[cfg(not(target_os = "windows"))]
pub fn escape_path_for_windows(path_str: &str) -> String {
    path_str.to_string()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyDisplayLine {
    pub seq: usize,
    pub keys: &'static str, // Or String if you plan to modify the keys later
    pub desc: &'static str, // Or String for modifiability
}

impl PartialOrd for KeyDisplayLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for KeyDisplayLine {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        usize::cmp(&self.seq, &other.seq)
    }
}

impl KeyDisplayLine {
    #[must_use]
    pub const fn new(seq: usize, keys: &'static str, desc: &'static str) -> Self {
        Self { seq, keys, desc }
    }
}

/// A trait to allow mocking of the event reader for testing purposes.
#[automock]
pub trait EventReader {
    /// Read a terminal event.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
    fn read_event(&self) -> ThagResult<Event>;
    /// Poll for a terminal event.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
    fn poll(&self, timeout: Duration) -> ThagResult<bool>;
}

/// A struct to implement real-world use of the event reader, as opposed to use in testing.
#[derive(Debug)]
pub struct CrosstermEventReader;

impl EventReader for CrosstermEventReader {
    fn read_event(&self) -> ThagResult<Event> {
        crossterm::event::read().map_err(Into::<ThagError>::into)
    }

    fn poll(&self, timeout: Duration) -> ThagResult<bool> {
        crossterm::event::poll(timeout).map_err(Into::<ThagError>::into)
    }
}

/// Control debug logging
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        // If the `debug-logs` feature is enabled, always log
        #[cfg(any(feature = "debug-logs", feature = "simplelog"))]
        {
            log::debug!($($arg)*);
        }

        // In all builds, log if runtime debug logging is enabled (e.g., via `-vv`)
        #[cfg(not(any(feature = "debug-logs", feature = "simplelog")))]
        {
            if $crate::logging::is_debug_logging_enabled() {
                log::debug!($($arg)*);
            } else {
                // Avoid unused variable warnings in release mode if logging isn't enabled
                let _ = format_args!($($arg)*);
            }
        }
    };
}
