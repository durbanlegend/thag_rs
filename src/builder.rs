//!
//! The main driver for the `thag_rs` binary command `thag`, which controls execution based on input from
//! the command line or caller and possibly from a script file or standard input.
//!
//! Preprocessing functions interpret the command-line arguments and processing flags, as pre-digested by
//! the `cmd_args` module.
//!
//! The `execute` function does a preliminary assessment and invokes the `process` function, which starts
//! by pre-configuring a `BuildState` struct instance to drive the build. In the case of script file input
//! this includes individual target directory information under `temp_dir()` corresponding to the input
//! filename. If standard input, edit or REPL command-line options were chosen, `process` will invoke the
//! `stdin` or `repl` modules to solicit the script or expression input, otherwise it will obtain it from a
//! script file path passed in as the main argument, or from a command-line argument such as `--expr (-e)`.
//!
//! Once the input is finalised, `builder` broadly speaking handles the three major processing stages,
//! named `generate`, `build` and `run`. First a function called `gen_build_run` invokes the `ast` module to
//! perform AST analysis of the input to extract explicit manifest information from an optional "toml" block
//! embedded in script comments, as well as any dependencies inferred from the source and features inferred
//! from configuration settings or system defaults. It invokes the `manifest` module to merge this manifest
//! information into a Cargo manifest struct.
//!
//! From the AST analysis, `gen_build_run` also determines whether the script is an expression or snippet
//! as opposed to a program. If so, it calls the `code_utils` module to wrap the snippet in a template with
//! a main function to make a well-formed program. If the generated output is stale or does not yet exist,
//! or if `force (-f)` was specified, it then calls fn `generate` to write out the manifest to the target
//! directory as a Cargo.toml file. If the input was not a program it also writes out the wrapped input to
//! the target directory as a `.rs` program file, optionally formatting it with `prettyplease`.
//!
//! The `build` stage invokes a Cargo command, which by default is `build`, but may be `check`, `expand` or
//! another arbitrary Cargo subcommand that may be specified by the `--cargo (-A)` command line option. As
//! with `generate`, a regular build is only invoked if the build output is stale or does not yet exist, or
//! if `force (-f)` was specified. If the `--executable (-x)` option was specified, a Cargo release build
//! is invoked and the executable output is moved to the user's `.cargo/bin` directory, which the user
//! should ensure is in the execution path so that it can be run as a command without further ado.
//!
//! Finally, if a conflicting option is not specified, the `run` function invokes `cargo run` to run the
//! built output. Note that because of the staleness checks, a normal script that has not been modified
//! since it was last built (and not been cleared from `temp_dir()` by the operating system) will skip the
//! generation and build steps and execute almost immediately, similarly to a Cargo `run`. In this case
//! the `build` module  will display an informational message to this effect at normal verbosity levels.
//!
use crate::ast::is_unit_return_type;
use crate::code_utils::{
    self, build_loop, create_temp_source_file, extract_ast_expr, get_source_path,
    read_file_contents, remove_inner_attributes, strip_curly_braces, to_ast, wrap_snippet,
    write_source,
};
use crate::config::{self, DependencyInference, RealContext};
use crate::manifest::extract;
use crate::styling::{style_string, ColorInitStrategy, TermAttributes};
use crate::{
    ast, cvprtln, debug_log, get_home_dir, get_proc_flags, manifest, maybe_config,
    modified_since_compiled, profile, profile_method, profile_section, regex, repeat_dash, shared,
    validate_args, vlog, Ast, Cli, Dependencies, Lvl, ProcFlags, ThagError, ThagResult,
    DYNAMIC_SUBDIR, FLOWER_BOX_LEN, PACKAGE_NAME, REPL_SCRIPT_NAME, REPL_SUBDIR, RS_SUFFIX,
    TEMP_DIR_NAME, TEMP_SCRIPT_NAME, TMPDIR, TOML_NAME, V,
};
use cargo_toml::Manifest;
use regex::Regex;
use side_by_side_diff::create_side_by_side_diff;
use std::env;
use std::{
    env::current_dir,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    string::ToString,
    time::Instant,
};

