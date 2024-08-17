use crate::code_utils::{
    self, build_loop, create_next_repl_file, create_temp_source_file, extract_ast,
    extract_manifest, process_expr, read_file_contents, remove_inner_attributes, rustfmt,
    strip_curly_braces, wrap_snippet, write_source,
};
use crate::colors::{nu_resolve_style, MessageLevel};
use crate::errors::BuildRunError;
use crate::log;
use crate::logging::Verbosity;
use crate::manifest;
use crate::repl::run_repl;
#[cfg(debug_assertions)]
use crate::shared::debug_timings;
use crate::shared::{display_timings, Ast, BuildState};
use crate::stdin::CrosstermEventReader;
use crate::stdin::{edit, read};
#[cfg(debug_assertions)]
use crate::VERSION;
use crate::{
    cmd_args::{get_proc_flags, validate_args, Cli, ProcFlags},
    ScriptState,
};
use crate::{
    debug_log, nu_color_println, DYNAMIC_SUBDIR, FLOWER_BOX_LEN, PACKAGE_NAME, REPL_SUBDIR,
    RS_SUFFIX, TEMP_SCRIPT_NAME, TMPDIR,
};

use cargo_toml::Manifest;
#[cfg(debug_assertions)]
use env_logger::{Builder, Env, WriteStyle};
use lazy_static::lazy_static;
#[cfg(debug_assertions)]
use log::{log_enabled, Level::Debug};
use regex::Regex;
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

