use crate::{debug_log, lazy_static_var, ColorSupport, TermBgLuma, ThagCommonError, Verbosity};
use documented::{Documented, DocumentedFields, DocumentedVariants};
use edit::edit_file;
use mockall::{automock, predicate::str};
use serde::{
    de::{self, Error as DeError},
    Deserialize, Deserializer, Serialize,
};
#[cfg(target_os = "windows")]
use std::env;
use std::{
    collections::HashMap,
    env::{current_dir, var},
    error::Error,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use strum::{Display, EnumString};
use toml::Value;
use toml_edit::DocumentMut;

const DEFAULT_CONFIG: &str = include_str!("../../assets/default_config.toml");

/// Configuration categories
#[derive(Clone, Debug, Default, Deserialize, Serialize, Documented, DocumentedFields)]
#[serde(default)]
pub struct Config {
    /// Logging configuration
    pub logging: Logging,
    /// Color settings
    #[serde(alias = "colors")]
    pub styling: Styling,
    /// Proc macros directory location, e.g. `demo/proc_macros`
    pub proc_macros: ProcMacros,
    /// Dependency handling settings
    pub dependencies: Dependencies, // New section
    /// Miscellaneous settings
    pub misc: Misc,
}

/// Result type alias for config operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Config-specific error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Common error
    #[error("Common error: {0}")]
    Common(#[from] ThagCommonError),
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    /// TOML edit error
    #[error("TOML edit error: {0}")]
    TomlEdit(#[from] toml_edit::TomlError),
    /// Generic error
    #[error("{0}")]
    Generic(String),
}

#[derive(Debug, Deserialize)]
struct PartialConfig {
    #[serde(default)]
    logging: Option<Logging>,
    #[serde(default, alias = "colors")]
    styling: Option<Styling>,
    #[serde(default)]
    proc_macros: Option<ProcMacros>,
    #[serde(default)]
    dependencies: Option<Dependencies>,
    #[serde(default)]
    misc: Option<Misc>,
}

impl Config {
    /// Load the user's config file, or if there is none, load the default.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered.
    pub fn load_or_create_default(ctx: &impl Context) -> Result<Self, Box<dyn Error>> {
        let config_path = ctx.get_config_path();

        #[cfg(debug_assertions)]
        debug_log!(
            "1. config_path={config_path:#?}, exists={}",
            config_path.exists()
        );

        if !config_path.exists() {
            let path = config_path.parent().ok_or_else(|| {
                ConfigError::Generic(format!("No parent for {}", config_path.display()))
            })?;
            fs::create_dir_all(path)?;

            // Try to find default config in different locations
            let default_config = if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
                // First try cargo installed assets location
                let user_config = PathBuf::from(cargo_home)
                    .join("assets")
                    .join("default_config.toml");

                #[cfg(debug_assertions)]
                debug_log!(
                    "2. dist_config={user_config:#?}, exists={}",
                    user_config.exists()
                );
                if user_config.exists() {
                    fs::read_to_string(user_config)?
                } else {
                    // Fallback to embedded config
                    DEFAULT_CONFIG.to_string()
                }
            } else {
                DEFAULT_CONFIG.to_string()
            };

            #[cfg(debug_assertions)]
            debug_log!("3. default_config={default_config}");
            fs::write(&config_path, default_config)?;
        }

        #[cfg(debug_assertions)]
        debug_log!(
            "4. config_path={config_path:#?}, exists={}",
            config_path.exists()
        );
        // let config_str = fs::read_to_string(&config_path)?;
        // let maybe_config = toml::from_str(&config_str);
        let maybe_config = Self::load(&config_path);

        #[cfg(debug_assertions)]
        debug_log!("5. maybe_config={maybe_config:#?}");
        Ok(maybe_config?)
    }

    /// Load a configuration.
    ///
    /// # Errors
    ///
    /// This function will bubble up any errors encountered.
    pub fn load(path: &Path) -> ConfigResult<Self> {
        let content = std::fs::read_to_string(path)?;

        match toml::from_str::<Self>(&content) {
            Ok(config) => {
                config.validate()?;
                validate_config_format(&content)?;
                Ok(config)
            }
            Err(e) => {
                // If parsing failed, try to salvage what we can
                eprintln!(
                    "Warning: Config parse error ({e}). Attempting to preserve valid settings."
                );
                let mut default_config = Self::default();

                // Try to parse what we can
                if let Ok(partial_config) = toml::from_str::<PartialConfig>(&content) {
                    default_config.update_from_partial(partial_config);
                    eprintln!("Info: Successfully preserved valid config settings.");
                }

                Ok(default_config)
            }
        }
    }

    fn update_from_partial(&mut self, partial: PartialConfig) {
        if let Some(logging) = partial.logging {
            self.logging = logging;
        }
        if let Some(styling) = partial.styling {
            self.styling = styling;
        }
        if let Some(proc_macros) = partial.proc_macros {
            self.proc_macros = proc_macros;
        }
        if let Some(dependencies) = partial.dependencies {
            self.dependencies = dependencies;
        }
        if let Some(misc) = partial.misc {
            self.misc = misc;
        }
    }

    fn validate(&self) -> ConfigResult<()> {
        // Validate Dependencies section
        self.dependencies
            .validate()
            .map_err(|e| ConfigError::Generic(format!("Dependencies validation failed: {e}")))?;

        // Add validation for other sections as needed
        Ok(())
    }
}

/// Dependency handling
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Deserialize, Serialize, Documented, DocumentedFields)]
#[serde(default)]
pub struct Dependencies {
    /// Exclude features containing "unstable"
    pub exclude_unstable_features: bool,
    /// Exclude the "std" feature
    pub exclude_std_feature: bool,
    /// Features that should always be included if present, e.g. `derive`
    pub always_include_features: Vec<String>,
    /// Exclude releases with pre-release markers such as -beta.
    pub exclude_prerelease: bool, // New option
    /// Crate-specific feature overrides
    pub feature_overrides: HashMap<String, FeatureOverride>,
    /// Features that should always be excluded
    pub global_excluded_features: Vec<String>,
    /// How much `thag_rs` should intervene in inferring dependencies from code.
    pub inference_level: DependencyInference,
    // /// `false` specifies a detailed dependency with `default-features = false`.
    // pub default_features: bool,
}

