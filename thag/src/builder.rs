use crate::code_utils::{
    self, build_loop, create_temp_source_file, extract_manifest, get_source_path,
    read_file_contents, remove_inner_attributes, strip_curly_braces, wrap_snippet, write_source,
};
use crate::code_utils::{extract_ast_expr, to_ast};
use crate::colors::init_styles;
use crate::config::{self, RealContext};
#[cfg(debug_assertions)]
use crate::debug_timings;
use crate::logging::is_debug_logging_enabled;
use crate::repl::run_repl;
use crate::shared::{find_crates, find_metadata};
use crate::stdin::{edit, read};
#[cfg(debug_assertions)]
use crate::VERSION;
use crate::{
    coloring, cvprtln, debug_log, display_timings, get_proc_flags, manifest, maybe_config, regex,
    repeat_dash, validate_args, vlog, Ast, BuildResult, BuildState, Cli, CrosstermEventReader,
    Dependencies, Lvl, ProcFlags, ScriptState, DYNAMIC_SUBDIR, FLOWER_BOX_LEN, PACKAGE_NAME,
    REPL_SCRIPT_NAME, REPL_SUBDIR, RS_SUFFIX, TEMP_SCRIPT_NAME, TMPDIR, V,
};
use cargo_toml::Manifest;
#[cfg(debug_assertions)]
use log::{log_enabled, Level::Debug};
use nu_ansi_term::Style;
use regex::Regex;
use side_by_side_diff::create_side_by_side_diff;
use std::{
    env::current_dir,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    string::ToString,
    time::Instant,
};
use thag_core::{profile, profile_section};

/// Execute the script runner.
/// # Errors
///
/// Will return `Err` if there is an error returned by any of the subordinate functions.
/// # Panics
/// Will panic if it fails to strip a .rs extension off the script name,
pub fn execute(args: &mut Cli) -> BuildResult<()> {
    // Instrument the entire function
    // profile!("execute");

    let start = Instant::now();

    // Access lazy_static variables whose initialisation may have side-effects that could
    // affect the behaviour of the terminal, to get these out of the way. (Belt and braces.)
    let (maybe_color_support, term_theme) = coloring();

    let proc_flags = get_proc_flags(args)?;

    #[cfg(debug_assertions)]
    if log_enabled!(Debug) {
        log_init_setup(start, args, &proc_flags);
    }

    init_styles(term_theme, maybe_color_support);

    // set_verbosity(args)?;

    if args.config {
        config::edit(&RealContext::new())?;
        return Ok(());
    }

    let is_repl = args.repl;
    let working_dir_path = if is_repl {
        TMPDIR.join(REPL_SUBDIR)
    } else {
        current_dir()?.canonicalize()?
    };
    validate_args(args, &proc_flags)?;
    let repl_source_path = if is_repl && args.script.is_none() {
        // Some(create_next_repl_file()?)
        let gen_repl_temp_dir_path = TMPDIR.join(REPL_SUBDIR);
        debug_log!("repl_temp_dir = std::env::temp_dir() = {gen_repl_temp_dir_path:?}");

        // Ensure REPL subdirectory exists
        fs::create_dir_all(&gen_repl_temp_dir_path)?;

        // Create REPL file if necessary
        let path = gen_repl_temp_dir_path.join(REPL_SCRIPT_NAME);
        let _ = fs::File::create(&path)?;
        Some(path)
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
        repl_source_path.as_ref(),
        is_dynamic,
    )?;

    let script_state =
        set_script_state(args, script_dir_path, is_repl, repl_source_path, is_dynamic)?;

    process(&proc_flags, args, &script_state, start)
}

