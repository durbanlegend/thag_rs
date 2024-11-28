/*[toml]
[dependencies]
colored = "2.1.0"
dirs = "5.0"
inquire = "0.7.5"
semver = "1.0.23"
serde = { version = "1.0.215", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
toml = "0.8"
*/

/// Prompted config file builder for `thag`, intended to be saved as a command with `-x`.
//# Purpose: Handy configuration file builder.
//# Categories: crates, technique, tools
use colored::Colorize;
use inquire::error::CustomUserError;
use inquire::validator::{CustomTypeValidator, Validation};
use inquire::{Confirm, Select, Text};
use serde::Serialize;
use std::{fs, path::PathBuf};

// Custom validators
#[derive(Clone)]
struct VersionValidator;

type Error = CustomUserError;

impl CustomTypeValidator<String> for VersionValidator {
    fn validate(&self, input: &String) -> Result<Validation, Error> {
        match semver::Version::parse(input) {
            Ok(_) => Ok(Validation::Valid),
            Err(_) => Ok(Validation::Invalid(
                "Please enter a valid semver version (e.g., 1.0.0)".into(),
            )),
        }
    }
}

#[derive(Clone)]
struct PathValidator;

impl CustomTypeValidator<String> for PathValidator {
    fn validate(&self, input: &String) -> Result<Validation, Error> {
        let path = PathBuf::from(input);
        if path.exists() {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Path does not exist".into()))
        }
    }
}

#[derive(Serialize, Default)]
struct ConfigBuilder {
    #[doc = "Logging configuration"]
    logging: Option<LoggingConfig>,

    #[doc = "Dependency handling settings"]
    dependencies: Option<DependencyConfig>,
}

impl ConfigBuilder {
    fn preview(&self) -> Result<String, Box<dyn std::error::Error>> {
        let toml_str = toml::to_string_pretty(&self)?;
        Ok(format!(
            "\nPreview of config.toml:\n{}\n{}\n{}\n",
            "=".repeat(40).blue(),
            toml_str.green(),
            "=".repeat(40).blue()
        ))
    }

    fn validate(&self) -> Result<(), String> {
        if let Some(deps) = &self.dependencies {
            // Check for conflicting settings
            if let Some(ref features) = deps.always_include_features {
                if let Some(ref excluded) = deps.global_excluded_features {
                    let conflicts: Vec<_> =
                        features.iter().filter(|f| excluded.contains(*f)).collect();
                    if !conflicts.is_empty() {
                        return Err(format!(
                            "Features cannot be both always included and excluded: {:?}",
                            conflicts
                        ));
                    }
                }
            }
        }
        Ok(())
    }
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

async fn prompt_feature_override() -> Result<(String, FeatureOverride), Box<dyn std::error::Error>>
{
    let crate_name = Text::new("Crate name:")
        .with_help_message("Enter the name of the crate to override")
        .prompt()?;

    let excluded = Text::new("Excluded features (comma-separated):").prompt()?;
    let excluded_features = excluded
        .split(',')
        .map(str::trim)
        .map(String::from)
        .collect::<Vec<_>>();

    let required = Text::new("Required features (comma-separated):").prompt()?;
    let required_features = required
        .split(',')
        .map(str::trim)
        .map(String::from)
        .collect();

    Ok((
        crate_name.clone(),
        FeatureOverride {
            crate_name,
            excluded_features,
            required_features,
        },
    ))
}

async fn prompt_config() -> Result<ConfigBuilder, Box<dyn std::error::Error>> {
    let mut config = ConfigBuilder::default();

    loop {
        let action = Select::new(
            "Configure:",
            vec![
                "Logging",
                "Dependencies",
                "Preview Configuration",
                "Save and Exit",
                "Cancel",
            ],
        )
        .prompt()?;

        match action {
            "Logging" => {
                config.logging = Some(prompt_logging_config()?);
            }
            "Dependencies" => {
                config.dependencies = Some(prompt_dependency_config().await?);
            }
            "Preview Configuration" => {
                println!("{}", config.preview()?);
            }
            "Save and Exit" => {
                if let Err(e) = config.validate() {
                    println!("{}", format!("Configuration Error: {}", e).red());
                    continue;
                }
                break;
            }
            "Cancel" => {
                return Err("Configuration cancelled".into());
            }
            _ => unreachable!(),
        }
    }

    Ok(config)
}

fn prompt_logging_config() -> Result<LoggingConfig, Box<dyn std::error::Error>> {
    let mut config = LoggingConfig::default();

    let level = Select::new(
        "Default verbosity level:",
        vec!["error", "warn", "info", "debug"],
    )
    .prompt()?;

    config.default_verbosity = Some(level.to_string());

    Ok(config)
}

async fn prompt_dependency_config() -> Result<DependencyConfig, Box<dyn std::error::Error>> {
    let mut config = DependencyConfig::default();

    config.exclude_unstable_features = Some(Confirm::new("Exclude unstable features?").prompt()?);

    if Confirm::new("Configure always-included features?").prompt()? {
        let features = Text::new("Enter features (comma-separated):").prompt()?;
        config.always_include_features = Some(
            features
                .split(',')
                .map(str::trim)
                .map(String::from)
                .collect(),
        );
    }

    if Confirm::new("Configure globally excluded features?").prompt()? {
        let features = Text::new("Enter features to exclude (comma-separated):").prompt()?;
        config.global_excluded_features = Some(
            features
                .split(',')
                .map(str::trim)
                .map(String::from)
                .collect(),
        );
    }

    if Confirm::new("Add crate-specific feature overrides?").prompt()? {
        let mut overrides = Vec::new();
        while Confirm::new("Add another crate override?").prompt()? {
            let crate_name = Text::new("Crate name:").prompt()?;

            let excluded = Text::new("Excluded features (comma-separated):").prompt()?;
            let excluded_features = excluded
                .split(',')
                .map(str::trim)
                .map(String::from)
                .collect();

            let required = Text::new("Required features (comma-separated):").prompt()?;
            let required_features = required
                .split(',')
                .map(str::trim)
                .map(String::from)
                .collect();

            overrides.push(FeatureOverride {
                crate_name,
                excluded_features,
                required_features,
            });
        }
        if !overrides.is_empty() {
            config.feature_overrides = Some(overrides);
        }
    }

    Ok(config)
}

fn get_doc_comments<T>() -> Vec<(String, String)> {
    // For now, just return empty vec
    // We'll implement proper doc extraction later
    Vec::new()
}

fn add_toml_comments(toml_str: &str, _doc_comments: &[(String, String)]) -> String {
    // For now, just add some basic comments
    format!("# Generated by thag_config_builder\n\n{}", toml_str)
}

fn save_config(config: &ConfigBuilder, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Create backup if exists
    if path.exists() {
        let backup_path = path.with_extension("toml.bak");
        fs::rename(path, &backup_path)?;
        println!("{}", format!("Created backup at {:?}", backup_path).blue());
    }

    // Generate TOML with comments
    let toml_str = toml::to_string_pretty(&config)?;
    let doc_comments = get_doc_comments::<ConfigBuilder>();
    let final_config = add_toml_comments(&toml_str, &doc_comments);

    // Save
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, final_config)?;

    println!("{}", format!("Configuration saved to {:?}", path).green());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Welcome to thag config generator!".bold());

    let config = prompt_config().await?;

    let config_path = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join("thag_rs")
        .join("config.toml");

    // Show final preview
    println!("{}", config.preview()?);
    if Confirm::new("Save this configuration?").prompt()? {
        save_config(&config, &config_path)?;
    } else {
        println!("Configuration not saved.");
    }

    Ok(())
}
