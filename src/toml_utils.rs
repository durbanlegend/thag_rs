#![allow(clippy::uninlined_format_args)]
use log::debug;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::io::BufRead;
use std::process::Command;
use std::time::Instant;
use std::{fs, str::FromStr};

use crate::errors::BuildRunError;

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
#[allow(dead_code)]
pub(crate) fn read_cargo_toml() -> Result<CargoManifest, BuildRunError> {
    let toml_str = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml file");

    let cargo_toml: CargoManifest = toml::from_str(&toml_str)?;

    // debug!("cargo_toml={cargo_toml:#?}");
    Ok(cargo_toml)
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Workspace {}

pub(crate) fn rs_extract_manifest(rs_contents: &str) -> Result<CargoManifest, BuildRunError> {
    let rs_toml_str = rs_extract_toml(rs_contents);
    CargoManifest::from_str(&rs_toml_str)
}

fn rs_extract_toml(rs_contents: &str) -> String {
    use std::fmt::Write;
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
    rs_toml_str
}

pub(crate) fn cargo_search(dep_crate: &str) -> Result<(String, String), Box<dyn Error>> {
    let start_search = Instant::now();
    // let mut dummy_command = Command::new("cargo");
    // dummy_command.args(["build", "--verbose"]);
    // debug!("\nCargo dummy command={dummy_command:#?}\n");

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
    // debug!("\nCargo search command={search_command:#?}\n");
    let search_output = search_command.output()?;

    let first_line: String = if search_output.status.success() {
        // let success_msg = String::from_utf8_lossy(&search_output.stdout);
        // info!("##### cargo search succeeded!");
        // if let Some(msg) = success_msg.lines().next() {
        //     msg.to_string()
        // };
        // // }?;
        use std::fmt::Write;

        search_output
            .stdout
            .lines()
            .take(1)
            .filter_map(Result::ok)
            .fold(String::new(), |mut output, b| {
                let _ = writeln!(output, "{b}");
                output
            })
    } else {
        let error_msg = String::from_utf8_lossy(&search_output.stderr);
        error_msg.lines().for_each(|line| {
            debug!("{line}");
        });
        return Err(Box::new(BuildRunError::Command(
            "Cargo search failed".to_string(),
        )));
    };

    // debug!("!!!!!!!! first_line={first_line}");

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

    let dur = start_search.elapsed();
    debug!(
        "Completed search in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );

    Ok((name, version))
}

pub(crate) fn capture_dep(first_line: &str) -> Result<(String, String), Box<dyn Error>> {
    let regex_str = r#"^(?P<name>\w+) = "(?P<version>\d+\.\d+\.\d+)"#;
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

pub(crate) fn default_manifest(
    build_dir: &str,
    source_stem: &str,
) -> Result<CargoManifest, BuildRunError> {
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

// let source_manifest_toml = cargo_manifest.parse::<Table>()?;
// debug!("source_manifest_toml={source_manifest_toml:#?}\n");

// let toml = toml::to_string(&source_manifest_toml)?;
// // debug!("Raw cargo_manifest = {toml:#?}\n");

// debug!("Cargo_manifest reconstituted:");
// toml.lines().for_each(|l| println!("{l}"));
