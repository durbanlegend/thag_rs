#![allow(clippy::uninlined_format_args)]
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::BufRead;
use std::process::Command;
use std::str::FromStr;
use std::time::Instant;

use crate::code_utils::{infer_deps_from_ast, infer_deps_from_source}; // Valid if no circular dependency
use crate::debug_log;
use crate::errors::BuildRunError;
use crate::log;
use crate::logging::Verbosity;
use crate::shared::{debug_timings, Ast, BuildState, CargoManifest, Dependency, Feature};
use crate::term_colors::{nu_resolve_style, MessageLevel};
pub fn cargo_search(dep_crate: &str) -> Result<(String, String), Box<dyn Error>> {
    let start_search = Instant::now();

    let dep_crate_styled = nu_resolve_style(MessageLevel::Emphasis).paint(dep_crate);
    log!(
        Verbosity::Normal,
        r#"Doing a Cargo search for crate {dep_crate_styled} referenced in your script.
See below for how to avoid this and speed up future builds.
"#,
    );

    let mut search_command = Command::new("cargo");
    search_command.args(["search", dep_crate, "--limit", "1"]);
    debug_log!("\nCargo search command={search_command:#?}\n");
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
            debug_log!("{line}");
        });
        return Err(Box::new(BuildRunError::Command(format!(
            "Cargo search failed for [{dep_crate}]"
        ))));
    };

    debug_log!("first_line={first_line}");
    let result = capture_dep(&first_line);
    let (name, version) = match result {
        Ok((name, version)) => {
            if name != dep_crate && name.replace('-', "_") != dep_crate {
                debug_log!("First line of cargo search for crate {dep_crate} found non-matching crate {name}");
                return Err(Box::new(BuildRunError::Command(format!(
                    "Cargo search failed for [{dep_crate}]: returned non-matching crate [{name}]"
                ))));
            }

            let dep_crate_styled = nu_resolve_style(MessageLevel::Emphasis).paint(&name);
            let dep_version_styled = nu_resolve_style(MessageLevel::Emphasis).paint(&version);

            log!(
                Verbosity::Normal,
                r#"Cargo found the following dependency, which you can copy into the toml block
as shown if you don't need special features:
/*[toml]
[dependencies]
{dep_crate_styled} = "{dep_version_styled}"
*/
"#
            );
            (name, version)
        }
        Err(err) => {
            debug_log!("Failure! err={err}");
            return Err(err);
        }
    };

    debug_timings(&start_search, "Completed search");

    Ok((name, version))
}

pub fn capture_dep(first_line: &str) -> Result<(String, String), Box<dyn Error>> {
    debug_log!("first_line={first_line}");
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"^(?P<name>[\w-]+) = "(?P<version>\d+\.\d+\.\d+)"#).unwrap();
    }
    let (name, version) = if RE.is_match(first_line) {
        let captures = RE.captures(first_line).unwrap();
        let name = captures.get(1).unwrap().as_str();
        let version = captures.get(2).unwrap().as_str();
        // log!(Verbosity::Normal, "Dependency name: {}", name);
        // log!(Verbosity::Normal, "Dependency version: {}", version);
        (String::from(name), String::from(version))
    } else {
        log!(Verbosity::Quiet, "Not a valid Cargo dependency format.");
        return Err(Box::new(BuildRunError::Command(
            "Not a valid Cargo dependency format".to_string(),
        )));
    };
    Ok((name, version))
}