impl Default for Dependencies {
    fn default() -> Self {
        Self {
            exclude_unstable_features: true,
            exclude_std_feature: true,
            always_include_features: vec!["derive".to_string()],
            exclude_prerelease: true,
            feature_overrides: HashMap::<String, FeatureOverride>::new(),
            global_excluded_features: vec![],
            inference_level: DependencyInference::Config,
        }
    }
}

impl Dependencies {
    /// Filter features based on configuration rules and return filtered features with `default_features` flag
    #[allow(clippy::unnecessary_map_or)]
    #[must_use]
    pub fn filter_maximal_features(
        &self,
        crate_name: &str,
        features: &[String],
    ) -> (Vec<String>, bool) {
        let mut filtered = features.to_owned();

        #[cfg(debug_assertions)]
        debug_log!(
            "Filtering features for crate {}: {:?}",
            crate_name,
            filtered
        );

        // Apply global exclusions
        if !self.global_excluded_features.is_empty() {
            #[cfg(debug_assertions)]
            let before_len = filtered.len();
            filtered.retain(|f| {
                let keep = !self
                    .global_excluded_features
                    .iter()
                    .any(|ex| f.contains(ex));
                if !keep {
                    debug_log!("Excluding feature '{}' due to global exclusion", f);
                }
                keep
            });
            #[cfg(debug_assertions)]
            if filtered.len() < before_len {
                debug_log!(
                    "Removed {} features due to global exclusions",
                    before_len - filtered.len()
                );
            }
        }

        let mut default_features = true;

        // Apply crate-specific overrides
        if let Some(override_config) = self.feature_overrides.get(crate_name) {
            #[cfg(debug_assertions)]
            debug_log!("Applying overrides for crate {}", crate_name);

            // Remove excluded features
            // let before_len = filtered.len();
            filtered.retain(|f| {
                let keep = self.always_include_features.contains(f) || {
                    override_config
                        .excluded_features
                        .as_ref()
                        .map_or(true, |excluded_features| !excluded_features.contains(f))
                };
                if !keep {
                    debug_log!("Excluding feature '{}' due to crate-specific override", f);
                }
                keep
            });

            // Add required features
            if let Some(ref required_features) = &override_config.required_features {
                for f in required_features {
                    if f.is_empty() {
                        continue;
                    }
                    if !filtered.contains(f) {
                        debug_log!("Adding required feature '{}'", f);
                        filtered.push(f.clone());
                    }
                }
            }

            default_features = override_config.default_features.unwrap_or(true);
        }

        // Apply other existing filters
        if self.exclude_unstable_features {
            filtered.retain(|f| {
                let keep = !f.contains("unstable") || self.always_include_features.contains(f);
                if !keep {
                    debug_log!("Excluding unstable feature '{}'", f);
                }
                keep
            });
        }

        if self.exclude_std_feature {
            filtered.retain(|f| {
                let keep = f != "std" || self.always_include_features.contains(f);
                if !keep {
                    debug_log!("Excluding std feature");
                }
                keep
            });
        }

        // Sort and remove duplicates
        filtered.sort();
        filtered.dedup();

        #[cfg(debug_assertions)]
        debug_log!("Final features for {}: {:?}", crate_name, filtered);

        (filtered.clone(), default_features)
    }

