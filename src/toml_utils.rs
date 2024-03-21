use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::Path;
use std::{fs, str::FromStr};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::errors::BuildRunError;
use crate::read_file_contents;

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct CargoManifest {
    #[serde(default = "default_package")]
    pub(self) package: Package,
    // #[serde(default)]
    // pub(self) dependencies: Option<Vec<Dependency>>,
    #[serde(default, skip_serializing_if = "Dependencies::is_empty")]
    dependencies: Dependencies,
    #[serde(default = "default_edition")]
    pub edition: String,
    #[serde(default)]
    pub workspaces: Option<Vec<String>>,
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
            let this = &self;
            toml::to_string(&this)
        }
        .unwrap()
    }
}

#[allow(dead_code)]
impl CargoManifest {
    // Save the CargoManifest struct to a Cargo.toml file
    fn save_to_file(&self, path: &str) -> Result<(), BuildRunError> {
        let toml_string = {
            let this = &self;
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
        authors: Vec::new(),
    }
}

// Default function for the `edition` field
fn default_edition() -> String {
    String::from("2021")
}

#[derive(Debug, Deserialize, Serialize)]
struct Package {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub authors: Vec<String>,
}

impl Default for Package {
    fn default() -> Self {
        Package {
            version: String::from("0.0.1"),
            name: String::from("your_script_name_stem"),
            authors: Vec::<String>::new(),
        }
    }
}

pub(crate) type Dependencies = BTreeMap<String, Dependency>;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Version requirement (e.g. `^1.5`)
    Simple(String),
    // /// Incomplete data
    // Inherited(InheritedDependencyDetail), // order is important for serde
    /// `{ version = "^1.5", features = ["a", "b"] }` etc.
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

// #[derive(Debug, Deserialize, Serialize)]
// pub(crate) struct Dependencies {
//     #[allow(dead_code)]
//     entry: Dependency,
//     // #[allow(dead_code)]
//     // toml: String,
// }

// #[derive(Debug, Default, Deserialize, Serialize)]
// pub(crate) struct Dependency {
//     // Add fields for dependency name, version, etc. as needed
//     pub name: String,
//     #[serde(default = "default_version")]
//     pub version: Option<String>,
//     #[serde(default)]
//     pub features: Option<Vec<String>>,
// }

// Default function for the `version` field of Dependency
#[allow(dead_code)]
fn default_version() -> Option<String> {
    None
}

#[allow(dead_code)]
fn default_package_version() -> String {
    "0.0.1".to_string()
}

// Old
// #[derive(Debug, Deserialize)]
// pub(crate) struct CargoToml {
//     #[allow(dead_code)] // Disable dead code warning for the entire struct
//     package: Package,
//     #[allow(dead_code)]
//     dependencies: Dependencies,
// }

// #[derive(Debug, Deserialize)]
// struct Package {
//     #[allow(dead_code)]
//     name: String,
//     #[allow(dead_code)]
//     version: String,
//     #[allow(dead_code)]
//     edition: String,
// }

// #[derive(Debug, Deserialize)]
// struct Dependencies {
//     #[allow(dead_code)]
//     serde: SerdeDependency,
//     #[allow(dead_code)]
//     toml: String,
// }

// #[derive(Debug, Deserialize)]
// struct SerdeDependency {
//     #[allow(dead_code)]
//     version: String,
//     #[allow(dead_code)]
//     features: Vec<String>,
// }

#[allow(dead_code)]
pub(crate) fn read_cargo_toml() -> Result<CargoManifest, BuildRunError> {
    let toml_str = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml file");

    let cargo_toml: CargoManifest = toml::from_str(&toml_str)?;

    // debug!("cargo_toml={cargo_toml:#?}");
    Ok(cargo_toml)
}

pub(crate) fn read_rs_toml(code_path: &Path) -> Result<CargoManifest, BuildRunError> {
    let rs_contents = read_file_contents(code_path)?;
    let rs_toml_str = rs_contents
        .lines()
        .map(str::trim_start)
        .filter(|&line| line.starts_with("//!"))
        .map(|line| line.trim_start_matches('/').trim_start_matches('!'))
        .fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "{b}");
            output
        });
    debug!("Rust source manifest info (rs_toml_str) = {rs_toml_str}");

    CargoManifest::from_str(&rs_toml_str)
}
// end old
