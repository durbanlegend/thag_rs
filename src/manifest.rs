#![allow(clippy::uninlined_format_args)]
use log::debug;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::io::BufRead;
use std::process::Command;
use std::str::FromStr;
use std::time::Instant;
use syn::File;

use crate::code_utils::{debug_timings, infer_deps_from_ast, infer_deps_from_source};
use crate::errors::BuildRunError;
use crate::term_colors::{MessageStyle, OwoThemeStyle};
use crate::BuildState;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CargoManifest {
    #[serde(default = "default_package")]
    pub(crate) package: Package,
    pub(crate) dependencies: Option<Dependencies>,
    pub(crate) features: Option<Features>,
    #[serde(default)]
    pub(crate) workspace: Workspace,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bin: Vec<Product>,
}

impl FromStr for CargoManifest {
    type Err = BuildRunError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str::<CargoManifest>(s).map_err(|e| BuildRunError::FromStr(e.to_string()))
    }
}

impl ToString for CargoManifest {
    fn to_string(&self) -> String {
        {
            let this = self;
            toml::to_string(&this)
        }
        .unwrap()
    }
}

#[allow(dead_code)]
impl CargoManifest {
    // Save the CargoManifest struct to a Cargo.toml file
    pub(crate) fn save_to_file(&self, path: &str) -> Result<(), BuildRunError> {
        let toml_string = {
            let this = self;
            toml::to_string(&this)
        }?;
        std::fs::write(path, toml_string.as_bytes())?;
        Ok(())
    }
}

// Default function for the `package` field
fn default_package() -> Package {
    Package {
        name: String::from("your_project_name"),
        version: String::from("0.1.0"),
        edition: default_edition(),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) version: String,
    #[serde(default = "default_edition")]
    pub(crate) edition: String,
}

// Default function for the `edition` field
fn default_edition() -> String {
    String::from("2021")
}

impl Default for Package {
    fn default() -> Self {
        Package {
            version: String::from("0.0.0"),
            name: String::from("your_script_name_stem"),
            edition: default_edition(),
        }
    }
}

pub(crate) type Dependencies = BTreeMap<String, Dependency>;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed(Box<DependencyDetail>),
}

fn default_true() -> bool {
    true
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_true(val: &bool) -> bool {
    *val
}

/// When definition of a dependency is more than just a version string.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub path: Option<String>,
    pub registry: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub default_features: bool,
}

