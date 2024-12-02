use crate::{debug_log, lazy_static_var, ColorSupport, TermTheme, ThagResult, Verbosity};
use documented::{Documented, DocumentedFields};
use edit::edit_file;
use firestorm::{profile_fn, profile_method};
use mockall::{automock, predicate::str};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
#[cfg(target_os = "windows")]
use std::env;
use std::{
    collections::HashMap,
    env::current_dir,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Arc,
};

/// Initializes and returns the configuration.
#[allow(clippy::module_name_repetitions)]
pub fn maybe_config() -> Option<Config> {
    profile_fn!(maybe_config);
    lazy_static_var!(Option<Config>, maybe_load_config()).clone()
}

fn maybe_load_config() -> Option<Config> {
    profile_fn!(maybe_load_config);
    eprintln!("In maybe_load_config, should not see this message more than once");

    let context = get_context();

    match load(&context) {
        Ok(Some(config)) => {
            // eprintln!("Loaded config: {config:?}");
            Some(config)
        }
        Ok(None) => {
            eprintln!("No config file found - this is allowed");
            None
        }
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            None
        }
    }
}

pub fn get_context() -> Arc<dyn Context> {
    let context: Arc<dyn Context> = if std::env::var("TEST_ENV").is_ok() {
        let current_dir = current_dir().expect("Could not get current dir");
        let config_path = current_dir.clone().join("tests/assets").join("config.toml");
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
}

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

/// Dependency handling
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Deserialize, Serialize, Documented, DocumentedFields)]
#[serde(default)]
pub struct Dependencies {
    /// Exclude features containing "unstable"
    pub exclude_unstable_features: bool,
    /// Exclude the "std" feature
    pub exclude_std_feature: bool,
    /// Detailed dependencies with features vs simple `name = "version"`
    pub use_detailed_dependencies: bool,
    /// Features that should always be included if present, e.g. `derive`
    pub always_include_features: Vec<String>,
    /// TODO Group related features together
    pub group_related_features: bool,
    /// TODO Show feature dependencies
    pub show_feature_dependencies: bool,
    /// Exclude releases with pre-release markers such as -beta.
    pub exclude_prerelease: bool, // New option
    // pub minimum_downloads: Option<u64>,  // New option
    // pub minimum_version: Option<String>, // New option
    /// Crate-level feature overrides
    pub feature_overrides: HashMap<String, FeatureOverride>,
    /// Features that should always be excluded
    pub global_excluded_features: Vec<String>,
}

impl Default for Dependencies {
    fn default() -> Self {
        Dependencies {
            exclude_unstable_features: true,
            exclude_std_feature: true,
            use_detailed_dependencies: true,
            always_include_features: vec!["derive".to_string()],
            group_related_features: true,
            show_feature_dependencies: true,
            exclude_prerelease: true,
            feature_overrides: HashMap::<String, FeatureOverride>::new(),
            global_excluded_features: vec![],
        }
    }
}

impl Dependencies {
    #[must_use]
    pub fn filter_features(&self, crate_name: &str, features: Vec<String>) -> Vec<String> {
        let mut filtered = features;

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

        // Apply crate-specific overrides
        if let Some(override_config) = self.feature_overrides.get(crate_name) {
            #[cfg(debug_assertions)]
            debug_log!("Applying overrides for crate {}", crate_name);

            // Remove excluded features
            let before_len = filtered.len();
            filtered.retain(|f| {
                let keep = self.always_include_features.contains(f)
                    || !override_config.excluded_features.contains(f);
                if !keep {
                    debug_log!("Excluding feature '{}' due to crate-specific override", f);
                }
                keep
            });

            // Add required features
            for f in &override_config.required_features {
                if !filtered.contains(f) {
                    debug_log!("Adding required feature '{}'", f);
                    filtered.push(f.clone());
                }
            }

            // Replace excluded features with alternatives if any were excluded
            if filtered.len() < before_len {
                for f in &override_config.alternative_features {
                    if !filtered.contains(f) {
                        debug_log!("Adding alternative feature '{}'", f);
                        filtered.push(f.clone());
                    }
                }
            }
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

        // Remove duplicates
        filtered.sort();
        filtered.dedup();

        #[cfg(debug_assertions)]
        debug_log!("Final features for {}: {:?}", crate_name, filtered);

        filtered
    }

    // Make should_include_feature use filter_features
    #[must_use]
    pub fn should_include_feature(&self, feature: &str, crate_name: &str) -> bool {
        self.filter_features(crate_name, vec![feature.to_string()])
            .contains(&feature.to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeatureOverride {
    pub excluded_features: Vec<String>,
    pub required_features: Vec<String>,
    pub alternative_features: Vec<String>,
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

/// Terminal color settings
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Documented, DocumentedFields, Serialize)]
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
    pub term_theme: TermTheme,
}

/// Demo proc macro settings
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Documented, DocumentedFields, Serialize)]
#[serde(default)]
pub struct ProcMacros {
    /// Absolute or relative path to demo proc macros crate, e.g. demo/proc_macros.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub proc_macro_crate_path: Option<String>,
}

