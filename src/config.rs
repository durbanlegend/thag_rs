use firestorm::profile_fn;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
#[cfg(target_os = "windows")]
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::colors::{ColorSupport, TermTheme};
use crate::logging::Verbosity;
use crate::{debug_log, ThagError};

lazy_static! {
    #[derive(Debug)]
    pub static ref MAYBE_CONFIG: Option<Config> = {
        let maybe_config = load();
        #[cfg(debug_assertions)]
        if let Some(ref config) = maybe_config {
                debug_log!("Loaded config: {config:?}");
                debug_log!(
                    "default_verbosity={:?}, color_support={:?}, term_theme={:?}",
                    config.logging.default_verbosity,
                    config.colors.color_support,
                    config.colors.term_theme
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

#[cfg(target_os = "windows")]
fn get_config_path() -> PathBuf {
    PathBuf::from(env::var("APPDATA").expect("Error resolving path from $APPDATA"))
        .join("thag_rs")
        .join("config.toml")
}

#[cfg(not(target_os = "windows"))]
fn get_config_path() -> PathBuf {
    home::home_dir()
        .expect("Error resolving home::home_dir()")
        .join(".config")
        .join("thag_rs")
        .join("config.toml")
}

#[must_use]
pub fn load() -> Option<Config> {
    profile_fn!(load);
    let config_path = get_config_path();
    #[cfg(debug_assertions)]
    debug_log!("config_path={config_path:?}");

    if config_path.exists() {
        let config_str = fs::read_to_string(config_path).ok()?;
        // println!("config_str={config_str}");
        // let config: Result<Config, toml::de::Error> = toml::from_str(&config_str);
        // println!("config={config:#?}");
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
pub fn edit() -> Result<Option<String>, ThagError> {
    profile_fn!(edit);
    let config_path = get_config_path();
    #[cfg(debug_assertions)]
    debug_log!("config_path={config_path:?}");

    // Create the target directory if it doesn't exist
    let exists = config_path.exists();
    if !exists {
        let dir_path = &config_path.parent().expect("Can't create directory");
        fs::create_dir_all(dir_path).unwrap_or_else(|_| panic!("Failed to create {dir_path:#?}"));
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&config_path)?;
    //https://github.com/durbanlegend/thag_rs/blob/master/assets/config.toml.template
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
    edit::edit_file(&config_path)?;
    Ok(Some(String::from("End of history file edit")))
}

/// Main function for use by testing or the script runner.
#[allow(dead_code)]
fn main() {
    let maybe_config = load();
    #[cfg(debug_assertions)]
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