#[cfg(feature = "tui")]
use crate::{
    stdin::{edit, read},
    CrosstermEventReader,
};

#[cfg(debug_assertions)]
use {
    crate::{debug_timings, logging::is_debug_logging_enabled, VERSION},
    log::{log_enabled, Level::Debug},
};

#[cfg(feature = "repl")]
use crate::repl::run_repl;

#[cfg(feature = "build")]
struct ExecutionFlags {
    is_repl: bool,
    is_dynamic: bool,
}

#[cfg(feature = "build")]
impl ExecutionFlags {
    const fn new(proc_flags: &ProcFlags, cli: &Cli) -> Self {
        let is_repl = proc_flags.contains(ProcFlags::REPL);
        let is_expr = cli.expression.is_some();
        let is_stdin = proc_flags.contains(ProcFlags::STDIN);
        let is_edit = proc_flags.contains(ProcFlags::EDIT);
        // let is_url = proc_flags.contains(ProcFlags::URL); // TODO reinstate
        let is_loop = proc_flags.contains(ProcFlags::LOOP);
        let is_dynamic = is_expr | is_stdin | is_edit | is_loop;

        Self {
            is_repl,
            is_dynamic,
        }
    }
}

#[cfg(feature = "build")]
struct BuildPaths {
    working_dir_path: PathBuf,
    source_path: PathBuf,
    source_dir_path: PathBuf,
    cargo_home: PathBuf,
    target_dir_path: PathBuf,
    target_path: PathBuf,
    cargo_toml_path: PathBuf,
}

/// A struct to encapsulate the attributes of the current build as needed by the various
/// functions co-operating in the generation, build and execution of the code.
#[derive(Clone, Debug, Default)]
#[cfg(feature = "build")]
pub struct BuildState {
    #[allow(dead_code)]
    pub working_dir_path: PathBuf,
    pub source_stem: String,
    pub source_name: String,
    #[allow(dead_code)]
    pub source_dir_path: PathBuf,
    pub source_path: PathBuf,
    pub cargo_home: PathBuf,
    pub target_dir_path: PathBuf,
    pub target_path: PathBuf,
    pub cargo_toml_path: PathBuf,
    pub rs_manifest: Option<Manifest>,
    pub cargo_manifest: Option<Manifest>,
    pub must_gen: bool,
    pub must_build: bool,
    pub build_from_orig_source: bool,
    pub ast: Option<Ast>,
    pub crates_finder: Option<ast::CratesFinder>,
    pub metadata_finder: Option<ast::MetadataFinder>,
    pub infer: DependencyInference,
    pub args: Vec<String>,
}

