#![allow(clippy::uninlined_format_args)]
use crate::code_utils::{get_source_path, infer_deps_from_ast, infer_deps_from_source}; // Valid if no circular dependency
#[cfg(debug_assertions)]
use crate::debug_timings;
use crate::{cvprtln, debug_log, maybe_config, regex, vlog, BuildState, Lvl, ThagResult, V};
use cargo_toml::{Dependency, DependencyDetail, Manifest};
use firestorm::{profile_fn, profile_method, profile_section};
use mockall::automock;
use nu_ansi_term::Style;
use regex::Regex;
use serde_merge::omerge;
#[cfg(debug_assertions)]
use std::time::Instant;
use std::{
    collections::BTreeMap,
    io::{self, BufRead},
    path::PathBuf,
    process::{Command, Output},
    str::FromStr,
};

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
        profile_method!(run_command);
        Command::new(program).args(args).output()
    }
}

/// Attempt to find a matching dependency name and version from Cargo by searching by
/// crate name and inspecting the first line of Cargo's response.
/// # Errors
/// Will return `Err` if the first line does not match the expected crate name and a valid version number.
pub fn cargo_search<R: CommandRunner>(runner: &R, dep_crate: &str) -> Option<(String, String)> {
    profile_fn!(cargo_search);
    #[cfg(debug_assertions)]
    let start_search = Instant::now();

    let dep_crate_styled = Style::from(&Lvl::EMPH).paint(dep_crate);
    vlog!(
        V::N,
        "Doing a Cargo search for crate {dep_crate_styled} referenced in your script.",
    );

    let args = vec![
        "search".to_string(),
        dep_crate.to_string(),
        "--limit".to_string(),
        "1".to_string(),
    ];

    let search_output = match runner.run_command("cargo", &args) {
        Ok(output) => output,
        Err(_) => return None,
    };

    if !search_output.status.success() {
        #[allow(unused_variables)]
        let error_msg = String::from_utf8_lossy(&search_output.stderr);
        error_msg.lines().for_each(|line| {
            debug_log!("{line}");
        });
        return None;
    }

    let first_line = search_output.stdout.lines().map_while(Result::ok).next()?;

    debug_log!("first_line={first_line}");

    match capture_dep(&first_line) {
        Ok((name, version)) if name == dep_crate || name.replace('-', "_") == dep_crate => {
            #[cfg(debug_assertions)]
            debug_timings(&start_search, "Completed search");

            Some((name, version))
        }
        Ok((name, _)) => {
            debug_log!(
                "First line of cargo search for crate {dep_crate} found non-matching crate {name}"
            );
            None
        }
        Err(err) => {
            debug_log!("Failure! err={err}");
            None
        }
    }
}

