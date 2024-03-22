use log::debug;
use regex::Regex;
use std::{collections::HashSet, fs, path::Path};

use crate::errors::BuildRunError;

#[allow(dead_code)]
fn main() {
    let code_snippet = r#"
  use std::io;

  #[macro_use]
  extern crate serde_derive;

  fn main() {
    println!("Hello, world!");
  }
  "#;

    let dependencies = infer_dependencies(code_snippet);
    println!("Potential dependencies: {dependencies:?}");
}
pub(crate) fn read_file_contents(path: &Path) -> Result<String, BuildRunError> {
    debug!("Reading from {path:?}");
    Ok(fs::read_to_string(path)?)
}

pub(crate) fn rs_extract_src(rs_contents: &str) -> String {
    use std::fmt::Write;
    let rs_source = rs_contents
        .lines()
        .map(str::trim_start)
        .filter(|&line| !line.starts_with("//!"))
        .fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "{b}");
            output
        });
    debug!("Rust source string (rs_source) = {rs_source}");
    rs_source
}

// Make a best effort to help the user by inferring dependencies from the source code.
pub(crate) fn infer_dependencies(code: &str) -> HashSet<String> {
    let mut dependencies = HashSet::new();

    let use_regex = Regex::new(r"(?i)use\s+([^;{]+)").unwrap();
    let macro_use_regex = Regex::new(r"(?i)#\[macro_use\]\s+::\s+([^;{]+)").unwrap();
    let extern_crate_regex = Regex::new(r"(?i)extern\s+crate\s+([^;{]+)").unwrap();

    let built_in_crates = &["std", "core", "alloc", "collections", "fmt"];

    for cap in use_regex.captures_iter(code) {
        let dependency = cap[1].to_string();
        if !built_in_crates
            .iter()
            .any(|builtin| dependency.starts_with(builtin))
        {
            if let Some((dep, _)) = dependency.split_once(':') {
                dependencies.insert(dep.to_owned());
            }
        }
    }

    // Similar checks for other regex patterns

    for cap in macro_use_regex.captures_iter(code) {
        let dependency = cap[1].to_string();
        if !built_in_crates
            .iter()
            .any(|builtin| dependency.starts_with(builtin))
        {
            dependencies.insert(dependency);
        }
    }

    for cap in extern_crate_regex.captures_iter(code) {
        let dependency = cap[1].to_string();
        if !built_in_crates
            .iter()
            .any(|builtin| dependency.starts_with(builtin))
        {
            dependencies.insert(dependency);
        }
    }

    dependencies
}
