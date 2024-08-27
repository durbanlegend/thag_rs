#[cfg(feature = "profile")]
use firestorm::profile_fn;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::path::PathBuf;
use std::{env, fs};

use crate::colors::{ColorSupport, TermTheme};
use crate::debug_log;
use crate::logging::Verbosity;

lazy_static! {
    #[derive(Debug)]
    pub static ref MAYBE_CONFIG: Option<Config> = {
        let maybe_config = load();
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

// impl Default for Config {
//     fn default() -> Self {
//         Config {
//             logging: Logging {
//                 default_verbosity: Verbosity::Normal,
//             },
//             colors: Colors {
//                 color_support: ColorSupport::Ansi16,
//                 term_theme: TermTheme::Dark,
//             },
//         }
//     }
// }

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

fn get_config_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        PathBuf::from(env::var("APPDATA").unwrap())
            .join("thag_rs")
            .join("config.toml")
    } else {
        home::home_dir()
            .unwrap()
            .join(".config")
            .join("thag_rs")
            .join("config.toml")
    }
}

#[must_use]
pub fn load() -> Option<Config> {
    #[cfg(feature = "profile")]
    {
        profile_fn!(load);
    }
    let config_path = get_config_path();
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

/// Main function for use by testing or the script runner.
#[allow(dead_code)]
fn main() {
    if let Some(config) = load() {
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
