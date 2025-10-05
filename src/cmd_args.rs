use crate::{
    config::{maybe_config, DependencyInference},
    ThagError, ThagResult, RS_SUFFIX,
};
use bitflags::bitflags;
use clap::{ArgGroup /*, ColorChoice */, Parser};
use std::{fmt, str};
use thag_common::{set_global_verbosity, Verbosity, V};
use thag_profiler::{end, profile, profiled};

/// The `clap` command-line interface for the `thag_rs` script runner and REPL.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Default, Parser, Debug)]
#[command(name = "thag_rs", version, about, long_about/*, color = ColorChoice::Always, styles=get_styles(), next_help_heading = Some("Information Options")*/)]
#[command(group(
            ArgGroup::new("commands")
                .required(true)
                .args(&["script", "expression", "repl", "filter", "stdin", "edit", "config", "clean"]),
   ))]
#[command(group(
            ArgGroup::new("verbosity")
                .required(false)
                .args(&["quiet", "normal_verbosity", "verbose"]),
        ))]
#[command(group(
            ArgGroup::new("norun_options")
                .required(false)
                .args(&["generate", "build", "check", "executable", "expand", "cargo"]),
        ))]
// #[command(group(
//             ArgGroup::new("dep_in")
//                 .required(false)
//                 .args(&["none", "min", "config", "max"]),
//         ))]
pub struct Cli {
    /// Optional path of a script to run (`path`/`stem`.rs)
    pub script: Option<String>,
    /// Features to enable when building the script (comma separated)
    #[arg(long, help_heading = Some("Processing Options"))]
    pub features: Option<String>,
    /// Any arguments for the script
    #[arg(last = true, requires = "script")]
    pub args: Vec<String>,
    /// Force the generation and build steps, even if the script is unchanged since a previous build. Required if there are updates to dependencies.
    #[arg(short, long, requires = "script", help_heading = Some("Processing Options"))]
    pub force: bool,
    // /// Don't run the script after generating and building
    // #[arg(short, long, conflicts_with_all(["edit", "expression", "filter", "repl", "stdin"]))]
    // pub norun: bool,
    /// Evaluate a quoted Rust expression on the fly
    #[arg(short, long = "expr", help_heading = Some("Dynamic Options (no script)"), conflicts_with_all(["generate", "build"]))]
    pub expression: Option<String>,
    /// REPL mode (read–eval–print loop) for Rust expressions, or for dynamic scripts using TUI or external editors.
    #[arg(short = 'r', long, help_heading = Some("Dynamic Options (no script)"), conflicts_with_all(["generate", "build"]))]
    pub repl: bool,
    /// Read script from stdin
    #[arg(short, long, help_heading = Some("Dynamic Options (no script)"), conflicts_with_all(["generate", "build"]))]
    pub stdin: bool,
    /// Simple TUI edit-submit with history. Editor will also capture any stdin input
    #[arg(short = 'd', long, help_heading = Some("Dynamic Options (no script)"), conflicts_with_all(["generate", "build"]))]
    pub edit: bool,
    /// Run the given filter expression in a loop against every line of stdin, with optional pre- and/or post-loop logic via -M, -B and -E.
    /// Expression may optionally print or return a value for line output.
    #[arg(short = 'l', long = "loop", help_heading = Some("Dynamic Options (no script)"), conflicts_with_all(["generate", "build"]))]
    pub filter: Option<String>,
    /// Optional manifest info for --loop in Cargo.toml format, such as a `[dependencies]` section
    //  clap issue 4707 may prevent `requires` from working, as I've experienced.
    #[arg(short = 'M', long, help_heading = Some("Filter Options"), requires = "filter", value_name = "CARGO-TOML")]
    pub toml: Option<String>,
    /// Optional pre-loop Rust statements for --loop, somewhat like awk BEGIN
    //  clap issue 4707 may prevent `requires` from working, as I've experienced.
    #[arg(short = 'B', long, help_heading = Some("Filter Options"), requires = "filter", value_name = "PRE-LOOP")]
    pub begin: Option<String>,
    /// Optional post-loop Rust statements for --loop, somewhat like awk END
    //  clap issue 4707 may prevent `requires` from working, as I've experienced.
    #[arg(short = 'E', long, help_heading = Some("Filter Options"), requires = "filter", value_name = "POST-LOOP")]
    pub end: Option<String>,
    /// Allow multiple main methods for the current script
    #[arg(short, long, help_heading = Some("Processing Options"))]
    pub multimain: bool,
    /// Display timings
    #[arg(short, long, help_heading = Some("Output Options"))]
    pub timings: bool,
    /// Set verbose mode. Double up for debug mode with destination app.log.
    #[arg(short, long, help_heading = Some("Output Options"), action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Set normal verbosity. Only needed in the case of overriding a different configured value
    #[arg(short = 'N', long = "normal", help_heading = Some("Output Options"))]
    pub normal_verbosity: bool,
    /// Suppress unnecessary output. Double up to show only errors, or when piping output to another command.
    #[arg(short, long, help_heading = Some("Output Options"), action = clap::ArgAction::Count)]
    pub quiet: u8,
    /// Just generate individual Cargo.toml and any required Rust scaffolding for script, unless script unchanged from a previous build.
    #[arg(short, long = "gen", help_heading = Some("No-run Options"), default_value_ifs([
        /*("force", "true", "true"),*/
        ("expression", "_", "true"),
        ("executable", "true", "true"),
        ("check", "true", "true"),
    ]))]
    pub generate: bool,
    /// Just build script (generating first if necessary), unless unchanged from a previous build
    #[arg(short, long, help_heading = Some("No-run Options"), default_value_ifs([
        /*("force", "true", "true"),*/
        ("expression", "_", "true"),
        ("executable", "true", "true"),
    ]))]
    pub build: bool,
    /// Just build executable `home_dir`/.cargo/bin/`stem` from script `path`/`stem`.rs using `cargo build --release`
    #[arg(short = 'x', long, help_heading = Some("No-run Options"))]
    pub executable: bool,
    /// Just cargo check script, unless unchanged from a previous build. Less thorough than build.
    /// Used by integration test to check all demo scripts
    #[arg(short, long, help_heading = Some("No-run Options"))]
    pub check: bool,
    /// Just generate script, unless unchanged from a previous build, and show the version with expanded
    /// macros side by side with the original version.
    /// Requires the `cargo-expand` binary to be installed.
    #[arg(short = 'X', long, help_heading = Some("No-run Options"))]
    pub expand: bool,
    /// Strip double quotes from string result of expression (true/false). Default: config value / false.
    #[arg(
        short,
        long, help_heading = Some("Output Options"),
        // require_equals = true,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",  // Default to true if -u is present but no value is given
        conflicts_with("multimain")
    )]
    pub unquote: Option<bool>,
    /// Edit the configuration file
    #[arg(short = 'C', long, help_heading = Some("Dynamic Options (no script)"), conflicts_with_all(["generate", "build", "executable"]))]
    pub config: bool,
    /// Dependency inference: none, min, config (default & recommended), max.
    /// `thag` infers dependencies from imports and Rust paths (`x::y::z`), with configurable default features.
    #[arg(short = 'i', long, help_heading = Some("Processing Options"))]
    pub infer: Option<DependencyInference>,
    /// Just generate script, unless unchanged from a previous build, and run the specified
    /// Cargo subcommand against the generated project `temp_dir`/`thag_rs`/`stem`. E.g. `thag demo/hello.rs -A tree`
    #[arg(short = 'A', long, requires = "script", help_heading = Some("No-run Options"))]
    pub cargo: bool,
    /// Test a module in isolation. Just generate the Cargo.toml and use it to run the internal unit tests without
    /// wrapping or modifying the source code.
    #[arg(short = 'T', long, requires = "script", help_heading = Some("No-run Options"))]
    pub test_only: bool,
    /// Clean cached build artifacts. Options: 'bins' (executables only), 'target' (shared build cache), 'all' (both). Default: 'all'
    #[arg(
        long,
        help_heading = Some("Maintenance Options"),
        value_name = "WHAT",
        default_missing_value = "all",
        num_args = 0..=1,
    )]
    pub clean: Option<String>,
}

