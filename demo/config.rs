/*[toml]
[dependencies]
dirs = "5.0"
serde = { version = "1.0", features = ["derive"] }
serde_derive = "1.0"
serde_json = "1.0"
serde_with = "3.9"
strum = "0.26"
strum_macros = "0.26"
supports-color = "3.0.0"
toml = "0.8"
*/

/// Prototype of configuration file implementation. Delegated the grunt work to ChatGPT.
//# Purpose: Develop a configuration file implementation for `thag_rs`.
//# Categories: prototype, technique
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::fs;
use std::path::PathBuf;
use strum_macros::EnumString;

#[derive(Clone, Copy, Debug, Deserialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Verbosity {
    Quieter,
    Quiet,
    Normal,
    Verbose,
}

#[derive(Debug, Deserialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ColorSupport {
    Xterm256,
    Ansi16,
    None,
}

#[derive(Debug, Deserialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TermTheme {
    Light,
    Dark,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    logging: LoggingConfig,
    colors: ColorsConfig,
}

#[allow(dead_code)]
#[serde_as]
#[derive(Debug, Deserialize)]
struct LoggingConfig {
    #[serde_as(as = "DisplayFromStr")]
    default_verbosity: Verbosity,
}

#[allow(dead_code)]
#[serde_as]
#[derive(Debug, Deserialize)]
struct ColorsConfig {
    #[serde_as(as = "DisplayFromStr")]
    color_support: ColorSupport,
    #[serde_as(as = "DisplayFromStr")]
    term_theme: TermTheme,
}

fn get_config_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        dirs::config_dir()
            .unwrap()
            .join("thag_rs")
            .join("config.toml")
    } else {
        dirs::home_dir()
            .unwrap()
            .join(".config")
            .join("thag_rs")
            .join("config.toml")
    }
}

fn load_config() -> Option<Config> {
    let config_path = get_config_path();
    println!("config_path={config_path:?}");

    if config_path.exists() {
        let config_str = fs::read_to_string(config_path).ok()?;
        eprintln!("config_str={config_str}");
        let config: Result<Config, toml::de::Error> = toml::from_str(&config_str);
        eprintln!("config={config:#?}");
        let config: Config = toml::from_str(&config_str).ok()?;
        Some(config)
    } else {
        None
    }
}

fn main() {
    if let Some(config) = load_config() {
        println!("Loaded config: {:?}", config);
        println!(
            "default_verbosity={:?}, color_support={:?}, term_theme={:?}",
            config.logging.default_verbosity, config.colors.color_support, config.colors.term_theme
        );
    } else {
        eprintln!("Configuration file not found or not valid.");
    }
}