pub fn default_manifest(build_state: &BuildState) -> Result<CargoManifest, BuildRunError> {
    let source_stem = &build_state.source_stem;
    let source_name = &build_state.source_name;
    let binding = build_state.target_dir_path.join(source_name);
    let gen_src_path = &binding.to_string_lossy();

    let gen_src_path = escape_path_for_windows(gen_src_path);

    let cargo_manifest = format!(
        r##"[package]
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

    // log!(Verbosity::Normal, "cargo_manifest=\n{cargo_manifest}");

    CargoManifest::from_str(&cargo_manifest)
}

pub fn escape_path_for_windows(path: &str) -> String {
    #[cfg(windows)]
    {
        path.replace('\\', "\\\\")
    }
    #[cfg(not(windows))]
    {
        path.to_string()
    }
}

pub fn merge_manifest(
    build_state: &mut BuildState,
    rs_source: &str,
    syntax_tree: &Option<Ast>,
) -> Result<CargoManifest, Box<dyn Error>> {
    let start_merge_manifest = Instant::now();

    let mut default_manifest = default_manifest(build_state)?;
    let cargo_manifest: &mut CargoManifest =
        if let Some(ref mut manifest) = build_state.cargo_manifest {
            manifest
        } else {
            &mut default_manifest
        };

    debug_log!("cargo_manifest (before deps)={cargo_manifest:#?}");

    let rs_inferred_deps = if let Some(ref syntax_tree) = syntax_tree {
        infer_deps_from_ast(syntax_tree)
    } else {
        infer_deps_from_source(rs_source)
    };

    debug_log!("rs_inferred_deps={rs_inferred_deps:#?}\n");
    // if let Some(rs_manifest) = &build_state.rs_manifest {
    //     debug_log!(
    //         "build_state.rs_manifest.dependencies={:#?}",
    //         rs_manifest.dependencies
    //     );
    // }

    let mut rs_dep_map: BTreeMap<std::string::String, Dependency> =
        if let Some(ref rs_manifest) = &build_state.rs_manifest {
            if let Some(ref rs_dep_map) = rs_manifest.dependencies {
                rs_dep_map.clone()
            } else {
                BTreeMap::new()
            }
        } else {
            BTreeMap::new()
        };

    if !rs_inferred_deps.is_empty() {
        debug_log!("rs_dep_map (before inferred) {rs_dep_map:#?}");
        for dep_name in rs_inferred_deps {
            if rs_dep_map.contains_key(&dep_name)
                || rs_dep_map.contains_key(&dep_name.replace('_', "-"))
                || ["crate", "macro_rules"].contains(&dep_name.as_str())
            {
                continue;
            }
            debug_log!("Starting Cargo search for key dep_name [{dep_name}]");
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
                log!(
                    Verbosity::Quiet,
                    "Cargo search couldn't find crate [{dep_name}]"
                );
                continue;
            };
            rs_dep_map.insert(dep_name, dep);
        }
        debug_log!("rs_dep_map (after inferred) = {rs_dep_map:#?}");
    }

    // Clone and merge dependencies
    let manifest_deps = cargo_manifest.dependencies.as_ref().unwrap();

    let mut manifest_deps_clone: BTreeMap<std::string::String, Dependency> = manifest_deps.clone();
    debug_log!("manifest_deps  (before merge) {manifest_deps_clone:?}");

    // Insert any entries from source and inferred deps that are not already in default manifest
    rs_dep_map
        .iter()
        .filter(|&(name, _dep)| !(manifest_deps.contains_key(name)))
        .for_each(|(key, value)| {
            manifest_deps_clone.insert(key.to_string(), value.clone());
        });
    cargo_manifest.dependencies = Some(manifest_deps_clone);
    debug_log!(
        "cargo_manifest.dependencies (after merge) {:#?}",
        cargo_manifest.dependencies
    );

    // Clone and merge features
    let manifest_feats = cargo_manifest.features.as_ref().unwrap();

    let rs_feat_map: BTreeMap<std::string::String, Vec<Feature>> =
        if let Some(ref mut rs_manifest) = build_state.rs_manifest {
            if let Some(ref mut rs_feat_map) = rs_manifest.features {
                rs_feat_map.clone()
            } else {
                BTreeMap::new()
            }
        } else {
            BTreeMap::new()
        };

    let mut manifest_features_clone: BTreeMap<std::string::String, Vec<Feature>> =
        manifest_feats.clone();
    debug_log!("manifest_features (before merge) {manifest_features_clone:?}");

    // Insert any entries from source features that are not already in default manifest
    rs_feat_map
        .iter()
        .filter(|&(name, _dep)| !(manifest_feats.contains_key(name)))
        .for_each(|(key, value)| {
            manifest_features_clone.insert(key.to_string(), value.clone());
        });
    cargo_manifest.features = Some(manifest_features_clone);
    debug_log!(
        "cargo_manifest.features (after merge) {:#?}",
        cargo_manifest.features
    );

    debug_timings(&start_merge_manifest, "Processed features");

    Ok(cargo_manifest.clone())
}