/// Attempt to capture the dependency name and version from the first line returned by
/// Cargo from the search by dependency name.
/// # Errors
/// Will return `Err` if the first line does not match the expected crate name and a valid version number.
/// # Panics
/// Will panic if the regular expression is malformed.
pub fn capture_dep(first_line: &str) -> ThagResult<(String, String)> {
    profile_fn!(capture_dep);

    debug_log!("first_line={first_line}");
    let re: &Regex = regex!(r#"^(?P<name>[\w-]+) = "(?P<version>\d+\.\d+\.\d+)"#);

    let (name, version) = if re.is_match(first_line) {
        let captures = re.captures(first_line).unwrap();
        let name = captures.get(1).unwrap().as_str();
        let version = captures.get(2).unwrap().as_str();
        // vlog!(V::N, "Dependency name: {}", name);
        // vlog!(V::N, "Dependency version: {}", version);
        (String::from(name), String::from(version))
    } else {
        vlog!(V::QQ, "Not a valid Cargo dependency format.");
        return Err("Not a valid Cargo dependency format".into());
    };
    Ok((name, version))
}

/// Configure the default manifest from the `BuildState` instance.
/// # Errors
/// Will return `Err` if there is any error parsing the default manifest.
pub fn configure_default(build_state: &BuildState) -> ThagResult<Manifest> {
    profile_fn!(configure_default);
    let source_stem = &build_state.source_stem;

    let gen_src_path = get_source_path(build_state);

    debug_log!(
        r"build_state.build_from_orig_source={}
gen_src_path={gen_src_path}",
        build_state.build_from_orig_source
    );

    default(source_stem, &gen_src_path)
}

/// Parse the default manifest from a string template.
/// # Errors
/// Will return `Err` if there is any error parsing the default manifest.
pub fn default(source_stem: &str, gen_src_path: &str) -> ThagResult<Manifest> {
    profile_fn!(default);
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

    // vlog!(V::N, "cargo_manifest=\n{cargo_manifest}");

    Ok(Manifest::from_str(&cargo_manifest)?)
}

/// Merge manifest data harvested from the source script and its optional embedded toml block
/// into the default manifest.
/// # Errors
/// Will return `Err` if there is any error parsing the default manifest.
pub fn merge(build_state: &mut BuildState, rs_source: &str) -> ThagResult<()> {
    profile_fn!(merge);
    #[cfg(debug_assertions)]
    let start_merge_manifest = Instant::now();

    // Take ownership of the default manifest
    let default_cargo_manifest = configure_default(build_state)?;
    let cargo_manifest = build_state
        .cargo_manifest
        .take()
        .map_or(default_cargo_manifest, |manifest| manifest);

    // let rs_inferred_deps = syntax_tree
    //     .as_ref()
    //     .map_or_else(|| infer_deps_from_source(rs_source), infer_deps_from_ast);

    profile_section!(infer_deps_and_merge);
    let rs_inferred_deps = if let Some(ref use_crates) = build_state.crates_finder {
        build_state.metadata_finder.as_ref().map_or_else(
            || infer_deps_from_source(rs_source),
            |metadata_finder| infer_deps_from_ast(use_crates, metadata_finder),
        )
    } else {
        infer_deps_from_source(rs_source)
    };

    // debug_log!("build_state.rs_manifest={0:#?}\n", build_state.rs_manifest);

    profile_section!(merge_manifest);
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

        call_omerge(&cargo_manifest, rs_manifest)?
    } else {
        cargo_manifest
    };

    // Reassign the merged manifest back to build_state
    build_state.cargo_manifest = Some(merged_manifest);

    #[cfg(debug_assertions)]
    debug_timings(&start_merge_manifest, "Processed features");
    Ok(())
}

fn call_omerge(
    cargo_manifest: &Manifest,
    rs_manifest: &mut Manifest,
) -> Result<Manifest, crate::ThagError> {
    profile_fn!(call_omerge);
    Ok(omerge(cargo_manifest, rs_manifest)?)
}

struct CrateInfo {
    name: String,
    version: String,
    features: Option<Vec<String>>,
}

fn get_crate_features(name: &str) -> Option<Vec<String>> {
    let output = Command::new("cargo")
        .args(["lookup", name, "-t=features", "-f", "no-prefix"])
        .output()
        .ok()?;

    if output.status.success() {
        let features_str = String::from_utf8_lossy(&output.stdout);
        let features: Vec<String> = features_str
            .trim()
            .split_whitespace()
            .map(String::from)
            .collect();
        if features.is_empty() {
            None
        } else {
            Some(features)
        }
    } else {
        None
    }
}

