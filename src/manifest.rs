#![allow(clippy::uninlined_format_args)]
#[cfg(debug_assertions)]
use crate::debug_timings;
use crate::{
    ast, code_utils::get_source_path, config::DependencyInference, cvprtln, debug_log,
    get_verbosity, maybe_config, profile, profile_section, regex, vlog, Ast, BuildState,
    Dependencies, Lvl, ThagResult, BUILT_IN_CRATES, V,
};
use cargo_lookup::Query;
use cargo_toml::{Dependency, DependencyDetail, Edition, Manifest};
use nu_ansi_term::Style;
use regex::Regex;
use semver::VersionReq;
use serde_merge::omerge;
#[cfg(debug_assertions)]
use std::time::Instant;
use std::{
    collections::{BTreeMap, HashSet},
    ops::Deref,
    path::PathBuf,
    str::FromStr,
};
use syn::{parse_file, File};

#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn cargo_lookup(dep_crate: &str) -> Option<(String, String)> {
    profile!("cargo_lookup");

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
    profile!("capture_dep");

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
    profile!("configure_default");
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
    profile!("default");
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
    profile!("merge");
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

    profile_section!("infer_deps_and_merge");
    let rs_inferred_deps = if let Some(ref use_crates) = build_state.crates_finder {
        build_state.metadata_finder.as_ref().map_or_else(
            || infer_deps_from_source(rs_source),
            |metadata_finder| infer_deps_from_ast(use_crates, metadata_finder),
        )
    } else {
        infer_deps_from_source(rs_source)
    };

    // debug_log!("build_state.rs_manifest={0:#?}\n", build_state.rs_manifest);

    profile_section!("merge_manifest");
    let merged_manifest = if let Some(ref mut rs_manifest) = build_state.rs_manifest {
        if !rs_inferred_deps.is_empty() {
            #[cfg(debug_assertions)]
            debug_log!(
                "rs_dep_map (before inferred) {:#?}",
                rs_manifest.dependencies
            );
            lookup_deps(
                &build_state.infer,
                &rs_inferred_deps,
                &mut rs_manifest.dependencies,
            );

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

fn call_omerge(cargo_manifest: &Manifest, rs_manifest: &mut Manifest) -> ThagResult<Manifest> {
    profile!("call_omerge");
    // eprintln!("cargo_manifest={cargo_manifest:#?}, rs_manifest={rs_manifest:#?}");
    Ok(omerge(cargo_manifest, rs_manifest)?)
}

/// Identify use ... as statements for inclusion in / exclusion from Cargo.toml metadata.
///
/// Include the "from" name and exclude the "to" name.
/// Fallback version for when an abstract syntax tree cannot be parsed.
#[must_use]
pub fn find_use_renames_source(code: &str) -> (Vec<String>, Vec<String>) {
    profile!("find_use_renames_source");

    debug_log!("In code_utils::find_use_renames_source");
    let use_as_regex: &Regex = regex!(r"(?m)^\s*use\s+(\w+).*? as\s+(\w+)");

    let mut use_renames_from: Vec<String> = vec![];
    let mut use_renames_to: Vec<String> = vec![];

    for cap in use_as_regex.captures_iter(code) {
        let from_name = cap[1].to_string();
        let to_name = cap[2].to_string();

        debug_log!("use_rename: from={from_name}, to={to_name}");
        use_renames_from.push(from_name);
        use_renames_to.push(to_name);
    }

    use_renames_from.sort();
    use_renames_from.dedup();

    debug_log!("use_renames from source: from={use_renames_from:#?}, to={use_renames_to:#?}");
    (use_renames_from, use_renames_to)
}

/// Extract embedded Cargo.toml metadata from a Rust source string.
/// # Errors
/// Will return `Err` if there is any error in parsing the toml data into a manifest.
pub fn extract(
    rs_full_source: &str,
    #[allow(unused_variables)] start_parsing_rs: Instant,
) -> ThagResult<Manifest> {
    profile!("extract");
    let maybe_rs_toml = extract_toml_block(rs_full_source);

    profile_section!("parse_and_set_edition");
    let mut rs_manifest = if let Some(rs_toml_str) = maybe_rs_toml {
        // debug_log!("rs_toml_str={rs_toml_str}");
        Manifest::from_str(&rs_toml_str)?
    } else {
        Manifest::from_str("")?
    };

    {
        profile_section!("set_edition");
        if let Some(package) = rs_manifest.package.as_mut() {
            package.edition = cargo_toml::Inheritable::Set(Edition::E2021);
        }
    }

    // debug_log!("rs_manifest={rs_manifest:#?}");

    #[cfg(debug_assertions)]
    debug_timings(&start_parsing_rs, "extract_manifest parsed source");
    Ok(rs_manifest)
}

fn extract_toml_block(input: &str) -> Option<String> {
    profile!("extract_toml_block");
    let re: &Regex = regex!(r"(?s)/\*\[toml\](.*?)\*/");
    re.captures(input)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Infer dependencies from AST-derived metadata to put in a Cargo.toml.
#[must_use]
pub fn infer_deps_from_ast(
    crates_finder: &ast::CratesFinder,
    metadata_finder: &ast::MetadataFinder,
) -> Vec<String> {
    profile!("infer_deps_from_ast");
    let mut dependencies = vec![];
    dependencies.extend_from_slice(&crates_finder.crates);

    let to_remove: HashSet<String> = crates_finder
        .names_to_exclude
        .iter()
        .cloned()
        .chain(metadata_finder.names_to_exclude.iter().cloned())
        .chain(metadata_finder.mods_to_exclude.iter().cloned())
        .chain(BUILT_IN_CRATES.iter().map(Deref::deref).map(String::from))
        .collect();
    // eprintln!("to_remove={to_remove:#?}");

    dependencies.retain(|e| !to_remove.contains(e));
    // eprintln!("dependencies (after)={dependencies:#?}");

    // Similar check for other regex pattern
    for crate_name in &metadata_finder.extern_crates {
        if !&to_remove.contains(crate_name) {
            dependencies.push(crate_name.to_owned());
        }
    }

    // Deduplicate the list of dependencies
    dependencies.sort();
    dependencies.dedup();

    dependencies
}

/// Infer dependencies from source code to put in a Cargo.toml.
/// Fallback version for when an abstract syntax tree cannot be parsed.
#[must_use]
pub fn infer_deps_from_source(code: &str) -> Vec<String> {
    profile!("infer_deps_from_source");

    if code.trim().is_empty() {
        return vec![];
    }

    let maybe_ast = extract_and_wrap_uses(code);
    let mut dependencies = maybe_ast.map_or_else(
        |_| {
            cvprtln!(
                Lvl::ERR,
                V::QQ,
                "Could not parse code into an abstract syntax tree"
            );
            vec![]
        },
        |ast| {
            let crates_finder = ast::find_crates(&ast);
            let metadata_finder = ast::find_metadata(&ast);
            infer_deps_from_ast(&crates_finder, &metadata_finder)
        },
    );

    let macro_use_regex: &Regex = regex!(r"(?m)^[\s]*#\[macro_use\((\w+)\)");
    let extern_crate_regex: &Regex = regex!(r"(?m)^[\s]*extern\s+crate\s+([^;{]+)");

    let modules = find_modules_source(code);

    dependencies.retain(|e| !modules.contains(e));
    // eprintln!("dependencies (after)={dependencies:#?}");

    for cap in macro_use_regex.captures_iter(code) {
        let crate_name = cap[1].to_string();
        // eprintln!("macro-use crate_name={crate_name:#?}");
        if !modules.contains(&crate_name) {
            dependencies.push(crate_name);
        }
    }

    for cap in extern_crate_regex.captures_iter(code) {
        let crate_name = cap[1].to_string();
        // eprintln!("extern-crate crate_name={crate_name:#?}");
        if !modules.contains(&crate_name) {
            dependencies.push(crate_name);
        }
    }
    dependencies.sort();
    dependencies
}

/// Extract the `use` statements from source and parse them to a `syn::File` in order to
/// extract the dependencies..
///
/// # Errors
///
/// This function will return an error if `syn` fails to parse the `use` statements as a `syn::File`.
pub fn extract_and_wrap_uses(source: &str) -> Result<Ast, syn::Error> {
    profile!("extract_and_wrap_uses");
    // Step 1: Capture `use` statements
    let use_simple_regex: &Regex = regex!(r"(?m)(^\s*use\s+[^;{]+;\s*$)");
    let use_nested_regex: &Regex = regex!(r"(?ms)(^\s*use\s+\{.*\};\s*$)");

    let mut use_statements: Vec<String> = vec![];

    for cap in use_simple_regex.captures_iter(source) {
        let use_string = cap[1].to_string();
        use_statements.push(use_string);
    }
    for cap in use_nested_regex.captures_iter(source) {
        let use_string = cap[1].to_string();
        use_statements.push(use_string);
    }

    // Step 2: Parse as `syn::File`
    let ast: File = parse_file(&use_statements.join("\n"))?;
    // eprintln!("ast={ast:#?}");

    // Return wrapped in `Ast::File`
    Ok(Ast::File(ast))
}

fn clean_features(features: Vec<String>) -> Vec<String> {
    profile!("clean_features");
    let mut features: Vec<String> = features
        .into_iter()
        .filter(|f| !f.contains('/')) // Filter out features with slashes
        .collect();
    features.sort();
    features
}

fn get_crate_features(name: &str) -> Option<Vec<String>> {
    profile!("get_crate_features");
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
pub fn lookup_deps(
    inference_level: &DependencyInference,
    rs_inferred_deps: &[String],
    rs_dep_map: &mut BTreeMap<String, Dependency>,
) {
    profile!("lookup_deps");

    // #[cfg(debug_assertions)]
    // eprintln!("In lookup_deps: rs_inferred_deps={rs_inferred_deps:#?}");
    if rs_inferred_deps.is_empty() {
        return;
    }

    let existing_toml_block = !&rs_dep_map.is_empty();
    let mut new_inferred_deps: Vec<String> = vec![];
    let recomm_style = Style::from(&Lvl::SUBH);
    let recomm_inf_level = &DependencyInference::Config;
    let actual_style = if inference_level == recomm_inf_level {
        recomm_style
    } else {
        Style::from(&Lvl::EMPH)
    };
    let styled_inference_level = actual_style.paint(inference_level.to_string());
    let styled_recomm_inf_level = recomm_style.paint(recomm_inf_level.to_string());
    // Hack: use reset string \x1b[0m here to avoid mystery white-on-white bug.
    cvprtln!(
        &Lvl::NORM,
        V::V,
        "\x1b[0mRecommended dependency inference_level={styled_recomm_inf_level}, actual={styled_inference_level}"
    );

    let config = maybe_config();
    let binding = Dependencies::default();
    let dep_config = config.as_ref().map_or(&binding, |c| &c.dependencies);
    for dep_name in rs_inferred_deps {
        if dep_name == "thag_demo_proc_macros" {
            proc_macros_magic(rs_dep_map, dep_name, "demo");
            continue;
        } else if dep_name == "thag_bank_proc_macros" {
            proc_macros_magic(rs_dep_map, dep_name, "bank");
            continue;
        } else if rs_dep_map.contains_key(dep_name) {
            continue;
        }

        if let Some((name, version)) = cargo_lookup(dep_name) {
            if rs_dep_map.contains_key(&name) || rs_dep_map.contains_key(dep_name.as_str()) {
                continue;
            }
            // Only do it after lookup in case the found crate name has hyphens instead of the underscores it has in code.
            new_inferred_deps.push(name.clone());
            let features = get_crate_features(&name);

            match inference_level {
                DependencyInference::None => {
                    // Skip dependency entirely
                    continue;
                }
                DependencyInference::Min => {
                    // Just add basic dependency
                    insert_simple(rs_dep_map, name, version);
                }
                DependencyInference::Config | DependencyInference::Max => {
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
                        } else {
                            insert_simple(rs_dep_map, name, version);
                        }
                    } else {
                        insert_simple(rs_dep_map, name, version);
                    }
                }
            }
        }
    }
    // Not sure we need this
    // if rs_dep_map.is_empty() {
    //     return;
    // }

    if get_verbosity() < V::V
        || matches!(inference_level, DependencyInference::None)
        || new_inferred_deps.is_empty()
    {
        // No generated manifest info to show.
        return;
    }
    display_toml_info(
        existing_toml_block,
        &new_inferred_deps,
        rs_dep_map,
        inference_level,
    );
}

fn insert_simple(rs_dep_map: &mut BTreeMap<String, Dependency>, name: String, version: String) {
    rs_dep_map
        .entry(name)
        .or_insert_with(|| Dependency::Simple(version));
}

fn display_toml_info(
    existing_toml_block: bool,
    new_inferred_deps: &[String],
    rs_dep_map: &BTreeMap<String, Dependency>,
    inference_level: &DependencyInference,
) {
    let mut toml_block = String::new();
    if !existing_toml_block {
        toml_block.push_str("/*[toml]\n[dependencies]\n");
    }
    for dep_name in new_inferred_deps {
        // eprintln!("dep_name={dep_name}");
        let value = rs_dep_map.get(dep_name);
        match value {
            Some(Dependency::Simple(string)) => {
                let dep_line = format!("{dep_name} = \"{string}\"\n");
                toml_block.push_str(&dep_line);
            }
            Some(Dependency::Detailed(dep)) => {
                if dep.features.is_empty() {
                    let dep_line = format!(
                        "{dep_name} = \"{}\"\n",
                        dep.version
                            .as_ref()
                            .expect("Error unwrapping version for {dep_name}"),
                    );
                    toml_block.push_str(&dep_line);
                } else {
                    let maybe_default_features = if dep.default_features {
                        ""
                    } else {
                        ", default-features = false"
                    };
                    let dep_line = format!(
                        "{} = {{ version = \"{}\"{maybe_default_features}, features = [{}] }}\n",
                        dep_name,
                        dep.version
                            .as_ref()
                            .expect("Error unwrapping version for {dep_name}"),
                        dep.features
                            .iter()
                            .map(|f| format!("\"{}\"", f))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );

                    toml_block.push_str(&dep_line);
                }
            }
            Some(Dependency::Inherited(_)) | None => (),
        }
    }
    if !existing_toml_block {
        toml_block.push_str("*/");
    }
    let styled_toml_block = Style::from(&Lvl::SUBH).paint(&toml_block);
    let styled_inference_level = Style::from(&Lvl::EMPH).paint(inference_level.to_string());
    let wording = if existing_toml_block {
        format!("This is the {styled_inference_level} manifest information that was generated for this run. If you want to, you can merge it into the existing toml block at")
    } else {
        format!("This toml block contains the same {styled_inference_level} manifest information that was generated for this run. If you want to, you can copy it into")
    };
    vlog!(
        V::N,
        "\n{wording} the top of your script for faster execution in future:\n{styled_toml_block}\n"
    );
}

// fn show_all_toml_variants(
//     name: &str,
//     version: &str,
//     features: Option<&Vec<String>>,
//     dep_config: &Dependencies,
// ) {
//     profile!("show_all_toml_variants");
//     if let Some(all_features) = features {
//         println!("\nAvailable dependency configurations for {}:", name);

//         println!("\nMinimal:");
//         println!("{} = \"{}\"\n", name, version);

//         if let (Some(max_features), default_features) = dep_config.get_features_for_inference_level(
//             name,
//             all_features,
//             &DependencyInference::Maximal,
//         ) {
//             let maybe_default_features = if default_features {
//                 ""
//             } else {
//                 ", default-features = false "
//             };
//             println!("Maximal:");
//             println!(
//                 "{} = {{ version = \"{}{maybe_default_features}\", features = [{}] }}\n",
//                 name,
//                 version,
//                 max_features
//                     .iter()
//                     .map(|f| format!("\"{}\"", f))
//                     .collect::<Vec<_>>()
//                     .join(", ")
//             );
//         }
//     }
// }

fn proc_macros_magic(
    rs_dep_map: &mut BTreeMap<String, Dependency>,
    dep_name: &str,
    dir_name: &str,
) {
    profile!("proc_macros_magic");
    cvprtln!(
        Lvl::BRI,
        V::V,
        r#"Found magic import `{dep_name}`: attempting to generate path dependency from `proc_macros.(...)proc_macro_crate_path` in config file ".../config.toml"."#
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
            debug_log!("Found config.proc_macros={:#?}", config.proc_macros);
            if dep_name == "thag_demo_proc_macros" {
                config.proc_macros.demo_proc_macro_crate_path
            } else if dep_name == "thag_bank_proc_macros" {
                config.proc_macros.bank_proc_macro_crate_path
            } else {
                None
            }
        },
    );
    let magic_proc_macros_dir = maybe_magic_proc_macros_dir.as_ref().map_or_else(|| {
        cvprtln!(
            Lvl::BRI,
            V::V,
            r#"No `config.proc_macros.proc_macro_crate_path` in config file for "use {dep_name};": defaulting to "{default_proc_macros_dir}"."#
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

/// Identify mod statements for exclusion from Cargo.toml metadata.
/// Fallback version for when an abstract syntax tree cannot be parsed.
#[must_use]
pub fn find_modules_source(code: &str) -> Vec<String> {
    profile!("find_modules_source");
    let module_regex: &Regex = regex!(r"(?m)^[\s]*mod\s+([^;{\s]+)");
    debug_log!("In code_utils::find_use_renames_source");
    let mut modules: Vec<String> = vec![];
    for cap in module_regex.captures_iter(code) {
        let module = cap[1].to_string();
        debug_log!("module={module}");
        modules.push(module);
    }
    debug_log!("modules from source={modules:#?}");
    modules
}
