#![allow(clippy::uninlined_format_args)]
use crate::code_utils::get_proc_flags;
use crate::errors::BuildRunError;
use core::str;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::time::Instant;
use std::{fs, io::Write as OtherWrite}; // Use PathBuf for paths

use log::{debug, info};

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
const RS_SUFFIX: &str = ".rs";
pub(crate) const TOML_NAME: &str = "Cargo.toml";

#[derive(Debug, Default)]
pub(crate) struct BuildState {
    pub(crate) source_stem: String,
    pub(crate) source_name: String,
    pub(crate) cargo_manifest: CargoManifest,
    pub(crate) cargo_toml_path: PathBuf,
    pub(crate) source_path: PathBuf,
    // pub(crate) source_str: String,
    pub(crate) target_dir_path: PathBuf,
    pub(crate) target_dir_str: String,
}

//      TODO:
//       1.  Disallow scripts not named *.rs.
//       2.  Move generate method to code_utils? etc.
//       3.  snippets
//       4.  features
//       5.  bool -> 2-value enums?
//       6.  Generate source to same directory as Cargo.toml? Nah
//       7.  Rethink flags: -r just to run instead of -n and running by default
//       8.  Print a warning if no options chosen.

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    configure_log();

    debug!("PACKAGE_DIR={PACKAGE_DIR}");
    debug!("PACKAGE_NAME={PACKAGE_NAME}");
    debug!("VERSION={VERSION}");
    let gen_build_dir = format!("{PACKAGE_DIR}/.cargo/{PACKAGE_NAME}");
    debug!("gen_build_dir={gen_build_dir:?}");

    let options = cmd_args::get_opt();
    let proc_flags = get_proc_flags(&options)?;
    debug!("flags={proc_flags:#?}");

    let dur = start.elapsed();
    debug!(
        "Set up processing flags in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );

    debug!(
        "options.script={}; options.script.ends_with(RS_SUFFIX)? {})",
        options.script,
        options.script.ends_with(RS_SUFFIX)
    );

    let current_dir_path = std::env::current_dir()?.canonicalize()?;
    // let current_dir_str = path_to_str(&current_dir_path)?;

    let script_path = current_dir_path.join(PathBuf::from(options.script.clone()));
    debug!("script_path={script_path:#?}");
    let source_dir_path = script_path.canonicalize()?;
    debug!("source_dir_path={source_dir_path:#?}");
    // let source_dir_str = path_to_str(&source_dir_path)?;
    let target_dir_str = gen_build_dir.clone();
    let target_dir_path = PathBuf::from_str(&target_dir_str)?;
    let mut build_state = BuildState {
        source_path: source_dir_path,
        // source_str: source_dir_str,
        target_dir_path,
        target_dir_str,
        ..Default::default()
    };

    debug!("1. build_state={build_state:#?}");

    // info!("options.script={}", &options.script);
    // info!(
    //     "build_state.source_dir_path={:#?}",
    //     build_state.source_dir_path
    // );

    let path = Path::new(&options.script);
    let source_name = path.file_name().unwrap().to_str().unwrap();
    debug!("source_name = {source_name}");
    (build_state.source_stem, build_state.source_name) = if source_name.ends_with(RS_SUFFIX) {
        let Some(stem) = source_name.strip_suffix(RS_SUFFIX) else {
            return Err(Box::new(BuildRunError::Command(format!(
                "Error stripping suffix from {}",
                source_name
            ))));
        };
        (stem.to_string(), source_name.to_string())
    } else {
        (source_name.to_string(), source_name.to_string() + RS_SUFFIX)
    };

    let (source_stem, source_name) = (&build_state.source_stem, &build_state.source_name);
    // let code_path = code_utils::build_code_path(source_stem)?;
    let (mut rs_manifest, rs_source): (CargoManifest, String) =
        code_utils::parse_source(&build_state.source_path)?;

    let source_path = build_state.source_path.clone();
    // debug!("code_path={code_path:?}");
    if !source_path.exists() {
        return Err(Box::new(BuildRunError::Command(format!(
            "No script named {source_stem} or {source_name} in path {source_path:?}"
        ))));
    }

    build_state.cargo_manifest = if proc_flags.contains(ProcFlags::GENERATE) {
        resolve_deps(&gen_build_dir, source_stem, &rs_source, &mut rs_manifest)?
    } else {
        default_manifest(&gen_build_dir, source_stem)?
    };

    debug!("build_state (after inferred) = {build_state:#?}");

    // let build_dir = PathBuf::from(".cargo/build_run");
    // if !build_dir.exists() {
    //     fs::create_dir_all(&build_dir)?; // Use fs::create_dir_all for directories
    // }

    // let build_dir = build_dir.canonicalize()?;

    // let (source_path, cargo_toml_path) = {
    //     // let source: &str = &rs_source;
    //     // let source_name = format!("{source_stem}.rs");
    //     let source_path = build_dir.join(source_name);
    //     let cargo_toml_path = build_dir.join(toml_name);
    //     (source_path, cargo_toml_path)
    // };

    let cargo_toml_path = &build_state
        .target_dir_path
        .join(source_stem)
        .join(TOML_NAME)
        .clone();
    build_state.cargo_toml_path = cargo_toml_path.clone();
    debug!("3. build_state={build_state:#?}");
    if proc_flags.contains(ProcFlags::GENERATE) {
        generate(&build_state, &rs_source, &proc_flags)?;
    }

    // let build_dir_str = build_dir.to_str().ok_or_else(|| {
    //     BuildRunError::Command(format!("Failed to convert path {build_dir:#?} to a string"))
    // })?;

    // assert_eq!(
    //     build_dir_str, gen_build_dir,
    //     "build_dir_str != gen_build_dir"
    // );

    debug!("2. build_state={build_state:#?}");

    if proc_flags.contains(ProcFlags::BUILD) {
        build(&proc_flags, &build_state)?;
    }

    if proc_flags.contains(ProcFlags::RUN) {
        run(&proc_flags, source_stem, &build_state)?;
    }

    let dur = start.elapsed();
    debug!("Completed in {}.{}s", dur.as_secs(), dur.subsec_millis());
    if proc_flags.intersects(ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        println!("Completed in {}.{}s", dur.as_secs(), dur.subsec_millis());
    }

    Ok(())
}

