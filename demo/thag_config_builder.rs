/*[toml]
[dependencies]
colored = "2.1.0"
convert_case = "0.6.0"
dirs = "5.0"
documented = "0.9.1"
inquire = "0.7.5"
semver = "1.0.23"
serde = { version = "1.0.215", features = ["derive"] }
strum = { version = "0.26.3", features = ["derive"] }
syn = { version = "2.0.90", features = ["full"] }
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", rev = "1d665b6c1dca651fc80e49e7bf4f2f96980e6468" }
# thag_rs = { path = "/Users/donf/projects/thag_rs/" }
tokio = { version = "1", features = ["full"] }
toml = "0.8"
*/

/// Prompted config file builder for `thag`, intended to be saved as a command with `-x`.
//# Purpose: Handy configuration file builder.
//# Categories: crates, technique, tools
// use convert_case::{Case, Casing};
use convert_case::{Converter, Pattern};
use documented::{Documented, DocumentedFields, DocumentedVariants};
use inquire::error::CustomUserError;
use inquire::validator::{StringValidator, Validation};
use inquire::{Confirm, Select, Text};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use strum::IntoEnumIterator;
use syn::{parse_file, Attribute, Item, ItemUse, Meta, /*Path as SynPath,*/ UseTree};
use thag_rs::config::DependencyInference;
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
    fn new() -> Self {
        let system_defaults = Config::default();
        let user_config = maybe_config();
        let current = user_config.clone().unwrap_or_default();

        Self {
            system_defaults,
            user_config,
            current,
        }
    }

    fn preview(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Get doc comments from Config and its fields
        let doc_comments = get_doc_comments::<Config>();
        let toml_str = toml::to_string_pretty(&self.current)?;
        Ok(add_doc_comments(&toml_str, doc_comments))
    }
}

// Helper trait for DisplayFromStr types
trait PromptableEnum:
    Sized + Display + Clone + IntoEnumIterator + DocumentedVariants + Into<&'static str>
// where
//     &str: for<'a> From<&'a Self>,
{
    fn variants() -> Vec<Self> {
        Self::iter().collect()
    }

    fn display_name(&self) -> &'static str {
        self.clone().into()
    }

    fn get_docs() -> Vec<(&'static str, &'static str)> {
        Self::iter()
            .map(|variant| (variant.display_name(), variant.get_variant_docs()))
            .collect()
    }
}

impl PromptableEnum for Verbosity {}
impl PromptableEnum for ColorSupport {}
impl PromptableEnum for TermTheme {}

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

struct ModuleInfo {
    items: Vec<Item>,
    uses: Vec<(String, String)>, // (name, path)
}

fn collect_modules(project_root: &Path) -> HashMap<String, ModuleInfo> {
    let mut modules = HashMap::new();

    // Start with main modules
    for entry in &["config.rs", "logging.rs", "colors.rs"] {
        let path = project_root.join("src").join(entry);
        if path.exists() {
            if let Ok(source) = fs::read_to_string(&path) {
                if let Ok(syntax) = parse_file(&source) {
                    let module_name = entry.trim_end_matches(".rs").to_string();
                    let mut uses = Vec::new();

                    // Collect use declarations
                    for item in &syntax.items {
                        if let Item::Use(use_item) = item {
                            if let Some((name, path)) = extract_use_path(use_item) {
                                uses.push((name, path));
                            }
                        }
                    }

                    modules.insert(
                        module_name,
                        ModuleInfo {
                            items: syntax.items,
                            uses,
                        },
                    );
                }
            }
        }
    }

    modules
}

fn extract_use_path(use_item: &ItemUse) -> Option<(String, String)> {
    fn process_use_tree(tree: &UseTree, base_path: &str) -> Vec<(String, String)> {
        match tree {
            // Simple path like "use crate::logging::Verbosity"
            UseTree::Path(use_path) => {
                let new_base = if base_path.is_empty() {
                    use_path.ident.to_string()
                } else {
                    format!("{base_path}::{}", use_path.ident)
                };
                process_use_tree(&use_path.tree, &new_base)
            }

            // Named item like "use crate::logging::Verbosity as VerbosityLevel"
            UseTree::Rename(rename) => {
                vec![(rename.rename.to_string(), base_path.to_string())]
            }

            // Simple name like the "Verbosity" in "use crate::logging::Verbosity"
            UseTree::Name(name) => {
                vec![(name.ident.to_string(), base_path.to_string())]
            }

            // Group like "use crate::logging::{Verbosity, ColorSupport}"
            UseTree::Group(group) => group
                .items
                .iter()
                .flat_map(|tree| process_use_tree(tree, base_path))
                .collect(),

            // Global import like "use *"
            UseTree::Glob(_) => Vec::new(),
        }
    }

    // Get the full path
    let full_path = use_item.tree.clone();
    let results = process_use_tree(&full_path, "");

    // We're primarily interested in enum imports
    results.into_iter().find(|(name, _)| {
        // Basic heuristic: enum names typically start with uppercase
        !name.is_empty() && name.chars().next().unwrap().is_uppercase()
    })
}

