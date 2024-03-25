#![allow(clippy::uninlined_format_args)]
use crate::code_utils::get_proc_flags;
use crate::errors::BuildRunError;
use core::str;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::{fs, io::Write as OtherWrite}; // Use PathBuf for paths

use log::{debug, info, LevelFilter};

mod cmd_args;
// mod cmd_args_old;
mod code_utils;
mod errors;
mod toml_utils;

use crate::cmd_args::ProcFlags;
use crate::code_utils::{build_code_path, read_file_contents, rs_extract_src};
use crate::toml_utils::{cargo_search, default_manifest, rs_extract_manifest, Dependency};

const PACKAGE_DIR: &str = env!("CARGO_MANIFEST_DIR");
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    configure_log();

    let gen_build_dir = format!("{}/.cargo/{PACKAGE_NAME}", PACKAGE_DIR.to_owned());
    debug!("PACKAGE_DIR={PACKAGE_DIR}");
    debug!("PACKAGE_NAME={PACKAGE_NAME}");
    debug!("VERSION={VERSION}");
    debug!("gen_build_dir={gen_build_dir:?}",);

    let options = cmd_args::get_opt();
    let proc_flags = get_proc_flags(&options)?;
    debug!("flags={proc_flags:#?}");

    // let source_stem = "factorial_main"; // Replace with actual program name
    // let source_name = options.script.clone();

    let rs_suffix = ".rs";

    // let strip_suffix = &options
    //     .script
    //     .strip_suffix(rs_suffix)
    //     .ok_or_else(|| BuildRunError::NoneOption(String::from("Failed to strip .rs suffix")))?;

    // let (a, b) = (String::from(options.script.strip_suffix(rs_suffix).ok_or_else(|| BuildRunError::NoneOption(String::from("Failed to strip .rs suffix")))?), options.script);
    // let (c, d) = (options.script, options.script + rs_suffix);

    // let (source_stem, source_name) = if options.script.ends_with(rs_suffix) { (a, b) } else { (c, d) };

    debug!(
        "options.script={}; options.script.ends_with(rs_suffix)? {})",
        options.script,
        options.script.ends_with(rs_suffix)
    );

    let (source_stem, source_name) = if options.script.ends_with(rs_suffix) {
        let stem = String::from(options.script.strip_suffix(rs_suffix).ok_or_else(|| {
            BuildRunError::NoneOption(format!("Failed to strip {rs_suffix} suffix"))
        })?);
        (stem, options.script.clone())
    } else {
        (options.script.clone(), options.script.clone() + rs_suffix)
    };
    debug!("source_stem={source_stem}; source_name={source_name}");

    let code_path = build_code_path(&source_stem)?;
    let source_path = code_path.clone();
    // source_path.push(PathBuf::from_str(&source_name)?);
    debug!("code_path={code_path:?}");

    // Check it exists
    if !source_path.exists() {
        return Err(Box::new(BuildRunError::Command(format!(
            "No script named {source_stem} or {source_name} in path {code_path:?}"
        ))));
    }

    // let default_manifest = CargoManifest::default();
    // println!("default_manifest: {default_manifest:#?}");

    // let rs_toml_to_string = toml::to_string(&default_manifest)?;
    // println!("rs_toml_to_string: {rs_toml_to_string}");

    // Read manifest from source file
    // let _ = toml_utils::read_cargo_toml()?;

    // default_manifest.save_to_file(default_toml_path.to_str().ok_or("Missing path?")?)?;

    let rs_full_source = read_file_contents(&code_path)?;

    let mut cargo_manifest = default_manifest(&gen_build_dir, &source_stem)?;

    let mut rs_manifest = rs_extract_manifest(&rs_full_source)?;

    // cargo_manifest.dependencies
    // Depletes rs_manifest.bin, don't reuse
    // cargo_manifest.bin.append(&mut rs_manifest.bin);
    debug!("@@@@ cargo_manifest (before deps)={cargo_manifest:#?}");

    // Exclude the embedded cargo manifest information
    let rs_source = rs_extract_src(&rs_full_source);

    // Infer dependencies from imports
    let rs_deps = code_utils::infer_dependencies(&rs_source);
    debug!("rs_deps={rs_deps:#?\n}");

    if !rs_deps.is_empty() {
        let dep_map: &mut std::collections::BTreeMap<std::string::String, toml_utils::Dependency> =
            if let Some(Some(ref mut dep_map)) = rs_manifest.dependencies {
                dep_map
            } else {
                return Err(Box::new(BuildRunError::Command(String::from(
                    "No dependency map found",
                ))));
            };

        debug!("dep_map= (before inferred) {dep_map:?}");
        for dep_name in rs_deps {
            let cargo_search_result = cargo_search(&dep_name);
            if dep_map.contains_key(&dep_name) {
                // Already in manifest
                continue;
            }
            let dep = if let Ok((_dep_name, version)) = cargo_search_result {
                Dependency::Simple(version)
            } else {
                return Err(Box::new(BuildRunError::Command(format!(
                    "Cargo search couldn't find crate {dep_name}"
                ))));
            };
            dep_map.insert(dep_name, dep);
        }
        debug!("dep_map= (after inferred) {dep_map:?}");

        let manifest_deps = cargo_manifest
            .dependencies
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap();

        // Clone dependencies
        let mut manifest_deps_clone: std::collections::BTreeMap<
            std::string::String,
            toml_utils::Dependency,
        > = manifest_deps.clone();
        debug!("manifest_deps= (before inferred) {manifest_deps_clone:?}");

        // Insert any entries from source and inferred deps that are not already in default manifest
        dep_map
            .iter()
            .filter(|&(name, _dep)| !(manifest_deps.contains_key(name)))
            .for_each(|(key, value)| {
                manifest_deps_clone.insert(key.to_string(), value.clone());
            });
        cargo_manifest.dependencies = Some(Some(manifest_deps_clone));
        debug!(
            "cargo_manifest.dependencies= (after inferred) {:#?}",
            cargo_manifest.dependencies
        );
    }

    debug!("cargo_manifest= (after inferred) {cargo_manifest:#?}");

    let build_dir = PathBuf::from(".cargo/build_run");
    if !build_dir.exists() {
        fs::create_dir_all(&build_dir)?; // Use fs::create_dir_all for directories
    }

    // rs_manifest.package.name = source_stem.clone();
    // rs_manifest.edition = default_manifest.edition;
    // rs_manifest.bin.insert(
    //     0,
    //     Product {
    //         path: Some(format!("{gen_build_dir}/{source_name}")),
    //         name: Some(source_stem.clone()),
    //         required_features: None,
    //     },
    // );

    // let cargo_manifest = rs_manifest.to_string();
    // debug!("cargo_manifest={cargo_manifest}");

    // intersection
    let gen_either = ProcFlags::GEN_SRC | ProcFlags::GEN_TOML;
    // debug!(
    //     "flags.intersects(gen_either)?: {}",
    //     flags.intersects(gen_either)
    // );

    // let result: Result<(), errors::BuildRunError> =
    // Implement generate logic with optional verbose and timings
    // println!("Generating code (verbose: {}, timings: {})", verbose, timings);

    // match options.action {
    if proc_flags.intersects(gen_either) {
        generate(
            &proc_flags,
            &source_name,
            &rs_source,
            &cargo_manifest.to_string(),
            &build_dir,
        )?;
    }

    if proc_flags.intersects(ProcFlags::BUILD) {
        build(&proc_flags, &build_dir)?;
    }

    if proc_flags.intersects(ProcFlags::RUN) {
        run(&proc_flags, &source_stem, build_dir)?;
    }

    let dur = start.elapsed();
    debug!("Completed in {}.{}s", dur.as_secs(), dur.subsec_millis());

    Ok(())
}