/// Executes the script runner
/// # Errors
///
/// Will return `Err` if there is an error returned by any of the subordinate functions.
/// # Panics
/// Will panic if it fails to strip a .rs extension off the script name,
#[allow(clippy::too_many_lines)]
pub fn execute(mut args: Cli) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    #[cfg(debug_assertions)]
    configure_log();
    let proc_flags = get_proc_flags(&args)?;
    #[cfg(debug_assertions)]
    if log_enabled!(Debug) {
        debug_print_config();
        debug_timings(&start, "Set up processing flags");
        debug_log!("proc_flags={proc_flags:#?}");

        if !&args.args.is_empty() {
            debug_log!("... args:");
            for arg in &args.args {
                debug_log!("{}", arg);
            }
        }
    }
    let is_repl = proc_flags.contains(ProcFlags::REPL);
    let working_dir_path = if is_repl {
        TMPDIR.join(REPL_SUBDIR)
    } else {
        std::env::current_dir()?.canonicalize()?
    };
    validate_args(&args, &proc_flags)?;
    let repl_source_path = if is_repl && args.script.is_none() {
        Some(create_next_repl_file())
    } else {
        None
    };
    let is_expr = proc_flags.contains(ProcFlags::EXPR);
    let is_stdin = proc_flags.contains(ProcFlags::STDIN);
    let is_edit = proc_flags.contains(ProcFlags::EDIT);
    let is_loop = proc_flags.contains(ProcFlags::LOOP);
    let is_dynamic = is_expr | is_stdin | is_edit | is_loop;
    if is_dynamic {
        let _ = create_temp_source_file();
    }
    let script_dir_path = if is_repl {
        if let Some(ref script) = args.script {
            // REPL with repeat of named script
            let source_stem = script
                .strip_suffix(RS_SUFFIX)
                .expect("Failed to strip extension off the script name");
            working_dir_path.join(source_stem)
        } else {
            // Normal REPL with no script name
            repl_source_path
                .as_ref()
                .expect("Missing path of newly created REPL souece file")
                .parent()
                .expect("Could not find parent directory of repl source file")
                .to_path_buf()
        }
    } else if is_dynamic {
        debug_log!("is_dynamic={is_dynamic}");
        <std::path::PathBuf as std::convert::AsRef<Path>>::as_ref(&TMPDIR)
            .join(DYNAMIC_SUBDIR)
            .clone()
    } else {
        // Normal script file prepared beforehand
        let script = args.script.clone().expect("Problem resolving script path");
        let script_path = PathBuf::from(script);
        let script_dir_path = script_path
            .parent()
            .expect("Problem resolving script parent path");
        // working_dir_path.join(PathBuf::from(script.clone()))
        script_dir_path
            .canonicalize()
            .expect("Problem resolving script dir path")
    };

    let script_state: ScriptState = if let Some(ref script) = args.script {
        let script = script.to_owned();
        ScriptState::Named {
            script,
            script_dir_path,
        }
    } else if is_repl {
        let script = repl_source_path
            .expect("Missing newly created REPL source path")
            .display()
            .to_string();
        ScriptState::NamedEmpty {
            script,
            script_dir_path,
        }
    } else {
        assert!(is_dynamic);
        ScriptState::NamedEmpty {
            script: String::from(TEMP_SCRIPT_NAME),
            script_dir_path,
        }
    };
    let mut build_state = BuildState::pre_configure(&proc_flags, &args, &script_state)?;
    if is_repl {
        debug_log!("build_state.source_path={:?}", build_state.source_path);
        run_repl(&mut args, &proc_flags, &mut build_state, start)
    } else if is_dynamic {
        let rs_source = if is_expr {
            let Some(rs_source) = args.expression.clone() else {
                return Err(Box::new(BuildRunError::Command(
                    "Missing expression for --expr option".to_string(),
                )));
            };
            rs_source
        } else if is_loop {
            let Some(filter) = args.filter.clone() else {
                return Err(Box::new(BuildRunError::Command(
                    "Missing expression for --loop option".to_string(),
                )));
            };
            build_loop(&args, filter)
        } else if is_edit {
            debug_log!("About to call stdin::edit()");
            let event_reader = CrosstermEventReader;
            let vec = edit(&event_reader)?;
            debug_log!("vec={vec:#?}");
            vec.join("\n")
        } else {
            assert!(is_stdin);
            debug_log!("About to call stdin::read())");
            let str = read()? + "\n";
            debug_log!("str={str}");
            str
        };

        debug_log!("rs_source={rs_source}");

        let rs_manifest = extract_manifest(&rs_source, Instant::now())
            .map_err(|_err| BuildRunError::FromStr("Error parsing rs_source".to_string()))?;
        build_state.rs_manifest = Some(rs_manifest);

        let maybe_ast = extract_ast(&rs_source);

        if let Ok(expr_ast) = maybe_ast {
            debug_log!("expr_ast={expr_ast:#?}");
            process_expr(
                &expr_ast,
                &mut build_state,
                &rs_source,
                &mut args,
                &proc_flags,
                &start,
            )
        } else {
            nu_color_println!(
                nu_resolve_style(MessageLevel::Error),
                "Error parsing code: {:#?}",
                maybe_ast
            );
            Err(Box::new(BuildRunError::Command(
                "Error parsing code".to_string(),
            )))
        }
    } else {
        gen_build_run(
            &mut args,
            &proc_flags,
            &mut build_state,
            None::<Ast>,
            &start,
        )
    }
}

#[cfg(debug_assertions)]
fn debug_print_config() {
    debug_log!("PACKAGE_NAME={PACKAGE_NAME}");
    debug_log!("VERSION={VERSION}");
    debug_log!("REPL_SUBDIR={REPL_SUBDIR}");
}

// Configure log level
#[cfg(debug_assertions)]
fn configure_log() {
    let env = Env::new().filter("RUST_LOG"); //.default_write_style_or("auto");
    let mut binding = Builder::new();
    let builder = binding.parse_env(env);
    builder.write_style(WriteStyle::Always);
    let _ = builder.try_init();

    // Builder::new().filter_level(log::LevelFilter::Debug).init();
}