fn get_doc_comments<T>() -> Vec<(String, String)> {
    let _type_name = std::any::type_name::<T>();
    let project_root = std::env::current_dir().expect("Could not get current directory");

    let modules = collect_modules(&project_root);
    let mut comments = Vec::new();

    // Process config.rs first
    if let Some(config_module) = modules.get("config") {
        extract_doc_comments(&config_module.items, "", &mut comments);

        // For each field that's an enum type, look up its module
        for (field_path, type_name) in find_enum_fields(&config_module.items) {
            if let Some(module_name) = find_enum_module(&type_name, &config_module.uses, &modules) {
                if let Some(module_info) = modules.get(&module_name) {
                    // Find and extract enum documentation
                    if let Some(enum_docs) = extract_enum_docs(&module_info.items, &type_name) {
                        let key = format!("{field_path}_type").to_lowercase();
                        eprintln!("Pushing ({key}, {enum_docs} to comments");
                        comments.push((key, enum_docs));
                    }
                }
            }
        }
    }

    eprintln!("Found {} doc comments:", comments.len());
    for (path, comment) in &comments {
        println!("  {path}: {comment}");
    }

    comments
}

fn find_enum_fields(items: &[Item]) -> Vec<(String, String)> {
    let mut fields = Vec::new();

    for item in items {
        if let Item::Struct(struct_item) = item {
            let struct_name = struct_item.ident.to_string();

            for field in &struct_item.fields {
                if let Some(field_name) = &field.ident {
                    if let syn::Type::Path(type_path) = &field.ty {
                        if let Some(last_seg) = type_path.path.segments.last() {
                            fields.push((
                                format!("{struct_name}.{field_name}"),
                                last_seg.ident.to_string(),
                            ));
                        }
                    }
                }
            }
        }
    }

    fields
}

fn find_enum_module(
    type_name: &str,
    uses: &[(String, String)],
    modules: &HashMap<String, ModuleInfo>,
) -> Option<String> {
    // Find which module contains the enum definition
    // First check use declarations
    for (name, path) in uses {
        if name == type_name {
            // Extract module name from path
            return path.split("::").nth(1).map(String::from);
        }
    }

    // Then check each module directly
    for (module_name, module_info) in modules {
        if contains_enum(&module_info.items, type_name) {
            return Some(module_name.clone());
        }
    }

    None
}

fn contains_enum(items: &[Item], enum_name: &str) -> bool {
    items.iter().any(|item| {
        if let Item::Enum(enum_item) = item {
            enum_item.ident == enum_name
        } else {
            false
        }
    })
}

fn extract_enum_docs(items: &[Item], enum_name: &str) -> Option<String> {
    for item in items {
        if let Item::Enum(enum_item) = item {
            if enum_item.ident == enum_name {
                let mut docs = extract_attrs_docs(&enum_item.attrs);
                docs.push("\nAvailable options:".to_string());

                for variant in &enum_item.variants {
                    let variant_docs = extract_attrs_docs(&variant.attrs);
                    let x = if variant_docs.is_empty() {
                        "No documentation"
                    } else {
                        &variant_docs.join("\n    ")
                    };
                    docs.push(format!("  {} - {x}", variant.ident));
                }

                return Some(docs.join("\n"));
            }
        }
    }

    None
}

