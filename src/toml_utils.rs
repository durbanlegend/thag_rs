use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct CargoToml {
    #[allow(dead_code)] // Disable dead code warning for the entire struct
    package: Package,
    #[allow(dead_code)]
    dependencies: Dependencies,
}

#[derive(Debug, Deserialize)]
struct Package {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    edition: String,
}

#[derive(Debug, Deserialize)]
struct Dependencies {
    #[allow(dead_code)]
    serde: SerdeDependency,
    #[allow(dead_code)]
    toml: String,
}

#[derive(Debug, Deserialize)]
struct SerdeDependency {
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    features: Vec<String>,
}

pub(crate) fn read_cargo_toml() {
    let toml_str = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml file");

    let cargo_toml: CargoToml =
        toml::from_str(&toml_str).expect("Failed to deserialize Cargo.toml");

    println!("{cargo_toml:#?}");
    // Example source code and Cargo.toml content
}