/// Getter for clap command-line arguments
#[must_use]
#[profiled]
pub fn get_args() -> Cli {
    Cli::parse()
}

/// Validates the command-line arguments
/// # Errors
/// Will return `Err` if there is a missing script name or missing .rs suffix.
#[profiled]
pub fn validate_args(args: &Cli, proc_flags: &ProcFlags) -> ThagResult<()> {
    if let Some(ref script) = args.script {
        if !script.ends_with(RS_SUFFIX) && script != "t" && script != "tools" {
            return Err(format!("Script name {script} must end in {RS_SUFFIX}").into());
        }
    } else if !proc_flags.contains(ProcFlags::EXPR)
        && !proc_flags.contains(ProcFlags::REPL)
        && !proc_flags.contains(ProcFlags::STDIN)
        && !proc_flags.contains(ProcFlags::EDIT)
        && !proc_flags.contains(ProcFlags::LOOP)
        && !proc_flags.contains(ProcFlags::CONFIG)
        && !proc_flags.contains(ProcFlags::CLEAN)
    {
        return Err("Missing script name".into());
    }
    Ok(())
}

#[inline]
/// Determine the desired logging verbosity for the current execution.
/// # Errors
/// Will return `Err` if the logger mutex cannot be locked.
#[profiled]
pub fn set_verbosity(args: &Cli) -> ThagResult<()> {
    let verbosity = if args.verbose >= 2 {
        Verbosity::Debug
    } else if args.verbose == 1 {
        Verbosity::Verbose
    } else if args.quiet == 1 {
        V::Quiet
    } else if args.quiet >= 2 {
        V::Quieter
    } else if args.normal_verbosity {
        V::Normal
    } else if args.repl {
        // Default to quiet mode for REPL
        V::Quiet
    } else if let Some(config) = maybe_config() {
        config.logging.default_verbosity
    } else {
        V::Normal
    };
    set_global_verbosity(verbosity);
    Ok(())
}

