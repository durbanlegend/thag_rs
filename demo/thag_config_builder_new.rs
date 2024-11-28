/*[toml]
[dependencies]
colored = "2.1.0"
dirs = "5.0"
inquire = "0.7.5"
semver = "1.0.23"
serde = { version = "1.0.215", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
thag_rs = { path = "/Users/donf/projects/thag_rs/" }
toml = "0.8"
*/

use inquire::error::CustomUserError;
use inquire::validator::{StringValidator, Validation};
use inquire::{Confirm, Select, Text};
// use serde::{Deserialize, Serialize};
// use serde_with::DisplayFromStr;
use std::collections::HashMap;
use std::fmt::Display;
// use std::fs::{create_dir_all, rename};
use std::fs;
use std::path::PathBuf;
use syn::{parse_file, Attribute, Item, Meta};
use thag_rs::{
    maybe_config, ColorSupport, Colors, Config, Dependencies, FeatureOverride, Logging, Misc,
    ProcMacros, TermTheme, Verbosity,
};

type Error = CustomUserError;

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

struct ConfigBuilder {
    system_defaults: Config,
    user_config: Option<Config>,
    current: Config,
}

impl ConfigBuilder {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let system_defaults = Config::default();
        let user_config = maybe_config();
        let current = user_config.clone().unwrap_or_else(Config::default);

        Ok(Self {
            system_defaults,
            user_config,
            current,
        })
    }

    fn preview(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Get doc comments from Config and its fields
        let doc_comments = get_doc_comments::<Config>();
        let toml_str = toml::to_string_pretty(&self.current)?;
        Ok(add_doc_comments(&toml_str, &doc_comments))
    }
}

// Helper trait for DisplayFromStr types
#[allow(dead_code)]
trait PromptableEnum: Sized + Display + Clone {
    fn variants() -> Vec<Self>;
    fn display_name(&self) -> &'static str;
}

impl PromptableEnum for Verbosity {
    fn variants() -> Vec<Self> {
        vec![
            Self::Quieter,
            Self::Quiet,
            Self::Normal,
            Self::Verbose,
            Self::Debug,
        ]
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::Quieter => "quieter",
            Self::Quiet => "quiet",
            Self::Normal => "normal",
            Self::Verbose => "verbose",
            Self::Debug => "debug",
        }
    }
}

impl PromptableEnum for ColorSupport {
    fn variants() -> Vec<Self> {
        vec![Self::Xterm256, Self::Ansi16, Self::None, Self::Default]
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::Xterm256 => "xterm256",
            Self::Ansi16 => "ansi16",
            Self::None => "none",
            Self::Default => "default",
        }
    }
}

impl PromptableEnum for TermTheme {
    fn variants() -> Vec<Self> {
        vec![Self::Light, Self::Dark, Self::None]
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::None => "none",
        }
    }
}

// Generic prompt function for DisplayFromStr types
fn prompt_enum<T: PromptableEnum>(
    prompt: &str,
    help: &str,
    _current: &T,
) -> Result<Option<T>, Box<dyn std::error::Error>> {
    let variants = T::variants();

    Select::new(prompt, variants)
        .with_help_message(help)
        .prompt_skippable()
        .map_err(Into::into)
}

fn get_doc_comments<T>() -> Vec<(String, String)> {
    let type_name = std::any::type_name::<T>();
    let source_path = find_source_file(type_name).expect("Could not find source file");
    let source = fs::read_to_string(source_path).expect("Could not read source file");

    let syntax = parse_file(&source).expect("Could not parse source file");

    let mut comments = Vec::new();
    extract_doc_comments(&syntax.items, "", &mut comments);

    comments
}

fn find_source_file(type_name: &str) -> Option<PathBuf> {
    // For now, assume the Config is in the current project
    // You might want to make this more sophisticated
    let project_root = std::env::current_dir().ok()?;
    let src_dir = project_root.join("src");

    // Try common locations
    let possible_paths = vec![
        src_dir.join("config.rs"),
        src_dir.join("lib.rs"),
        src_dir.join("main.rs"),
    ];

    possible_paths.into_iter().find(|p| p.exists())
}

