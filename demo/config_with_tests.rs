/*[toml]
[dependencies]
# thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["config", "core", "simplelog"] }
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["config", "core", "simplelog"] }
*/

/// Demo of unit testing a non-snippet source file such as a library module using `thag --test-only (-T)`.
///
/// The unit tests must be in mod `tests` in the file.
///
/// `thag` will leave the file as is, but generate a temporary Cargo.toml for it in the usual way as a prerequisite for running `cargo test`.
///
/// `thag` will then invoke `cargo test` on the file, specifying the Cargo.toml location via `--manifest-path`.
///
/// `thag <filepath> -T [-- <cargo test options>]`
///
/// E.g.:
///
/// `TEST_CONFIG_PATH=/absolute/path/to/test/config.toml cargo run demo/config_with_tests.rs -Tv -- --nocapture --show-output`
///
//# Purpose: Demonstrate unit testing a file in situ without wrapping it if it doesn't have a main method.
//# Categories: technique, testing
use documented::{Documented, DocumentedFields, DocumentedVariants};
use edit::edit_file;
use mockall::{automock, predicate::str};
use serde::de;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
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
use thag_rs::{
    clog, clog_error, cprtln, cvprtln, debug_log, lazy_static_var, profile, profile_method, Color,
    ColorSupport, Level, Lvl, TermBgLuma, ThagError, ThagResult, Verbosity, V,
};
use toml_edit::DocumentMut;

const DEFAULT_CONFIG: &str = include_str!("../assets/default_config.toml");

/// Configuration categories
#[derive(Clone, Debug, Default, Deserialize, Serialize, Documented, DocumentedFields)]
#[serde(default)]
pub struct Config {
    /// Logging configuration
    pub logging: Logging,
    /// Color settings
    pub colors: Colors,
    /// Proc macros directory location, e.g. `demo/proc_macros`
    pub proc_macros: ProcMacros,
    /// Dependency handling settings
    pub dependencies: Dependencies, // New section
    /// Miscellaneous settings
    pub misc: Misc,
}

