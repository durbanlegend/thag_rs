#![allow(clippy::uninlined_format_args)]
#[cfg(debug_assertions)]
use crate::debug_timings;
use crate::{
    code_utils::{get_source_path, infer_deps_from_ast, infer_deps_from_source},
    config::DependencyInference,
    get_verbosity,
}; // Valid if no circular dependency
use crate::{
    cvprtln, debug_log, maybe_config, regex, vlog, BuildState, Dependencies, Lvl, ThagResult, V,
};
use cargo_lookup::Query;
use cargo_toml::{Dependency, DependencyDetail, Manifest};
use firestorm::{profile_fn, profile_section};
// use nu_ansi_term::Style;
use regex::Regex;
use semver::VersionReq;
use serde_merge::omerge;

#[cfg(debug_assertions)]
use std::time::Instant;
use std::{collections::BTreeMap, path::PathBuf, str::FromStr};

#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn cargo_lookup(dep_crate: &str) -> Option<(String, String)> {
    profile_fn!(cargo_lookup);

    // Try both original and hyphenated versions
    let crate_variants = vec![dep_crate.to_string(), dep_crate.replace('_', "-")];

    for crate_name in crate_variants {
        let query: Query = match crate_name.parse() {
            Ok(q) => q,
            Err(e) => {
                debug_log!("Failed to parse query for crate {}: {}", crate_name, e);
                continue;
            }
        };

        match query.package() {
            Ok(package) => {
                debug_log!(
                    "Found package {} with {} releases",
                    package.name(),
                    package.releases().len()
                );

                // Request only stable versions (no pre-release)
                let req = VersionReq::parse("*").unwrap();

                // Log all available versions and their pre-release status
                // #[cfg(debug_assertions)]
                // for release in package.releases() {
                //     debug_log!(
                //         "Version {} {}",
                //         release.vers,
                //         if release.vers.pre.is_empty() {
                //             "(stable)"
                //         } else {
                //             "(pre-release)"
                //         }
                //     );
                // }

                let release = package.version(&req).filter(|r| r.vers.pre.is_empty());

                match release {
                    Some(r) => {
                        debug_log!("Selected stable version: {}", r.vers);
                        let name = r.name.clone();
                        let version = r.vers.to_string();

                        // Check if either variant matches
                        if name == dep_crate || name == dep_crate.replace('_', "-") {
                            return Some((name, version));
                        }
                    }
                    None => {
                        debug_log!("No stable version found for {}", crate_name);
                    }
                }
            }
            Err(e) => {
                debug_log!("Failed to look up crate {}: {}", crate_name, e);
                continue;
            }
        }
    }

    None
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
            #[cfg(debug_assertions)]
            debug_log!(
                "rs_dep_map (before inferred) {:#?}",
                rs_manifest.dependencies
            );
            lookup_deps(rs_inferred_deps, &mut rs_manifest.dependencies);

            #[cfg(debug_assertions)]
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
    // eprintln!("cargo_manifest={cargo_manifest:#?}, rs_manifest={rs_manifest:#?}");
    Ok(omerge(cargo_manifest, rs_manifest)?)
}

fn clean_features(features: Vec<String>) -> Vec<String> {
    profile_fn!(clean_features);
    let mut features: Vec<String> = features
        .into_iter()
        .filter(|f| !f.contains('/')) // Filter out features with slashes
        .collect();
    features.sort();
    features
}

fn get_crate_features(name: &str) -> Option<Vec<String>> {
    profile_fn!(get_crate_features);
    let query: Query = match name.parse() {
        Ok(q) => q,
        Err(e) => {
            debug_log!("Failed to parse query for crate {}: {}", name, e);
            return None;
        }
    };

    match query.package() {
        Ok(package) => {
            let latest = package.into_latest()?;

            // Collect features from both fields
            let mut all_features: Vec<String> = latest.features.keys().cloned().collect();

            // Add features2 if present
            if let Some(features2) = latest.features2 {
                all_features.extend(features2.keys().cloned());
            }

            if all_features.is_empty() {
                None
            } else {
                Some(clean_features(all_features))
            }
        }
        Err(e) => {
            debug_log!("Failed to get features for crate {}: {}", name, e);
            None
        }
    }
}