#[inline]
fn resolve_script_dir_path(
    is_repl: bool,
    args: &Cli,
    working_dir_path: &Path,
    repl_source_path: Option<&PathBuf>,
    is_dynamic: bool,
) -> BuildResult<PathBuf> {
    profile!("resolve_script_dir_path");

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
) -> BuildResult<ScriptState> {
    profile!("set_script_state");
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
) -> BuildResult<()> {
    // profile!("process");
    let is_repl = args.repl;
    let is_expr = proc_flags.contains(ProcFlags::EXPR);
    let is_stdin = proc_flags.contains(ProcFlags::STDIN);
    let is_edit = proc_flags.contains(ProcFlags::EDIT);
    let is_loop = proc_flags.contains(ProcFlags::LOOP);
    let is_dynamic = is_expr | is_stdin | is_edit | is_loop;

    let mut build_state = BuildState::pre_configure(proc_flags, args, script_state)?;
    if is_repl {
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
            debug_log!("About to call stdin::edit()");
            let event_reader = CrosstermEventReader;
            let vec = edit(&event_reader)?;
            debug_log!("vec={vec:#?}");
            if vec.is_empty() {
                // User chose Quit
                return Ok(());
            }
            vec.join("\n")
        } else {
            assert!(is_stdin);

            debug_log!("About to call stdin::read())");
            let str = read()? + "\n";

            debug_log!("str={str}");
            str
        };

        vlog!(V::V, "rs_source={rs_source}");

        let rs_manifest = extract_manifest(&rs_source, Instant::now())
            // .map_err(|_err| "Error parsing rs_source")
            ?;
        build_state.rs_manifest = Some(rs_manifest);

        debug_log!(
            r"About to try to parse following source to syn::Expr:
{rs_source}"
        );

        let expr_ast = extract_ast_expr(&rs_source)?;

        debug_log!("expr_ast={expr_ast:#?}");
        process_expr(&mut build_state, &rs_source, args, proc_flags, &start)
    } else {
        gen_build_run(args, proc_flags, &mut build_state, &start)
    }
}

/// Process a Rust expression
/// # Errors
/// Will return `Err` if there is any error encountered opening or writing to the file.
pub fn process_expr(
    build_state: &mut BuildState,
    rs_source: &str,
    args: &Cli,
    proc_flags: &ProcFlags,
    start: &Instant,
) -> BuildResult<()> {
    profile!("process_expr");
    // let syntax_tree = Some(Ast::Expr(expr_ast));
    write_source(&build_state.source_path, rs_source)?;
    let result = gen_build_run(args, proc_flags, build_state, start);
    vlog!(V::N, "{result:?}");
    Ok(())
}

