/*[toml]
[dependencies]
dirs = "5.0"
inquire = "0.7.5"
serde = { version = "1.0.215", features = ["derive"] }
toml = "0.8"
*/

/// Prompted config file builder for `thag`, intended to be saved as a command with `-x`.
//# Purpose: Handy configuration file builder.
//# Categories: crates, technique, tools
use inquire::{Confirm, MultiSelect, Text};
use serde::Serialize;
use std::fs;

/// Extracts doc comments from a type using rustdoc
fn get_doc_comments<T>() -> Vec<(String, String)> {
    let type_name = std::any::type_name::<T>();
    let output = std::process::Command::new("rustdoc")
        .args(["--document-private-items", "--output-format=json"])
        .output()
        .expect("Failed to run rustdoc");

    // Parse JSON output to extract doc comments
    // (simplified for example, would need proper JSON parsing)
    vec![] // Placeholder
}

fn add_toml_comments(toml_str: &str, doc_comments: &[(String, String)]) -> String {
    let mut result = String::new();
    for line in toml_str.lines() {
        // If line defines a field, find and add its doc comment
        if let Some(field_name) = line.split('=').next().map(str::trim) {
            if let Some((_, comment)) = doc_comments.iter().find(|(name, _)| name == field_name) {
                result.push_str(&format!("# {}\n", comment));
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

#[derive(Default, Serialize)]
struct ConfigBuilder {
    #[doc = "Control verbosity level of logging"]
    logging: Option<LoggingConfig>,

    #[doc = "Configure dependency handling"]
    dependencies: Option<DependencyConfig>,
}

#[derive(Default, Serialize)]
struct LoggingConfig {
    #[doc = "Default verbosity level (error, warn, info, debug)"]
    default_verbosity: Option<String>,
}

#[derive(Default, Serialize)]
struct DependencyConfig {
    #[doc = "Exclude unstable features from dependencies"]
    exclude_unstable_features: Option<bool>,

    #[doc = "Features that should always be included"]
    always_include_features: Option<Vec<String>>,

    #[doc = "Features that should be globally excluded"]
    global_excluded_features: Option<Vec<String>>,

    #[doc = "Crate-specific feature overrides"]
    feature_overrides: Option<Vec<FeatureOverride>>,
}

#[derive(Default, Serialize)]
struct FeatureOverride {
    #[doc = "Name of the crate to override"]
    crate_name: String,

    #[doc = "Features to exclude for this crate"]
    excluded_features: Vec<String>,

    #[doc = "Features to always include for this crate"]
    required_features: Vec<String>,
}

fn prompt_config() -> Result<ConfigBuilder, Box<dyn std::error::Error>> {
    let mut config = ConfigBuilder::default();

    if Confirm::new("Configure logging?").prompt()? {
        let mut logging = LoggingConfig::default();

        let level = inquire::Select::new(
            "Default verbosity level:",
            vec!["error", "warn", "info", "debug"],
        )
        .prompt()?;
        logging.default_verbosity = Some(level.to_string());

        config.logging = Some(logging);
    }

    if Confirm::new("Configure dependency handling?").prompt()? {
        let mut deps = DependencyConfig::default();

        deps.exclude_unstable_features = Some(Confirm::new("Exclude unstable features?").prompt()?);

        if Confirm::new("Configure always-included features?").prompt()? {
            let features = Text::new("Enter features (comma-separated):").prompt()?;
            deps.always_include_features = Some(
                features
                    .split(',')
                    .map(str::trim)
                    .map(String::from)
                    .collect(),
            );
        }

        // Similar prompts for other fields...

        config.dependencies = Some(deps);
    }

    Ok(config)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Welcome to thag config generator!");

    let config = prompt_config()?;

    // Generate TOML
    let toml_str = toml::to_string_pretty(&config)?;

    // Add doc comments as TOML comments
    let doc_comments = get_doc_comments::<ConfigBuilder>();
    let final_config = add_toml_comments(&toml_str, &doc_comments);

    // Save with backup
    let config_path = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join("thag_rs")
        .join("config.toml");

    if config_path.exists() {
        let backup_path = config_path.with_extension("toml.bak");
        fs::rename(&config_path, &backup_path)?;
        println!("Created backup at {:?}", backup_path);
    }

    fs::create_dir_all(config_path.parent().unwrap())?;
    fs::write(&config_path, final_config)?;

    println!("Configuration saved to {:?}", config_path);
    Ok(())
}