bitflags! {
    /// Processing flags for ease of handling command-line options.
    // You can `#[derive]` the `Debug` trait, but implementing it manually
    // can produce output like `A | B` instead of `Flags(A | B)`.
    // #[derive(Debug)]
    #[derive(Clone, Default, PartialEq, Eq)]
    pub struct ProcFlags: u32 {
        /// Generate flag
        const GENERATE      = 1;
        /// Build flag
        const BUILD         = 2;
        /// Force flag
        const FORCE         = 4;
        /// Run flag
        const RUN           = 8;
        /// No-run flag
        const NORUN         = 16;
        /// Executable flag
        const EXECUTABLE    = 32;
        /// Check flag
        const CHECK         = 64;
        /// REPL flag
        const REPL          = 128;
        /// Expression flag
        const EXPR          = 256;
        /// Stdin flag
        const STDIN         = 512;
        /// Edit flag
        const EDIT          = 1024;
        /// Loop flag
        const LOOP          = 2048;
        /// Multi flag
        const MULTI         = 4096;
        /// Timings flag
        const TIMINGS       = 8192;
        /// Debug flag
        const DEBUG         = 16384;
        /// Verbose flag
        const VERBOSE       = 32768;
        /// Normal flag
        const NORMAL        = 65536;
        /// Quiet flag
        const QUIET         = 131_072;
        /// Quieter flag
        const QUIETER       = 262_144;
        /// Unquote flag
        const UNQUOTE       = 524_288;
        /// Config flag
        const CONFIG        = 1_048_576;
        /// Expand flag
        const EXPAND        = 2_097_152;
        /// Cargo flag
        const CARGO         = 4_194_304;
        /// Infer flag
        const INFER         = 8_388_608;
        /// Test only flag
        const TEST_ONLY     = 16_777_216;
        /// Tools flag
        const TOOLS         = 33_554_432;
        /// Features flag
        const FEATURES      = 67_108_864;
        /// Clean flag
        const CLEAN         = 134_217_728;
    }
}

impl fmt::Debug for ProcFlags {
    #[profiled]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

impl fmt::Display for ProcFlags {
    #[profiled]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

impl str::FromStr for ProcFlags {
    type Err = bitflags::parser::ParseError;