pub(crate) type Features = BTreeMap<String, Vec<Feature>>;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Feature {
    Simple(String),
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Product {
    pub path: Option<String>,
    pub name: Option<String>,
    pub required_features: Option<Vec<String>>,
}

#[allow(dead_code)]
fn default_package_version() -> String {
    "0.0.1".to_string()
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Workspace {}

pub(crate) fn cargo_search(dep_crate: &str) -> Result<(String, String), Box<dyn Error>> {
    let start_search = Instant::now();

    let dep_crate_styled =
        // TODO remove hard coded colour support and theme variant
        owo_colors::Style::style(&MessageStyle::Ansi16DarkEmphasis.get_owo_style().unwrap(), dep_crate);
    println!(
        r#"
            Doing a Cargo search for crate {dep_crate_styled} referenced in your script.
            To speed up build, consider embedding the required {dep_crate_styled} = "<version>"
            in a block comment at the top of the script, in the form:
            /*[toml]
            [dependencies]
            {dep_crate_styled} = "n.n.n"
            */
            E.g.:
    //! [dependencies]
    //! {dep_crate_styled} = "<version n.n.n goes here>"
            "#,
    );

    // let content = format!(r#"hello"#);
    // println!(
    //     "Why, {} world!",
    //     owo_colors::Style::style(&MessageStyle::Emphasis.get_style().unwrap(), content)
    // );

    let mut search_command = Command::new("cargo");
    search_command.args(["search", dep_crate, "--limit", "1"]);
    debug!("\nCargo search command={search_command:#?}\n");
    let search_output = search_command.output()?;

    let first_line = if search_output.status.success() {
        search_output
            .stdout
            .lines()
            .map_while(Result::ok)
            .next()
            .ok_or_else(|| {
                Box::new(BuildRunError::Command(format!(
                    "Something went wrong with Cargo search for [{dep_crate}]"
                )))
            })?
    } else {
        let error_msg = String::from_utf8_lossy(&search_output.stderr);
        error_msg.lines().for_each(|line| {
            debug!("{line}");
        });
        return Err(Box::new(BuildRunError::Command(format!(
            "Cargo search failed for [{dep_crate}]"
        ))));
    };

    debug!("!!!!!!!! first_line={first_line}");
    let result = capture_dep(&first_line);
    let (name, version) = match result {
        Ok((name, version)) => {
            if name != dep_crate && name.replace('-', "_") != dep_crate {
                debug!("First line of cargo search for crate {dep_crate} found non-matching crate {name}");
                return Err(Box::new(BuildRunError::Command(format!(
                    "Cargo search failed for [{dep_crate}]: returned non-matching crate [{name}]"
                ))));
            }
            debug!("Success! value={:?}", (&name, &version));
            (name, version)
        }
        Err(err) => {
            debug!("Failure! err={err}");
            return Err(err);
        }
    };
    debug!("!!!!!!!! found name={name}, version={version}");

    debug_timings(&start_search, "Completed search");

    Ok((name, version))
}

pub(crate) fn capture_dep(first_line: &str) -> Result<(String, String), Box<dyn Error>> {
    debug!("first_line={first_line}");
    let regex_str = r#"^(?P<name>[\w-]+) = "(?P<version>\d+\.\d+\.\d+)"#;
    let re = Regex::new(regex_str).unwrap();
    let (name, version) = if re.is_match(first_line) {
        let captures = re.captures(first_line).unwrap();
        let name = captures.get(1).unwrap().as_str();
        let version = captures.get(2).unwrap().as_str();
        println!("Dependency name: {}", name);
        println!("Dependency version: {}", version);
        (String::from(name), String::from(version))
    } else {
        println!("Not a valid Cargo dependency format.");
        return Err(Box::new(BuildRunError::Command(
            "Not a valid Cargo dependency format".to_string(),
        )));
    };
    Ok((name, version))
}

pub(crate) fn default_manifest(build_state: &BuildState) -> Result<CargoManifest, BuildRunError> {
    let source_stem = &build_state.source_stem;
    let source_name = &build_state.source_name;
    let binding = build_state.target_dir_path.join(source_name);
    let gen_src_path = &binding.to_string_lossy();

    let gen_src_path = escape_path_for_windows(gen_src_path);

    let cargo_manifest = format!(
        r##"
[package]
name = "{source_stem}"
version = "0.0.1"
edition = "2021"

[dependencies]

[features]

[workspace]

[[bin]]
name = "{source_stem}"
path = "{gen_src_path}"
"##
    );

    eprintln!("cargo_manifest=\n{cargo_manifest}");

    CargoManifest::from_str(&cargo_manifest)
}

fn escape_path_for_windows(path: &str) -> String {
    #[cfg(windows)]
    {
        path.replace('\\', "\\\\")
    }
    #[cfg(not(windows))]
    {
        path.to_string()
    }
}

pub(crate) fn merge_manifest(
    build_state: &BuildState,
    maybe_syntax_tree: Option<&File>,
    maybe_rs_source: Option<&String>,
    rs_manifest: &mut CargoManifest,
) -> Result<CargoManifest, Box<dyn Error>> {
    let start_merge_manifest = Instant::now();

    let mut cargo_manifest = default_manifest(build_state)?;
    debug!("@@@@ cargo_manifest (before deps)={cargo_manifest:#?}");

    // // TODO temp debug out
    // infer_deps_from_source(maybe_rs_source.ok_or("Missing source code")?);

    let rs_inferred_deps = if let Some(syntax_tree) = maybe_syntax_tree {
        infer_deps_from_ast(syntax_tree)
    } else {
        infer_deps_from_source(maybe_rs_source.ok_or("Missing source code")?)
    };
    debug!("rs_inferred_deps={rs_inferred_deps:#?}\n");
    debug!("rs_manifest.dependencies={:#?}", rs_manifest.dependencies);

    let mut rs_dep_map: BTreeMap<std::string::String, Dependency> =
        if let Some(ref mut rs_dep_map) = rs_manifest.dependencies {
            rs_dep_map.clone()
        } else {
            // return Err(Box::new(BuildRunError::Command(String::from(
            //     "No dependency map found",
            // ))));
            BTreeMap::new()
        };

    if !rs_inferred_deps.is_empty() {
        debug!("dep_map (before inferred) {rs_dep_map:#?}");
        for dep_name in rs_inferred_deps {
            if rs_dep_map.contains_key(&dep_name)
                || rs_dep_map.contains_key(&dep_name.replace('_', "-"))
                || ["crate", "macro_rules"].contains(&dep_name.as_str())
            {
                continue;
            }
            debug!("############ Starting Cargo search for key dep_name [{dep_name}]");
            let cargo_search_result = cargo_search(&dep_name);
            // If the crate name is hyphenated, Cargo search will nicely search for underscore version and return the correct
            // hyphenated name. So we must replace the incorrect underscored version we searched on with the corrected
            // hyphenated version that the Cargo search returned.
            let (dep_name, dep) = if let Ok((dep_name, version)) = cargo_search_result {
                (dep_name, Dependency::Simple(version))
            } else {
                // return Err(Box::new(BuildRunError::Command(format!(
                //     "Cargo search couldn't find crate [{dep_name}]"
                // ))));
                println!("Cargo search couldn't find crate [{dep_name}]");
                continue;
            };
            rs_dep_map.insert(dep_name, dep);
        }
        debug!("rs_dep_map (after inferred) = {rs_dep_map:#?}");
    }

    // Clone and merge dependencies
    let manifest_deps = cargo_manifest.dependencies.as_ref().unwrap();

    let mut manifest_deps_clone: BTreeMap<std::string::String, Dependency> = manifest_deps.clone();
    debug!("manifest_deps  (before merge) {manifest_deps_clone:?}");

    // Insert any entries from source and inferred deps that are not already in default manifest
    rs_dep_map
        .iter()
        .filter(|&(name, _dep)| !(manifest_deps.contains_key(name)))
        .for_each(|(key, value)| {
            manifest_deps_clone.insert(key.to_string(), value.clone());
        });
    cargo_manifest.dependencies = Some(manifest_deps_clone);
    debug!(
        "cargo_manifest.dependencies (after merge) {:#?}",
        cargo_manifest.dependencies
    );

    // Clone and merge features
    let manifest_feats = cargo_manifest.features.as_ref().unwrap();

    let rs_feat_map: BTreeMap<std::string::String, Vec<Feature>> =
        if let Some(ref mut rs_feat_map) = rs_manifest.features {
            rs_feat_map.clone()
        } else {
            // return Err(Box::new(BuildRunError::Command(String::from(
            //     "No feature map found",
            // ))));
            BTreeMap::new()
        };

    let mut manifest_features_clone: BTreeMap<std::string::String, Vec<Feature>> =
        manifest_feats.clone();
    // debug!("manifest_features (before merge) {manifest_features_clone:?}");

    // Insert any entries from source features that are not already in default manifest
    rs_feat_map
        .iter()
        .filter(|&(name, _dep)| !(manifest_feats.contains_key(name)))
        .for_each(|(key, value)| {
            manifest_features_clone.insert(key.to_string(), value.clone());
        });
    cargo_manifest.features = Some(manifest_features_clone);
    debug!(
        "cargo_manifest.features (after merge) {:#?}",
        cargo_manifest.features
    );

    debug_timings(&start_merge_manifest, "Processed features");

    Ok(cargo_manifest)
}
