use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::Path;
use std::{fs, str::FromStr};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::errors::BuildRunError;
use crate::read_file_contents;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CargoManifest {
    #[serde(default = "default_package")]
    pub(self) package: Package,
    dependencies: Option<Dependencies>,
    #[serde(default = "default_edition")]
    pub edition: String,
    #[serde(default)]
    pub workspace: Workspace,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bin: Vec<Product>,
}

impl Default for CargoManifest {
    fn default() -> Self {
        CargoManifest {
            package: Package::default(),
            dependencies: None,
            edition: "2021".to_string(),
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
            let this = &self;
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
}

impl Default for Package {
    fn default() -> Self {
        Package {
            version: String::from("0.0.0"),
            name: String::from("your_script_name_stem"),
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

// Default function for the `version` field of Dependency
// #[allow(dead_code)]
// fn default_version() -> Option<String> {
//     None
// }

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Product {
    pub path: Option<String>,
    pub name: Option<String>,
    pub required_features: Option<Vec<String>>,
}

// #[derive(Debug, Default, Deserialize, Serialize)]
// pub struct Bin {
//     pub products: Vec<Products>,
// }

#[allow(dead_code)]
fn default_package_version() -> String {
    "0.0.1".to_string()
}
#[allow(dead_code)]
pub(crate) fn read_cargo_toml() -> Result<CargoManifest, BuildRunError> {
    let toml_str = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml file");

    let cargo_toml: CargoManifest = toml::from_str(&toml_str)?;

    // debug!("cargo_toml={cargo_toml:#?}");
    Ok(cargo_toml)
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Workspace {}

// impl Default for Workspace {
//     fn default() -> Self {
//         Workspace {}
//     }
// }

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
