use crate::errors::BuildRunError;
use home::home_dir;
use log::debug;
use proc_macro2::TokenStream;
use quote::ToTokens;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;
use std::str::FromStr;
use std::{path::PathBuf, time::Instant};
use strum::Display;

use crate::cmd_args::{Cli, ProcFlags};
use crate::modified_since_compiled;
use crate::DYNAMIC_SUBDIR;
use crate::REPL_SUBDIR;
use crate::RS_SUFFIX;
use crate::TEMP_SCRIPT_NAME;
use crate::TMPDIR;
use crate::TOML_NAME;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CargoManifest {
    #[serde(default = "default_package")]
    pub(crate) package: Package,
    pub(crate) dependencies: Option<Dependencies>,
    pub(crate) features: Option<Features>,
    #[serde(default)]
    pub(crate) workspace: Workspace,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bin: Vec<Product>,
}

impl FromStr for CargoManifest {
    type Err = BuildRunError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str::<CargoManifest>(s).map_err(|e| BuildRunError::FromStr(e.to_string()))
    }
}

impl ToString for CargoManifest {
    fn to_string(&self) -> String {
        {
            let this = self;
            toml::to_string(&this)
        }
        .unwrap()
    }
}

#[allow(dead_code)]
impl CargoManifest {
    // Save the CargoManifest struct to a Cargo.toml file
    pub(crate) fn save_to_file(&self, path: &str) -> Result<(), BuildRunError> {
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
        name: String::from("your_project_name"),
        version: String::from("0.1.0"),
        edition: default_edition(),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) version: String,
    #[serde(default = "default_edition")]
    pub(crate) edition: String,
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

pub(crate) type Dependencies = BTreeMap<String, Dependency>;
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
fn is_true(val: &bool) -> bool {
    *val
}

/// When definition of a dependency is more than just a version string.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub path: Option<String>,
    pub registry: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub default_features: bool,
}

pub(crate) type Features = BTreeMap<String, Vec<Feature>>;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Feature {
    Simple(String),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Product {
    pub path: Option<String>,
    pub name: Option<String>,
    pub required_features: Option<Vec<String>>,
}

#[allow(dead_code)]
fn default_package_version() -> String {
    "0.0.1".to_string()
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Workspace {}

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
    pub rs_manifest: Option<CargoManifest>,
    pub cargo_manifest: Option<CargoManifest>,
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
pub(crate) fn debug_timings(start: &Instant, process: &str) {
    let dur = start.elapsed();
    debug!("{} in {}.{}s", process, dur.as_secs(), dur.subsec_millis());
}

#[inline]
/// Display method timings when either the --verbose or --timings option is chosen.
pub(crate) fn display_timings(start: &Instant, process: &str, proc_flags: &ProcFlags) {
    let dur = start.elapsed();
    let msg = format!("{process} in {}.{}s", dur.as_secs(), dur.subsec_millis());

    debug!("{msg}");
    if proc_flags.intersects(ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        println!("{msg}");
    }
}

// Add other shared functions and types here
