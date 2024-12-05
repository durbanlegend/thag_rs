/// Use the `documented` crate to iterate through struct fields and their docs at runtime.
//# Purpose: Prototype for `thag_config_builder`.
//# Categories: crates, prototype, technique
use documented::{Documented, DocumentedFields, DocumentedVariants};
use phf;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;

/// Dependency handling
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Deserialize, Serialize, Documented, DocumentedFields)]
#[serde(default)]
pub struct Dependencies {
    /// Exclude features containing "unstable"
    pub exclude_unstable_features: bool,
    /// Exclude the "std" feature
    pub exclude_std_feature: bool,
    /// Features that should always be included if present, e.g. `derive`
    pub always_include_features: Vec<String>,
    /// Exclude releases with pre-release markers such as -beta.
    pub exclude_prerelease: bool, // New option
    // pub minimum_downloads: Option<u64>,  // New option
    // pub minimum_version: Option<String>, // New option
    /// Crate-level feature overrides
    pub feature_overrides: HashMap<String, FeatureOverride>,
    /// Features that should always be excluded
    pub global_excluded_features: Vec<String>,
}

impl Default for Dependencies {
    fn default() -> Self {
        Dependencies {
            exclude_unstable_features: true,
            exclude_std_feature: true,
            always_include_features: vec!["derive".to_string()],
            exclude_prerelease: true,
            feature_overrides: HashMap::<String, FeatureOverride>::new(),
            global_excluded_features: vec![],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeatureOverride {
    /// Features to be excluded for crate
    pub excluded_features: Vec<String>,
    /// Features required for crate
    pub required_features: Vec<String>,
}


for field_name in Dependencies::FIELD_NAMES {
    println!("field_name={field_name}, field_doc={:#?}", Dependencies::get_field_docs(field_name));
}