/// Generate, build and run the script or expression.
/// # Errors
///
/// Will return `Err` if there is an error returned by any of the generate, build or run functions.
///
/// # Panics
/// Will panic if it fails to parse the shebang, if any.
#[allow(clippy::too_many_lines)]
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

        // Strip off any shebang: it may have got us here but we don't need it
        // in the gen_build_run process.
        rs_source = if rs_source.starts_with("#!") && !rs_source.starts_with("#![") {
            // debug_log!("rs_source (before)={rs_source}");
            let split_once = rs_source.split_once('\n');
            let (shebang, rust_code) = split_once.expect("Failed to strip shebang");
            debug_log!("Successfully stripped shebang {shebang}");
            // debug_log!("rs_source (after)={rust_code}");
            rust_code.to_string()
        } else {
            rs_source
        };

        let rs_manifest: Manifest = {
            // debug_timings(&start_parsing_rs, "Parsed source");
            extract_manifest(&rs_source, start_parsing_rs)
        }?;
        // debug_log!("rs_manifest={rs_manifest:#?}");
        debug_log!("rs_source={rs_source}");
        if build_state.rs_manifest.is_none() {
            build_state.rs_manifest = Some(rs_manifest);
        }
        // let mut rs_source = read_file_contents(&build_state.source_path)?;
        let mut syntax_tree: Option<Ast> = if syntax_tree.is_none() {
            code_utils::to_ast(&rs_source)
        } else {
            syntax_tree
        };

        // debug_log!("syntax_tree={syntax_tree:#?}");

        if build_state.rs_manifest.is_some() {
            build_state.cargo_manifest =
                Some(manifest::merge(build_state, &rs_source, &syntax_tree)?);
        }

        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?m)^\s*(async\s+)?fn\s+main\s*\(\s*\)").unwrap();
        }
        let main_methods = match syntax_tree {
            Some(ref ast) => code_utils::count_main_methods(ast),
            None => RE.find_iter(&rs_source).count(),
        };
        let has_main = match main_methods {
            0 => false,
            1 => true,
            _ => {
                if options.multimain {
                    true
                } else {
                    writeln!(
                    &mut std::io::stderr(),
                    "{main_methods} main methods found, only one allowed by default. Specify --multimain (-m) option to allow more"
                )
                .unwrap();
                    std::process::exit(1);
                }
            }
        };

        // println!("build_state={build_state:#?}");
        rs_source = if has_main {
            // Strip off any enclosing braces we may have added
            if rs_source.starts_with('{') {
                strip_curly_braces(&rs_source).unwrap_or(rs_source)
            } else {
                rs_source
            }
        } else {
            // let start_quote = Instant::now();

            // Remove any inner attributes from the syntax tree
            let found = if let Some(Ast::Expr(syn::Expr::Block(ref mut expr_block))) = syntax_tree {
                // Apply the RemoveInnerAttributes visitor to the expression block
                remove_inner_attributes(expr_block)
            } else {
                false
            };

            let (inner_attribs, body) = if found {
                code_utils::prep_snippet(&rs_source)
            } else {
                (String::new(), rs_source)
            };

            let rust_code = if let Some(ref syntax_tree_ref) = syntax_tree {
                let returns_unit = match syntax_tree_ref {
                    Ast::Expr(expr) => code_utils::is_unit_return_type(expr),
                    Ast::File(file) => {
                        let expr = code_utils::extract_expr_from_file(file)
                            .expect("Error extracting syn::Expr from syn::File");
                        code_utils::returns_unit(&expr)
                    }
                };
                if returns_unit {
                    debug_log!("Option B: returns unit type");
                    quote::quote!(
                        #syntax_tree_ref
                    )
                    .to_string()
                } else {
                    debug_log!("Option A: returns a substantive type");
                    quote::quote!(#[macro_export]
                    macro_rules! println_trimmed {
                        ($val:expr) => {{
                                // Format the value using Debug
                                let formatted = format!("{:?}", $val);

                                // Trim surrounding double quotes if present
                                let trimmed = formatted.trim_matches('"');

                                // Print the result using Display
                                println!("{}", trimmed);
                            }};
                        }

                        println_trimmed!(#syntax_tree_ref);
                    )
                    .to_string()
                }
            } else {
                // demo/fizz_buzz.rs broke this: not an expression but still a valid snippet.
                // format!(r#"println!("Expression returned {{}}", {rs_source});"#)
                // debug_log!("dbg!(rs_source)={}", dbg!(rs_source.clone()));
                // dbg!(rs_source)
                body
            };
            // display_timings(&start_quote, "Completed quote", proc_flags);
            wrap_snippet(&inner_attribs, &rust_code)
        };
        generate(build_state, &rs_source, proc_flags)?;
    } else {
        log!(
            Verbosity::Normal,
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
        log!(
            Verbosity::Normal,
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
        PACKAGE_NAME,
        nu_resolve_style(MessageLevel::Emphasis).paint(&build_state.source_name)
    );
    display_timings(start, process, proc_flags);
    Ok(())
}

/// Generate the source code and Cargo.toml file for the script.
/// # Errors
///
/// Will return `Err` if there is an error creating the directory path, writing to the
/// target source or `Cargo.toml` file or formatting the source file with rustfmt.
///
/// # Panics
///
/// Will panic if it fails to unwrap the `BuildState.cargo_manifest`.
pub fn generate(
    build_state: &BuildState,
    rs_source: &str,
    proc_flags: &ProcFlags,
) -> Result<(), Box<dyn Error>> {
    let start_gen = Instant::now();

    debug_log!("In generate, proc_flags={proc_flags}");

    debug_log!(
        "build_state.target_dir_path={:#?}",
        build_state.target_dir_path
    );

    if !build_state.target_dir_path.exists() {
        fs::create_dir_all(&build_state.target_dir_path)?;
    }

    let target_rs_path = build_state
        .target_dir_path
        .clone()
        .join(&build_state.source_name);
    // let is_repl = proc_flags.contains(ProcFlags::REPL);
    log!(
        Verbosity::Verbose,
        "GGGGGGGG Creating source file: {target_rs_path:?}"
    );

    write_source(&target_rs_path, rs_source)?;
    rustfmt(build_state)?;

    // debug_log!("cargo_toml_path will be {:?}", &build_state.cargo_toml_path);
    if !Path::try_exists(&build_state.cargo_toml_path)? {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&build_state.cargo_toml_path)?;
    }
    // debug_log!("cargo_toml: {cargo_toml:?}");

    let manifest = &build_state
        .cargo_manifest
        .as_ref()
        .expect("Could not unwrap BuildState.cargo_manifest");
    let cargo_manifest_str: &str = &toml::to_string(manifest)?;

    debug_log!(
        "cargo_manifest_str: {}",
        code_utils::disentangle(cargo_manifest_str)
    );

    let mut toml_file = fs::File::create(&build_state.cargo_toml_path)?;
    toml_file.write_all(cargo_manifest_str.as_bytes())?;
    // debug_log!("cargo_toml_path={:?}", &build_state.cargo_toml_path);
    // debug_log!("##### Cargo.toml generation succeeded!");

    display_timings(&start_gen, "Completed generation", proc_flags);

    Ok(())
}

