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
use crate::toml_utils::{default_manifest, resolve_deps, CargoManifest};

const PACKAGE_DIR: &str = env!("CARGO_MANIFEST_DIR");
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

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

    let dur = start.elapsed();
    debug!(
        "Set up processing flags in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );

    let (source_stem, source_name, mut rs_manifest, rs_source): (
        String,
        String,
        CargoManifest,
        String,
    ) = code_utils::parse_source(&options)?;

    let cargo_manifest: CargoManifest =
        if proc_flags.intersects(ProcFlags::GEN_SRC | ProcFlags::GEN_TOML) {
            resolve_deps(&gen_build_dir, &source_stem, &rs_source, &mut rs_manifest)?
        } else {
            default_manifest(&gen_build_dir, &source_stem)?
        };

    // debug!("cargo_manifest= (after inferred) {cargo_manifest:#?}");

    let build_dir = PathBuf::from(".cargo/build_run");
    if !build_dir.exists() {
        fs::create_dir_all(&build_dir)?; // Use fs::create_dir_all for directories
    }

    // match options.action {
    if proc_flags.intersects(ProcFlags::GEN_SRC | ProcFlags::GEN_TOML) {
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
    let build_output = build_command.spawn()?.wait_with_output()?;

    if flags.intersects(ProcFlags::VERBOSE) {
        let output = String::from_utf8_lossy(&build_output.stdout);

        println!("Build output:");
        output.lines().for_each(|line| debug!("{line}"));
    }

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
