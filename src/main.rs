use std::env;
use std::error::Error;
use std::process::Command;
use std::time::Instant;
use std::{fs, io::Write};

// use env_logger::Builder;
// use env_logger::{Env, WriteStyle};
use std::path::{Path, PathBuf}; // Use PathBuf for paths

use errors::BuildRunError;

use log::{debug, info, LevelFilter};

// use crate::cmd_args::{Action, Opt};
mod cmd_args;
mod errors;
mod toml_utils;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "build_run",
    about = "Build and run n given Rust programs, with separate and combined options for stages"
)]
struct Opt {
    #[structopt(subcommand)]
    action: Action,
    #[structopt(short, long, help = "Print help message")]
    help: bool,
    #[structopt(
        short = "C",
        long = "check-cargo",
        help = "Check for changes in Cargo.toml"
    )]
    check_cargo: bool,
    #[structopt(
        short = "S",
        long = "check-source",
        help = "Check for changes in source code"
    )]
    check_source: bool,
    #[structopt(short = "v", long = "verbose", help = "Enable verbose output")]
    verbose: bool,
    #[structopt(short = "t", long = "timings", help = "Print timings for each stage")]
    timings: bool,
}

#[derive(Debug, PartialEq, StructOpt)]
pub(crate) enum Action {
    #[structopt(
        name = "all",
        about = "Generate, build and run a Rust program from source code"
    )]
    All,
    #[structopt(name = "generate", about = "Generate Cargo.toml and source code")]
    Generate,
    #[structopt(name = "build", about = "Build the executable from generated code")]
    Build,
    #[structopt(
        name = "gen-and-build",
        about = "Generate Cargo.toml and source code, then build"
    )]
    GenAndBuild,
    #[structopt(name = "run", about = "Run the generated program (if already built)")]
    Run,
    #[structopt(name = "build-and-run", about = "Build and run the generated program")]
    BuildAndRun,
}

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
    // debug!("XYZ={:#?}", *XYZ);

    // Next: read manifest from source file?

    toml_utils::read_cargo_toml();

    let source_stem = "factorial_main"; // Replace with actual program name
    let source_name = format!("{source_stem}.rs");
    let project_dir = env::var("PWD")?; // Set during cargo build
    let project_path = PathBuf::from(project_dir);
    let mut code_path: PathBuf = project_path.join("examples");

    code_path.push(source_name);
    let source = read_file_contents(&code_path)?;

    let cargo_manifest = format!(
        r##"
    [package]
    name = "{source_stem}"
    version = "0.0.1"
    edition = "2021"

    [dependencies]
    rug = {{ version = "1.24.0", features = ["integer"] }}

    [workspace]

    [[bin]]
    name = "{source_stem}"
    path = "/Users/donf/projects/build_run/.cargo/build_run/tmp_source.rs"
    "##
    );

    let source: &str = &source;
    let cargo_manifest: &str = &cargo_manifest;
    let build_dir = PathBuf::from(".cargo/build_run");
    if !build_dir.exists() {
        fs::create_dir_all(&build_dir)?; // Use fs::create_dir_all for directories
    }

    let options = Opt::from_args();

    debug!("########options={options:?}");
    // println!("########options={options:?}");

    // options=Opt { action: Generate, help: false, check_cargo: false, check_source: false, verbose: false, timings: false }

    let verbose = options.verbose;
    let timings = options.timings;

    let result: Result<(), errors::BuildRunError> = match options.action {
        // Implement generate logic with optional verbose and timings
        // println!("Generating code (verbose: {}, timings: {})", verbose, timings);

        // match options.action {
        Action::All => {
            generate(source, cargo_manifest, &build_dir)?;
            build(&build_dir)?;
            run(source_stem, build_dir)
        } /* Generate code and Cargo.toml, then build */
        Action::Generate => generate(source, cargo_manifest, &build_dir),
        Action::Build => build(&build_dir),
        Action::GenAndBuild => {
            generate(source, cargo_manifest, &build_dir)?;
            build(&build_dir)
        } /* Generate code and Cargo.toml, then build */
        Action::Run => run(source_stem, build_dir),
        Action::BuildAndRun => {
            build(&build_dir)?;
            run(source_stem, build_dir)
        } /* Generate, build, and run */
    };

    match result {
        Ok(()) => {
            let dur = start.elapsed();
            debug!("Completed in {}.{}s", dur.as_secs(), dur.subsec_millis());
        }
        Err(ref error) => {
            println!("Error: {error}");
        }
    }
    // result?
    Ok(result?)
}