fn generate(
    build_state: &BuildState,
    rs_source: &String,
    proc_flags: &ProcFlags,
) -> Result<(), Box<dyn Error>> {
    let start_gen = Instant::now();

    info!("In generate, proc_flags={proc_flags}");

    let target_rs_path = build_state.target_dir_path.clone();
    let target_rs_path = target_rs_path.join(&build_state.source_name);
    debug!("GGGGGGGG Creating source file: {target_rs_path:?}");
    let mut target_rs_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(target_rs_path)?;
    debug!("GGGGGGGG Done!");

    use std::fmt::Write;
    debug!(
        "Writing out source {}",
        rs_source
            .clone()
            .lines()
            .fold(String::new(), |mut output, b| {
                let _ = writeln!(output, "{b}");
                output
            })
    );

    target_rs_file.write_all(rs_source.as_bytes())?;

    // let relative_path = source_path;
    // let mut absolute_path = PathBuf::from(PACKAGE_DIR);
    // absolute_path.push(relative_path);
    // debug!("Absolute path of generated program: {absolute_path:?}");
    // info!("##### Source code generation succeeded!");

    debug!("cargo_toml_path will be {:?}", &build_state.cargo_toml_path);
    let cargo_toml_dir_path = &build_state.cargo_toml_path.parent().unwrap();
    fs::create_dir_all(cargo_toml_dir_path)?;
    if !Path::try_exists(&build_state.cargo_toml_path)? {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&build_state.cargo_toml_path)?;
    }
    // debug!("cargo_toml: {cargo_toml:?}");

    let cargo_manifest_str: &str = &build_state.cargo_manifest.to_string();

    debug!("cargo_manifest_str: {}", {
        use std::fmt::Write;
        cargo_manifest_str
            .lines()
            .fold(String::new(), |mut output, b| {
                let _ = writeln!(output, "{b}");
                output
            })
    });

    let mut toml_file = fs::File::create(&build_state.cargo_toml_path)?;
    toml_file.write_all(cargo_manifest_str.as_bytes())?;
    debug!("cargo_toml_path={:?}", &build_state.cargo_toml_path);
    info!("##### Cargo.toml generation succeeded!");

    let dur = start_gen.elapsed();
    debug!(
        "Completed generation in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );
    if proc_flags.intersects(ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        println!(
            "Completed generation in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );
    }
    Ok(())
}

// Configure log level
fn configure_log() {
    use env_logger::fmt::WriteStyle;
    use env_logger::Builder;
    use env_logger::Env;

    let env = Env::new().filter("RUST_LOG"); //.default_write_style_or("auto");
    let mut binding = Builder::new();
    let builder = binding.parse_env(env);
    builder.write_style(WriteStyle::Always);
    builder.init();

    // Builder::new().filter_level(log::LevelFilter::Debug).init();
}

/// Build the Rust program using Cargo (with manifest path)
fn build(proc_flags: &ProcFlags, build_state: &BuildState) -> Result<(), BuildRunError> {
    let start_build = Instant::now();
    let verbose = proc_flags.contains(ProcFlags::VERBOSE);

    debug!("BBBBBBBB In build!");

    let build_dir = PathBuf::from(build_state.target_dir_str.clone());
    let Ok(cargo_toml_path_str) = code_utils::path_to_str(&build_state.cargo_toml_path) else {
        return Err(BuildRunError::OsString(
            build_state.cargo_toml_path.clone().into_os_string(),
        ));
    };
    let mut build_command = Command::new("cargo");
    // Rustc writes to std
    let mut args = vec!["build", "--manifest-path", &cargo_toml_path_str];
    if verbose {
        args.push("-vv");
    };

    build_command.args(&args).current_dir(build_dir);

    debug!("&&&&&&&& build_command={build_command:#?}");

    let build_output = build_command.spawn()?.wait_with_output()?;

    if verbose {
        let stdout = String::from_utf8_lossy(&build_output.stdout);
        // TODO make println
        info!("Cargo output:");
        stdout.lines().for_each(|line| debug!("{line}"));

        let stderr = String::from_utf8_lossy(&build_output.stderr);
        // TODO make println
        debug!("Stderr output including rustc:");
        stderr.lines().for_each(|line| debug!("{line}"));
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

    if proc_flags.intersects(ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        println!(
            "Completed build in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );
    }

    Ok(())
}

// Run the built program
fn run(
    flags: &ProcFlags,
    source_stem: &str,
    build_state: &BuildState,
) -> Result<(), BuildRunError> {
    let start_run = Instant::now();

    let relative_path = format!("./target/debug/{source_stem}");
    let mut absolute_path = build_state.target_dir_path.clone();
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

    println!("Run output:");
    output.lines().for_each(|line| debug!("{line}"));

    let dur = start_run.elapsed();
    debug!(
        "Completed run in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );

    if flags.intersects(ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        println!(
            "Completed run in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );
    }

    Ok(())
}