impl Config {
    /// Load the user's config file, or if there is none, load the default.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered.
    pub fn load_or_create_default(ctx: &impl Context) -> Result<Self, Box<dyn Error>> {
        profile_method!("load_or_create_default");

        let config_path = ctx.get_config_path();

        #[cfg(debug_assertions)]
        debug_log!(
            "1. config_path={config_path:#?}, exists={}",
            config_path.exists()
        );

        if !config_path.exists() {
            let path = config_path
                .parent()
                .ok_or(ThagError::NoneOption("No parent for {config_path:#?}"))?;
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
        let config_str = fs::read_to_string(&config_path)?;
        let maybe_config = toml::from_str(&config_str);
        #[cfg(debug_assertions)]
        debug_log!("5. maybe_config={maybe_config:#?}");
        Ok(maybe_config?)
    }

    /// Load a configuration.
    ///
    /// # Errors
    ///
    /// This function will bubble up any errors encountered.
    pub fn load(path: &Path) -> Result<Self, ThagError> {
        profile_method!("load");
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        validate_config_format(&content)?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ThagError> {
        profile_method!("validate");
        // Validate Dependencies section
        self.dependencies
            .validate()
            .map_err(|e| ThagError::Validation(format!("Dependencies validation failed: {e}")))?;

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
        profile_method!("default");
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
    #[must_use]
    pub fn filter_maximal_features(
        &self,
        crate_name: &str,
        features: &[String],
    ) -> (Vec<String>, bool) {
        profile_method!("filter_maximal_features");
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

    // Make should_include_feature use filter_features
    #[must_use]
    pub fn should_include_feature(&self, feature: &str, crate_name: &str) -> bool {
        profile_method!("should_include_feature");
        self.filter_maximal_features(crate_name, &[feature.to_string()])
            .0
            .contains(&feature.to_string())
    }

    // New method for config features based on overrides
    #[must_use]
    pub fn apply_config_features(
        &self,
        crate_name: &str,
        all_features: &[String],
    ) -> (Vec<String>, bool) {
        profile_method!("apply_config_features");

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
                        cvprtln!(
                            Lvl::WARN,
                            V::QQ,
                            "Configured feature `{}` does not exist in crate {}. Available features are:",
                            feature,
                            crate_name
                        );
                        for available in all_features {
                            cvprtln!(Lvl::BRI, V::QQ, "{}", available);
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

    // Method to get features based on inference level
    #[must_use]
    pub fn get_features_for_inference_level(
        &self,
        crate_name: &str,
        all_features: &[String],
        level: &DependencyInference,
    ) -> (Option<Vec<String>>, bool) {
        profile_method!("get_features_for_inference_level");
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

    fn validate(&self) -> Result<(), String> {
        profile_method!("validate");
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
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Documented, DocumentedFields)]
#[serde(default)]
pub struct Logging {
    /// Default verbosity setting
    #[serde_as(as = "DisplayFromStr")]
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
#[serde(rename_all = "snake_case")] // Add this line
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
        profile_method!("deserialize");
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
#[serde_as]
#[derive(Clone, Debug, Deserialize, Documented, DocumentedFields, Serialize)]
pub struct Colors {
    /// Color support override. Sets the terminal's color support level. The alternative is
    /// to leave it up to thag_rs, which depending on the platform may call 3rd-party crates
    /// to interrogate the terminal, which could cause misbehaviour, or may choose a default,
    /// which might not take advantage of the full capabilities of the terminal.
    /// If the terminal can't handle your chosen level, this may cause unwanted control strings
    /// to be interleaved with the messages.
    /// If your terminal can handle 16m colors, choose xterm256
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default)]
    pub color_support: ColorSupport,
    #[serde(default)]
    /// Light or dark terminal background override
    #[serde_as(as = "DisplayFromStr")]
    pub term_theme: TermBgLuma,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            color_support: ColorSupport::Undetermined,
            term_theme: TermBgLuma::Dark,
        }
    }
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

/// Miscellaneous configuration parameters
#[serde_as]
#[derive(Clone, Debug, Default, Documented, DocumentedFields, Deserialize, Serialize)]
#[serde(default)]
pub struct Misc {
    /// Strip double quotes from around string literals returned by snippets
    // #[serde_as(as = "DisplayFromStr")]
    pub unquote: bool,
}

#[automock]
pub trait Context: Debug {
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
        profile_method!("new_real_contexr");
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
        profile_method!("new_real_context");
        let base_dir =
            PathBuf::from(thag_rs::get_home_dir_string().expect("Could not find home directory"))
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
    lazy_static_var!(Option<Config>, {
        let context = RealContext::new();
        let load_or_default = Config::load_or_create_default(&context);
        load_or_default.map_or_else(|_| maybe_load_config(), Some)
    })
    .clone()
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
            clog_error!("Failed to load config: {e}");
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
        clog!(
            Level::Warning,
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
pub fn open(context: &dyn Context) -> ThagResult<Option<String>> {
    profile!("open");
    let config_path = context.get_config_path();
    debug_log!("config_path={config_path:?}");

    let exists = config_path.exists();
    if !exists {
        let dir_path = &config_path.parent().ok_or("Can't create directory")?;
        fs::create_dir_all(dir_path)?;

        cprtln!(
            &Color::yellow().bold(), // using our Color type
            "No configuration file found at {}. Creating one using system defaults...",
            config_path.display()
        );

        fs::write(&config_path, DEFAULT_CONFIG)?;
    }

    eprintln!("About to edit {config_path:#?}");
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
pub fn validate_config_format(content: &str) -> Result<(), ThagError> {
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

/// Main function for use by testing or the script runner.
#[allow(dead_code, unused_variables)]
fn main() {
    let maybe_config = load(&get_context());

    if let Ok(Some(config)) = maybe_config {
        // #[cfg(debug_assertions)]
        // debug_log!("Loaded config: {config:?}");
        #[cfg(debug_assertions)]
        debug_log!(
            "verbosity={:?}, ColorSupport={:?}, TermBgLuma={:?}",
            config.logging.default_verbosity,
            config.colors.color_support,
            config.colors.term_theme
        );
    } else {
        debug_log!("No configuration file found.");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        load, open, validate_config_format, Config, Context, Dependencies, FeatureOverride,
        MockContext, RealContext,
    };
    use simplelog::{
        ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode, WriteLogger,
    };
    use std::{
        fs::File,
        path::PathBuf,
        sync::{Arc, OnceLock},
    };
    use tempfile::TempDir;
    use thag_rs::{
        cvprtln, debug_log, logging::Verbosity, ColorSupport, Lvl, TermBgLuma, ThagResult, V,
    };

    static LOGGER: OnceLock<()> = OnceLock::new();

    fn init_logger() {
        // Choose between simplelog and env_logger based on compile feature
        LOGGER.get_or_init(|| {
            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Debug,
                    simplelog::Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    LevelFilter::Debug,
                    simplelog::Config::default(),
                    File::create("app.log").unwrap(),
                ),
            ])
            .unwrap();
            debug_log!("Initialized simplelog");
        });

        // #[cfg(not(feature = "simplelog"))] // This will use env_logger if simplelog is not active
        // {
        //     let _ = env_logger::builder().is_test(true).try_init();
        // }
    }

    // Set environment variables before running tests
    fn set_up() {
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");
    }

    #[test]
    fn test_config_load_config_success() -> ThagResult<()> {
        set_up();
        init_logger();

        let test_cfg_path = "TEST_CONFIG_PATH";
        let config_path = match std::env::var(test_cfg_path) {
            Ok(config_path) => config_path,
            Err(err) => {
                cvprtln!(
                    Lvl::ERR,
                    V::QQ,
                    "Environment variable {test_cfg_path} must be set to location of test config.toml"
                );
                return Err(err.into());
            }
        };

        let get_context = || -> Arc<dyn Context> {
            let context: Arc<dyn Context> = if std::env::var("TEST_ENV").is_ok() {
                let mut mock_context = MockContext::default();
                mock_context
                    .expect_get_config_path()
                    .return_const(config_path.clone());
                mock_context.expect_is_real().return_const(false);
                eprintln!("Using MockContext");
                Arc::new(mock_context)
            } else {
                eprintln!("Using RealContext");
                Arc::new(RealContext::new())
            };
            context
        };

        let config = load(&get_context())
            .expect("Failed to load config")
            .unwrap();

        // eprintln!("config={config:#?}");

        assert_eq!(config.logging.default_verbosity, Verbosity::Normal);
        assert_eq!(config.colors.color_support, ColorSupport::default());
        assert_eq!(config.colors.term_theme, TermBgLuma::default());
        Ok(())
    }

    #[test]
    fn test_config_load_config_file_not_found() {
        set_up();
        init_logger();

        let get_context = || -> Arc<dyn Context> {
            let context: Arc<dyn Context> = if std::env::var("TEST_ENV").is_ok() {
                let mut mock_context = MockContext::default();
                mock_context
                    .expect_get_config_path()
                    .return_const(PathBuf::from("/non/existent/path/config.toml"));
                mock_context.expect_is_real().return_const(false);
                Arc::new(mock_context)
            } else {
                Arc::new(RealContext::new())
            };
            context
        };

        let config = load(&get_context()).expect("Failed to load config");

        assert!(
            config.is_some(),
            "Expected to load default config when config file is not found"
        );
    }

    #[test]
    fn test_config_load_config_invalid_format() {
        set_up();
        init_logger();
        let config_content = r#"invalid = toml"#;
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, config_content).expect("Failed to write to temp config file");

        let get_context = || -> Arc<dyn Context> {
            let context: Arc<dyn Context> = if std::env::var("TEST_ENV").is_ok() {
                let mut mock_context = MockContext::default();
                mock_context
                    .expect_get_config_path()
                    .return_const(config_path.clone());
                mock_context.expect_is_real().return_const(false);
                Arc::new(mock_context)
            } else {
                Arc::new(RealContext::new())
            };
            context
        };

        let config = load(&get_context());
        // eprintln!("config={config:#?}");
        assert!(config.is_err());
    }

    // #[ignore = "Opens file and expects human interaction"]
    #[test]
    fn test_config_edit_creates_config_file_if_not_exists() {
        set_up();
        init_logger();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");

        let mut mock_context = MockContext::default();
        mock_context
            .expect_get_config_path()
            .return_const(config_path.clone());
        mock_context.expect_is_real().return_const(false);

        let result = open(&mock_context).expect("Failed to edit config");

        assert!(config_path.exists(), "Config file should be created");
        let config_content =
            std::fs::read_to_string(&config_path).expect("Failed to read config file");
        // eprintln!("config_content={config_content}");
        #[cfg(target_os = "windows")]
        assert!(
            config_content.contains("[dependencies.feature_overrides.syn]"),
            "Config file should contain the expected `syn` crate overrides"
        );
        #[cfg(target_os = "windows")]
        assert!(
            config_content.contains("[dependencies.feature_overrides.syn]"),
            "Config file should contain the expected `syn` crate overrides"
        );
        #[cfg(target_os = "windows")]
        assert!(
            config_content.contains("visit-mut"),
            "Config file should contain the expected `syn` crate overrides"
        );
        #[cfg(not(target_os = "windows"))]
        assert!(
            config_content.contains(
                r#"[dependencies.feature_overrides.syn]
required_features = [
    "extra-traits",
    "fold",
    "full",
    "parsing",
    "visit",
    "visit-mut",
]
default_features = false"#
            ),
            "Config file should contain the expected `syn` crate overrides"
        );
        assert_eq!(result, Some(String::from("End of edit")));
    }

    fn create_test_config() -> Dependencies {
        set_up();
        init_logger();
        let mut config = Dependencies::default();
        config.exclude_unstable_features = true;
        config.exclude_std_feature = true;
        config.global_excluded_features = vec!["default".to_string(), "sqlite".to_string()];
        config.always_include_features = vec!["derive".to_string()];

        let rustyline_override = FeatureOverride {
            excluded_features: Some(vec!["with-sqlite-history".to_string()]),
            required_features: Some(vec!["with-file-history".to_string()]),
            default_features: Some(true),
            // alternative_features: vec![],
        };

        config
            .feature_overrides
            .insert("rustyline".to_string(), rustyline_override);
        config
    }

    #[test]
    fn test_config_filter_features_global_exclusions() {
        set_up();
        init_logger();
        let config = create_test_config();
        let features = &[
            "default".to_string(),
            "derive".to_string(),
            "std".to_string(),
        ];
        let filtered = config.filter_maximal_features("some_crate", features).0;
        assert!(!filtered.contains(&"default".to_string()));
        assert!(filtered.contains(&"derive".to_string())); // Always included
        assert!(!filtered.contains(&"std".to_string()));
        eprintln!("config={}", toml::to_string_pretty(&config).unwrap());
    }

    #[test]
    fn test_config_filter_features_crate_specific() {
        set_up();
        init_logger();
        let config = create_test_config();
        let features = &[
            "with-sqlite-history".to_string(),
            "derive".to_string(),
            "with-fuzzy".to_string(),
        ];
        let filtered = config.filter_maximal_features("rustyline", features).0;
        assert!(!filtered.contains(&"with-sqlite-history".to_string()));
        assert!(filtered.contains(&"with-file-history".to_string())); // Required
        assert!(filtered.contains(&"derive".to_string()));
        assert!(filtered.contains(&"with-fuzzy".to_string()));
    }

    #[test]
    fn test_config_should_include_feature() {
        set_up();
        init_logger();
        let config = create_test_config();
        assert!(!config.should_include_feature("default", "some_crate"));
        assert!(config.should_include_feature("derive", "some_crate"));
        assert!(!config.should_include_feature("with-sqlite-history", "rustyline"));
        assert!(config.should_include_feature("with-file-history", "rustyline"));
    }

    #[test]
    fn test_config_validation() {
        // Test valid config
        let config = r#"
            [dependencies]
            inference_level = "custom"
            exclude_unstable_features = true

            [dependencies.feature_overrides.clap]
            required_features = ["derive"]
            excluded_features = ["unstable"]
            default_features = true
        "#;

        assert!(validate_config_format(config).is_ok());

        // Test invalid config
        let invalid_config = r#"
            [dependencies]
            inference_level = "Custom"  # Wrong case

            [dependencies.feature_overrides.tokio]
            required_features = ["rt"]
            excluded_features = ["rt"]  # Conflict
        "#;

        assert!(validate_config_format(invalid_config).is_err());
    }

    #[test]
    fn test_config_load_or_create_default_when_config_doesnt_exist() -> ThagResult<()> {
        // Create a temporary directory that will be automatically cleaned up when the test ends
        let temp_dir = TempDir::new()?;
        let mut mock_context = MockContext::new();

        // Set up the config path inside the temporary directory
        let config_path = temp_dir.path().join("thag_rs").join("config.toml");

        mock_context
            .expect_get_config_path()
            .return_const(config_path.clone());

        mock_context.expect_is_real().return_const(false);

        let maybe_config = Config::load_or_create_default(&mock_context);

        assert!(
            maybe_config.is_ok(),
            "Expected Ok result, got {:?}",
            maybe_config
        );
        assert!(config_path.exists(), "Config file was not created");

        // TempDir will automatically clean up when it goes out of scope
        Ok(())
    }
}
