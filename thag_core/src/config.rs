use crate::cprtln;
use crate::debug_log;
use crate::error::{ThagError, ThagResult};
use crate::lazy_static_var;
use crate::logging::Verbosity;
use crate::{profile, profile_method};
use dirs;
use documented::{Documented, DocumentedFields};
use mockall::automock;
use nu_ansi_term::Style;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
#[cfg(target_os = "windows")]
use std::env;
use std::{
    env::{current_dir, var},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
}; // use strum::{Display, EnumString};
   // use thiserror::Error;
use toml_edit::DocumentMut;

/// Core configuration structure
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Config {
    // Core configuration fields
    // Note: we'll need to decide which fields from the original Config
    // are truly "core" vs build-specific
    /// Logging configuration
    pub logging: Logging,
    /// Proc macros directory location, e.g. `demo/proc_macros`
    pub proc_macros: ProcMacros,
    /// Path to configuration file
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

/// Logging settings
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Documented, DocumentedFields)]
#[serde(default)]
pub struct Logging {
    /// Default verbosity setting
    #[serde_as(as = "DisplayFromStr")]
    pub default_verbosity: Verbosity,
}

/// Demo proc macro settings
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Documented, DocumentedFields, Serialize)]
#[serde(default)]
pub struct ProcMacros {
    /// Absolute or relative path to bank proc macros crate, e.g. bank/proc_macros.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bank_proc_macro_crate_path: Option<String>,
    /// Absolute or relative path to demo proc macros crate, e.g. demo/proc_macros.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub demo_proc_macro_crate_path: Option<String>,
}

impl Config {
    /// Load the user's config file, or if there is none, load the default.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered.
    pub fn load_or_create_default() -> ThagResult<Self> {
        profile_method!("load_or_create_default");
        let config_dir = if let Some(cargo_home) = std::env::var_os("CARGO_HOME") {
            PathBuf::from(cargo_home).join(".config").join("thag_rs")
        } else {
            dirs::config_dir()
                .ok_or("Could not determine config directory")?
                .join("thag_rs")
        };

        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            fs::create_dir_all(&config_dir)?;

            // Try to find default config in different locations
            let default_config = if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
                // First try cargo-dist installed location
                let dist_config = PathBuf::from(cargo_home)
                    .join("assets")
                    .join("default_config.toml");

                if dist_config.exists() {
                    fs::read_to_string(dist_config)?
                } else {
                    // Fallback to embedded config
                    include_str!("../../assets/default_config.toml").to_string()
                }
            } else {
                include_str!("../../assets/default_config.toml").to_string()
            };

            fs::write(&config_path, default_config)?;
        }

        let config_str = fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&config_str)?)
    }

    /// Load a configuration.
    ///
    /// # Errors
    ///
    /// This function will bubble up any errors encountered.
    pub fn load(path: &Path) -> ThagResult<Self> {
        profile_method!("load");
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        validate_config_format(&content)?;
        Ok(config)
    }
}

#[automock]
pub trait Context {
    fn get_config_path(&self) -> PathBuf;
    fn is_real(&self) -> bool;
}

/// A struct for use in normal execution, as opposed to use in testing.
#[derive(Debug, Default)]
pub struct RealContext {
    pub base_dir: PathBuf,
}

impl RealContext {
    /// Creates a new [`RealContext`].
    ///
    /// # Panics
    ///
    /// Panics if it fails to resolve the $APPDATA path.
    #[cfg(target_os = "windows")]
    #[must_use]
    pub fn new() -> Self {
        profile_method!("new");
        let base_dir =
            PathBuf::from(env::var("APPDATA").expect("Error resolving path from $APPDATA"));
        Self { base_dir }
    }

    /// Creates a new [`RealContext`].
    ///
    /// # Panics
    ///
    /// Panics if it fails to resolve the home directory.
    #[cfg(not(target_os = "windows"))]
    #[must_use]
    pub fn new() -> Self {
        profile_method!("new");
        let base_dir = home::home_dir()
            .expect("Error resolving home::home_dir()")
            .join(".config");
        Self { base_dir }
    }
}