#[cfg(debug_assertions)]
fn log_init_setup(start: Instant, args: &Cli, proc_flags: &ProcFlags) {
    profile!("log_init_setup");
    debug_log_config();
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
    profile!("debug_log_config");
    debug_log!("PACKAGE_NAME={PACKAGE_NAME}");
    debug_log!("VERSION={VERSION}");
    debug_log!("REPL_SUBDIR={REPL_SUBDIR}");
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
    start: &Instant,
) -> BuildResult<()> {
    // Instrument the entire function
    // profile!("gen_build_run");

    if build_state.must_gen {
        let source_path: &Path = &build_state.source_path;
        let start_parsing_rs = Instant::now();
        let mut rs_source = read_file_contents(source_path)?;

        // Strip off any shebang: it may have got us here but we don't want or need it
        // in the gen_build_run process.
        rs_source = if rs_source.starts_with("#!") && !rs_source.starts_with("#![") {
            // debug_log!("rs_source (before)={rs_source}");
            let split_once = rs_source.split_once('\n');
            #[allow(unused_variables)]
            let (shebang, rust_code) = split_once.ok_or("Failed to strip shebang")?;

            debug_log!("Successfully stripped shebang {shebang}");
            // debug_log!("rs_source (after)={rust_code}");
            rust_code.to_string()
        } else {
            rs_source
        };

        // let sourch_path_string = source_path.display().to_string();
        let sourch_path_string = source_path.to_string_lossy();
        // let mut rs_source = read_file_contents(&build_state.source_path)?;
        if build_state.ast.is_none() {
            build_state.ast = to_ast(&sourch_path_string, &rs_source);
        }
        if let Some(ref ast) = build_state.ast {
            build_state.crates_finder = Some(find_crates(ast));
            build_state.metadata_finder = Some(find_metadata(ast));
        }

        let metadata_finder = build_state.metadata_finder.as_ref();
        let main_methods = metadata_finder.map_or_else(
            || {
                let re: &Regex = regex!(r"(?m)^\s*(async\s+)?fn\s+main\s*\(\s*\)");
                re.find_iter(&rs_source).count()
            },
            |metadata_finder| metadata_finder.main_count,
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
        let is_file = build_state.ast.as_ref().map_or(false, Ast::is_file);
        build_state.build_from_orig_source = has_main && args.script.is_some() && is_file;

        debug_log!(
            "has_main={has_main}; build_state.build_from_orig_source={}",
            build_state.build_from_orig_source
        );

        let rs_manifest: Manifest = { extract_manifest(&rs_source, start_parsing_rs) }?;

        // debug_log!("rs_manifest={rs_manifest:#?}");

        debug_log!("rs_source={rs_source}");
        if build_state.rs_manifest.is_none() {
            build_state.rs_manifest = Some(rs_manifest);
        }

        // debug_log!("syntax_tree={syntax_tree:#?}");

        if build_state.rs_manifest.is_some() {
            manifest::merge(build_state, &rs_source)?;
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
            let found =
                if let Some(Ast::Expr(syn::Expr::Block(ref mut expr_block))) = build_state.ast {
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

            let rust_code = build_state.ast.as_ref().map_or(body, |syntax_tree_ref| {
                let returns_unit = match syntax_tree_ref {
                    Ast::Expr(expr) => code_utils::is_unit_return_type(expr),
                    Ast::File(_) => true, // Trivially true since we're here because there's no main
                };
                if returns_unit {
                    debug_log!("Option B: returns unit type");
                    quote::quote!(
                        #syntax_tree_ref
                    )
                    .to_string()
                } else {
                    debug_log!("Option A: returns a substantive type");
                    debug_log!(
                        "args.unquote={:?}, MAYBE_CONFIG={:?}",
                        args.unquote,
                        maybe_config()
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
        cvprtln!(
            Lvl::EMPH,
            V::N,
            "Skipping unnecessary generation step.  Use --force (-f) to override."
        );
        // build_state.cargo_manifest = Some(default_manifest(build_state)?);
        build_state.cargo_manifest = None; // Don't need it in memory, build will find it on disk
    }
    if build_state.must_build {
        build(proc_flags, build_state)?;
    } else {
        let build_qualifier =
            if proc_flags.contains(ProcFlags::NORUN) && !proc_flags.contains(ProcFlags::BUILD) {
                "Skipping cargo build step because --gen specified without --build."
            } else {
                "Skipping unnecessary cargo build step. Use --force (-f) to override."
            };
        cvprtln!(Lvl::EMPH, V::N, "{build_qualifier}");
    }
    if proc_flags.contains(ProcFlags::RUN) {
        run(proc_flags, &args.args, build_state)?;
    }
    let process = &format!(
        "{} completed processing script {}",
        PACKAGE_NAME,
        Style::from(&Lvl::EMPH).paint(&build_state.source_name)
    );
    display_timings(start, process, proc_flags);
    Ok(())
}

/// Generate the source code and Cargo.toml file for the script.
/// # Errors
///
/// Will return `Err` if there is an error creating the directory path, writing to the
/// target source or `Cargo.toml` file or formatting the source file with `prettyplease`.
///
/// # Panics
///
/// Will panic if it fails to unwrap the `BuildState.cargo_manifest`.
pub fn generate(
    build_state: &BuildState,
    rs_source: Option<&str>,
    proc_flags: &ProcFlags,
) -> BuildResult<()> {
    profile!("generate");
    let start_gen = Instant::now();

    if is_debug_logging_enabled() {
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
    vlog!(V::V, "GGGGGGGG Creating source file: {target_rs_path:?}");

    if !build_state.build_from_orig_source {
        profile_section!("transform_snippet");
        // TODO make this configurable
        let rs_source: &str = {
            #[cfg(feature = "format_snippet")]
            {
                let syntax_tree = syn_parse_file(rs_source)?;
                prettyplease_unparse(&syntax_tree)
            }
            #[cfg(not(feature = "format_snippet"))]
            rs_source.expect("Logic error retrieving rs_source")
        };

        write_source(&target_rs_path, rs_source)?;
    }

    // Remove any existing Cargo.lock as this may raise spurious compatibility issues with new dependency versions.
    let lock_path = &build_state.target_dir_path.join("Cargo.lock");
    // eprintln!("Lock path {lock_path:?} exists? - {}", lock_path.exists());
    if lock_path.exists() {
        fs::remove_file(lock_path)?;
    }

    let manifest = &build_state
        .cargo_manifest
        .as_ref()
        .ok_or("Could not unwrap BuildState.cargo_manifest")?;
    let cargo_manifest_str: &str = &toml::to_string(manifest)?;

    debug_log!(
        "cargo_manifest_str: {}",
        code_utils::disentangle(cargo_manifest_str)
    );

    // Create or truncate the Cargo.toml file and write the content
    let mut toml_file = OpenOptions::new()
        .write(true)
        .create(true) // Creates the file if it doesn't exist
        .truncate(true) // Ensures the file is emptied if it exists
        .open(&build_state.cargo_toml_path)?;

    toml_file.write_all(cargo_manifest_str.as_bytes())?;
    display_timings(&start_gen, "Completed generation", proc_flags);

    Ok(())
}

#[inline]
#[cfg(feature = "format_snippet")]
fn syn_parse_file(rs_source: Option<&str>) -> BuildResult<syn::File> {
    profile!("syn_parse_file");
    let syntax_tree = syn::parse_file(rs_source.ok_or("Logic error retrieving rs_source")?)?;
    Ok(syntax_tree)
}

#[inline]
#[cfg(feature = "format_snippet")]
fn prettyplease_unparse(syntax_tree: &syn::File) -> String {
    profile!("prettyplease_unparse");
    prettyplease::unparse(syntax_tree)
}

/// Call Cargo to build, check or expand the prepared script.
///
/// # Errors
///
/// This function will bubble up any errors encountered.
pub fn build(proc_flags: &ProcFlags, build_state: &BuildState) -> BuildResult<()> {
    let start_build = Instant::now();
    profile!("build");
    vlog!(V::V, "BBBBBBBB In build");

    if proc_flags.contains(ProcFlags::EXPAND) {
        handle_expand(proc_flags, build_state)
    } else {
        handle_build_or_check(proc_flags, build_state)
    }?;

    display_timings(&start_build, "Completed build", proc_flags);
    Ok(())
}

fn create_cargo_command(proc_flags: &ProcFlags, build_state: &BuildState) -> BuildResult<Command> {
    profile!("create_cargo_command");
    let cargo_toml_path_str = code_utils::path_to_str(&build_state.cargo_toml_path)?;
    let mut cargo_command = Command::new("cargo");

    let args = build_command_args(proc_flags, build_state, &cargo_toml_path_str);
    cargo_command.args(&args);

    configure_command_output(&mut cargo_command, proc_flags);
    Ok(cargo_command)
}

fn build_command_args(
    proc_flags: &ProcFlags,
    build_state: &BuildState,
    cargo_toml_path: &str,
) -> Vec<String> {
    profile!("build_command_args");
    let mut args = vec![
        get_cargo_subcommand(proc_flags, build_state).to_string(),
        "--manifest-path".to_string(),
        cargo_toml_path.to_string(),
    ];

    if proc_flags.contains(ProcFlags::QUIET) || proc_flags.contains(ProcFlags::QUIETER) {
        args.push("--quiet".to_string());
    }

    if proc_flags.contains(ProcFlags::EXECUTABLE) {
        args.push("--release".to_string());
    } else if proc_flags.contains(ProcFlags::EXPAND) {
        args.extend_from_slice(&[
            "--bin".to_string(),
            build_state.source_stem.clone(),
            "--theme=gruvbox-dark".to_string(),
        ]);
    } else if proc_flags.contains(ProcFlags::CARGO) {
        args.extend_from_slice(&build_state.args[1..]);
    }

    args
}

fn get_cargo_subcommand(proc_flags: &ProcFlags, build_state: &BuildState) -> &'static str {
    profile!("get_cargo_subcommand");
    if proc_flags.contains(ProcFlags::CHECK) {
        "check"
    } else if proc_flags.contains(ProcFlags::EXPAND) {
        "expand"
    } else if proc_flags.contains(ProcFlags::CARGO) {
        // Convert to owned String then get static str to avoid lifetime issues
        Box::leak(build_state.args[0].clone().into_boxed_str())
    } else {
        "build"
    }
}

fn configure_command_output(command: &mut Command, proc_flags: &ProcFlags) {
    profile!("configure_command_output");
    if proc_flags.contains(ProcFlags::QUIETER) || proc_flags.contains(ProcFlags::EXPAND) {
        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
    } else {
        command
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());
    }
}

fn handle_expand(proc_flags: &ProcFlags, build_state: &BuildState) -> BuildResult<()> {
    profile!("handle_expand");
    let mut cargo_command = create_cargo_command(proc_flags, build_state)?;

    // eprintln!("cargo_command={cargo_command:#?}");

    let output = cargo_command.output()?;

    if !output.status.success() {
        eprintln!(
            "Error running `cargo expand`: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err("Expansion failed".into());
    }

    display_expansion_diff(output.stdout, build_state)?;
    Ok(())
}

fn handle_build_or_check(proc_flags: &ProcFlags, build_state: &BuildState) -> BuildResult<()> {
    profile!("handle_build_or_check");
    let mut cargo_command = create_cargo_command(proc_flags, build_state)?;

    // eprintln!("cargo_command={cargo_command:#?}");

    let status = cargo_command.spawn()?.wait()?;

    if !status.success() {
        display_build_failure();
        return Err("Build failed".into());
    }

    if proc_flags.contains(ProcFlags::EXECUTABLE) {
        deploy_executable(build_state)?;
    }
    Ok(())
}

fn display_expansion_diff(stdout: Vec<u8>, build_state: &BuildState) -> BuildResult<()> {
    profile!("display_expansion_diff");
    let expanded_source = String::from_utf8(stdout)?;
    let unexpanded_path = get_source_path(build_state);
    let unexpanded_source = std::fs::read_to_string(unexpanded_path)?;

    let max_width = if let Ok((width, _height)) = crossterm::terminal::size() {
        (width - 26) / 2
    } else {
        80
    };

    let diff = create_side_by_side_diff(&unexpanded_source, &expanded_source, max_width.into());
    println!("{diff}");
    Ok(())
}

fn display_build_failure() {
    profile!("display_build_failure");
    cvprtln!(&Lvl::ERR, V::N, "Build failed");
    let config = maybe_config();
    let binding = Dependencies::default();
    let dep_config = config.as_ref().map_or(&binding, |c| &c.dependencies);
    let inference_level = &dep_config.inference_level;

    let advice = match inference_level {
        config::DependencyInference::None => "You are running without dependency inference.",
        config::DependencyInference::Min => "You may be missing features or `thag` may not be picking up dependencies.",
        config::DependencyInference::Config => "You may need to tweak your config feature overrides or 'toml` block",
        config::DependencyInference::Max => "It may be that maximal dependency inference is specifying conflicting features. Consider trying `config` or failing that, a `toml` block",
    };

    cvprtln!(
        &Lvl::EMPH,
        V::N,
        r#"Dependency inference_level={inference_level:#?}
If the problem is a dependency error, consider the following advice:
{advice}
{}"#,
        if matches!(inference_level, config::DependencyInference::Config) {
            ""
        } else {
            "Consider running with dependency inference_level configured as `config` or an embedded `toml` block."
        }
    );
}

fn deploy_executable(build_state: &BuildState) -> BuildResult<()> {
    profile!("deploy_executable");
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

    // let dash_line = "-".repeat(&FLOWER_BOX_LEN);
    repeat_dash!(70);
    cvprtln!(Lvl::EMPH, V::Q, "{DASH_LINE}");

    vlog!(
        V::QQ,
        "Executable built and moved to ~/{cargo_bin_subdir}/{executable_name}"
    );

    cvprtln!(Lvl::EMPH, V::Q, "{DASH_LINE}");
    Ok(())
}

/// Run the built program
/// # Errors
///
/// Will return `Err` if there is an error waiting for the spawned command
/// that runs the user script.
pub fn run(proc_flags: &ProcFlags, args: &[String], build_state: &BuildState) -> BuildResult<()> {
    // profile!("run");

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
    cvprtln!(Lvl::EMPH, V::Q, "{dash_line}");

    let _exit_status = run_command.spawn()?.wait()?;

    cvprtln!(Lvl::EMPH, V::Q, "{dash_line}");

    // #[cfg(debug_assertions)]
    // debug_log!("Exit status={exit_status:#?}");

    display_timings(&start_run, "Completed run", proc_flags);

    Ok(())
}
