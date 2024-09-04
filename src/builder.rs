use crate::code_utils::{
    self, build_loop, create_next_repl_file, create_temp_source_file, extract_ast_expr,
    extract_manifest, process_expr, read_file_contents, remove_inner_attributes,
    strip_curly_braces, wrap_snippet, write_source,
};
use crate::colors::{nu_resolve_style, MessageLevel};
use crate::config::{self, RealContext, MAYBE_CONFIG};
use crate::errors::ThagError;
use crate::log;
use crate::logging;
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
    debug_log, DYNAMIC_SUBDIR, FLOWER_BOX_LEN, PACKAGE_NAME, REPL_SUBDIR, RS_SUFFIX,
    TEMP_SCRIPT_NAME, TMPDIR,
};

use cargo_toml::Manifest;
#[cfg(debug_assertions)]
use env_logger::{Builder, Env, WriteStyle};
use firestorm::{profile_fn, profile_section};
use lazy_static::lazy_static;
#[cfg(debug_assertions)]
use log::{log_enabled, Level::Debug};
use regex::Regex;
use std::string::ToString;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

/// Execute the script runner.
/// # Errors
///
/// Will return `Err` if there is an error returned by any of the subordinate functions.
/// # Panics
/// Will panic if it fails to strip a .rs extension off the script name,
pub fn execute(args: &mut Cli) -> Result<(), ThagError> {
    // Instrument the entire function
    // profile_fn!(execute);

    let start = Instant::now();
    #[cfg(debug_assertions)]
    configure_log();

    let proc_flags = get_proc_flags(args)?;

    #[cfg(debug_assertions)]
    if log_enabled!(Debug) {
        log_init_setup(start, args, &proc_flags);
    }

    set_verbosity(args)?;

    if args.config {
        config::edit(&RealContext::new())?;
        return Ok(());
    }

    let is_repl = args.repl;
    let working_dir_path = if is_repl {
        TMPDIR.join(REPL_SUBDIR)
    } else {
        std::env::current_dir()?.canonicalize()?
    };
    validate_args(args, &proc_flags)?;
    let repl_source_path = if is_repl && args.script.is_none() {
        Some(create_next_repl_file()?)
    } else {
        None
    };
    let is_expr = proc_flags.contains(ProcFlags::EXPR);
    let is_stdin = proc_flags.contains(ProcFlags::STDIN);
    let is_edit = proc_flags.contains(ProcFlags::EDIT);
    let is_loop = proc_flags.contains(ProcFlags::LOOP);
    let is_dynamic = is_expr | is_stdin | is_edit | is_loop;
    if is_dynamic {
        let _ = create_temp_source_file()?;
    }

    let script_dir_path = resolve_script_dir_path(
        is_repl,
        args,
        &working_dir_path,
        &repl_source_path,
        is_dynamic,
    )?;

    let script_state =
        set_script_state(args, script_dir_path, is_repl, repl_source_path, is_dynamic)?;

    process(&proc_flags, args, &script_state, start)
}

#[inline]
fn set_verbosity(args: &Cli) -> Result<(), ThagError> {
    profile_fn!(set_verbosity);

    let verbosity = if args.verbose {
        Verbosity::Verbose
    } else if args.quiet == 1 {
        Verbosity::Quiet
    } else if args.quiet == 2 {
        Verbosity::Quieter
    } else if args.normal {
        Verbosity::Normal
    } else if let Some(config) = &*MAYBE_CONFIG {
        config.logging.default_verbosity
    } else {
        Verbosity::Normal
    };
    logging::set_global_verbosity(verbosity)
}