impl Context for RealContext {
    fn get_config_path(&self) -> PathBuf {
        profile_method!("get_config_path");

        self.base_dir.join("thag_rs").join("config.toml")
    }

    fn is_real(&self) -> bool {
        true
    }
}

/// Initializes and returns the configuration.
#[allow(clippy::module_name_repetitions)]
pub fn maybe_config() -> Option<Config> {
    profile!("maybe_config");
    lazy_static_var!(Option<Config>, maybe_load_config()).clone()
}

fn maybe_load_config() -> Option<Config> {
    profile!("maybe_load_config");
    // eprintln!("In maybe_load_config, should not see this message more than once");

    let context = get_context();

    match load(&context) {
        Ok(Some(config)) => Some(config),
        Ok(None) => {
            eprintln!("No config file found - this is allowed");
            None
        }
        Err(e) => {
            cprtln!(
                &Style::from(nu_ansi_term::Color::LightRed),
                "Failed to load config: {e}"
            );
            // sleep(Duration::from_secs(1));
            // println!("Failed to load config: {e}");
            std::process::exit(1);
        }
    }
}

/// Gets the real or mock context according to whether test mode is detected via the `TEST_ENV` sstem variable.
///
/// # Panics
///
/// Panics if there is any issue accessing the current directory, e.g. if it doesn't exist or we don't have sufficient permissions to access it.
#[must_use]
pub fn get_context() -> Arc<dyn Context> {
    profile!("get_context");
    let context: Arc<dyn Context> = if var("TEST_ENV").is_ok() {
        let current_dir = current_dir().expect("Could not get current dir");
        let config_path = current_dir.join("tests/assets").join("config.toml");
        let mut mock_context = MockContext::default();
        mock_context
            .expect_get_config_path()
            .return_const(config_path);
        mock_context.expect_is_real().return_const(false);
        Arc::new(mock_context)
    } else {
        Arc::new(RealContext::new())
    };
    context
}

/// Load the existing configuration file, if one exists at the specified location.
/// The absence of a configuration file is not an error.
///
/// # Errors
///
/// This function will return an error if it either finds a file and fails to read it,
/// or reads the file and fails to parse it.
pub fn load(context: &Arc<dyn Context>) -> ThagResult<Option<Config>> {
    profile!("load");
    let config_path = context.get_config_path();

    debug_log!("config_path={config_path:?}");

    if !config_path.exists() {
        return Ok(Some(Config::default()));
    }

    let config = Config::load(&config_path)?;

    // Log validation success
    debug_log!("Config validation successful");
    Ok(Some(config))
}

/// Validate the content of the `config.toml` file.
///
/// # Errors
///
/// This function will bubble up any Toml parsing errors encountered.
pub fn validate_config_format(content: &str) -> ThagResult<()> {
    profile!("validate_config_format");
    // Try to parse as generic TOML first
    let doc = content
        .parse::<DocumentMut>()
        .map_err(|e| ThagError::Validation(format!("Invalid TOML syntax: {e}")))?;

    // Check for required sections
    if !doc.contains_key("dependencies") {
        return Err(ThagError::Validation(
            "Missing [dependencies] section in config".into(),
        ));
    }

    // Check for common mistakes
    if let Some(table) = doc.get("dependencies").and_then(|v| v.as_table()) {
        for (key, value) in table {
            #[allow(clippy::single_match)]
            match key {
                "inference_level" => {
                    if let Some(v) = value.as_str() {
                        if v.chars().next().unwrap_or('_').is_uppercase() {
                            return Err(ThagError::Validation(format!(
                                "inference_level should be lowercase: '{v}' should be '{}'",
                                v.to_lowercase()
                            )));
                        }
                    }
                }
                // Add checks for other fields
                _ => {}
            }
        }
    }

    Ok(())
}
