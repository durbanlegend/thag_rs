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
use ratatui::crossterm::cursor::{MoveTo, Show};
use ratatui::crossterm::terminal::{Clear, ClearType};
use ratatui::crossterm::ExecutableCommand;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{stdout, Write};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Instant,
};
use strum::Display;
use toml::Value;

pub fn clear_screen() {
    let mut out = stdout();
    // out.execute(Hide).unwrap();
    out.execute(Clear(ClearType::All)).unwrap();
    out.execute(MoveTo(0, 0)).unwrap();
    out.execute(Show).unwrap();
    out.flush().unwrap();
}

#[derive(Clone, Debug, Display)]
/// Abstract syntax tree wrapper for use with syn.
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

pub type Patches = BTreeMap<String, Dependencies>;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CargoManifest {
    #[serde(default = "default_package")]
    pub package: Package,
    pub dependencies: Option<Dependencies>,
    pub features: Option<Features>,
    pub patch: Option<Patches>,
    #[serde(default)]
    pub workspace: Workspace,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bin: Vec<Product>,
    pub lib: Option<Product>,
}

impl FromStr for CargoManifest {
    type Err = BuildRunError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str::<CargoManifest>(s).map_err(|e| BuildRunError::FromStr(e.to_string()))
    }
}

impl std::fmt::Display for CargoManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(&self).unwrap())
    }
}

#[allow(dead_code)]
impl CargoManifest {
    // Save the CargoManifest struct to a Cargo.toml file
    pub fn save_to_file(&self, path: &str) -> Result<(), BuildRunError> {
        let toml_string = {
            let this = self;
            toml::to_string(&this)
        }?;
        std::fs::write(path, toml_string.as_bytes())?;
        Ok(())
    }
}

// Implementation of manifest

// Default function for the `package` field
fn default_package() -> Package {
    Package {
        name: String::from("your_script_name"),
        version: String::from("0.1.0"),
        edition: default_edition(),
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    #[serde(default = "default_edition")]
    pub edition: String,
}

// Default function for the `edition` field
fn default_edition() -> String {
    String::from("2021")
}

impl Default for Package {
    fn default() -> Self {
        Package {
            version: String::from("0.0.0"),
            name: String::from("your_script_name_stem"),
            edition: default_edition(),
        }
    }
}

pub type Dependencies = BTreeMap<String, Dependency>;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed(Box<DependencyDetail>),
}

fn default_true() -> bool {
    true
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(val: &bool) -> bool {
    !*val
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_true(val: &bool) -> bool {
    *val
}

/// Credit to cargo_toml crate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyDetail {
    /// Semver requirement. Note that a plain version number implies this version *or newer* compatible one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Fetch this dependency from a custom 3rd party registry (alias defined in Cargo config), not crates-io.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,

    /// Directly define custom 3rd party registry URL (may be `sparse+https:`) instead of a config nickname.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_index: Option<String>,

    /// This path is usually relative to the crate's manifest, but when using workspace inheritance, it may be relative to the workspace!
    ///
    /// When calling [`Manifest::complete_from_path_and_workspace`] use absolute path for the workspace manifest, and then this will be corrected to be an absolute
    /// path when inherited from the workspace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// If true, the dependency has been defined at the workspace level, so the `path` is joined with workspace's base path.
    ///
    /// Note that `Dependency::Simple` won't have this flag, even if it was inherited.
    #[serde(skip)]
    pub inherited: bool,

    /// Read dependency from git repo URL, not allowed on crates-io.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    /// Read dependency from git branch, not allowed on crates-io.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Read dependency from git tag, not allowed on crates-io.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Read dependency from git commit, not allowed on crates-io.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,

    /// Enable these features of the dependency. `default` is handled in a special way.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,

    /// NB: Not allowed at workspace level
    ///
    /// If not used with `dep:` or `?/` syntax in `[features]`, this also creates an implicit feature.
    /// See the [`features`] module for more info.
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,

    /// Enable the `default` set of features of the dependency (enabled by default).
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub default_features: bool,

    /// Use this crate name instead of table key.
    ///
    /// By using this, a crate can have multiple versions of the same dependency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,

    /// Contains the remaining unstable keys and values for the dependency.
    #[serde(flatten)]
    pub unstable: BTreeMap<String, Value>,
}

impl Default for DependencyDetail {
    fn default() -> Self {
        DependencyDetail {
            version: None,
            registry: None,
            registry_index: None,
            path: None,
            inherited: false,
            git: None,
            branch: None,
            tag: None,
            rev: None,
            features: Vec::new(),
            optional: false,
            default_features: true, // != bool::default()
            package: None,
            unstable: BTreeMap::new(),
        }
    }
}

pub type Features = BTreeMap<String, Vec<Feature>>;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Feature {
    Simple(String),
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Product {
    pub path: Option<String>,
    pub name: Option<String>,
    pub required_features: Option<Vec<String>>,
    pub crate_type: Option<Vec<String>>,
}

#[allow(dead_code)]
fn default_package_version() -> String {
    "0.0.1".to_string()
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {}

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
    pub fn pre_configure(
        proc_flags: &ProcFlags,
        options: &Cli,
        script_state: &ScriptState,
    ) -> Result<Self, Box<dyn Error>> {
        let is_repl = proc_flags.contains(ProcFlags::REPL);
        let is_expr = options.expression.is_some();
        let is_stdin = proc_flags.contains(ProcFlags::STDIN);
        let is_edit = proc_flags.contains(ProcFlags::EDIT);
        let is_dynamic = is_expr | is_stdin | is_edit;
        let maybe_script = script_state.get_script();
        if maybe_script.is_none() {
            return Err(Box::new(BuildRunError::NoneOption(
                "No script specified".to_string(),
            )));
        }
        let script = (maybe_script).clone().unwrap();
        debug_log!("script={script}");
        let path = Path::new(&script);
        debug_log!("path={path:#?}");
        let source_name: String = path.file_name().unwrap().to_str().unwrap().to_string();
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
            let build_requested = proc_flags.contains(ProcFlags::BUILD);
            let must_gen = force || is_repl || (gen_requested && stale_executable);
            let must_build = force || is_repl || (build_requested && stale_executable);
            (must_gen, must_build)
        };

        debug_log!("build_state={build_state:#?}");

        Ok(build_state)
    }
}

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
    pub fn get_script(&self) -> Option<String> {
        match self {
            ScriptState::Anonymous => None,
            ScriptState::NamedEmpty { script, .. } | ScriptState::Named { script, .. } => {
                Some(script.to_string())
            }
        }
    }
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
        log!(Verbosity::Quiet, "{msg}");
    }
}

// Helper function to sort out using the escape character as the file separator.
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
