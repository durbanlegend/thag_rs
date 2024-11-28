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
use inquire::validator::Validation;
use inquire::{Confirm, Select, Text};
use serde::Serialize;
use std::{fs, path::PathBuf};

type Error = CustomUserError;

// Custom validators
#[derive(Clone)]
struct VersionValidator;

impl StringValidator for VersionValidator {
    fn validate(&self, input: &str) -> Result<Validation, Error> {
        match semver::Version::parse(input) {
            Ok(_) => Ok(Validation::Valid),
            Err(_) => Ok(Validation::Invalid(
                "Please enter a valid semver version (e.g., 1.0.0)".into(),
            )),
        }
    }
}

use inquire::validator::StringValidator;

#[derive(Clone)]
struct PathValidator;

impl StringValidator for PathValidator {
    fn validate(&self, input: &str) -> Result<Validation, Error> {
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

    #[doc = "Color settings"]
    colors: Option<Colors>,

    #[doc = "Proc macros configuration"]
    proc_macros: Option<ProcMacros>,

    #[doc = "Miscellaneous settings"]
    misc: Option<Misc>,
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
struct Colors {
    #[doc = "Color support level (auto, always, never)"]
    color_support: Option<String>,
    #[doc = "Terminal theme (dark, light)"]
    term_theme: Option<String>,
}

#[derive(Default, Serialize)]
struct ProcMacros {
    #[doc = "Path to proc macro crate"]
    proc_macro_crate_path: Option<String>,
}

#[derive(Default, Serialize)]
struct Misc {
    #[doc = "Unquote option for string handling"]
    unquote: Option<bool>,
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
                "Colors",
                "Proc Macros",
                "Misc Settings",
                "Preview Configuration",
                "Save and Exit",
                "Cancel",
            ],
        )
        .with_help_message("Use ↑↓ to navigate, Enter to select, Esc to go back")
        .prompt()?;

        match action {
            "Logging" => {
                if let Ok(logging_config) = prompt_logging_config_with_escape() {
                    if let Some(_) = logging_config {
                        // None means user escaped
                        config.logging = logging_config;
                    }
                }
            }
            "Dependencies" => {
                // if let Ok(dependency_config) = prompt_dependency_config().await? {
                //     if let Some(_) = dependency_config {
                //         // None means user escaped
                //         config.dependencies = dependency_config;
                //     }
                // }
                config.dependencies = Some(prompt_dependency_config().await?);
            }
            "Colors" => {
                // config.colors = Some(prompt_colors_config()?);
                if let Ok(colors_config) = prompt_colors_config_with_escape() {
                    if let Some(_) = colors_config {
                        // None means user escaped
                        config.colors = colors_config;
                    }
                }
            }

            "Proc Macros" => {
                // config.proc_macros = Some(prompt_proc_macros_config()?);
                if let Ok(proc_macros_config) = prompt_proc_macros_config_with_escape() {
                    if let Some(_) = proc_macros_config {
                        // None means user escaped
                        config.proc_macros = proc_macros_config;
                    }
                }
            }
            "Misc Settings" => {
                // config.misc = Some(prompt_misc_config()?);
                if let Ok(misc_config) = prompt_misc_config_with_escape() {
                    if let Some(_) = misc_config {
                        // None means user escaped
                        config.misc = misc_config;
                    }
                }
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

fn prompt_logging_config_with_escape() -> Result<Option<LoggingConfig>, Box<dyn std::error::Error>>
{
    let level = Select::new(
        "Default verbosity level:",
        vec!["error", "warn", "info", "debug"],
    )
    .with_help_message("Controls detail level of log messages (Esc to go back)")
    .prompt_skippable()?; // prompt_skippable returns None if user hits Esc

    // If user escaped, return None
    let Some(level) = level else {
        return Ok(None);
    };

    Ok(Some(LoggingConfig {
        default_verbosity: Some(level.to_string()),
    }))
}

fn prompt_colors_config_with_escape() -> Result<Option<Colors>, Box<dyn std::error::Error>> {
    let color_support = Select::new("Color support:", vec!["auto", "always", "never"])
        .with_help_message("When to use colored output (Esc to go back)")
        .prompt_skippable()?;

    let Some(color_support) = color_support else {
        return Ok(None);
    };

    let term_theme = Select::new("Terminal theme:", vec!["dark", "light"])
        .with_help_message("Choose theme based on your terminal background")
        .prompt_skippable()?;

    let Some(term_theme) = term_theme else {
        return Ok(None);
    };

    Ok(Some(Colors {
        color_support: Some(color_support.to_string()),
        term_theme: Some(term_theme.to_string()),
    }))
}

fn prompt_proc_macros_config_with_escape() -> Result<Option<ProcMacros>, Box<dyn std::error::Error>>
{
    let configure = Confirm::new("Configure proc macro path?")
        .with_help_message("Set custom path for proc macro crates")
        .prompt_skippable()?;

    let Some(configure) = configure else {
        return Ok(None);
    };

    if configure {
        let path = Text::new("Proc macro crate path:")
            .with_help_message("Path to directory containing proc macro crates")
            .with_validator(PathValidator)
            .prompt_skippable()?;

        let Some(path) = path else {
            return Ok(None);
        };

        Ok(Some(ProcMacros {
            proc_macro_crate_path: Some(path),
        }))
    } else {
        Ok(Some(ProcMacros::default()))
    }
}

fn prompt_misc_config_with_escape() -> Result<Option<Misc>, Box<dyn std::error::Error>> {
    let unquote = Confirm::new("Enable unquote option?")
        .with_help_message("Controls string literal handling in scripts")
        .prompt_skippable()?;

    let Some(unquote) = unquote else {
        return Ok(None);
    };

    Ok(Some(Misc {
        unquote: Some(unquote),
    }))
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
