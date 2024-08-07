use crate::errors::BuildRunError;
use crate::logging::{self, Verbosity};
use crate::RS_SUFFIX;

use bitflags::bitflags;
use clap::{ArgGroup, Parser};
use core::{fmt, str};
use std::error::Error;

/// rs-script script runner and REPL
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Default, Parser, Debug)]
#[command(name = "rs_script", version, about, long_about)]
#[command(group(
            ArgGroup::new("commands")
                .required(true)
                .args(&["script", "expression", "repl", "filter", "stdin", "edit", "executable"]),
        ))]
pub struct Cli {
    /// Optional name of a script to run (`stem`.rs)
    pub script: Option<String>,
    /// Set the arguments for the script
    #[arg(last = true, requires = "script")]
    pub args: Vec<String>,
    /// Generate Rust source and individual cargo .toml if compiled file is stale
    #[arg(short = 'g', long = "gen")]
    pub generate: bool,
    /// Build script if compiled file is stale
    #[arg(short, long)]
    pub build: bool,
    /// Force generation of Rust source and individual Cargo.toml, and build, even if compiled file is not stale
    #[arg(short, long)]
    pub force: bool,
    /// Don't run the script after generating and building
    #[arg(short, long, conflicts_with_all(["edit","expression", "filter", "repl", "stdin"]))]
    pub norun: bool,
    /// Evaluate a quoted expression on the fly
    #[arg(short, long = "expr", conflicts_with_all(["generate", "build"]))]
    pub expression: Option<String>,
    /// REPL mode (read–eval–print loop) for Rust expressions. Option: existing script name
    #[arg(short = 'r', long, conflicts_with_all(["generate", "build"]))]
    pub repl: bool,
    /// Read script from stdin
    #[arg(short, long, conflicts_with_all(["generate", "build"]))]
    pub stdin: bool,
    /// Simple TUI edit-submit with history; editor will also capture any stdin input
    #[arg(short = 'd', long, conflicts_with_all(["generate", "build"]))]
    pub edit: bool,
    /// Filter expression to be run in a loop against every line in stdin, with optional pre- and post-loop logic.
    #[arg(short = 'l', long = "loop", conflicts_with_all(["generate", "build"]))]
    pub filter: Option<String>,
    /// Optional manifesto info in format ready for Cargo.toml
    #[arg(short = 'C', long, requires = "filter", value_name = "CARGO-TOML")]
    pub cargo: Option<String>,
    /// Optional awk-style pre-loop logic for --loop, somewhat like awk BEGIN
    #[arg(short = 'B', long, requires = "filter", value_name = "PRE-LOOP")]
    pub begin: Option<String>,
    /// Optional post-loop logic for --loop, somewhat like awk END
    #[arg(short = 'E', long, requires = "filter", value_name = "POST-LOOP")]
    pub end: Option<String>,
    /// Allow multiple main methods
    #[arg(short, long)]
    pub multimain: bool,
    /// Cargo check script if compiled file is stale. Less thorough than build.
    /// Used by by integration test suite for mass sanity check.
    #[arg(short, long, conflicts_with_all(["build", "executable"]))]
    pub check: bool,
    /// Build executable `home_dir`/.cargo/bin/`stem` from script `stem`.rs using `cargo build --release`.
    #[arg(short = 'x', long)]
    pub executable: bool,
    /// Set verbose mode
    #[arg(short, long)]
    pub verbose: bool,
    /// Suppress unnecessary output, double up to show only errors
    #[arg(short, long, action = clap::ArgAction::Count, conflicts_with("verbose"))]
    pub quiet: u8,
    /// Display timings
    #[arg(short, long)]
    pub timings: bool,
}

/// Getter for clap command-line arguments
#[must_use]
pub fn get_args() -> Cli {
    Cli::parse()
}

/// Validates the command-line arguments
/// # Errors
/// Will return `Err` if there is a missing script name or missing .rs suffix.
pub fn validate_args(args: &Cli, proc_flags: &ProcFlags) -> Result<(), Box<dyn Error>> {
    if let Some(ref script) = args.script {
        if !script.ends_with(RS_SUFFIX) {
            return Err(Box::new(BuildRunError::Command(format!(
                "Script name {script} must end in {RS_SUFFIX}"
            ))));
        }
    } else if !proc_flags.contains(ProcFlags::EXPR)
        && !proc_flags.contains(ProcFlags::REPL)
        && !proc_flags.contains(ProcFlags::STDIN)
        && !proc_flags.contains(ProcFlags::EDIT)
        && !proc_flags.contains(ProcFlags::LOOP)
    {
        return Err(Box::new(BuildRunError::Command(
            "Missing script name".to_string(),
        )));
    }
    Ok(())
}