#[cfg(feature = "build")]
impl BuildState {
    /// Configures a new `BuildState` instance based on processing flags, CLI arguments, and script state.
    ///
    /// This function coordinates the complete setup process by:
    /// 1. Extracting and validating script information
    /// 2. Determining execution mode flags
    /// 3. Setting up all required directory paths
    /// 4. Creating the initial build state
    /// 5. Determining build requirements
    ///
    /// # Arguments
    /// * `proc_flags` - Processing flags that control build and execution behavior
    /// * `cli` - Command-line arguments parsed from the CLI
    /// * `script_state` - Current state of the script being processed
    ///
    /// # Returns
    /// * `ThagResult<Self>` - Configured `BuildState` instance if successful
    ///
    /// # Errors
    /// Returns a `ThagError` if:
    /// * No script is specified in the script state
    /// * Script filename is invalid or cannot be converted to a string
    /// * Unable to strip .rs suffix from script name
    /// * Cannot resolve working directory or home directory
    /// * Cannot resolve script directory path
    /// * Script file does not exist at the specified path
    /// * Cannot resolve parent directory of script
    /// * Cannot determine if source has been modified since last compilation
    ///
    /// # Example
    /// ```ignore
    /// let proc_flags = ProcFlags::default();
    /// let cli = Cli::parse();
    /// let script_state = ScriptState::new("example.rs");
    /// let build_state = BuildState::pre_configure(&proc_flags, &cli, &script_state)?;
    /// ```
    pub fn pre_configure(
        proc_flags: &ProcFlags,
        cli: &Cli,
        script_state: &ScriptState,
    ) -> ThagResult<Self> {
        profile!("pre_configure");

        // 1. Validate and extract basic script info
        let (source_name, source_stem) = Self::extract_script_info(script_state)?;

        // 2. Determine execution mode flags
        let execution_flags = ExecutionFlags::new(proc_flags, cli);

        // 3. Set up directory paths
        let paths = Self::set_up_paths(&execution_flags, script_state, &source_name, &source_stem)?;

        // 4. Create initial build state
        let mut build_state = Self::create_initial_state(paths, source_name, source_stem, cli);

        // 5. Determine build requirements
        build_state.determine_build_requirements(proc_flags, script_state, &execution_flags)?;

        // 6. Validate state (debug only)
        #[cfg(debug_assertions)]
        build_state.validate_state(proc_flags);

        Ok(build_state)
    }

    fn extract_script_info(script_state: &ScriptState) -> ThagResult<(String, String)> {
        profile!("extract_script_info");
        let script = script_state
            .get_script()
            .ok_or(ThagError::NoneOption("No script specified"))?;

        let path = Path::new(&script);
        let filename = path
            .file_name()
            .ok_or(ThagError::NoneOption("No filename specified"))?;

        let source_name = filename
            .to_str()
            .ok_or(ThagError::NoneOption(
                "Error converting filename to a string",
            ))?
            .to_string();

        let source_stem = source_name
            .strip_suffix(RS_SUFFIX)
            .ok_or_else(|| -> ThagError {
                format!("Error stripping suffix from {source_name}").into()
            })?
            .to_string();

        Ok((source_name, source_stem))
    }

    fn set_up_paths(
        flags: &ExecutionFlags,
        script_state: &ScriptState,
        source_name: &str,
        source_stem: &str,
    ) -> ThagResult<BuildPaths> {
        profile!("set_up_paths");
        // Working directory setup
        let working_dir_path = if flags.is_repl {
            TMPDIR.join(REPL_SUBDIR)
        } else {
            env::current_dir()?.canonicalize()?
        };

        // Script path setup
        let script_path = if flags.is_repl {
            script_state
                .get_script_dir_path()
                .ok_or("Missing script path")?
                .join(source_name)
        } else if flags.is_dynamic {
            script_state
                .get_script_dir_path()
                .ok_or("Missing script path")?
                .join(TEMP_SCRIPT_NAME)
        } else {
            working_dir_path.join(script_state.get_script().unwrap()) // Safe due to prior validation
        };

        // Source path setup and validation
        let source_path = script_path.canonicalize()?;
        if !source_path.exists() {
            return Err(format!(
                "No script named {source_stem} or {source_name} in path {source_path:?}"
            )
            .into());
        }

        // Source directory path
        let source_dir_path = source_path
            .parent()
            .ok_or("Problem resolving to parent directory")?
            .to_path_buf();

        // Cargo home setup
        let cargo_home_var = env::var("CARGO_HOME")?;
        let cargo_home = PathBuf::from(if cargo_home_var == String::new() {
            get_home_dir()?.join(".cargo").display().to_string()
        } else {
            cargo_home_var
        });

        // Target directory setup
        let target_dir_path = if flags.is_repl {
            script_state
                .get_script_dir_path()
                .ok_or("Missing ScriptState::NamedEmpty.repl_path")?
                .join(TEMP_DIR_NAME)
        } else if flags.is_dynamic {
            TMPDIR.join(DYNAMIC_SUBDIR)
        } else {
            TMPDIR.join(PACKAGE_NAME).join(source_stem)
        };

        // Target path setup
        let mut target_path = target_dir_path.join("target").join("debug");
        #[cfg(target_os = "windows")]
        {
            target_path = target_path.join(format!("{source_stem}.exe"));
        }
        #[cfg(not(target_os = "windows"))]
        {
            target_path = target_path.join(source_stem);
        }

        let cargo_toml_path = target_dir_path.join(TOML_NAME);

        Ok(BuildPaths {
            working_dir_path,
            source_path,
            source_dir_path,
            cargo_home,
            target_dir_path,
            target_path,
            cargo_toml_path,
        })
    }

