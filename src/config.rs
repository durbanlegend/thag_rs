use firestorm::profile_fn;
use lazy_static::lazy_static;
use mockall::{automock, predicate::str};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
#[cfg(target_os = "windows")]
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::colors::{ColorSupport, TermTheme};

use crate::debug_log;
use crate::logging::Verbosity;
use crate::ThagError;

lazy_static! {
    #[derive(Debug)]
    pub static ref MAYBE_CONFIG: Option<Config> = {
        let maybe_config = load(&RealContext::new());

        if let Some(ref config) = maybe_config {
                debug_log!("Loaded config: {config:?}");
                debug_log!(
                    "default_verbosity={:?}, color_support={:?}, term_theme={:?}, unquote={}",
                    config.logging.default_verbosity,
                    config.colors.color_support,
                    config.colors.term_theme,
                    config.misc.unquote
                );
        }
        maybe_config
    };
}

// #[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub logging: Logging,
    pub colors: Colors,
    pub misc: Misc,
}

#[allow(dead_code)]
#[serde_as]
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Logging {
    #[serde_as(as = "DisplayFromStr")]
    pub default_verbosity: Verbosity,
}

#[allow(dead_code)]
#[serde_as]
#[derive(Debug, Default, Deserialize)]
pub struct Colors {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default)]
    pub color_support: ColorSupport,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default)]
    pub term_theme: TermTheme,
}

#[allow(dead_code)]
#[serde_as]
#[derive(Debug, Default, Deserialize)]
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
        let base_dir = home::home_dir()
            .expect("Error resolving home::home_dir()")
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
pub fn edit(context: &dyn Context) -> Result<Option<String>, ThagError> {
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
        edit::edit_file(&config_path)?;
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