/// Build the Rust program using Cargo (with manifest path)
/// # Errors
/// Will return `Err` if there is an error composing the Cargo TOML path or running the Cargo build command.
/// # Panics
/// Will panic if the cargo build process fails to spawn or if it can't move the executable.
pub fn build(proc_flags: &ProcFlags, build_state: &BuildState) -> Result<(), BuildRunError> {
    let start_build = Instant::now();
    // let verbose = proc_flags.contains(ProcFlags::VERBOSE);
    let quiet = proc_flags.contains(ProcFlags::QUIET);
    let quieter = proc_flags.contains(ProcFlags::QUIETER);
    let executable = proc_flags.contains(ProcFlags::EXECUTABLE);
    let check = proc_flags.contains(ProcFlags::CHECK);

    debug_log!("BBBBBBBB In build");

    let Ok(cargo_toml_path_str) = code_utils::path_to_str(&build_state.cargo_toml_path) else {
        return Err(BuildRunError::OsString(
            build_state.cargo_toml_path.clone().into_os_string(),
        ));
    };

    let mut cargo_command = Command::new("cargo");
    let cargo_subcommand = if check { "check" } else { "build" };
    // Rustc writes to std
    let mut args = vec![cargo_subcommand, "--manifest-path", &cargo_toml_path_str];
    // if verbose {
    //     args.push("--verbose");
    // };
    if quiet || quieter {
        args.push("--quiet");
    }
    if executable {
        args.push("--release");
    }

    cargo_command.args(&args); // .current_dir(build_dir);

    // Show sign of life in case build takes a while
    log!(
        Verbosity::Normal,
        "{} {} ...",
        if check { "Checking" } else { "Building" },
        nu_resolve_style(MessageLevel::Emphasis).paint(&build_state.source_name)
    );

    if quieter {
        // Pipe output: TODO: debug
        cargo_command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
    } else {
        // Redirect stdout and stderr to inherit from the parent process (terminal)
        cargo_command
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());
    }

    // Execute the command and handle the result
    let output = cargo_command
        .spawn()
        .expect("failed to spawn cargo build process");

    // Wait for the process to finish
    let exit_status = output
        .wait_with_output()
        .expect("failed to wait on cargo build");

    if exit_status.status.success() {
        debug_log!("Build succeeded");
        if executable {
            // Determine the output directory
            let mut cargo_bin_path = home::home_dir().expect("Could not find home directory");
            let cargo_bin_subdir = ".cargo/bin";
            cargo_bin_path.push(cargo_bin_subdir);

            // Create the target directory if it doesn't exist
            if !cargo_bin_path.exists() {
                fs::create_dir_all(&cargo_bin_path).expect("Failed to create target directory");
            }

            let executable_name: String;
            #[cfg(windows)]
            {
                executable_name = format!("{}.exe", build_state.source_stem);
            }
            #[cfg(not(windows))]
            {
                executable_name = build_state.source_stem.to_string();
            }

            let executable_path = build_state
                .target_dir_path
                .clone()
                .join("target/release")
                .join(&executable_name);
            let output_path = cargo_bin_path.join(&build_state.source_stem);

            fs::rename(executable_path, output_path).expect("Failed to move the executable");

            let dash_line = "-".repeat(FLOWER_BOX_LEN);
            log!(
                Verbosity::Quiet,
                "{}",
                nu_ansi_term::Color::Yellow.paint(&dash_line)
            );

            log!(
                Verbosity::Quieter,
                "Executable built and moved to ~/{cargo_bin_subdir}/{executable_name}"
            );

            log!(
                Verbosity::Quiet,
                "{}",
                nu_ansi_term::Color::Yellow.paint(&dash_line)
            );
        }
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
    debug_log!("RRRRRRRR In run");

    // debug_log!("BuildState={build_state:#?}");
    let target_path = build_state.target_path.clone();
    // debug_log!("Absolute path of generated program: {absolute_path:?}");

    let mut run_command = Command::new(format!("{}", target_path.display()));
    run_command.args(args);

    debug_log!("Run command is {run_command:?}");

    // Sandwich command between two lines of dashes in the terminal

    let dash_line = "-".repeat(FLOWER_BOX_LEN);
    log!(
        Verbosity::Quiet,
        "{}",
        nu_ansi_term::Color::Yellow.paint(&dash_line)
    );

    let _exit_status = run_command.spawn()?.wait()?;

    log!(
        Verbosity::Quiet,
        "{}",
        nu_ansi_term::Color::Yellow.paint(&dash_line)
    );

    // debug_log!("Exit status={exit_status:#?}");

    display_timings(&start_run, "Completed run", proc_flags);

    Ok(())
}
