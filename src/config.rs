use crate::{debug_log, lazy_static_var, ColorSupport, TermTheme, ThagResult, Verbosity};
use edit::edit_file;
use firestorm::{profile_fn, profile_method};
use mockall::{automock, predicate::str};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
#[cfg(target_os = "windows")]
use std::env;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

/// Initializes and returns the configuration.
#[allow(clippy::module_name_repetitions)]
pub fn maybe_config() -> Option<Config> {
    profile_fn!(maybe_config);
    lazy_static_var!(Option<Config>, maybe_load_config()).clone()
}

fn maybe_load_config() -> Option<Config> {
    profile_fn!(maybe_load_config);
    // eprintln!("In maybe_load_config, should not see this message more than once");
    let maybe_config = load(&RealContext::new());
    if let Some(config) = maybe_config {
        // debug_log!("Loaded config: {config:?}");
        return Some(config);
    }
    None::<Config>
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub logging: Logging,
    pub colors: Colors,
    pub proc_macros: ProcMacros,
    pub misc: Misc,
    pub dependencies: Dependencies, // New section
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Dependencies {
    pub exclude_unstable_features: bool,
    pub exclude_std_feature: bool,
    pub use_detailed_dependencies: bool,
    pub exclude_feature_patterns: Vec<String>,
    pub always_include_features: Vec<String>,
    pub group_related_features: bool,
    pub show_feature_dependencies: bool,
    pub exclude_prerelease: bool,        // New option
    pub minimum_downloads: Option<u64>,  // New option
    pub minimum_version: Option<String>, // New option
}

impl Dependencies {
    pub fn should_include_feature(&self, feature: &str) -> bool {
        if self.always_include_features.contains(&feature.to_string()) {
            return true;
        }
        if self.exclude_unstable_features && feature.contains("unstable") {
            return false;
        }
        if self.exclude_std_feature && feature == "std" {
            return false;
        }
        !self
            .exclude_feature_patterns
            .iter()
            .any(|pattern| feature.contains(pattern))
    }
}

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Logging {
    #[serde_as(as = "DisplayFromStr")]
    pub default_verbosity: Verbosity,
}

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Colors {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default)]
    pub color_support: ColorSupport,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub term_theme: TermTheme,
}

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ProcMacros {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub proc_macro_crate_path: Option<String>,
}

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Misc {
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

#[must_use]
pub fn load(context: &dyn Context) -> Option<Config> {
    profile_fn!(load);
    let config_path = context.get_config_path();

    debug_log!("config_path={config_path:?}");

    if config_path.exists() {
        let config_str = fs::read_to_string(config_path).ok()?;
        let config: Config = toml::from_str(&config_str).ok()?;
        Some(config)
    } else {
        None
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
#[allow(dead_code)]
fn main() {
    let maybe_config = load(&RealContext::new());

    if let Some(config) = maybe_config {
        debug_log!("Loaded config: {config:?}");
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
