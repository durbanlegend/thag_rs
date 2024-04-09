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

use crate::code_utils::{debug_timings, infer_dependencies};
use crate::errors::BuildRunError;
use crate::BuildState;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CargoManifest {
    #[serde(default = "default_package")]
    pub(crate) package: Package,
    pub(crate) dependencies: Option<Dependencies>,
    #[serde(default)]
    pub(crate) workspace: Workspace,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bin: Vec<Product>,
}

impl Default for CargoManifest {
    fn default() -> Self {
        CargoManifest {
            package: Package::default(),
            dependencies: None,
            workspace: Workspace::default(),
            bin: vec![Product::default()],
        }
    }
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

pub(crate) type Dependencies = Option<BTreeMap<String, Dependency>>;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed(Box<DependencyDetail>),
}

/// When definition of a dependency is more than just a version string.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
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

    println!(
        r#"
        Doing a Cargo search for crate {dep_crate} referenced in your script.
        To speed up build, consider embedding the required {dep_crate} = "<version>"
        in comments (//!) in the script.
        E.g. {dep_crate} = "<version n.n.n goes here>
        "#
    );

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

    let (name, version) = match capture_dep(&first_line) {
        Ok(value) => {
            debug!("Success! value={value:?}");
            value
        }
        Err(err) => {
            debug!("Failure! err={err}");
            return Err(err);
        }
    };
    debug!("!!!!!!!! found name={name}, version={version}");

    debug_timings(start_search, "Completed search");

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
    let build_dir = &build_state.target_dir_str;
    let source_stem = &build_state.source_stem;

    let cargo_manifest = format!(
        r##"
[package]
name = "{source_stem}"
version = "0.0.1"
edition = "2021"

[dependencies]

[workspace]

[[bin]]
name = "{source_stem}"
path = "{build_dir}/{source_stem}.rs"
"##
    );

    CargoManifest::from_str(&cargo_manifest)
}

pub(crate) fn resolve_deps(
    build_state: &BuildState,
    rs_source: &str,
    rs_manifest: &mut CargoManifest,
) -> Result<CargoManifest, Box<dyn Error>> {
    let start_deps_rs = Instant::now();

    let mut cargo_manifest = default_manifest(build_state)?;
    debug!("@@@@ cargo_manifest (before deps)={cargo_manifest:#?}");

    let rs_inferred_deps = infer_dependencies(rs_source);
    debug!("rs_inferred_deps={rs_inferred_deps:#?}\n");
    debug!("rs_manifest.dependencies={:#?}", rs_manifest.dependencies);

    let mut rs_dep_map: BTreeMap<std::string::String, Dependency> =
        if let Some(Some(ref mut rs_dep_map)) = rs_manifest.dependencies {
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
            {
                // Already in manifest
                // debug!(
                //     "============ rs_dep_map {rs_dep_map:#?} already contains key dep_name {dep_name}"
                // );
                continue;
            }
            debug!("############ Doing a Cargo search for key dep_name [{dep_name}]");
            let cargo_search_result = cargo_search(&dep_name);
            // If the crate name is hyphenated, Cargo search will nicely search for underscore version and return the correct
            // hyphenated name. So we must replace the incorrect underscored version we searched on with the corrected
            // hyphenated version that the Cargo search returned.
            let (dep_name, dep) = if let Ok((dep_name, version)) = cargo_search_result {
                (dep_name, Dependency::Simple(version))
            } else {
                return Err(Box::new(BuildRunError::Command(format!(
                    "Cargo search couldn't find crate [{dep_name}]"
                ))));
            };
            rs_dep_map.insert(dep_name, dep);
        }
        debug!("rs_dep_map (after inferred) = {rs_dep_map:?}");
    }

    let manifest_deps = cargo_manifest
        .dependencies
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();

    // Clone dependencies
    let mut manifest_deps_clone: BTreeMap<std::string::String, Dependency> = manifest_deps.clone();
    debug!("manifest_deps  (before inferred) {manifest_deps_clone:?}");

    // Insert any entries from source and inferred deps that are not already in default manifest
    rs_dep_map
        .iter()
        .filter(|&(name, _dep)| !(manifest_deps.contains_key(name)))
        .for_each(|(key, value)| {
            manifest_deps_clone.insert(key.to_string(), value.clone());
        });
    cargo_manifest.dependencies = Some(Some(manifest_deps_clone));
    debug!(
        "cargo_manifest.dependencies (after merge) {:#?}",
        cargo_manifest.dependencies
    );

    debug_timings(start_deps_rs, "Processed dependencies");

    Ok(cargo_manifest)
}
