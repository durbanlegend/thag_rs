use std::fs;

use serde::{Deserialize, Serialize};
use toml::to_string;

use crate::errors::BuildRunError;

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct CargoManifest {
    #[serde(default = "default_package")]
    pub(self) package: Package,
    #[serde(default)]
    pub(self) dependencies: Option<Vec<Dependency>>,
    #[serde(default = "default_edition")]
    pub edition: String,
    #[serde(default)]
    pub workspaces: Option<Vec<String>>,
}

impl CargoManifest {
    // Serialize the CargoManifest struct to a String
    fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        to_string(&self)
    }

    // Save the CargoManifest struct to a Cargo.toml file
    fn save_to_file(&self, path: &str) -> Result<(), BuildRunError> {
        let toml_string = self.to_toml_string()?;
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
    String::from("edition = \"2021\"")
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Package {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub authors: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Dependency {
    // Add fields for dependency name, version, etc. as needed
    pub name: String,
    #[serde(default = "default_version")]
    pub version: Option<String>,
    #[serde(default)]
    pub features: Option<Vec<String>>,
}

// Default function for the `version` field of Dependency
fn default_version() -> Option<String> {
    None
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

pub(crate) fn read_cargo_toml() -> Result<CargoManifest, BuildRunError> {
    let toml_str = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml file");

    let cargo_toml: CargoManifest = toml::from_str(&toml_str)?;

    println!("#####cargo_toml={cargo_toml:#?}");
    Ok(cargo_toml)
}

pub(crate) fn read_rs_toml() {
    let toml_str = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml file");

    let cargo_toml: CargoManifest =
        toml::from_str(&toml_str).expect("Failed to deserialize Cargo.toml");

    println!("{cargo_toml:#?}");
}
// end old