#[inline]
fn resolve_script_dir_path(
    is_repl: bool,
    args: &Cli,
    working_dir_path: &Path,
    repl_source_path: &Option<PathBuf>,
    is_dynamic: bool,
) -> Result<PathBuf, ThagError> {
    profile_fn!(resolve_script_dir_path);

    let script_dir_path = if is_repl {
        if let Some(ref script) = args.script {
            // REPL with repeat of named script
            let source_stem = script
                .strip_suffix(RS_SUFFIX)
                .ok_or("Failed to strip extension off the script name")?;
            working_dir_path.join(source_stem)
        } else {
            // Normal REPL with no script name
            repl_source_path
                .as_ref()
                .ok_or("Missing path of newly created REPL source file")?
                .parent()
                .ok_or("Could not find parent directory of repl source file")?
                .to_path_buf()
        }
    } else if is_dynamic {
        #[cfg(debug_assertions)]
        debug_log!("is_dynamic={is_dynamic}");
        TMPDIR.join(DYNAMIC_SUBDIR)
    } else {
        // Normal script file prepared beforehand
        let script = args
            .script
            .as_ref()
            .ok_or("Problem resolving script path")?;
        let script_path = PathBuf::from(script);
        let script_dir_path = script_path
            .parent()
            .ok_or("Problem resolving script parent path")?;
        script_dir_path.canonicalize()?
    };
    Ok(script_dir_path)
}

