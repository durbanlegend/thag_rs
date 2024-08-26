#![allow(clippy::uninlined_format_args)]
use cargo_toml::{Dependency, Manifest};
use lazy_static::lazy_static;
use mockall::automock;
use regex::Regex;
use serde_merge::omerge;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{self, BufRead};
use std::process::{Command, Output};
use std::time::Instant;

use crate::code_utils::{infer_deps_from_ast, infer_deps_from_source}; // Valid if no circular dependency
use crate::colors::{nu_resolve_style, MessageLevel};
use crate::debug_log;
use crate::errors::ThagError;
use crate::log;
use crate::logging::Verbosity;
use crate::shared::{debug_timings, escape_path_for_windows, Ast, BuildState};

/// A trait to allow mocking of the command for testing purposes.
#[automock]
pub trait CommandRunner {
    /// Run the Cargo search, real or mocked.
    /// # Errors
    /// Will return `Err` if the first line does not match the expected crate name and a valid version number.
    fn run_command(&self, program: &str, args: &[String]) -> io::Result<Output>;
}

/// A struct for use in actual running of the command, as opposed to use in testing.
pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    /// Run the Cargo search, real or mocked.
    /// # Errors
    /// Will return `Err` if the first line does not match the expected crate name and a valid version number.
    fn run_command(&self, program: &str, args: &[String]) -> io::Result<Output> {
        Command::new(program).args(args).output()
    }
}