    /// Check if a specific feature should be included for a crate
    #[must_use]
    pub fn should_include_feature(&self, feature: &str, crate_name: &str) -> bool {
        self.filter_maximal_features(crate_name, &[feature.to_string()])
            .0
            .contains(&feature.to_string())
    }

    /// Apply configuration-based feature selection for a crate
    #[must_use]
    pub fn apply_config_features(
        &self,
        crate_name: &str,
        all_features: &[String],
    ) -> (Vec<String>, bool) {
        let (mut config_features, default_features) = self.feature_overrides.get(crate_name).map_or_else(|| {
            // Only include features from always_include_features that exist in all_features
            let intersection: Vec<String> = self.always_include_features
                .iter()
                .filter(|feature| all_features.contains(*feature))
                .cloned()
                .collect();
            (intersection, true)
        }, |override_config| {
            // Only include features from always_include_features that exist in all_features
            let mut config_features: Vec<String> = self.always_include_features
                .iter()
                .filter(|feature| all_features.contains(*feature))
                .cloned()
                .collect();

            if let Some(ref required_features) = &override_config.required_features {
                for feature in required_features {
                    if feature.is_empty() {
                        continue;
                    }
                    // Validate required features exist
                    if all_features.contains(feature) {
                        config_features.push(feature.clone());
                    } else {
                        println!(
                            "Configured feature `{feature}` does not exist in crate {crate_name}. Available features are:"
                        );
                        for available in all_features {
                            println!("{available}");
                        }
                    }
                }
            }
            (config_features, override_config.default_features.unwrap_or(true))
        });

        // Sort and remove duplicates
        config_features.sort();
        config_features.dedup();

        (config_features, default_features)
    }

    /// Get features based on the specified dependency inference level
    #[must_use]
    pub fn get_features_for_inference_level(
        &self,
        crate_name: &str,
        all_features: &[String],
        level: &DependencyInference,
    ) -> (Option<Vec<String>>, bool) {
        match level {
            DependencyInference::None | DependencyInference::Min => (None, true),
            DependencyInference::Config => {
                let (features, default_features) =
                    self.apply_config_features(crate_name, all_features);
                (Some(features), default_features)
            }
            DependencyInference::Max => {
                let (features, default_features) =
                    self.filter_maximal_features(crate_name, all_features);
                (Some(features), default_features)
            }
        }
    }

    /// Validate the dependency configuration for consistency
    fn validate(&self) -> Result<(), String> {
        // Validate feature overrides
        for (crate_name, override_config) in &self.feature_overrides {
            // Check for conflicts between required and excluded features
            if let Some(ref required_features) = override_config.required_features {
                if let Some(ref excluded_features) = override_config.excluded_features {
                    let conflicts: Vec<_> = required_features
                        .iter()
                        .filter(|f| excluded_features.contains(*f))
                        .collect();

                    if !conflicts.is_empty() {
                        return Err(format!(
                            "Crate {crate_name} has features that are both required and excluded: {conflicts:?}",
                        ));
                    }

                    // Check for empty feature lists
                    if required_features.is_empty() && excluded_features.is_empty() {
                        return Err(format!(
                            "Crate {crate_name} has empty feature override lists. Remove the override if not needed"
                        ));
                    }
                }
            }
        }

        // Validate global exclusions don't conflict with always-include features
        let global_conflicts: Vec<_> = self
            .always_include_features
            .iter()
            .filter(|f| self.global_excluded_features.contains(*f))
            .collect();

        if !global_conflicts.is_empty() {
            return Err(format!(
                "Features cannot be both always included and globally excluded: {global_conflicts:?}"
            ));
        }

        Ok(())
    }
}

/// Crate-specific feature overrides
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeatureOverride {
    /// Features to be excluded for crate
    pub excluded_features: Option<Vec<String>>,
    /// Features required for crate
    pub required_features: Option<Vec<String>>,
    /// `false` specifies a detailed dependency with `default-features = false`.
    /// Default: true, in line with the Cargo default.
    pub default_features: Option<bool>,
}