    fn create_initial_state(
        paths: BuildPaths,
        source_name: String,
        source_stem: String,
        cli: &Cli,
    ) -> Self {
        profile!("create_initial_state");

        Self {
            working_dir_path: paths.working_dir_path,
            source_stem,
            source_name,
            source_dir_path: paths.source_dir_path,
            source_path: paths.source_path,
            cargo_home: paths.cargo_home,
            target_dir_path: paths.target_dir_path,
            target_path: paths.target_path,
            cargo_toml_path: paths.cargo_toml_path,
            ast: None,
            crates_finder: None,
            metadata_finder: None,
            infer: cli.infer.as_ref().map_or_else(
                || {
                    let config = maybe_config();
                    let binding = Dependencies::default();
                    let dep_config = config.as_ref().map_or(&binding, |c| &c.dependencies);
                    let infer = &dep_config.inference_level;
                    infer.clone()
                },
                Clone::clone,
            ),
            args: cli.args.clone(),
            ..Default::default()
        }
    }

    fn determine_build_requirements(
        &mut self,
        proc_flags: &ProcFlags,
        script_state: &ScriptState,
        flags: &ExecutionFlags,
    ) -> ThagResult<()> {
        profile_method!("determine_build_requirements");
        // Case 1: Force generation and building
        if flags.is_dynamic
            || flags.is_repl
            || proc_flags.contains(ProcFlags::FORCE)
            || proc_flags.contains(ProcFlags::CHECK)
        {
            self.must_gen = true;
            self.must_build = true;
            return Ok(());
        }

        // Case 2: No-run mode
        if proc_flags.contains(ProcFlags::NORUN) {
            self.must_build = proc_flags.contains(ProcFlags::BUILD)
                || proc_flags.contains(ProcFlags::EXECUTABLE)
                // For EXPAND and CARGO, "build" step (becoming a bit of a misnomer)
                // is needed to run their alternative Cargo commands
                || proc_flags.contains(ProcFlags::EXPAND)
                || proc_flags.contains(ProcFlags::CARGO);
            self.must_gen = self.must_build
                || proc_flags.contains(ProcFlags::GENERATE)
                || !self.cargo_toml_path.exists();
            return Ok(());
        }

        // Case 3: Check if build is needed due to state or modifications
        if matches!(script_state, ScriptState::NamedEmpty { .. })
            || !self.target_path.exists()
            || modified_since_compiled(self)?.is_some()
        {
            self.must_gen = true;
            self.must_build = true;
            return Ok(());
        }

        // Default case: no generation or building needed
        self.must_gen = false;
        self.must_build = false;
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn validate_state(&self, proc_flags: &ProcFlags) {
        profile_method!("validate_state");
        // Validate build/check/executable/expand flags
        if proc_flags.contains(ProcFlags::BUILD)
            | proc_flags.contains(ProcFlags::CHECK)
            | proc_flags.contains(ProcFlags::EXECUTABLE)
            | proc_flags.contains(ProcFlags::EXPAND)
            | proc_flags.contains(ProcFlags::CARGO)
        {
            assert!(self.must_gen & self.must_build & proc_flags.contains(ProcFlags::NORUN));
        }

        // Validate force flag
        if proc_flags.contains(ProcFlags::FORCE) {
            assert!(self.must_gen & self.must_build);
        }

        // Validate expand and cargo flags
        if proc_flags.contains(ProcFlags::EXPAND) | proc_flags.contains(ProcFlags::CARGO) {
            assert!(self.must_gen & self.must_build & proc_flags.contains(ProcFlags::NORUN));
        }

        // Validate build dependency
        if self.must_build {
            assert!(self.must_gen);
        }

        // Log the final state in debug mode
        debug_log!("build_state={self:#?}");
    }
}

/// An enum to encapsulate the type of script in play.
#[derive(Debug)]
pub enum ScriptState {
    /// Repl with no script name provided by user
    #[allow(dead_code)]
    Anonymous,
    /// Repl with script name.
    NamedEmpty {
        script: String,
        script_dir_path: PathBuf,
    },
    /// Script name provided by user
    Named {
        script: String,
        script_dir_path: PathBuf,
    },
}

impl ScriptState {
    /// Return the script name wrapped in an Option.
    #[must_use]
    pub fn get_script(&self) -> Option<String> {
        profile_method!("get_script");
        match self {
            Self::Anonymous => None,
            Self::NamedEmpty { script, .. } | Self::Named { script, .. } => {
                Some(script.to_string())
            }
        }
    }
    /// Return the script's directory path wrapped in an Option.
    #[must_use]
    pub fn get_script_dir_path(&self) -> Option<PathBuf> {
        profile_method!("get_script_dir_path");
        match self {
            Self::Anonymous => None,
            Self::Named {
                script_dir_path, ..
            } => Some(script_dir_path.clone()),
            Self::NamedEmpty {
                script_dir_path: script_path,
                ..
            } => Some(script_path.clone()),
        }
    }
}

/// Execute the script runner.
/// # Errors
///
/// Will return `Err` if there is an error returned by any of the subordinate functions.
/// # Panics
/// Will panic if it fails to strip a .rs extension off the script name,
pub fn execute(args: &mut Cli) -> ThagResult<()> {
    // Instrument the entire function
    // profile!("execute");

    let start = Instant::now();

    // Initialize TermAttributes for message styling
    let strategy = if std::env::var("TEST_ENV").is_ok() {
        #[cfg(debug_assertions)]
        debug_log!("Avoiding colour detection for testing");
        ColorInitStrategy::Default
    } else if cfg!(feature = "color_detect") {
        ColorInitStrategy::Detect
    } else if let Some(config) = maybe_config() {
        ColorInitStrategy::Configure(config.colors.color_support, config.colors.term_theme)
    } else {
        ColorInitStrategy::Default
    };

    TermAttributes::initialize(strategy);

    let proc_flags = get_proc_flags(args)?;

    #[cfg(debug_assertions)]
    if log_enabled!(Debug) {
        log_init_setup(start, args, &proc_flags);
    }

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
) -> ThagResult<PathBuf> {
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
) -> ThagResult<ScriptState> {
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
) -> ThagResult<()> {
    // profile!("process");
    let is_repl = args.repl;
    let is_expr = proc_flags.contains(ProcFlags::EXPR);
    let is_stdin = proc_flags.contains(ProcFlags::STDIN);
    let is_edit = proc_flags.contains(ProcFlags::EDIT);
    let is_loop = proc_flags.contains(ProcFlags::LOOP);
    let is_dynamic = is_expr | is_stdin | is_edit | is_loop;

    let mut build_state = BuildState::pre_configure(proc_flags, args, script_state)?;
    if is_repl {
        #[cfg(not(feature = "repl"))]
        return Err("repl requires `repl` feature".into());
        #[cfg(feature = "repl")]
        {
            debug_log!("build_state.source_path={:?}", build_state.source_path);
            run_repl(args, proc_flags, &mut build_state, start)
        }
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
        } else {
            #[cfg(not(feature = "tui"))]
            return Err("`stdin` and `edit` options require `tui` feature".into());
            #[cfg(feature = "tui")]
            if is_edit {
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
                debug_log!("About to call stdin::read())");
                let str = read()? + "\n";

                debug_log!("str={str}");
                str
            }
        };

        vlog!(V::V, "rs_source={rs_source}");

        let rs_manifest = extract(&rs_source, Instant::now())
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
) -> ThagResult<()> {
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
) -> ThagResult<()> {
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
            build_state.crates_finder = Some(ast::find_crates(ast));
            build_state.metadata_finder = Some(ast::find_metadata(ast));
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

        let rs_manifest: Manifest = { extract(&rs_source, start_parsing_rs) }?;

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
                    Ast::Expr(expr) => is_unit_return_type(expr),
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
        "{PACKAGE_NAME} completed processing script {}",
        style_string(Lvl::EMPH, &build_state.source_name)
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
) -> ThagResult<()> {
    profile!("generate");
    let start_gen = Instant::now();

    #[cfg(debug_assertions)]
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
                &prettyplease_unparse(&syntax_tree)
            }
            #[cfg(not(feature = "format_snippet"))]
            {
                if proc_flags.contains(ProcFlags::CARGO) {
                    // Code needs to be human readable for clippy, test etc.
                    let syntax_tree = syn_parse_file(rs_source)?;
                    &prettyplease_unparse(&syntax_tree)
                } else {
                    rs_source.expect("Logic error retrieving rs_source")
                }
            }
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
        shared::disentangle(cargo_manifest_str)
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
fn syn_parse_file(rs_source: Option<&str>) -> ThagResult<syn::File> {
    profile!("syn_parse_file");
    let syntax_tree = syn::parse_file(rs_source.ok_or("Logic error retrieving rs_source")?)?;
    Ok(syntax_tree)
}

#[inline]
fn prettyplease_unparse(syntax_tree: &syn::File) -> String {
    profile!("prettyplease_unparse");
    prettyplease::unparse(syntax_tree)
}

/// Call Cargo to build, check or expand the prepared script.
///
/// # Errors
///
/// This function will bubble up any errors encountered.
pub fn build(proc_flags: &ProcFlags, build_state: &BuildState) -> ThagResult<()> {
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

fn create_cargo_command(proc_flags: &ProcFlags, build_state: &BuildState) -> ThagResult<Command> {
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

fn handle_expand(proc_flags: &ProcFlags, build_state: &BuildState) -> ThagResult<()> {
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

fn handle_build_or_check(proc_flags: &ProcFlags, build_state: &BuildState) -> ThagResult<()> {
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

fn display_expansion_diff(stdout: Vec<u8>, build_state: &BuildState) -> ThagResult<()> {
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
    cvprtln!(Lvl::ERR, V::N, "Build failed");
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
        Lvl::EMPH,
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

fn deploy_executable(build_state: &BuildState) -> ThagResult<()> {
    profile!("deploy_executable");
    // Determine the output directory
    let mut cargo_bin_path = PathBuf::from(crate::get_home_dir_string()?);
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
pub fn run(proc_flags: &ProcFlags, args: &[String], build_state: &BuildState) -> ThagResult<()> {
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

/// Display method timings when either the --verbose or --timings option is chosen.
#[inline]
pub fn display_timings(start: &Instant, process: &str, proc_flags: &ProcFlags) {
    profile!("display_timings");
    #[cfg(not(debug_assertions))]
    if !proc_flags.intersects(ProcFlags::DEBUG | ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        return;
    }
    let dur = start.elapsed();
    let msg = format!("{process} in {}.{}s", dur.as_secs(), dur.subsec_millis());

    #[cfg(debug_assertions)]
    debug_log!("{msg}");
    if proc_flags.intersects(ProcFlags::DEBUG | ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        vlog!(V::QQ, "{msg}");
    }
}