    #[profiled]
    fn from_str(flags: &str) -> Result<Self, Self::Err> {
        bitflags::parser::from_str(flags)
    }
}

/// Set up the processing flags from the command line arguments and pass them back.
/// # Errors
///
/// Will return `Err` if there is an error parsing the flags to set up and internal
/// correctness chack.
/// # Panics
///
/// Will panic if the internal correctness check fails.
#[profiled]
pub fn get_proc_flags(args: &Cli) -> ThagResult<ProcFlags> {
    // eprintln!("args={args:#?}");
    let is_expr = args.expression.is_some();
    let is_loop = args.filter.is_some();
    let is_infer = args.infer.is_some();
    let is_features = args.features.is_some();
    profile!(init_config_loop_assert, time);
    let proc_flags = {
        let mut proc_flags = ProcFlags::empty();
        // eprintln!("args={args:#?}");
        proc_flags.set(ProcFlags::GENERATE, args.generate);
        proc_flags.set(ProcFlags::BUILD, args.build);
        proc_flags.set(ProcFlags::CHECK, args.check);
        proc_flags.set(ProcFlags::FORCE, args.force);
        proc_flags.set(
            ProcFlags::QUIET,
            // Default for REPL is quiet
            args.quiet == 1
                || (args.repl && !args.normal_verbosity && args.quiet == 0 && args.verbose == 0),
        );
        proc_flags.set(ProcFlags::QUIETER, args.quiet >= 2);
        proc_flags.set(ProcFlags::MULTI, args.multimain);
        proc_flags.set(ProcFlags::VERBOSE, args.verbose == 1);
        proc_flags.set(ProcFlags::DEBUG, args.verbose >= 2);
        proc_flags.set(ProcFlags::TIMINGS, args.timings);
        proc_flags.set(
            ProcFlags::NORUN,
            args.generate
                | args.build
                | args.check
                | args.executable
                | args.expand
                | args.cargo
                | args.test_only,
        );
        proc_flags.set(ProcFlags::NORMAL, args.normal_verbosity);
        proc_flags.set(ProcFlags::RUN, !proc_flags.contains(ProcFlags::NORUN));
        proc_flags.set(ProcFlags::REPL, args.repl);
        proc_flags.set(ProcFlags::EXPR, is_expr);
        proc_flags.set(ProcFlags::STDIN, args.stdin);
        proc_flags.set(ProcFlags::EDIT, args.edit);
        proc_flags.set(ProcFlags::LOOP, is_loop);
        proc_flags.set(ProcFlags::EXECUTABLE, args.executable);
        proc_flags.set(ProcFlags::EXPAND, args.expand);
        proc_flags.set(ProcFlags::CARGO, args.cargo);
        proc_flags.set(ProcFlags::INFER, is_infer);
        proc_flags.set(ProcFlags::FEATURES, is_features);
        proc_flags.set(ProcFlags::TEST_ONLY, args.test_only);
        proc_flags.set(
            ProcFlags::TOOLS,
            args.script.as_ref().is_some_and(|script| script == "tools"),
        );
        proc_flags.set(ProcFlags::CLEAN, args.clean.is_some());
        end!(init_config_loop_assert);

        profile!(config_loop_assert, time);
        let unquote = args.unquote.map_or_else(
            || maybe_config().map_or_else(|| false, |config| config.misc.unquote),
            |unquote| {
                // debug_log!("args.unquote={:?}", args.unquote);
                unquote
            },
        );
        proc_flags.set(ProcFlags::UNQUOTE, unquote);
        proc_flags.set(ProcFlags::CONFIG, args.config);
        end!(config_loop_assert);

        profile!(loop_assert, time);
        if !is_loop && (args.toml.is_some() || args.begin.is_some() || args.end.is_some()) {
            if args.toml.is_some() {
                eprintln!("Option --toml (-M) requires --loop (-l)");
            }
            if args.begin.is_some() {
                eprintln!("Option --begin (-B) requires --loop (-l)");
            }
            if args.end.is_some() {
                eprintln!("Option --end (-E) requires --loop (-l)");
            }
            return Err("Missing --loop option".into());
        }
        end!(loop_assert);

        #[cfg(debug_assertions)]
        {
            profile!(assert_section, time);
            // Check all good
            let formatted = proc_flags.to_string();
            let parsed = formatted.parse::<ProcFlags>()?;
            assert_eq!(proc_flags, parsed);
            end!(assert_section);
        }

        Ok::<ProcFlags, ThagError>(proc_flags)
    }?;
    Ok(proc_flags)
}