fn extract_doc_comments(items: &[Item], prefix: &str, comments: &mut Vec<(String, String)>) {
    for item in items {
        match item {
            Item::Struct(item_struct) => {
                let struct_name = item_struct.ident.to_string();
                eprintln!("Found item struct={struct_name}");
                let struct_docs = extract_attrs_docs(&item_struct.attrs);
                eprintln!("struct_docs={struct_docs:#?}");
                if !struct_docs.is_empty() {
                    comments.push((
                        format!("{prefix}{struct_name}").to_lowercase(),
                        struct_docs.join("\n"),
                    ));
                }

                // Process struct fields
                for field in &item_struct.fields {
                    if let Some(ident) = &field.ident {
                        // Get field docs
                        let field_docs = extract_attrs_docs(&field.attrs);
                        eprintln!("field_docs={field_docs:#?}");
                        if !field_docs.is_empty() {
                            comments.push((
                                format!("{prefix}{struct_name}.{ident}").to_lowercase(),
                                field_docs.join("\n"),
                            ));
                        }

                        // Try to get field type docs (for enums)
                        if let syn::Type::Path(type_path) = &field.ty {
                            if let Some(last_seg) = type_path.path.segments.last() {
                                let type_name = last_seg.ident.to_string();
                                comments.push((
                                    format!("{prefix}{struct_name}.{ident}_type").to_lowercase(),
                                    type_name,
                                ));
                            }
                        }
                    }
                }
            }
            Item::Enum(item_enum) => {
                let enum_name = item_enum.ident.to_string();
                eprintln!("Found item enum={enum_name}");

                // Get enum-level docs
                let mut enum_docs = extract_attrs_docs(&item_enum.attrs);

                // Add variant documentation
                enum_docs.push("\nAvailable options:".to_string());
                for variant in &item_enum.variants {
                    let variant_docs = extract_attrs_docs(&variant.attrs);
                    let variant_name = variant.ident.to_string();
                    if variant_docs.is_empty() {
                        enum_docs.push(format!("  {variant_name} - No documentation"));
                    } else {
                        enum_docs.push(format!(
                            "  {variant_name} - {}",
                            variant_docs.join("\n    ")
                        ));
                    }
                }

                comments.push((enum_name.to_lowercase(), enum_docs.join("\n")));
            }
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
                        let attr_doc = s.value().trim().to_string();
                        eprintln!("attr_doc={attr_doc}");
                        return Some(attr_doc);
                    }
                }
            }
            None
        })
        .collect()
}

fn add_enum_docs<T: PromptableEnum>(result: &mut String, field_name: &str) {
    result.push_str(&format!("\n# Available options for {field_name}:\n"));
    for (name, docs) in T::get_docs() {
        result.push_str(&format!("#   {name} - {docs}\n"));
    }
    result.push('\n');
}

