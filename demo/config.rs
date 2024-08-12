/*[toml]
[dependencies]
dirs = "5.0"
#rs-script = { git = "https://github.com/durbanlegend/rs-script" }
serde = { version = "1.0", features = ["derive"] }
serde_derive = "1.0"
serde_json = "1.0"
serde_with = "1.0"
strum = "0.26"
strum_macros = "0.26"
supports-color = "3.0.0"
toml = "0.8"
*/

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use strum_macros::EnumString;

#[derive(Debug, Deserialize, EnumString)]
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
#[derive(Debug, Deserialize)]
struct LoggingConfig {
    #[serde(with = "serde_with::rust::display_fromstr")]
    verbosity: Verbosity,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ColorsConfig {
    #[serde(with = "serde_with::rust::display_fromstr")]
    color_support: ColorSupport,
    #[serde(with = "serde_with::rust::display_fromstr")]
    term_theme: TermTheme,
}

fn get_config_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        dirs::config_dir()
            .unwrap()
            .join("rs-script")
            .join("config.toml")
    } else {
        dirs::home_dir()
            .unwrap()
            .join(".config")
            .join("rs-script")
            .join("config.toml")
    }
}

fn load_config() -> Option<Config> {
    let config_path = get_config_path();
    println!("config_path={config_path:?}");

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

fn main() {
    if let Some(config) = load_config() {
        println!("Loaded config: {:?}", config);
        println!(
            "verbosity={:?}, ColorSupport={:?}, TermTheme={:?}",
            config.logging.verbosity, config.colors.color_support, config.colors.term_theme
        );
    } else {
        println!("No configuration file found.");
    }
}