bitflags! {
    // You can `#[derive]` the `Debug` trait, but implementing it manually
    // can produce output like `A | B` instead of `Flags(A | B)`.
    // #[derive(Debug)]
    #[derive(Clone, Default, PartialEq, Eq)]
    /// Processing flags for ease of handling command-line options
    pub struct ProcFlags: u32 {
        const GENERATE = 1;
        const BUILD = 2;
        const FORCE = 4;
        const RUN = 8;
        const ALL = 16;
        const VERBOSE = 32;
        const TIMINGS = 64;
        const REPL = 128;
        const EXPR = 256;
        const STDIN = 512;
        const EDIT = 1024;
        const QUIET = 2048;
        const QUIETER = 4096;
        const MULTI = 8192;
        const NORUN = 16384;
        const EXECUTABLE = 32768;
        const LOOP = 65536;
        const CHECK = 131_072;
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
pub fn get_proc_flags(args: &Cli) -> Result<ProcFlags, Box<dyn Error>> {
    // eprintln!("args={args:#?}");
    let is_expr = args.expression.is_some();
    let is_loop = args.filter.is_some();
    let proc_flags = {
        let mut proc_flags = ProcFlags::empty();
        // TODO: out? once clap default_value_ifs is working
        proc_flags.set(
            ProcFlags::GENERATE,
            args.generate | args.force | is_expr | args.executable | args.check,
        );
        proc_flags.set(
            ProcFlags::BUILD,
            args.build | args.force | is_expr | args.executable,
        );
        proc_flags.set(ProcFlags::CHECK, args.check);
        proc_flags.set(ProcFlags::FORCE, args.force);
        proc_flags.set(ProcFlags::QUIET, args.quiet == 1);
        proc_flags.set(ProcFlags::QUIETER, args.quiet >= 2);
        proc_flags.set(ProcFlags::MULTI, args.multimain);
        proc_flags.set(ProcFlags::VERBOSE, args.verbose);
        proc_flags.set(ProcFlags::TIMINGS, args.timings);
        proc_flags.set(ProcFlags::NORUN, args.norun | args.check | args.executable);
        proc_flags.set(
            ProcFlags::RUN,
            !args.norun && !args.build && !args.executable && !args.check,
        );
        proc_flags.set(
            ProcFlags::ALL,
            !args.norun && !args.build && !args.executable && !args.check,
        );
        if proc_flags.contains(ProcFlags::ALL) {
            proc_flags.set(ProcFlags::GENERATE, true);
            proc_flags.set(ProcFlags::BUILD, true);
        } else {
            proc_flags.set(ProcFlags::ALL, args.generate & args.build);
        }
        proc_flags.set(ProcFlags::REPL, args.repl);
        proc_flags.set(ProcFlags::EXPR, is_expr);
        proc_flags.set(ProcFlags::STDIN, args.stdin);
        proc_flags.set(ProcFlags::EDIT, args.edit);
        proc_flags.set(ProcFlags::LOOP, is_loop);
        // proc_flags.set(ProcFlags::TOML, is_toml);
        // proc_flags.set(ProcFlags::BEGIN, is_begin);
        // proc_flags.set(ProcFlags::END, is_end);
        proc_flags.set(ProcFlags::EXECUTABLE, args.executable);

        let verbosity = if args.verbose {
            Verbosity::Verbose
        } else if args.quiet == 1 {
            Verbosity::Quiet
        } else if args.quiet == 2 {
            Verbosity::Quieter
        } else {
            Verbosity::Normal
        };
        logging::set_global_verbosity(verbosity);

        // Check all good
        let formatted = proc_flags.to_string();
        let parsed = formatted
            .parse::<ProcFlags>()
            .map_err(|e| BuildRunError::FromStr(e.to_string()))?;

        assert_eq!(proc_flags, parsed);

        Ok::<ProcFlags, BuildRunError>(proc_flags)
    }?;
    Ok(proc_flags)
}