#[allow(clippy::missing_panics_doc)]
pub fn search_deps(rs_inferred_deps: Vec<String>, rs_dep_map: &mut BTreeMap<String, Dependency>) {
    profile_fn!(search_deps);
    if rs_inferred_deps.is_empty() {
        return;
    }

    let mut found_deps = Vec::new();

    for dep_name in rs_inferred_deps {
        if rs_dep_map.contains_key(&dep_name)
            || rs_dep_map.contains_key(&dep_name.replace('_', "-"))
            || ["crate", "macro_rules"].contains(&dep_name.as_str())
        {
            continue;
        }

        if &dep_name == "thag_demo_proc_macros" {
            proc_macros_magic(rs_dep_map, &dep_name, "demo");
            continue;
        } else if &dep_name == "thag_bank_proc_macros" {
            proc_macros_magic(rs_dep_map, &dep_name, "bank");
            continue;
        }

        #[cfg(debug_assertions)]
        debug_log!("Starting Cargo search for key dep_name [{dep_name}]");
        let command_runner = RealCommandRunner;

        if let Some((name, version)) = cargo_search(&command_runner, &dep_name) {
            let features = get_crate_features(&name);
            found_deps.push(CrateInfo {
                name: name.clone(),
                version: version.clone(),
                features,
            });
            rs_dep_map.insert(name, Dependency::Simple(version));
        } else {
            vlog!(V::QQ, "Cargo search couldn't find crate [{dep_name}]");
        }
    }

    // Generate both simple and full-featured toml blocks if any dependencies were found
    if !found_deps.is_empty() {
        // Simple block
        let mut simple_block = String::from("/*[toml]\n[dependencies]\n");
        for dep in &found_deps {
            let dep_line = format!("{} = \"{}\"\n", dep.name, dep.version);
            simple_block.push_str(&dep_line);
        }
        simple_block.push_str("*/");

        // Full-featured block
        let mut featured_block = String::from("/*[toml]\n[dependencies]\n");
        for dep in &found_deps {
            if let Some(features) = &dep.features {
                let features_str = features
                    .iter()
                    .map(|f| format!("\"{}\"", f))
                    .collect::<Vec<_>>()
                    .join(", ");
                let dep_line = format!(
                    "{} = {{ version = \"{}\", features = [{}] }}\n",
                    dep.name, dep.version, features_str
                );
                featured_block.push_str(&dep_line);
            } else {
                // Use simple format for dependencies without features
                let dep_line = format!("{} = \"{}\"\n", dep.name, dep.version);
                featured_block.push_str(&dep_line);
            }
        }
        featured_block.push_str("*/");

        let styled_simple = Style::from(&Lvl::EMPH).paint(&simple_block);
        let styled_featured = Style::from(&Lvl::EMPH).paint(&featured_block);
        vlog!(
            V::N,
            "\nYou can copy one of the following toml blocks into your script:\n\nSimple version:\n{}\n\nFull-featured version:\n{}\n",
            styled_simple,
            styled_featured
        );
    }
}

fn proc_macros_magic(
    rs_dep_map: &mut BTreeMap<String, Dependency>,
    dep_name: &str,
    dir_name: &str,
) {
    profile_fn!(proc_macros_magic);
    cvprtln!(
        Lvl::BRI,
        V::V,
        r#"Found magic import `{dep_name}`: attempting to generate path dependency from proc_macros.proc_macro_crate_path in config file ".../config.toml"."#
    );
    let default_proc_macros_dir = format!("{dir_name}/proc_macros");
    let maybe_magic_proc_macros_dir = maybe_config().map_or_else(
        || {
            debug_log!(
                r#"Missing config file for "use {dep_name};", defaulting to "{dir_name}/proc_macros"."#
            );
            Some(default_proc_macros_dir.clone())
        },
        |config| {
            debug_log!("Found config.proc_macros()={:#?}", config.proc_macros);
            config.proc_macros.proc_macro_crate_path
        },
    );
    let magic_proc_macros_dir = maybe_magic_proc_macros_dir.as_ref().map_or_else(|| {
        cvprtln!(
            Lvl::BRI,
            V::V,
            r#"Missing `config.proc_macros.proc_macro_crate_path` in config file for "use {dep_name};": defaulting to "{default_proc_macros_dir}"."#
        );
        default_proc_macros_dir
    }, |proc_macros_dir| {
        cvprtln!(Lvl::BRI, V::V, "Found {proc_macros_dir:#?}.");
        proc_macros_dir.to_string()
    });

    let path = PathBuf::from_str(&magic_proc_macros_dir).unwrap();
    let path = if path.is_absolute() {
        path
    } else {
        path.canonicalize()
            .unwrap_or_else(|_| panic!("Could not canonicalize path {}", path.display()))
    };
    let dep = Dependency::Detailed(Box::new(DependencyDetail {
        path: Some(path.display().to_string()),
        ..Default::default()
    }));
    rs_dep_map.insert(dep_name.to_string(), dep);
}