fn extract_doc_comments(items: &[Item], prefix: &str, comments: &mut Vec<(String, String)>) {
    for item in items {
        match item {
            Item::Struct(item_struct) => {
                let struct_name = item_struct.ident.to_string();
                let struct_docs = extract_attrs_docs(&item_struct.attrs);
                if !struct_docs.is_empty() {
                    comments.push((format!("{}{}", prefix, struct_name), struct_docs.join("\n")));
                }

                // Process struct fields
                for field in &item_struct.fields {
                    if let Some(ident) = &field.ident {
                        let field_docs = extract_attrs_docs(&field.attrs);
                        if !field_docs.is_empty() {
                            comments.push((
                                format!("{}{}.{}", prefix, struct_name, ident),
                                field_docs.join("\n"),
                            ));
                        }
                    }
                }
            }
            Item::Enum(item_enum) => {
                let enum_name = item_enum.ident.to_string();
                let enum_docs = extract_attrs_docs(&item_enum.attrs);
                if !enum_docs.is_empty() {
                    comments.push((format!("{}{}", prefix, enum_name), enum_docs.join("\n")));
                }

                // Process enum variants
                for variant in &item_enum.variants {
                    let variant_docs = extract_attrs_docs(&variant.attrs);
                    if !variant_docs.is_empty() {
                        comments.push((
                            format!("{}{}.{}", prefix, enum_name, variant.ident),
                            variant_docs.join("\n"),
                        ));
                    }
                }
            }
            // Add other item types as needed
            _ => {}
        }
    }
}

fn extract_attrs_docs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &meta.value
                    {
                        return Some(s.value().trim().to_string());
                    }
                }
            }
            None
        })
        .collect()
}