/// Miscellaneous configuration parameters
#[serde_as]
#[derive(Clone, Debug, Default, Documented, DocumentedFields, Deserialize, Serialize)]
#[serde(default)]
pub struct Misc {
    /// Strip double quotes from around string literals returned by snippets
    #[serde_as(as = "DisplayFromStr")]
    pub unquote: bool,
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
        profile_method!(new_real_contexr);
        let base_dir = home::home_dir()
            .expect("Error resolving home::home_dir()")
            .join(".config");
        Self { base_dir }
    }
}

impl Context for RealContext {
    fn get_config_path(&self) -> PathBuf {
        profile_method!(get_config_path);

        self.base_dir.join("thag_rs").join("config.toml")
    }

    fn is_real(&self) -> bool {
        true
    }
}

/// Load the existing configuration file, if one exists at the specified location.
/// The absence of a configuration file is not an error.
///
/// # Errors
///
/// This function will return an error if it either finds a file and fails to read it,
/// or reads the file and fails to parse it..
pub fn load(context: &Arc<dyn Context>) -> ThagResult<Option<Config>> {
    profile_fn!(load);
    let config_path = context.get_config_path();

    eprintln!("config_path={config_path:?}");

    if config_path.exists() {
        let config_str = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {e}"))?;

        match toml::from_str(&config_str) {
            Ok(config) => Ok(Some(config)),
            Err(e) => Err(format!(
                "Failed to parse config file: {e}\nConfig content:\n{config_str}"
            )
            .into()),
        }
    } else {
        Ok(None)
    }
}

/// Open the configuration file in an editor.
/// # Errors
/// Will return `Err` if there is an error editing the file.
/// # Panics
/// Will panic if it can't create the parent directory for the configuration.
#[allow(clippy::unnecessary_wraps)]
pub fn edit(context: &dyn Context) -> ThagResult<Option<String>> {
    profile_fn!(edit);
    let config_path = context.get_config_path();

    debug_log!("config_path={config_path:?}");

    let exists = config_path.exists();
    if !exists {
        let dir_path = &config_path.parent().ok_or("Can't create directory")?;
        fs::create_dir_all(dir_path)?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&config_path)?;
    if !exists {
        let text = r#"# Please set up the config file as follows:
# 1. Follow the link below to the template on Github.
# 2. Copy the config file template contents using the "Raw (Copy raw file)" icon in the viewer.
# 3. Paste the contents in here.
# 4. Make the configuration changes you want. Ensure you un-comment the options you want.
# 5. Save the file.
# 6. Exit or tab away to return.
#
# https://github.com/durbanlegend/thag_rs/blob/master/assets/config.toml.template
"#;
        file.write_all(text.as_bytes())?;
    }
    eprintln!("About to edit {config_path:#?}");
    if context.is_real() {
        edit_file(&config_path)?;
    }
    Ok(Some(String::from("End of edit")))
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
            config.colors.color_support,
            config.colors.term_theme
        );
    } else {
        debug_log!("No configuration file found.");
    }
}