// Configure log level
fn configure_log() {
    // let env = Env::new().filter("RUST_LOG"); //.default_write_style_or("auto");
    // let mut binding = Builder::new();
    // let builder = binding.parse_env(env);
    // builder.write_style(WriteStyle::Always);
    // builder.init();

    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();
}

fn generate(
    flags: &ProcFlags,
    source_name: &str,
    source: &str,
    cargo_manifest: &str,
    build_dir: &Path,
) -> Result<(), BuildRunError> {
    let start_gen = Instant::now();

    info!("In generate, flags={flags}");

    if flags.contains(ProcFlags::GEN_SRC) {
        let source_path = build_dir.join(source_name);
        let mut source_file = fs::File::create(&source_path)?;
        source_file.write_all(source.as_bytes())?;
        let relative_path = source_path;
        let mut absolute_path = PathBuf::from(PACKAGE_DIR); // std::env::current_dir()?.canonicalize()?;
                                                            // let absolute_path = absolute_path.canonicalize();
        absolute_path.push(relative_path);
        debug!("Absolute path of generated program: {absolute_path:?}");
        info!("##### Source code generation succeeded!");
    }

    if flags.contains(ProcFlags::GEN_TOML) {
        let cargo_toml_path = build_dir.join("Cargo.toml");

        info!("In generate of Cargo.toml, flags={flags}");

        // ? Don't overwrite Cargo.toml if not changed - see if it will remember it's compiled.
        // let prev_cargo_toml = read_file_contents(&cargo_toml_path)?;
        // if !cargo_manifest.eq(&prev_cargo_toml) {
        let mut cargo_toml = fs::File::create(&cargo_toml_path)?;

        OtherWrite::write_all(&mut cargo_toml, cargo_manifest.as_bytes())?;
        debug!("cargo_toml_path={cargo_toml_path:?}");
        info!("##### Cargo.toml generation succeeded!");
        // }
    }

    let dur = start_gen.elapsed();
    debug!(
        "Completed generation in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );
    if flags.contains(ProcFlags::TIMINGS) {
        println!(
            "Completed generation in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );
    }

    Ok(())
}

// Build the Rust program using Cargo (with manifest path)
fn build(flags: &ProcFlags, build_dir: &Path) -> Result<(), BuildRunError> {
    let start_build = Instant::now();
    let mut build_command = Command::new("cargo");
    build_command
        .args(["build", "--verbose"])
        .current_dir(build_dir);
    let build_output = build_command.output()?;
    if build_output.status.success() {
        let success_msg = String::from_utf8_lossy(&build_output.stdout);
        info!("##### Build succeeded!");
        success_msg.lines().for_each(|line| {
            debug!("{line}");
        });
    } else {
        let error_msg = String::from_utf8_lossy(&build_output.stderr);
        error_msg.lines().for_each(|line| {
            debug!("{line}");
        });
        return Err(BuildRunError::Command("Cargo build failed".to_string()));
    }

    let dur = start_build.elapsed();
    debug!(
        "Completed build in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );

    if flags.contains(ProcFlags::TIMINGS) {
        println!(
            "Completed build in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );
    }

    Ok(())
}

// Run the built program
fn run(flags: &ProcFlags, source_stem: &str, build_dir: PathBuf) -> Result<(), BuildRunError> {
    let start_run = Instant::now();

    let relative_path = format!("./target/debug/{source_stem}");
    let mut absolute_path = build_dir;
    absolute_path.push(relative_path);
    debug!("Absolute path of generated program: {absolute_path:?}");

    let mut run_command = Command::new(format!("{}", absolute_path.display()));
    debug!("Run command is {run_command:?}");

    let run_output = run_command.spawn()?.wait_with_output()?;

    if run_output.status.success() {
        let success_msg = String::from_utf8_lossy(&run_output.stdout);
        info!("##### Run succeeded!");
        success_msg.lines().for_each(|line| {
            debug!("{line}");
        });
    } else {
        let error_msg = String::from_utf8_lossy(&run_output.stderr);
        error_msg.lines().for_each(|line| {
            debug!("{line}");
        });
        return Err(BuildRunError::Command("Cargo run failed".to_string()));
    }

    let output = String::from_utf8_lossy(&run_output.stdout);

    println!("Build output:");
    output.lines().for_each(|line| debug!("{line}"));

    let dur = start_run.elapsed();
    debug!(
        "Completed run in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );

    if flags.contains(ProcFlags::TIMINGS) {
        println!(
            "Completed run in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );
    }

    Ok(())
}