fn add_doc_comments(toml_str: &str, doc_comments: &[(String, String)]) -> String {
    let mut result = String::from("# Generated by thag_config_builder\n\n");

    for line in toml_str.lines() {
        // If line defines a field, find and add its doc comment
        if let Some(field_path) = extract_field_path(line) {
            if let Some((_, comment)) = doc_comments.iter().find(|(path, _)| path == &field_path) {
                // Add doc comment before the field
                for comment_line in comment.lines() {
                    result.push_str(&format!("# {}\n", comment_line));
                }
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    result
}

fn extract_field_path(line: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    // Handle section headers [section]
    if line.starts_with('[') && line.ends_with(']') {
        return Some(line[1..line.len() - 1].to_string());
    }

    // Handle field definitions field = value
    if let Some(idx) = line.find('=') {
        return Some(line[..idx].trim().to_string());
    }

    None
}

fn prompt_verbosity(current: &Verbosity) -> Result<Option<Verbosity>, Box<dyn std::error::Error>> {
    prompt_enum(
        "Verbosity level:",
        "quieter (0) < quiet (1) < normal (2) < verbose (3) < debug (4)",
        current,
    )
}

fn prompt_color_support(
    current: &ColorSupport,
) -> Result<Option<ColorSupport>, Box<dyn std::error::Error>> {
    prompt_enum(
        "Color support:",
        "xterm256 (full color) / ansi16 (basic) / none (disabled) / default (auto-detect)",
        current,
    )
}

fn prompt_term_theme(current: &TermTheme) -> Result<Option<TermTheme>, Box<dyn std::error::Error>> {
    prompt_enum(
        "Terminal theme:",
        "light/dark background, or none for no adjustment",
        current,
    )
}

fn prompt_logging_config(current: &Logging) -> Result<Option<Logging>, Box<dyn std::error::Error>> {
    let verbosity = prompt_enum(
        "Default verbosity level:",
        "Set the default logging detail level",
        &current.default_verbosity,
    )?;

    // If user escaped, return None
    let Some(default_verbosity) = verbosity else {
        return Ok(None);
    };

    Ok(Some(Logging { default_verbosity }))
}

fn prompt_colors_config(current: &Colors) -> Result<Option<Colors>, Box<dyn std::error::Error>> {
    let color_support = prompt_enum(
        "Color support:",
        "Configure color output support",
        &current.color_support,
    )?;

    let Some(color_support) = color_support else {
        return Ok(None);
    };

    let term_theme = prompt_enum(
        "Terminal theme:",
        "Select theme based on your terminal background",
        &current.term_theme,
    )?;

    let Some(term_theme) = term_theme else {
        return Ok(None);
    };

    Ok(Some(Colors {
        color_support,
        term_theme,
    }))
}

fn prompt_dependencies_config(
    current: &Dependencies,
) -> Result<Option<Dependencies>, Box<dyn std::error::Error>> {
    let mut config = current.clone();

    config.exclude_unstable_features = Confirm::new("Exclude unstable features?")
        .with_default(current.exclude_unstable_features)
        .prompt_skippable()?
        .unwrap_or(current.exclude_unstable_features);

    config.exclude_std_feature = Confirm::new("Exclude std feature?")
        .with_default(current.exclude_std_feature)
        .prompt_skippable()?
        .unwrap_or(current.exclude_std_feature);

    if Confirm::new("Configure feature overrides?").prompt()? {
        config.feature_overrides = HashMap::new(); // Start fresh if user wants to configure
        while if config.feature_overrides.is_empty() {
            true
        } else {
            Confirm::new("Add another crate override?").prompt()?
        } {
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

            let alternative = Text::new("Alternative features (comma-separated):").prompt()?;
            let alternative_features = alternative
                .split(',')
                .map(str::trim)
                .map(String::from)
                .collect();

            config.feature_overrides.insert(
                crate_name,
                FeatureOverride {
                    excluded_features,
                    required_features,
                    alternative_features,
                },
            );
        }
    }

    Ok(Some(config))
}

fn prompt_misc_config(current: &Misc) -> Result<Option<Misc>, Box<dyn std::error::Error>> {
    let unquote = Confirm::new("Enable unquote option?")
        .with_default(current.unquote)
        .prompt_skippable()?;

    let Some(unquote) = unquote else {
        return Ok(None);
    };

    Ok(Some(Misc { unquote }))
}

fn prompt_proc_macros_config(
    _current: &ProcMacros,
) -> Result<Option<ProcMacros>, Box<dyn std::error::Error>> {
    let path = if Confirm::new("Configure proc macro path?").prompt()? {
        let input = Text::new("Proc macro crate path:")
            .with_help_message("Path to directory containing proc macro crates")
            .with_validator(PathValidator)
            .prompt_skippable()?;

        input
    } else {
        None
    };

    Ok(Some(ProcMacros {
        proc_macro_crate_path: path,
    }))
}

use colored::Colorize;

fn prompt_config() -> Result<Config, Box<dyn std::error::Error>> {
    let builder = ConfigBuilder::new()?;
    let mut config = builder.current.clone();

    loop {
        let action = Select::new(
            "Configure:",
            vec![
                "Logging",
                "Colors",
                "Dependencies",
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
                if let Some(new_config) = prompt_logging_config(&config.logging)? {
                    config.logging = new_config;
                }
            }
            "Colors" => {
                if let Some(new_config) = prompt_colors_config(&config.colors)? {
                    config.colors = new_config;
                }
            }
            "Dependencies" => {
                if let Some(new_config) = prompt_dependencies_config(&config.dependencies)? {
                    config.dependencies = new_config;
                }
            }
            "Proc Macros" => {
                if let Some(new_config) = prompt_proc_macros_config(&config.proc_macros)? {
                    config.proc_macros = new_config;
                }
            }
            "Misc Settings" => {
                if let Some(new_config) = prompt_misc_config(&config.misc)? {
                    config.misc = new_config;
                }
            }
            "Preview Configuration" => {
                let preview = ConfigBuilder {
                    system_defaults: builder.system_defaults.clone(),
                    user_config: builder.user_config.clone(),
                    current: config.clone(),
                }
                .preview()?;
                println!("\n{}", preview);
            }
            "Save and Exit" => {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Welcome to thag config builder!".bold());

    let config = prompt_config()?;

    let config_path = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join("thag_rs")
        .join("config.toml");

    // Show final preview
    let builder = ConfigBuilder {
        system_defaults: Config::default(),
        user_config: maybe_config(),
        current: config.clone(),
    };

    println!("{}", builder.preview()?);

    if Confirm::new("Save this configuration?").prompt()? {
        // Create backup if exists
        if config_path.exists() {
            let backup_path = config_path.with_extension("toml.bak");
            fs::rename(&config_path, &backup_path)?;
            println!("{}", format!("Created backup at {:?}", backup_path).blue());
        }

        fs::create_dir_all(config_path.parent().unwrap())?;
        fs::write(&config_path, builder.preview()?)?;
        println!(
            "{}",
            format!("Configuration saved to {:?}", config_path).green()
        );
    } else {
        println!("Configuration not saved.");
    }

    Ok(())
}
