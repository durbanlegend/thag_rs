/*[toml]
[dependencies]
log = "0.4.22"
# thag_rs = { path = "/Users/donf/projects/thag_rs" }
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", rev = "79582b0a889bca191a15a9d85f7d4c0ab5fbab69"}
*/

/// Prototype of configuration file implementation. Delegated the grunt work to ChatGPT.
//# Purpose: Develop a configuration file implementation for `thag_rs`.
//# Categories: prototype, technique
use edit::edit_file;
use firestorm::{profile_fn, profile_method};
use home;
use mockall::{automock, predicate::str};
use nu_ansi_term;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
#[cfg(target_os = "windows")]
use std::env;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};
use thag_rs::{
    cvprtln, debug_log, lazy_static_var, vlog, ColorSupport, Lvl, TermTheme, ThagResult, Verbosity,
    V,
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub logging: Logging,
    pub colors: Colors,
    pub proc_macros: ProcMacros,
    pub misc: Misc,
    pub dependencies: Dependencies, // New section
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Dependencies {
    pub exclude_unstable_features: bool,
    pub exclude_std_feature: bool,
    pub use_detailed_dependencies: bool,
    pub always_include_features: Vec<String>,
    pub group_related_features: bool,
    pub show_feature_dependencies: bool,
    pub exclude_prerelease: bool,        // New option
    pub minimum_downloads: Option<u64>,  // New option
    pub minimum_version: Option<String>, // New option
    pub feature_overrides: HashMap<String, FeatureOverride>,
    pub global_excluded_features: Vec<String>,
}

impl Dependencies {
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
            filtered = filtered
                .into_iter()
                .filter(|f| {
                    let keep = !self
                        .global_excluded_features
                        .iter()
                        .any(|ex| f.contains(ex));
                    if !keep {
                        debug_log!("Excluding feature '{}' due to global exclusion", f);
                    }
                    keep
                })
                .collect();
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
            filtered = filtered
                .into_iter()
                .filter(|f| {
                    let keep = !override_config.excluded_features.contains(f);
                    if !keep {
                        debug_log!("Excluding feature '{}' due to crate-specific override", f);
                    }
                    keep
                })
                .collect();

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
            filtered = filtered
                .into_iter()
                .filter(|f| {
                    let keep = !f.contains("unstable");
                    if !keep {
                        debug_log!("Excluding unstable feature '{}'", f);
                    }
                    keep
                })
                .collect();
        }

        if self.exclude_std_feature {
            filtered = filtered
                .into_iter()
                .filter(|f| {
                    let keep = f != "std";
                    if !keep {
                        debug_log!("Excluding std feature");
                    }
                    keep
                })
                .collect();
        }

        // Always include specified features
        for f in &self.always_include_features {
            if !filtered.contains(f) {
                debug_log!("Adding always-included feature '{}'", f);
                filtered.push(f.clone());
            }
        }

        // Remove duplicates
        filtered.sort();
        filtered.dedup();

        #[cfg(debug_assertions)]
        debug_log!("Final features for {}: {:?}", crate_name, filtered);

        filtered
    }

    // Make should_include_feature use filter_features
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

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Logging {
    #[serde_as(as = "DisplayFromStr")]
    pub default_verbosity: Verbosity,
}

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Colors {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default)]
    pub color_support: ColorSupport,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub term_theme: TermTheme,
}

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ProcMacros {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub proc_macro_crate_path: Option<String>,
}

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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

    eprintln!("config_path={config_path:?}");

    if config_path.exists() {
        let config_str = fs::read_to_string(config_path).ok()?;
        debug_log!("config_str={config_str:?}");
        let config: Config = toml::from_str(&config_str).ok()?;
        debug_log!("config={config:?}");
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

    eprintln!("config_path={config_path:?}");

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
fn main() -> ThagResult<()> {
    let maybe_config = load(&RealContext::new());

    if let Some(config) = maybe_config {
        cvprtln!(Lvl::EMPH, V::QQ, "Loaded config:");
        let toml = &toml::to_string_pretty(&config)?;
        for line in toml.lines() {
            cvprtln!(Lvl::BRI, V::QQ, "{line}");
        }
        eprintln!();
        eprintln!(
            "verbosity={:?}, ColorSupport={:?}, TermTheme={:?}",
            config.logging.default_verbosity, config.colors.color_support, config.colors.term_theme
        );
    } else {
        eprintln!("No configuration file found.");
    }
    Ok(())
}
