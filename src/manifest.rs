#![allow(clippy::uninlined_format_args)]
use crate::code_utils::{get_source_path, infer_deps_from_ast, infer_deps_from_source}; // Valid if no circular dependency
use crate::{
    cvprtln, debug_log, debug_timings, maybe_config, regex, vlog, BuildState, Lvl, ThagResult, V,
};
use cargo_toml::{Dependency, DependencyDetail, Manifest};
use firestorm::{profile_fn, profile_method};
use mockall::automock;
use nu_ansi_term::Style;
use regex::Regex;
use serde_merge::omerge;
use std::{
    collections::BTreeMap,
    io::{self, BufRead},
    path::PathBuf,
    process::{Command, Output},
    str::FromStr,
    time::Instant,
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
pub fn cargo_search<R: CommandRunner>(runner: &R, dep_crate: &str) -> ThagResult<(String, String)> {
    profile_fn!(cargo_search);
    let start_search = Instant::now();

    let dep_crate_styled = Style::from(&Lvl::EMPH).paint(dep_crate);
    vlog!(
        V::N,
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
            .ok_or_else(|| format!("Something went wrong with Cargo search for [{dep_crate}]"))?
    } else {
        #[allow(unused_variables)]
        let error_msg = String::from_utf8_lossy(&search_output.stderr);

        error_msg.lines().for_each(|line| {
            debug_log!("{line}");
        });
        return Err(format!("Cargo search failed for [{dep_crate}]").into());
    };

    debug_log!("first_line={first_line}");
    let result = capture_dep(&first_line);
    let (name, version) = match result {
        Ok((name, version)) => {
            if name != dep_crate && name.replace('-', "_") != dep_crate {
                debug_log!("First line of cargo search for crate {dep_crate} found non-matching crate {name}");
                return Err(format!(
                    "Cargo search failed for [{dep_crate}]: returned non-matching crate [{name}]"
                )
                .into());
            }

            let dep_crate_styled = Style::from(&Lvl::EMPH).paint(&name);
            let dep_version_styled = Style::from(&Lvl::EMPH).paint(&version);

            vlog!(
                V::N,
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

    let rs_inferred_deps = if let Some(ref use_crates) = build_state.crates_finder {
        build_state.metadata_finder.as_ref().map_or_else(
            || infer_deps_from_source(rs_source),
            |metadata_finder| infer_deps_from_ast(use_crates, metadata_finder),
        )
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

        omerge(&cargo_manifest, rs_manifest)?
    } else {
        cargo_manifest
    };

    // Reassign the merged manifest back to build_state
    build_state.cargo_manifest = Some(merged_manifest);

    debug_timings(&start_merge_manifest, "Processed features");
    Ok(())
}

#[allow(clippy::missing_panics_doc)]
pub fn search_deps(rs_inferred_deps: Vec<String>, rs_dep_map: &mut BTreeMap<String, Dependency>) {
    profile_fn!(search_deps);
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
        let cargo_search_result = cargo_search(&command_runner, &dep_name);
        // If the crate name is hyphenated, Cargo search will nicely search for underscore version and return the correct
        // hyphenated name. So we must replace the incorrect underscored version we searched on with the corrected
        // hyphenated version that the Cargo search returned.
        let (dep_name, dep) = if let Ok((dep_name, version)) = cargo_search_result {
            (dep_name, Dependency::Simple(version))
        } else {
            // return Err(format!(
            //     "Cargo search couldn't find crate [{dep_name}]").into()
            // );
            vlog!(V::QQ, "Cargo search couldn't find crate [{dep_name}]");
            continue;
        };
        rs_dep_map.insert(dep_name, dep);
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
