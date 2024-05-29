use log::debug;

use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
    time::Instant,
};

use crate::cmd_args::{Cli, ProcFlags};
use crate::code_utils::{
    self, extract_manifest, read_file_contents, rustfmt, strip_curly_braces, wrap_snippet,
    write_source,
};
use crate::errors::BuildRunError;
use crate::manifest;
use crate::shared::CargoManifest;
use crate::shared::{display_timings, Ast, BuildState};
use crate::FLOWER_BOX_LEN;
use crate::PACKAGE_NAME;

pub fn gen_build_run(
    options: &mut Cli,
    proc_flags: &ProcFlags,
    build_state: &mut BuildState,
    syntax_tree: Option<Ast>,
    start: &Instant,
) -> Result<(), Box<dyn Error>> {
    // let verbose = proc_flags.contains(ProcFlags::VERBOSE);
    let proc_flags = &proc_flags;
    let options = &options;

    if build_state.must_gen {
        let source_path: &Path = &build_state.source_path;
        let start_parsing_rs = Instant::now();
        let mut rs_source = read_file_contents(source_path)?;
        let rs_manifest: CargoManifest = {
            // debug_timings(&start_parsing_rs, "Parsed source");
            extract_manifest(&rs_source, start_parsing_rs)
        }?;
        debug!("&&&&&&&& rs_manifest={rs_manifest:#?}");
        // debug!("&&&&&&&& rs_source={rs_source}");
        if build_state.rs_manifest.is_none() {
            build_state.rs_manifest = Some(rs_manifest);
        }
        // let mut rs_source = read_file_contents(&build_state.source_path)?;
        let syntax_tree: Option<Ast> = if syntax_tree.is_none() {
            code_utils::to_ast(&rs_source)
        } else {
            syntax_tree
        };

        // debug!("syntax_tree={syntax_tree:#?}");

        if build_state.rs_manifest.is_some() {
            build_state.cargo_manifest = Some(manifest::merge_manifest(
                build_state,
                &rs_source,
                &syntax_tree,
            )?);
        }

        let has_main = if let Some(ref syntax_tree_ref) = syntax_tree {
            code_utils::has_main(syntax_tree_ref)
        } else {
            code_utils::has_main_alt(&rs_source)
        };

        // println!("######## build_state={build_state:#?}");
        rs_source = if has_main {
            // Strip off any enclosing braces we may have added
            if rs_source.starts_with('{') {
                strip_curly_braces(&rs_source).unwrap_or(rs_source)
            } else {
                rs_source
            }
        } else {
            // let start_quote = Instant::now();
            let rust_code = if let Some(ref syntax_tree_ref) = syntax_tree {
                quote::quote!(
                    // fn type_of<T>(_: T) -> &'static str {
                    //     std::any::type_name::<T>()
                    // }
                    //     println!("Type of expression is {}", type_of(#syntax_tree_ref));)
                    // .to_string()
                    // println!("Expression returned {:#?}", #syntax_tree_ref);)
                    // .to_string()
                    dbg!(#syntax_tree_ref);
                )
                .to_string()
            } else {
                // examples/fizz_buzz.rs broke this: not an expression but still a valid snippet.
                // format!(r#"println!("Expression returned {{}}", {rs_source});"#)
                debug!("dbg!(rs_source)={}", dbg!(rs_source.clone()));
                dbg!(rs_source)
            };
            // display_timings(&start_quote, "Completed quote", proc_flags);
            wrap_snippet(&rust_code.to_string())
        };
        generate(build_state, &rs_source, proc_flags)?;
    } else {
        println!(
            "{}",
            nu_ansi_term::Color::Yellow
                // .bold()
                .paint("Skipping unnecessary generation step.  Use --force (-f) to override.")
        );
        // build_state.cargo_manifest = Some(default_manifest(build_state)?);
        build_state.cargo_manifest = None; // Don't need it in memory, build will find it on disk
    }
    if build_state.must_build {
        build(proc_flags, build_state)?;
    } else {
        println!(
            "{}",
            nu_ansi_term::Color::Yellow
                // .bold()
                .paint("Skipping unnecessary cargo build step. Use --force (-f) to override.")
        );
    }
    if proc_flags.contains(ProcFlags::RUN) {
        run(proc_flags, &options.args, build_state)?;
    }
    let process = &format!(
        "{} completed processing script {}",
        PACKAGE_NAME, build_state.source_name
    );
    display_timings(start, process, proc_flags);
    Ok(())
}

/// # Errors
///
/// Will return `Err` if there is an error creating the directory path, writing to the
/// target source or `Cargo.toml` file or formatting the source file with rustfmt.
pub fn generate(
    build_state: &BuildState,
    rs_source: &str,
    proc_flags: &ProcFlags,
) -> Result<(), Box<dyn Error>> {
    let start_gen = Instant::now();
    let verbose = proc_flags.contains(ProcFlags::VERBOSE);

    debug!("In generate, proc_flags={proc_flags}");

    debug!(
        "build_state.target_dir_path={:#?}",
        build_state.target_dir_path
    );

    if !build_state.target_dir_path.exists() {
        fs::create_dir_all(&build_state.target_dir_path)?;
    }

    let target_rs_path = build_state.target_dir_path.clone();
    let target_rs_path = target_rs_path.join(&build_state.source_name);
    // let is_repl = proc_flags.contains(ProcFlags::REPL);
    if verbose {
        println!("GGGGGGGG Creating source file: {target_rs_path:?}");
    }
    write_source(&target_rs_path, rs_source)?;
    rustfmt(build_state)?;

    // debug!("cargo_toml_path will be {:?}", &build_state.cargo_toml_path);
    if !Path::try_exists(&build_state.cargo_toml_path)? {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&build_state.cargo_toml_path)?;
    }
    // debug!("cargo_toml: {cargo_toml:?}");

    let cargo_manifest_str: &str = &build_state.cargo_manifest.as_ref().unwrap().to_string();

    debug!(
        "cargo_manifest_str: {}",
        code_utils::disentangle(cargo_manifest_str)
    );

    let mut toml_file = fs::File::create(&build_state.cargo_toml_path)?;
    toml_file.write_all(cargo_manifest_str.as_bytes())?;
    // debug!("cargo_toml_path={:?}", &build_state.cargo_toml_path);
    // debug!("##### Cargo.toml generation succeeded!");

    display_timings(&start_gen, "Completed generation", proc_flags);

    Ok(())
}

