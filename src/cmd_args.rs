use crate::{config::maybe_config, debug_log, ThagError, ThagResult, RS_SUFFIX};

use bitflags::bitflags;
// use clap::builder::styling::{Ansi256Color, AnsiColor, Color, Style};
use clap::{ArgGroup /*, ColorChoice */, Parser};
use firestorm::profile_fn;
use std::{fmt, str};

/// The `clap` command-line interface for the `thag_rs` script runner and REPL.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Default, Parser, Debug)]
#[command(name = "thag_rs", version, about, long_about/*, color = ColorChoice::Always, styles=get_styles(), next_help_heading = Some("Information Options")*/)]
#[command(group(
            ArgGroup::new("commands")
                .required(true)
                .args(&["script", "expression", "repl", "filter", "stdin", "edit", "config"]),
   ))]
#[command(group(
            ArgGroup::new("verbosity")
                .required(false)
                .args(&["quiet", "normal", "verbose"]),
        ))]
pub struct Cli {
    /// Optional path of a script to run (`path`/`stem`.rs)
    pub script: Option<String>,
    /// Any arguments for the script
    #[arg(last = true, requires = "script")]
    pub args: Vec<String>,
    /// Force the generation and build steps, even if the script is unchanged since a previous build. Required if there are updates to dependencies.
    #[arg(short, long, help_heading = Some("Script Options"))]
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
    /// Run the given filter expression in a loop against every line of stdin, with optional pre- and/or post-loop logic via -T, -B and -E.
    #[arg(short = 'l', long = "loop", help_heading = Some("Dynamic Options (no script)"), conflicts_with_all(["generate", "build"]))]
    pub filter: Option<String>,
    /// Optional manifest info for --loop in Cargo.toml format, such as a [dependencies] section
    //  clap issue 4707 may prevent `requires` from working, as I've experienced.
    #[arg(short = 'T', long, help_heading = Some("Dynamic Options (no script)"), requires = "filter", value_name = "CARGO-TOML")]
    pub toml: Option<String>,
    /// Optional pre-loop Rust statements for --loop, somewhat like awk BEGIN
    //  clap issue 4707 may prevent `requires` from working, as I've experienced.
    #[arg(short = 'B', long, help_heading = Some("Dynamic Options (no script)"), requires = "filter", value_name = "PRE-LOOP")]
    pub begin: Option<String>,
    /// Optional post-loop Rust statements for --loop, somewhat like awk END
    //  clap issue 4707 may prevent `requires` from working, as I've experienced.
    #[arg(short = 'E', long, help_heading = Some("Dynamic Options (no script)"), requires = "filter", value_name = "POST-LOOP")]
    pub end: Option<String>,
    /// Required if multiple main methods are valid for the current script
    #[arg(short, long, help_heading = Some("Script Options"))]
    pub multimain: bool,
    /// Display timings
    #[arg(short, long, help_heading = Some("Output Options"))]
    pub timings: bool,
    /// Set verbose mode. Double up for debug mode with destination app.log.
    #[arg(short, long, help_heading = Some("Output Options"), action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Set normal verbosity. Only needed in the case of overriding a different configured value
    #[arg(short = 'N', long = "normal verbosity", help_heading = Some("Output Options"))]
    pub normal: bool,
    /// Suppress unnecessary output. Double up to show only errors, or to pipe output to another command.
    #[arg(short, long, help_heading = Some("Output Options"), action = clap::ArgAction::Count)]
    pub quiet: u8,
    /// Just generate individual Cargo.toml and any required Rust scaffolding for script, unless script unchanged from a previous build.
    #[arg(short, long = "gen"/*, requires = "norun"*/, help_heading = Some("No-run Options"), default_value_ifs([
        /*("force", "true", "true"),*/
        ("expression", "_", "true"),
        ("executable", "true", "true"),
        ("check", "true", "true"),
    ]))]
    pub generate: bool,
    /// Just build script (generating first if necessary), unless unchanged from a previous build
    #[arg(short, long/*, requires = "norun"*/, help_heading = Some("No-run Options"), default_value_ifs([
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
    #[arg(short, long, help_heading = Some("No-run Options"), conflicts_with_all(["build", "executable"]))]
    pub check: bool,
    /// Strip double quotes from string result of expression (true/false). Default: config value / false.
    #[arg(
        short,
        long, help_heading = Some("Dynamic Options (no script)"),
        // require_equals = true,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",  // Default to true if -u is present but no value is given
        conflicts_with("multimain")
    )]
    pub unquote: Option<bool>,
    /// Edit the configuration file
    #[arg(short = 'C', long, conflicts_with_all(["generate", "build", "executable"]))]
    pub config: bool,
}

/// Getter for clap command-line arguments
#[must_use]
pub fn get_args() -> Cli {
    profile_fn!(get_args);
    Cli::parse()
}