fn add_doc_comments(toml_str: &str, doc_comments: Vec<(String, String)>) -> String {
    let mut result = String::from("# Generated by thag_config_builder\n\n");

    // Add Config documentation
    result.push_str(&format!("# {}\n\n", Config::DOCS));

    let comments_hash = &doc_comments
        .into_iter()
        .collect::<HashMap<String, String>>();

    let mut section = String::new();
    for line in toml_str.lines() {
        let trimmed = line.trim();

        // Add section documentation
        if trimmed.starts_with('[') {
            section = trimmed.trim_matches(|c| c == '[' || c == ']').to_string();
            match section.as_str() {
                "logging" => result.push_str(&format!("# {}\n", Logging::DOCS)),
                "colors" => result.push_str(&format!("# {}\n", Colors::DOCS)),
                "dependencies" => result.push_str(&format!("# {}\n", Dependencies::DOCS)),
                "proc_macros" => result.push_str(&format!("# {}\n", ProcMacros::DOCS)),
                "misc" => result.push_str(&format!("# {}\n", Misc::DOCS)),
                _ => {}
            }
        }

        // Add enum documentation before relevant fields
        match trimmed {
            s if s.starts_with("default_verbosity =") => {
                add_enum_docs::<Verbosity>(&mut result, "Verbosity");
            }
            s if s.starts_with("color_support =") => {
                add_enum_docs::<ColorSupport>(&mut result, "ColorSupport");
            }
            s if s.starts_with("term_theme =") => {
                add_enum_docs::<TermTheme>(&mut result, "TermTheme");
            }
            s => {
                let maybe_setting = &s.split_once(' ');
                if let Some((setting, _)) = maybe_setting {
                    let key = format!("{section}.{setting}");
                    eprintln!("Trying to match key {key}");
                    let maybe_desc = comments_hash.get(&key);
                    if let Some(desc) = maybe_desc {
                        result.push_str(&format!("\n# {desc}\n"));
                    }
                }
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    result
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

#[allow(clippy::too_many_lines)]
fn prompt_dependencies_config(
    current: &Dependencies,
) -> Result<Option<Dependencies>, Box<dyn std::error::Error>> {
    let mut config = current.clone();

    loop {
        let options = vec![
            "Show Current Settings",
            "Inference Level",      // New option
            "Exclude Pre-releases", // New option
            "Unstable Features",
            "Std Feature",
            "Global Exclusions",
            "Always Included Features",
            "Feature Overrides",
            "Save and Return",
            "Cancel",
        ];

        let action = Select::new("Configure Dependencies:", options)
            .with_help_message("Use ↑↓ to navigate, Enter to select, Esc to go back")
            .prompt_skippable()?;

        let Some(action) = action else {
            return Ok(None);
        };

        match action {
            "Show Current Settings" => {
                println!("\nCurrent Settings:");
                let max_len = Dependencies::FIELD_NAMES
                    .iter()
                    .map(|k| k.len())
                    .max()
                    .unwrap_or(0);

                for field_name in Dependencies::FIELD_NAMES {
                    let doc = Dependencies::get_field_docs(field_name)
                        .unwrap_or("No documentation available");
                    match *field_name {
                        "inference_level" => {
                            println!("  Inference level: {:?} ({doc})", config.inference_level);
                        }
                        "exclude_prerelease" => println!(
                            "  Exclude pre-releases: {} ({doc})",
                            config.exclude_prerelease
                        ),
                        _ => {
                            let conv = Converter::new()
                                .set_delim(" ")
                                .set_pattern(Pattern::Sentence);
                            println!(
                                "  {:<width$}: {doc}",
                                conv.convert(field_name),
                                width = max_len + 2
                            );
                        }
                    }
                }
                println!();
            }
            "Inference Level" => {
                let options = vec![
                    DependencyInference::None,
                    DependencyInference::Minimal,
                    DependencyInference::Config,
                    DependencyInference::Maximal,
                ];

                // Find index of current value
                let current_index = options
                    .iter()
                    .position(|x| x == &config.inference_level)
                    .unwrap_or(2); // Default to Config if not found

                let level = Select::new("Dependency inference level:", options)
                    .with_starting_cursor(current_index)
                    .with_help_message(
                        Dependencies::get_field_docs("inference_level")
                            .unwrap_or("No documentation available"),
                    )
                    .prompt_skippable()?;

                if let Some(level) = level {
                    config.inference_level = level;
                }
            }
            "Exclude Pre-releases" => {
                if let Some(value) = Confirm::new("Exclude pre-release versions?")
                    .with_default(config.exclude_prerelease)
                    .with_help_message(
                        Dependencies::get_field_docs("exclude_prerelease")
                            .unwrap_or("No documentation available"),
                    )
                    .prompt_skippable()?
                {
                    config.exclude_prerelease = value;
                }
            }
            "Unstable Features" => {
                if let Some(value) = Confirm::new("Exclude unstable features?")
                    .with_default(config.exclude_unstable_features)
                    .with_help_message(
                        Dependencies::get_field_docs("exclude_unstable_features")
                            .unwrap_or("No documentation available"),
                    )
                    .prompt_skippable()?
                {
                    config.exclude_unstable_features = value;
                }
            }
            "Std Feature" => {
                if let Some(value) = Confirm::new("Exclude std feature?")
                    .with_default(config.exclude_std_feature)
                    .with_help_message(
                        Dependencies::get_field_docs("exclude_std_feature")
                            .unwrap_or("No documentation available"),
                    )
                    .prompt_skippable()?
                {
                    config.exclude_std_feature = value;
                }
            }
            "Global Exclusions" => {
                let doc = Dependencies::get_field_docs("global_excluded_features")
                    .unwrap_or("No documentation available");
                let current = config.global_excluded_features.join(", ");

                if let Some(input) = Text::new("Enter global exclusions (comma-separated):")
                    .with_default(&current)
                    .with_help_message(doc)
                    .prompt_skippable()?
                {
                    config.global_excluded_features = input
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(String::from)
                        .collect();
                }
            }
            "Always Included Features" => {
                let doc = Dependencies::get_field_docs("always_include_features")
                    .unwrap_or("No documentation available");
                let current = config.always_include_features.join(", ");

                if let Some(input) =
                    Text::new("Enter features to always include (comma-separated):")
                        .with_default(&current)
                        .with_help_message(doc)
                        .prompt_skippable()?
                {
                    config.always_include_features = if input.trim().is_empty() {
                        vec!["derive".to_string()]
                    } else {
                        input
                            .split(',')
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(String::from)
                            .collect()
                    };
                }
            }
            "Feature Overrides" => {
                if let Some(overrides) = prompt_feature_overrides()? {
                    config.feature_overrides = overrides;
                }
            }
            "Save and Return" => {
                return Ok(Some(config));
            }
            "Cancel" => {
                return Ok(None);
            }
            _ => unreachable!(),
        }
    }
}

fn prompt_feature_overrides(
) -> Result<Option<HashMap<String, FeatureOverride>>, Box<dyn std::error::Error>> {
    let mut overrides = HashMap::new();

    while Confirm::new("Add crate override?")
        .with_help_message("Press Esc to finish")
        .prompt_skippable()?
        .unwrap_or(false)
    {
        let Some(crate_name) = Text::new("Crate name:").prompt_skippable()? else {
            return Ok(None);
        };

        let default_features = match Confirm::new("Use default features?")
            .with_help_message("Include crate's default feature set")
            .prompt_skippable()?
        {
            Some(df) => Some(df),
            None => return Ok(None),
        };

        let excluded = match Text::new("Excluded features (comma-separated):")
            .with_help_message("Features to exclude for this crate")
            .prompt_skippable()?
        {
            Some(ex) => Some(
                ex.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect(),
            ),
            None => return Ok(None),
        };

        let required = match Text::new("Required features (comma-separated):")
            .with_help_message("Features to always include for this crate")
            .prompt_skippable()?
        {
            Some(req) => Some(
                req.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect(),
            ),
            None => return Ok(None),
        };

        overrides.insert(
            crate_name,
            FeatureOverride {
                excluded_features: excluded,
                required_features: required,
                default_features,
            },
        );
    }

    Ok(Some(overrides))
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
    let path = if Confirm::new("Configure demo proc macro path?").prompt()? {
        Text::new("demo proc macro crate path:")
            .with_help_message("Path to directory containing demo proc macro crates")
            .with_validator(PathValidator)
            .prompt_skippable()?
    } else {
        None
    };

    Ok(Some(ProcMacros {
        bank_proc_macro_crate_path: None,
        demo_proc_macro_crate_path: path,
    }))
}

use colored::Colorize;

fn prompt_config() -> Result<Config, Box<dyn std::error::Error>> {
    let builder = ConfigBuilder::new();
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
                println!("\n{preview}");
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

    let preview = builder.preview()?;
    println!("{preview}");

    if Confirm::new("Save this configuration?").prompt()? {
        // Create backup if exists
        if config_path.exists() {
            let backup_path = config_path.with_extension("toml.bak");
            fs::rename(&config_path, &backup_path)?;
            println!("{}", format!("Created backup at {backup_path:?}").blue());
        }

        fs::create_dir_all(config_path.parent().unwrap())?;
        fs::write(&config_path, preview)?;
        println!(
            "{}",
            format!("Configuration saved to {config_path:?}").green()
        );
    } else {
        println!("Configuration not saved.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_simple_use() {
        let use_item: ItemUse = parse_quote! {
            use crate::logging::Verbosity;
        };
        let result = extract_use_path(&use_item);
        assert_eq!(
            result,
            Some(("Verbosity".to_string(), "crate::logging".to_string()))
        );
    }

    #[test]
    fn test_renamed_use() {
        let use_item: ItemUse = parse_quote! {
            use crate::logging::Verbosity as VerbosityLevel;
        };
        let result = extract_use_path(&use_item);
        assert_eq!(
            result,
            Some((
                "VerbosityLevel".to_string(),
                "crate::logging::Verbosity".to_string()
            ))
        );
    }

    #[test]
    fn test_grouped_use() {
        let use_item: ItemUse = parse_quote! {
            use crate::logging::{Verbosity, ColorSupport};
        };
        let result = extract_use_path(&use_item);
        assert!(result.is_some());
        // Should find either Verbosity or ColorSupport
        let (name, path) = result.unwrap();
        assert!(name == "Verbosity" || name == "ColorSupport");
        assert_eq!(path, "crate::logging");
    }

    #[test]
    fn test_ignore_non_enum() {
        let use_item: ItemUse = parse_quote! {
            use std::io;
        };
        let result = extract_use_path(&use_item);
        assert!(result.is_none());
    }
}