/// Build the Rust program using Cargo (with manifest path)
/// # Panics
///
/// Will panic if the cargo build process fails to spawn.
pub fn build(proc_flags: &ProcFlags, build_state: &BuildState) -> Result<(), BuildRunError> {
    let start_build = Instant::now();
    // let verbose = proc_flags.contains(ProcFlags::VERBOSE);
    let quiet = proc_flags.contains(ProcFlags::QUIET);

    debug!("BBBBBBBB In build");

    let Ok(cargo_toml_path_str) = code_utils::path_to_str(&build_state.cargo_toml_path) else {
        return Err(BuildRunError::OsString(
            build_state.cargo_toml_path.clone().into_os_string(),
        ));
    };
    let mut build_command = Command::new("cargo");
    // Rustc writes to std
    let mut args = vec!["build", "--manifest-path", &cargo_toml_path_str];
    // if verbose {
    //     args.push("--verbose");
    // };
    if quiet {
        args.push("--quiet");
    }

    build_command.args(&args); // .current_dir(build_dir);

    // Show sign of life in case build takes a while
    eprintln!("Building...");

    // Redirect stdout and stderr to inherit from the parent process (terminal)
    build_command
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    // Execute the command and handle the result
    let output = build_command
        .spawn()
        .expect("failed to spawn cargo build process");

    // Wait for the process to finish
    let exit_status = output
        .wait_with_output()
        .expect("failed to wait on cargo build");

    if exit_status.status.success() {
        debug!("Build succeeded");
    } else {
        return Err(BuildRunError::Command(String::from("Build failed")));
    };

    display_timings(&start_build, "Completed build", proc_flags);

    Ok(())
}

/// Run the built program
/// # Errors
///
/// Will return `Err` if there is an error waiting for the spawned command
/// that runs the user script.
pub fn run(
    proc_flags: &ProcFlags,
    args: &[String],
    build_state: &BuildState,
) -> Result<(), BuildRunError> {
    let start_run = Instant::now();
    debug!("RRRRRRRR In run");

    // debug!("BuildState={build_state:#?}");
    let target_path = build_state.target_path.clone();
    // debug!("Absolute path of generated program: {absolute_path:?}");

    let mut run_command = Command::new(format!("{}", target_path.display()));
    run_command.args(args);

    debug!("Run command is {run_command:?}");

    // Sandwich command between two lines of dashes in the terminal

    let dash_line = "-".repeat(FLOWER_BOX_LEN);
    println!("{}", nu_ansi_term::Color::Yellow.paint(dash_line.clone()));

    let _exit_status = run_command.spawn()?.wait()?;

    println!("{}", nu_ansi_term::Color::Yellow.paint(dash_line.clone()));

    // debug!("Exit status={exit_status:#?}");

    display_timings(&start_run, "Completed run", proc_flags);

    Ok(())
}