// // Handle options and execute commands based on user selection
// // let var_name =
// match options {
//     Opt.help => {
//         // Implement help message logic here
//         println!("Help message...");
//         // Ok(())
//     }
//     Opt::CheckCargo => {
//         // Implement cargo check logic here
//         println!("Checking Cargo.toml...");
//         // Ok(())
//     }
//     Opt::CheckSource => {
//         // Implement source code check logic here
//         println!("Checking source code...");
//         // Ok(())
//     }
//     Opt::Action(action) => {
//         let verbose = matches!(options, Opt::Verbose);
//         let timings = matches!(options, Opt::Timings);
//         let result: Result<(), errors::BuildRunError> = match action {
//             // Action::Generate => {
//             // Implement generate logic with optional verbose and timings
//             // println!("Generating code (verbose: {}, timings: {})", verbose, timings);

//             // match options.action {
//             Action::Generate => generate(source, cargo_manifest, &build_dir),
//             Action::Build => build(&build_dir),
//             Action::GenAndBuild => {
//                 generate(source, cargo_manifest, &build_dir)?;
//                 build(&build_dir)
//             } /* Generate code and Cargo.toml, then build */
//             Action::Run => run(source_stem, build_dir),
//             Action::BuildAndRun => {
//                 build(&build_dir)?;
//                 run(source_stem, build_dir)
//             } /* Generate, build, and run */
//         };

//         match result {
//             Ok(()) => {
//                 let dur = start.elapsed();
//                 debug!("Completed in {}.{}s", dur.as_secs(), dur.subsec_millis());
//             }
//             Err(ref error) => {
//                 println!("Error: {error}");
//             }
//         }
//         result?
//         // Ok(result?)
//     }
// };

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
    source: &str,
    cargo_manifest: &str,
    build_dir: &Path,
) -> Result<(), errors::BuildRunError> {
    let start_gen = Instant::now();
    let source_path = build_dir.join("tmp_source.rs");
    let mut source_file = fs::File::create(&source_path)?;
    source_file.write_all(source.as_bytes())?;
    let relative_path = source_path;
    let mut absolute_path = std::env::current_dir()?;
    absolute_path.push(relative_path);
    debug!("Absolute path of generated program: {absolute_path:?}");

    let cargo_toml_path = build_dir.join("Cargo.toml");

    // Don't overwrite Cargo.toml if not changed - see if it will remember it's compiled.
    let prev_cargo_toml = read_file_contents(&cargo_toml_path)?;
    if !cargo_manifest.eq(&prev_cargo_toml) {
        let mut cargo_toml = fs::File::create(&cargo_toml_path)?;

        cargo_toml.write_all(cargo_manifest.as_bytes())?;
        debug!("cargo_toml_path={cargo_toml_path:?}");
    }

    let dur = start_gen.elapsed();
    debug!(
        "Completed generation in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );

    Ok(())
}

// Build the Rust program using Cargo (with manifest path)
fn build(build_dir: &Path) -> Result<(), errors::BuildRunError> {
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
        return Err(errors::BuildRunError::Command(
            "Cargo build failed".to_string(),
        ));
    }

    let dur = start_build.elapsed();
    debug!(
        "Completed build in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );
    Ok(())
}

// Run the built program
fn run(source_stem: &str, build_dir: PathBuf) -> Result<(), errors::BuildRunError> {
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
        info!("##### Build succeeded!");
        success_msg.lines().for_each(|line| {
            debug!("{line}");
        });
    } else {
        let error_msg = String::from_utf8_lossy(&run_output.stderr);
        error_msg.lines().for_each(|line| {
            debug!("{line}");
        });
        return Err(errors::BuildRunError::Command(
            "Cargo run failed".to_string(),
        ));
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

    Ok(())
}

fn read_file_contents(path: &Path) -> Result<String, BuildRunError> {
    debug!("Reading from {path:?}");
    Ok(fs::read_to_string(path)?)
}