#[inline]
fn set_script_state(
    args: &Cli,
    script_dir_path: PathBuf,
    is_repl: bool,
    repl_source_path: Option<PathBuf>,
    is_dynamic: bool,
) -> Result<ScriptState, ThagError> {
    profile_fn!(set_script_state);
    let script_state: ScriptState = if let Some(ref script) = args.script {
        let script = script.to_owned();
        ScriptState::Named {
            script,
            script_dir_path,
        }
    } else if is_repl {
        let script = repl_source_path
            .ok_or("Missing newly created REPL source path")?
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
    Ok(script_state)
}

#[inline]
fn process(
    proc_flags: &ProcFlags,
    args: &mut Cli,
    script_state: &ScriptState,
    start: Instant,
) -> Result<(), ThagError> {
    // profile_fn!(process);
    let is_repl = args.repl;
    let is_expr = proc_flags.contains(ProcFlags::EXPR);
    let is_stdin = proc_flags.contains(ProcFlags::STDIN);
    let is_edit = proc_flags.contains(ProcFlags::EDIT);
    let is_loop = proc_flags.contains(ProcFlags::LOOP);
    let is_dynamic = is_expr | is_stdin | is_edit | is_loop;

    let mut build_state = BuildState::pre_configure(proc_flags, args, script_state)?;
    if is_repl {
        #[cfg(debug_assertions)]
        debug_log!("build_state.source_path={:?}", build_state.source_path);
        run_repl(args, proc_flags, &mut build_state, start)
    } else if is_dynamic {
        let rs_source = if is_expr {
            // Consumes the expression argument
            let Some(rs_source) = args.expression.take() else {
                return Err("Missing expression for --expr option".into());
            };
            rs_source
        } else if is_loop {
            // Consumes the filter argument
            let Some(filter) = args.filter.take() else {
                return Err("Missing expression for --loop option".into());
            };
            build_loop(args, filter)
        } else if is_edit {
            #[cfg(debug_assertions)]
            debug_log!("About to call stdin::edit()");
            let event_reader = CrosstermEventReader;
            let vec = edit(&event_reader)?;
            #[cfg(debug_assertions)]
            debug_log!("vec={vec:#?}");
            vec.join("\n")
        } else {
            assert!(is_stdin);
            #[cfg(debug_assertions)]
            debug_log!("About to call stdin::read())");
            let str = read()? + "\n";
            #[cfg(debug_assertions)]
            debug_log!("str={str}");
            str
        };

        log!(Verbosity::Verbose, "rs_source={rs_source}");

        let rs_manifest = extract_manifest(&rs_source, Instant::now())
            // .map_err(|_err| ThagError::FromStr("Error parsing rs_source"))
            ?;
        build_state.rs_manifest = Some(rs_manifest);

        debug_log!(
            r"About to try to parse following source to syn::Expr:
{rs_source}"
        );

        let expr_ast = extract_ast_expr(&rs_source)?;

        #[cfg(debug_assertions)]
        debug_log!("expr_ast={expr_ast:#?}");
        process_expr(
            expr_ast,
            &mut build_state,
            &rs_source,
            args,
            proc_flags,
            &start,
        )
    } else {
        gen_build_run(args, proc_flags, &mut build_state, None::<Ast>, &start)
    }
}

#[cfg(debug_assertions)]
fn log_init_setup(start: Instant, args: &Cli, proc_flags: &ProcFlags) {
    profile_fn!(log_init_setup);
    debug_log_config();
    #[cfg(debug_assertions)]
    debug_timings(&start, "Set up processing flags");
    debug_log!("proc_flags={proc_flags:#?}");

    if !&args.args.is_empty() {
        debug_log!("... args:");
        for arg in &args.args {
            debug_log!("{}", arg);
        }
    }
}

#[cfg(debug_assertions)]
fn debug_log_config() {
    profile_fn!(debug_log_config);
    debug_log!("PACKAGE_NAME={PACKAGE_NAME}");
    debug_log!("VERSION={VERSION}");
    debug_log!("REPL_SUBDIR={REPL_SUBDIR}");
}

// Configure log level
#[cfg(debug_assertions)]
fn configure_log() {
    profile_fn!(configure_log);
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
    args: &Cli,
    proc_flags: &ProcFlags,
    build_state: &mut BuildState,
    syntax_tree: Option<Ast>,
    start: &Instant,
) -> Result<(), ThagError> {
    // Instrument the entire function
    // profile_fn!(gen_build_run);

    if build_state.must_gen {
        let source_path: &Path = &build_state.source_path;
        let start_parsing_rs = Instant::now();
        let mut rs_source = read_file_contents(source_path)?;

        // Strip off any shebang: it may have got us here but we don't want or need it
        // in the gen_build_run process.
        rs_source = if rs_source.starts_with("#!") && !rs_source.starts_with("#![") {
            // #[cfg(debug_assertions)]
            // debug_log!("rs_source (before)={rs_source}");
            let split_once = rs_source.split_once('\n');
            #[allow(unused_variables)]
            let (shebang, rust_code) = split_once.ok_or("Failed to strip shebang")?;
            #[cfg(debug_assertions)]
            debug_log!("Successfully stripped shebang {shebang}");
            // #[cfg(debug_assertions)]
            // debug_log!("rs_source (after)={rust_code}");
            rust_code.to_string()
        } else {
            rs_source
        };

        // let mut rs_source = read_file_contents(&build_state.source_path)?;
        let mut syntax_tree: Option<Ast> = if syntax_tree.is_none() {
            code_utils::to_ast(&rs_source)
        } else {
            syntax_tree
        };

        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?m)^\s*(async\s+)?fn\s+main\s*\(\s*\)").unwrap();
        }
        let main_methods = syntax_tree.as_ref().map_or_else(
            || RE.find_iter(&rs_source).count(),
            code_utils::count_main_methods,
        );
        let has_main = match main_methods {
            0 => false,
            1 => true,
            _ => {
                if args.multimain {
                    true
                } else {
                    writeln!(
                    &mut std::io::stderr(),
                    "{main_methods} main methods found, only one allowed by default. Specify --multimain (-m) option to allow more"
                )?;
                    std::process::exit(1);
                }
            }
        };

        // NB build scripts that are well-formed programs from the original source.
        // Fun fact: Rust compiler will ignore shebangs:
        // https://neosmart.net/blog/self-compiling-rust-code/
        let is_file = syntax_tree.as_ref().map_or(false, Ast::is_file);
        build_state.build_from_orig_source = has_main && args.script.is_some() && is_file;

        debug_log!(
            "has_main={has_main}; build_state.build_from_orig_source={}",
            build_state.build_from_orig_source
        );

        let rs_manifest: Manifest = { extract_manifest(&rs_source, start_parsing_rs) }?;
        // #[cfg(debug_assertions)]
        // debug_log!("rs_manifest={rs_manifest:#?}");
        #[cfg(debug_assertions)]
        debug_log!("rs_source={rs_source}");
        if build_state.rs_manifest.is_none() {
            build_state.rs_manifest = Some(rs_manifest);
        }

        // #[cfg(debug_assertions)]
        // debug_log!("syntax_tree={syntax_tree:#?}");

        if build_state.rs_manifest.is_some() {
            manifest::merge(build_state, &rs_source, &syntax_tree)?;
        }

        // println!("build_state={build_state:#?}");
        rs_source = if has_main {
            // Strip off any enclosing braces, e.g.
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
                code_utils::extract_inner_attribs(&rs_source)
            } else {
                (String::new(), rs_source)
            };

            let rust_code = syntax_tree.as_ref().map_or(body, |syntax_tree_ref| {
                let returns_unit = match syntax_tree_ref {
                    Ast::Expr(expr) => code_utils::is_unit_return_type(expr),
                    Ast::File(_) => true, // Trivially true since we're here because there's no main
                };
                if returns_unit {
                    #[cfg(debug_assertions)]
                    debug_log!("Option B: returns unit type");
                    quote::quote!(
                        #syntax_tree_ref
                    )
                    .to_string()
                } else {
                    #[cfg(debug_assertions)]
                    debug_log!("Option A: returns a substantive type");
                    debug_log!(
                        "args.unquote={:?}, MAYBE_CONFIG={:?}",
                        args.unquote,
                        MAYBE_CONFIG
                    );

                    if proc_flags.contains(ProcFlags::UNQUOTE) {
                        debug_log!("\nIn unquote leg\n");
                        quote::quote!(
                            println!("{}", format!("{:?}", #syntax_tree_ref).trim_matches('"'));
                        )
                        .to_string()
                    } else {
                        debug_log!("\nIn quote leg\n");
                        quote::quote!(
                            println!("{}", format!("{:?}", #syntax_tree_ref));
                        )
                        .to_string()
                    }
                }
            });

            // display_timings(&start_quote, "Completed quote", proc_flags);
            wrap_snippet(&inner_attribs, &rust_code)
        };

        let maybe_rs_source = if has_main && build_state.build_from_orig_source {
            None
        } else {
            Some(rs_source.as_str())
        };
        generate(build_state, maybe_rs_source, proc_flags)?;
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
        run(proc_flags, &args.args, build_state)?;
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
    rs_source: Option<&str>,
    proc_flags: &ProcFlags,
) -> Result<(), ThagError> {
    // profile_fn!(generate);
    let start_gen = Instant::now();

    #[cfg(debug_assertions)]
    {
        debug_log!("In generate, proc_flags={proc_flags}");
        debug_log!(
            "build_state.target_dir_path={:#?}",
            build_state.target_dir_path
        );
    }

    if !build_state.target_dir_path.exists() {
        fs::create_dir_all(&build_state.target_dir_path)?;
    }

    let target_rs_path = build_state.target_dir_path.join(&build_state.source_name);
    // let is_repl = proc_flags.contains(ProcFlags::REPL);
    log!(
        Verbosity::Verbose,
        "GGGGGGGG Creating source file: {target_rs_path:?}"
    );

    if !build_state.build_from_orig_source {
        profile_section!(transform);
        let syntax_tree = syn_parse_file(rs_source)?;
        let rs_source = prettyplease_unparse(&syntax_tree);
        write_source(&target_rs_path, &rs_source)?;
    }

    // #[cfg(debug_assertions)]
    // debug_log!("cargo_toml_path will be {:?}", &build_state.cargo_toml_path);
    if !Path::try_exists(&build_state.cargo_toml_path)? {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&build_state.cargo_toml_path)?;
    }
    // #[cfg(debug_assertions)]
    // debug_log!("cargo_toml: {cargo_toml:?}");

    let manifest = &build_state
        .cargo_manifest
        .as_ref()
        .ok_or("Could not unwrap BuildState.cargo_manifest")?;
    let cargo_manifest_str: &str = &toml::to_string(manifest)?;

    #[cfg(debug_assertions)]
    debug_log!(
        "cargo_manifest_str: {}",
        code_utils::disentangle(cargo_manifest_str)
    );

    let mut toml_file = fs::File::create(&build_state.cargo_toml_path)?;
    toml_file.write_all(cargo_manifest_str.as_bytes())?;
    // #[cfg(debug_assertions)]
    // {
    //     debug_log!("cargo_toml_path={:?}", &build_state.cargo_toml_path);
    //     debug_log!("##### Cargo.toml generation succeeded");
    // }
    display_timings(&start_gen, "Completed generation", proc_flags);

    Ok(())
}

#[inline]
fn syn_parse_file(rs_source: Option<&str>) -> Result<syn::File, ThagError> {
    profile_fn!(syn_parse_file);
    let syntax_tree = syn::parse_file(rs_source.ok_or("Logic error retrieving rs_source")?)?;
    Ok(syntax_tree)
}

#[inline]
fn prettyplease_unparse(syntax_tree: &syn::File) -> String {
    profile_fn!(prettyplease_unparse);
    prettyplease::unparse(syntax_tree)
}

/// Build the Rust program using Cargo (with manifest path)
/// # Errors
/// Will return `Err` if there is an error composing the Cargo TOML path or running the Cargo build command.
/// # Panics
/// Will panic if the cargo build process fails to spawn or if it can't move the executable.
pub fn build(proc_flags: &ProcFlags, build_state: &BuildState) -> Result<(), ThagError> {
    // profile_fn!(build);

    let start_build = Instant::now();
    let quiet = proc_flags.contains(ProcFlags::QUIET);
    let quieter = proc_flags.contains(ProcFlags::QUIETER);
    let executable = proc_flags.contains(ProcFlags::EXECUTABLE);
    let check = proc_flags.contains(ProcFlags::CHECK);

    #[cfg(debug_assertions)]
    debug_log!("BBBBBBBB In build");

    let cargo_toml_path_str = code_utils::path_to_str(&build_state.cargo_toml_path)?;

    let mut cargo_command = Command::new("cargo");
    let cargo_subcommand = if check { "check" } else { "build" };
    // Rustc writes to std
    let mut args = vec![cargo_subcommand, "--manifest-path", &cargo_toml_path_str];
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
        // Pipe output
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
    let output = cargo_command.spawn()?;

    // Wait for the process to finish
    let exit_status = output.wait_with_output()?;

    if exit_status.status.success() {
        #[cfg(debug_assertions)]
        debug_log!("Build succeeded");
        if executable {
            deploy_executable(build_state)?;
        }
    } else {
        return Err("Build failed".into());
    };

    display_timings(&start_build, "Completed build", proc_flags);

    Ok(())
}

fn deploy_executable(build_state: &BuildState) -> Result<(), ThagError> {
    profile_fn!(deploy_executable);
    // Determine the output directory
    let mut cargo_bin_path = home::home_dir().ok_or("Could not find home directory")?;
    let cargo_bin_subdir = ".cargo/bin";
    cargo_bin_path.push(cargo_bin_subdir);

    // Create the target directory if it doesn't exist
    if !cargo_bin_path.exists() {
        fs::create_dir_all(&cargo_bin_path)?;
    }

    // Logic change: from accepting the first of multiple [[bin]] entries to only allowing exactly one.
    let name_option = build_state.cargo_manifest.as_ref().and_then(|manifest| {
        let mut iter = manifest
            .bin
            .iter()
            .filter_map(|p: &cargo_toml::Product| p.name.as_ref().map(ToString::to_string));

        match (iter.next(), iter.next()) {
            (Some(name), None) => Some(name), // Return Some(name) if exactly one name is found
            _ => None,                        // Return None if zero or multiple names are found
        }
    });

    #[allow(clippy::option_if_let_else)]
    let executable_name = if let Some(name) = name_option {
        name
    } else {
        #[cfg(target_os = "windows")]
        {
            format!("{}.exe", build_state.source_stem)
        }
        #[cfg(not(target_os = "windows"))]
        {
            build_state.source_stem.to_string()
        }
    };

    let executable_path = &build_state
        .target_dir_path
        .join("target/release")
        .join(&executable_name);
    let output_path = cargo_bin_path.join(&build_state.source_stem);
    debug_log!("executable_path={executable_path:#?}, output_path={output_path:#?}");
    fs::rename(executable_path, output_path)?;

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
) -> Result<(), ThagError> {
    // profile_fn!(run);

    let start_run = Instant::now();
    #[cfg(debug_assertions)]
    debug_log!("RRRRRRRR In run");

    // #[cfg(debug_assertions)]
    // debug_log!("BuildState={build_state:#?}");
    let target_path: &Path = build_state.target_path.as_ref();
    // #[cfg(debug_assertions)]
    // debug_log!("Absolute path of generated program: {absolute_path:?}");

    let mut run_command = Command::new(format!("{}", target_path.display()));

    run_command.args(args);

    // #[cfg(debug_assertions)]
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

    // #[cfg(debug_assertions)]
    // debug_log!("Exit status={exit_status:#?}");

    display_timings(&start_run, "Completed run", proc_flags);

    Ok(())
}