/// Attempt to find a matching dependency name and version from Cargo by searching by
/// crate name and inspecting the first line of Cargo's response.
/// # Errors
/// Will return `Err` if the first line does not match the expected crate name and a valid version number.
pub fn cargo_search<R: CommandRunner>(
    runner: &R,
    dep_crate: &str,
) -> Result<(String, String), Box<dyn Error>> {
    let start_search = Instant::now();

    let dep_crate_styled = nu_resolve_style(MessageLevel::Emphasis).paint(dep_crate);
    log!(
        Verbosity::Normal,
        r#"Doing a Cargo search for crate {dep_crate_styled} referenced in your script.
See below for how to avoid this and speed up future builds.
"#,
    );

    let args = vec![
        "search".to_string(),
        dep_crate.to_string(),
        "--limit".to_string(),
        "1".to_string(),
    ];
    let search_output = runner.run_command("cargo", &args)?;

    let first_line = if search_output.status.success() {
        search_output
            .stdout
            .lines()
            .map_while(Result::ok)
            .next()
            .ok_or_else(|| {
                Box::new(ThagError::Command(format!(
                    "Something went wrong with Cargo search for [{dep_crate}]"
                )))
            })?
    } else {
        let error_msg = String::from_utf8_lossy(&search_output.stderr);
        error_msg.lines().for_each(|line| {
            debug_log!("{line}");
        });
        return Err(Box::new(ThagError::Command(format!(
            "Cargo search failed for [{dep_crate}]"
        ))));
    };

    debug_log!("first_line={first_line}");
    let result = capture_dep(&first_line);
    let (name, version) = match result {
        Ok((name, version)) => {
            if name != dep_crate && name.replace('-', "_") != dep_crate {
                debug_log!("First line of cargo search for crate {dep_crate} found non-matching crate {name}");
                return Err(Box::new(ThagError::Command(format!(
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

/// Attempt to capture the dependency name and version from the first line returned by
/// Cargo from the search by dependency name.
/// # Errors
/// Will return `Err` if the first line does not match the expected crate name and a valid version number.
/// # Panics
/// Will panic if the regular expression is malformed.
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
        log!(Verbosity::Quieter, "Not a valid Cargo dependency format.");
        return Err(Box::new(ThagError::Command(
            "Not a valid Cargo dependency format".to_string(),
        )));
    };
    Ok((name, version))
}

/// Retrieve the default manifest from the `BuildState` instance.
/// # Errors
/// Will return `Err` if there is any error parsing the default manifest.
pub fn default_manifest_from_build_state(build_state: &BuildState) -> Result<Manifest, ThagError> {
    let source_stem = &build_state.source_stem;
    let source_name = &build_state.source_name;
    let binding = build_state.target_dir_path.join(source_name);
    let gen_src_path = escape_path_for_windows(binding.to_string_lossy().as_ref());

    default(source_stem, &gen_src_path)
}

/// Parse the default manifest from a string template.
/// # Errors
/// Will return `Err` if there is any error parsing the default manifest.
pub fn default(source_stem: &str, gen_src_path: &str) -> Result<Manifest, ThagError> {
    let cargo_manifest = format!(
        r##"[package]
name = "{}"
version = "0.0.1"
edition = "2021"

[dependencies]

[features]

[patch]

[workspace]

[[bin]]
name = "{}"
path = "{}"
edition = "2021"
"##,
        source_stem, source_stem, gen_src_path
    );

    // log!(Verbosity::Normal, "cargo_manifest=\n{cargo_manifest}");

    Ok(Manifest::from_str(&cargo_manifest)?)
}

/// Merge manifest data harvested from the source script and its optional embedded toml block
/// into the default manifest.
/// # Errors
/// Will return `Err` if there is any error parsing the default manifest.
pub fn merge(
    build_state: &mut BuildState,
    rs_source: &str,
    syntax_tree: &Option<Ast>,
) -> Result<(), Box<dyn Error>> {
    let start_merge_manifest = Instant::now();

    // Take ownership of the default manifest
    let default_cargo_manifest = default_manifest_from_build_state(build_state)?;
    let cargo_manifest = if let Some(manifest) = build_state.cargo_manifest.take() {
        manifest
    } else {
        default_cargo_manifest
    };

    let rs_inferred_deps = if let Some(ref syntax_tree) = syntax_tree {
        infer_deps_from_ast(syntax_tree)
    } else {
        infer_deps_from_source(rs_source)
    };

    debug_log!("build_state.rs_manifest={0:#?}\n", build_state.rs_manifest);

    let merged_manifest = if let Some(ref mut rs_manifest) = build_state.rs_manifest {
        if !rs_inferred_deps.is_empty() {
            debug_log!(
                "rs_dep_map (before inferred) {:#?}",
                rs_manifest.dependencies
            );
            search_deps(rs_inferred_deps, &mut rs_manifest.dependencies);
            debug_log!(
                "rs_dep_map (after inferred) {:#?}",
                rs_manifest.dependencies
            );
        }

        // Perform the merge with ownership of both manifests
        omerge(cargo_manifest, rs_manifest.clone())?
    } else {
        cargo_manifest
    };

    // Reassign the merged manifest back to build_state
    build_state.cargo_manifest = Some(merged_manifest);

    debug_timings(&start_merge_manifest, "Processed features");
    Ok(())
}

pub fn search_deps(rs_inferred_deps: Vec<String>, rs_dep_map: &mut BTreeMap<String, Dependency>) {
    for dep_name in rs_inferred_deps {
        if rs_dep_map.contains_key(&dep_name)
            || rs_dep_map.contains_key(&dep_name.replace('_', "-"))
            || ["crate", "macro_rules"].contains(&dep_name.as_str())
        {
            continue;
        }
        debug_log!("Starting Cargo search for key dep_name [{dep_name}]");
        let command_runner = RealCommandRunner;
        let cargo_search_result = cargo_search(&command_runner, &dep_name);
        // If the crate name is hyphenated, Cargo search will nicely search for underscore version and return the correct
        // hyphenated name. So we must replace the incorrect underscored version we searched on with the corrected
        // hyphenated version that the Cargo search returned.
        let (dep_name, dep) = if let Ok((dep_name, version)) = cargo_search_result {
            (dep_name, Dependency::Simple(version))
        } else {
            // return Err(Box::new(ThagError::Command(format!(
            //     "Cargo search couldn't find crate [{dep_name}]"
            // ))));
            log!(
                Verbosity::Quieter,
                "Cargo search couldn't find crate [{dep_name}]"
            );
            continue;
        };
        rs_dep_map.insert(dep_name, dep);
    }
    debug_log!("rs_dep_map (after inferred) = {rs_dep_map:#?}");
}