/// Validates the command-line arguments
/// # Errors
/// Will return `Err` if there is a missing script name or missing .rs suffix.
pub fn validate_args(args: &Cli, proc_flags: &ProcFlags) -> ThagResult<()> {
    profile_fn!(validate_args);
    if let Some(ref script) = args.script {
        if !script.ends_with(RS_SUFFIX) {
            return Err(format!("Script name {script} must end in {RS_SUFFIX}").into());
        }
    } else if !proc_flags.contains(ProcFlags::EXPR)
        && !proc_flags.contains(ProcFlags::REPL)
        && !proc_flags.contains(ProcFlags::STDIN)
        && !proc_flags.contains(ProcFlags::EDIT)
        && !proc_flags.contains(ProcFlags::LOOP)
        && !proc_flags.contains(ProcFlags::CONFIG)
    {
        return Err("Missing script name".into());
    }
    Ok(())
}

bitflags! {
    /// Processing flags for ease of handling command-line options.
    // You can `#[derive]` the `Debug` trait, but implementing it manually
    // can produce output like `A | B` instead of `Flags(A | B)`.
    // #[derive(Debug)]
    #[derive(Clone, Default, PartialEq, Eq)]
    pub struct ProcFlags: u32 {
        const GENERATE      = 1;
        const BUILD         = 2;
        const FORCE         = 4;
        const RUN           = 8;
        const NORUN         = 16;
        const EXECUTABLE    = 32;
        const CHECK         = 64;
        const REPL          = 128;
        const EXPR          = 256;
        const STDIN         = 512;
        const EDIT          = 1024;
        const LOOP          = 2048;
        const MULTI         = 4096;
        const TIMINGS       = 8192;
        const DEBUG         = 16384;
        const VERBOSE       = 32768;
        const NORMAL        = 65536;
        const QUIET         = 131_072;
        const QUIETER       = 262_144;
        const UNQUOTE       = 524_288;
        const CONFIG        = 1_048_576;
    }
}

impl fmt::Debug for ProcFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

impl fmt::Display for ProcFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

impl str::FromStr for ProcFlags {
    type Err = bitflags::parser::ParseError;

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
pub fn get_proc_flags(args: &Cli) -> ThagResult<ProcFlags> {
    profile_fn!(get_proc_flags);
    // eprintln!("args={args:#?}");
    let is_expr = args.expression.is_some();
    let is_loop = args.filter.is_some();
    let proc_flags = {
        let mut proc_flags = ProcFlags::empty();
        // eprintln!("args={args:#?}");
        proc_flags.set(ProcFlags::GENERATE, args.generate);
        proc_flags.set(ProcFlags::BUILD, args.build);
        proc_flags.set(ProcFlags::CHECK, args.check);
        proc_flags.set(ProcFlags::FORCE, args.force);
        proc_flags.set(ProcFlags::QUIET, args.quiet == 1);
        proc_flags.set(ProcFlags::QUIETER, args.quiet >= 2);
        proc_flags.set(ProcFlags::MULTI, args.multimain);
        proc_flags.set(ProcFlags::VERBOSE, args.verbose == 1);
        proc_flags.set(ProcFlags::DEBUG, args.verbose >= 2);
        proc_flags.set(ProcFlags::TIMINGS, args.timings);
        proc_flags.set(
            ProcFlags::NORUN,
            args.generate | args.build | args.check | args.executable,
        );
        proc_flags.set(ProcFlags::NORMAL, args.normal);
        proc_flags.set(ProcFlags::RUN, !proc_flags.contains(ProcFlags::NORUN));
        proc_flags.set(ProcFlags::REPL, args.repl);
        proc_flags.set(ProcFlags::EXPR, is_expr);
        proc_flags.set(ProcFlags::STDIN, args.stdin);
        proc_flags.set(ProcFlags::EDIT, args.edit);
        proc_flags.set(ProcFlags::LOOP, is_loop);
        proc_flags.set(ProcFlags::EXECUTABLE, args.executable);

        let unquote = args.unquote.map_or_else(
            || {
                maybe_config().map_or_else(
                    || {
                        debug_log!(
                            "Found no arg or config file, returning default unquote = false"
                        );
                        false
                    },
                    |config| {
                        debug_log!(
                            "maybe_config()={:?}, returning config.misc.unquote={}",
                            maybe_config(),
                            config.misc.unquote
                        );
                        config.misc.unquote
                    },
                )
            },
            |unquote| {
                debug_log!("args.unquote={:?}", args.unquote);
                unquote
            },
        );
        proc_flags.set(ProcFlags::UNQUOTE, unquote);

        proc_flags.set(ProcFlags::CONFIG, args.config);

        if !is_loop && (args.toml.is_some() || args.begin.is_some() || args.end.is_some()) {
            if args.toml.is_some() {
                eprintln!("Option --toml (-T) requires --loop (-l)");
            }
            if args.begin.is_some() {
                eprintln!("Option --begin (-B) requires --loop (-l)");
            }
            if args.end.is_some() {
                eprintln!("Option --end (-E) requires --loop (-l)");
            }
            return Err("Missing --loop option".into());
        }

        // Check all good
        let formatted = proc_flags.to_string();
        let parsed = formatted.parse::<ProcFlags>()?;

        assert_eq!(proc_flags, parsed);

        Ok::<ProcFlags, ThagError>(proc_flags)
    }?;
    Ok(proc_flags)
}