/// Logging settings
#[derive(Clone, Debug, Default, Deserialize, Serialize, Documented, DocumentedFields)]
#[serde(default)]
pub struct Logging {
    /// Default verbosity setting
    // #[serde_as(as = "DisplayFromStr")]
    pub default_verbosity: Verbosity,
}

#[derive(
    Clone,
    Debug,
    Default,
    Serialize,
    EnumString,
    Display,
    PartialEq,
    Eq,
    Documented,
    DocumentedVariants,
)]
/// Dependency inference level
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DependencyInference {
    /// Don't infer any dependencies
    None,
    /// Basic dependencies without features
    Min,
    /// Use config.toml feature overrides
    #[default]
    Config,
    /// Include all features not excluded by config
    Max,
}

// Custom deserializer to provide better error messages
impl<'de> de::Deserialize<'de> for DependencyInference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "minimal" => Ok(Self::Min),
            "config" => Ok(Self::Config),
            "maximal" => Ok(Self::Max),
            _ => Err(de::Error::custom(format!(
                "Invalid dependency inference level '{s}'. Expected one of: none, minimal, config, maximal"
            ))),
        }
    }
}

/// Terminal color settings
// #[serde_as]
#[derive(Clone, Debug, Deserialize, Documented, DocumentedFields, Serialize)]
pub struct Styling {
    /// Color support override. Sets the terminal's color support level. The alternative is
    /// to leave it up to `thag_rs`, which depending on the platform may call 3rd-party crates
    /// to interrogate the terminal, which could cause misbehaviour, or may choose a default,
    /// which might not take advantage of the full capabilities of the terminal.
    /// If the terminal can't handle your chosen level, this may cause unwanted control strings
    /// to be interleaved with the messages.
    /// If your terminal can handle 16m colors, choose xterm256
    // #[serde_as(as = "DisplayFromStr")]
    #[serde(default)]
    pub color_support: ColorSupport,
    #[serde(default)]
    /// Terminal background RGB override (user should coordinate this with `term_bg_luma`)
    pub term_bg_rgb: Option<[u8; 3]>,
    #[serde(default)]
    /// Light or dark terminal background override (user should coordinate this with `term_bg_rgb`)
    pub term_bg_luma: TermBgLuma,
    /// Directory to search for user-defined themes (can be overridden by `THAG_THEME_DIR` env var)
    #[serde(default)]
    pub theme_dir: Option<String>,
    /// First one is primary
    #[serde(default)]
    pub backgrounds: Vec<String>,
    /// For backward compatibility
    // #[serde(default = "default_background")]
    pub background: Option<String>,
    /// Preferred light themes in descending order
    #[serde(default)]
    pub preferred_light: Vec<String>,
    /// Preferred dark themes in descending order
    #[serde(default)]
    pub preferred_dark: Vec<String>,
    /// Preferred fallback light themes in descending order
    #[serde(default)]
    pub fallback_light: Vec<String>,
    /// Preferred fallback dark themes in descending order
    #[serde(default)]
    pub fallback_dark: Vec<String>,
}

impl Default for Styling {
    fn default() -> Self {
        Self {
            color_support: ColorSupport::Undetermined,
            term_bg_rgb: None,
            #[cfg(feature = "color_detect")]
            term_bg_luma: TermBgLuma::Undetermined,
            #[cfg(not(feature = "color_detect"))]
            term_bg_luma: TermBgLuma::Dark,
            theme_dir: None,
            backgrounds: vec![],
            background: None,
            preferred_light: vec![],
            preferred_dark: vec![],
            fallback_light: vec![],
            fallback_dark: vec![],
        }
    }
}

/// Demo proc macro settings
// #[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Documented, DocumentedFields, Serialize)]
#[serde(default)]
pub struct ProcMacros {
    /// Absolute or relative path to bank proc macros crate, e.g. `bank/proc_macros`.
    // #[serde_as(as = "Option<DisplayFromStr>")]
    pub bank_proc_macro_crate_path: Option<String>,
    /// Absolute or relative path to demo proc macros crate, e.g. `demo/proc_macros`.
    // #[serde_as(as = "Option<DisplayFromStr>")]
    pub demo_proc_macro_crate_path: Option<String>,
}

/// Miscellaneous configuration parameters
#[derive(Clone, Debug, Default, Documented, DocumentedFields, Deserialize, Serialize)]
#[serde(default)]
pub struct Misc {
    /// Strip double quotes from around string literals returned by snippets
    #[serde(deserialize_with = "boolean")]
    pub unquote: bool,
}