#[allow(clippy::missing_panics_doc)]
pub fn lookup_deps(rs_inferred_deps: Vec<String>, rs_dep_map: &mut BTreeMap<String, Dependency>) {
    profile_fn!(lookup_deps);

    #[cfg(debug_assertions)]
    eprintln!("In lookup_deps: rs_inferred_deps={rs_inferred_deps:#?}");
    if rs_inferred_deps.is_empty() {
        return;
    }

    let config = maybe_config();
    let binding = Dependencies::default();
    let dep_config = config.as_ref().map_or(&binding, |c| &c.dependencies);
    let inference_level = &dep_config.inference_level/*.unwrap_or_default() */;
    // let mut found_deps: std::vec::Vec<String> = Vec::new();
    eprintln!("inference_level={inference_level:#?}");

    for dep_name in rs_inferred_deps {
        if &dep_name == "thag_demo_proc_macros" {
            proc_macros_magic(rs_dep_map, &dep_name, "demo");
            continue;
        } else if &dep_name == "thag_bank_proc_macros" {
            proc_macros_magic(rs_dep_map, &dep_name, "bank");
            continue;
        }

        if let Some((name, version)) = cargo_lookup(&dep_name) {
            let features = get_crate_features(&name);

            match inference_level {
                DependencyInference::None => {
                    // Skip dependency entirely
                    continue;
                }
                DependencyInference::Minimal => {
                    // Just add basic dependency
                    rs_dep_map.insert(name.clone(), Dependency::Simple(version.clone()));
                }
                DependencyInference::Custom | DependencyInference::Maximal => {
                    // eprintln!("crate={name}, features.is_some()? {}", features.is_some());
                    if let Some(ref all_features) = features {
                        let features_for_inference_level = dep_config
                            .get_features_for_inference_level(&name, all_features, inference_level);
                        // eprintln!("features_for_inference_level={features_for_inference_level:#?}");
                        if let (Some(final_features), default_features) =
                            features_for_inference_level
                        {
                            // let detail = DependencyDetail {
                            //     version: Some(version.clone()),
                            //     features: final_features,
                            //     default_features,
                            //     ..Default::default()
                            // };
                            rs_dep_map.entry(name.clone()).or_insert_with(|| {
                                Dependency::Detailed(Box::new(DependencyDetail {
                                    version: Some(version.clone()),
                                    features: final_features,
                                    default_features,
                                    ..Default::default()
                                }))
                            });
                        }
                    }
                }
            }

            // Maybe show different toml blocks based on verbosity
            let verbosity = get_verbosity();
            if verbosity >= V::N {
                dbg!();
                show_all_toml_variants(&name, &version, features.as_ref(), dep_config);
            }
        }
    }
}

fn show_all_toml_variants(
    name: &str,
    version: &str,
    features: Option<&Vec<String>>,
    dep_config: &Dependencies,
) {
    profile_fn!(show_all_toml_variants);
    if let Some(all_features) = features {
        println!("\nAvailable dependency configurations for {}:", name);

        println!("\nMinimal:");
        println!("{} = \"{}\"\n", name, version);

        if let (Some(custom_features), default_features) = dep_config
            .get_features_for_inference_level(name, all_features, &DependencyInference::Custom)
        {
            let maybe_default_features = if default_features {
                ""
            } else {
                ", default-features = false "
            };
            println!("Custom (from config):");
            println!(
                "{} = {{ version = \"{}{maybe_default_features}\", features = [{}] }}\n",
                name,
                version,
                custom_features
                    .iter()
                    .map(|f| format!("\"{}\"", f))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if let (Some(max_features), default_features) = dep_config.get_features_for_inference_level(
            name,
            all_features,
            &DependencyInference::Maximal,
        ) {
            let maybe_default_features = if default_features {
                ""
            } else {
                ", default-features = false "
            };
            println!("Maximal:");
            println!(
                "{} = {{ version = \"{}{maybe_default_features}\", features = [{}] }}\n",
                name,
                version,
                max_features
                    .iter()
                    .map(|f| format!("\"{}\"", f))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
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