/// Custom deserialisation method for booleans, to accept current true/false or legacy "true"/"false".
fn boolean<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    let deserialize = Deserialize::deserialize(deserializer);
    deserialize.map_or_else(Err, |val| match val {
        Value::Boolean(b) => Ok(b),
        Value::String(s) => Ok(&s == "true"),
        _ => Err(DeError::custom("Wrong type, expected boolean")),
    })
}

#[automock]
/// Trait for providing configuration context, allowing for different implementations
/// in production versus testing environments.
pub trait Context: Debug {
    /// Returns the path where the configuration file should be located.
    fn get_config_path(&self) -> PathBuf;
    /// Returns true if this is a real context (not a mock for testing).
    fn is_real(&self) -> bool;
}

/// A struct for use in normal execution, as opposed to use in testing.
#[derive(Debug, Default)]
pub struct RealContext {
    /// Base directory for configuration files
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
        let base_dir =
            PathBuf::from(crate::get_home_dir_string().expect("Could not find home directory"))
                .join(".config");
        Self { base_dir }
    }
}

impl Context for RealContext {
    fn get_config_path(&self) -> PathBuf {
        self.base_dir.join("thag_rs").join("config.toml")
    }

    fn is_real(&self) -> bool {
        true
    }
}

/// Initializes and returns the configuration.
#[allow(clippy::module_name_repetitions)]
pub fn maybe_config() -> Option<Config> {
    lazy_static_var!(Option<Config>, {
        let context = RealContext::new();
        let load_or_default = Config::load_or_create_default(&context);
        load_or_default.map_or_else(|_| maybe_load_config(), Some)
    })
    .clone()
}

fn maybe_load_config() -> Option<Config> {
    // eprintln!("In maybe_load_config, should not see this message more than once");

    let context = get_context();

    match load(&context) {
        Ok(Some(config)) => Some(config),
        Ok(None) => {
            eprintln!("No config file found - this is allowed");
            None
        }
        Err(e) => {
            println!("Failed to load config: {e}");
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
pub fn load(context: &Arc<dyn Context>) -> ConfigResult<Option<Config>> {
    let config_path = context.get_config_path();

    debug_log!("config_path={config_path:?}");

    if !config_path.exists() {
        println!(
            "Configuration file path {} not found. No config loaded. System defaults will be used.",
            config_path.display()
        );
        return Ok(Some(Config::default()));
    }

    let config = Config::load(&config_path)?;

    // Log validation success
    debug_log!("Config validation successful");
    Ok(Some(config))
}

/// Open the configuration file in an editor.
/// # Errors
/// Will return `Err` if there is an error editing the file.
/// # Panics
/// Will panic if it can't create the parent directory for the configuration.
#[allow(clippy::unnecessary_wraps)]
pub fn open(context: &dyn Context) -> ConfigResult<Option<String>> {
    let config_path = context.get_config_path();
    debug_log!("config_path={config_path:?}");

    let exists = config_path.exists();
    if !exists {
        let dir_path = &config_path
            .parent()
            .ok_or_else(|| ConfigError::Generic("Can't create directory".to_string()))?;
        fs::create_dir_all(dir_path)?;

        println!(
            "Configuration file path {} not found. Creating it.",
            config_path.display()
        );

        fs::write(&config_path, DEFAULT_CONFIG)?;
    }

    eprintln!("Editing {}...", config_path.display());
    if context.is_real() {
        edit_file(&config_path)?;
    }
    Ok(Some(String::from("End of edit")))
}

/// Validate the content of the `config.toml` file.
///
/// # Errors
///
/// This function will bubble up any Toml parsing errors encountered.
pub fn validate_config_format(content: &str) -> ConfigResult<()> {
    // Try to parse as generic TOML first
    let doc = content
        .parse::<DocumentMut>()
        .map_err(|e| ConfigError::Generic(format!("Invalid TOML syntax: {e}")))?;

    // Check for required sections
    if !doc.contains_key("dependencies") {
        return Err(ConfigError::Generic(
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
                            return Err(ConfigError::Generic(format!(
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

/// Main function for use by testing or the script runner.
#[allow(dead_code, unused_variables)]
fn main() {
    let maybe_config = load(&get_context());

    if let Ok(Some(config)) = maybe_config {
        // #[cfg(debug_assertions)]
        // debug_log!("Loaded config: {config:?}");
        #[cfg(debug_assertions)]
        debug_log!(
            "verbosity={:?}, ColorSupport={:?}, TermTheme={:?}",
            config.logging.default_verbosity,
            config.styling.color_support,
            config.styling.term_bg_luma
        );
    } else {
        debug_log!("No configuration file found.");
    }
}
