/*[toml]
[dependencies]
bitflags = "2.5.0"
clap = { version = "4.5.7", features = ["cargo", "derive"] }
thag_rs = "0.1.7"
*/

/// A prototype of the cmd_args module of thag_rs itself.
///
/// E.g. `thag -tv demo/cmd_args.rs -- -gbrtv demo/hello.rs -- -fq Hello world`
//# Purpose: Prototype CLI.
//# Categories: CLI, crates, prototype, technique
use bitflags::bitflags;
use clap::{ArgGroup, Parser};
use std::{error::Error, fmt, str};
use thag_rs::{errors::ThagError, logging::Verbosity, vlog, RS_SUFFIX};

// thag_rs script runner and REPL
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Default, Parser, Debug)]
#[command(name = "thag_rs", version, about, long_about)]
#[command(group(
            ArgGroup::new("commands")
                .required(true)
                .args(&["script", "expression", "repl", "filter", "stdin", "edit", "config"]),
        ))]
#[command(group(
            ArgGroup::new("volume")
                .required(false)
                .args(&["quiet", "normal", "verbose"]),
        ))]
pub struct Cli {
    /// Optional name of a script to run (`stem`.rs)
    pub script: Option<String>,
    /// Set the arguments for the script
    #[arg(last = true, requires = "script")]
    pub args: Vec<String>,
    /// Generate Rust source and individual cargo .toml if compiled file is stale
    #[arg(short, long = "gen", default_value_ifs([
        ("force", "true", "true"),
        ("expression", "_", "true"),
        ("executable", "true", "true"),
        ("check", "true", "true"),
    ]))]
    pub generate: bool,
    /// Build script if compiled file is stale
    #[arg(short, long, default_value_ifs([
        ("force", "true", "true"),
        ("expression", "_", "true"),
        ("executable", "true", "true"),
    ]))]
    pub build: bool,
    /// Force generation of Rust source and individual Cargo.toml, and build, even if compiled file is not stale
    #[arg(short, long)]
    pub force: bool,
    /// Don't run the script after generating and building
    #[arg(short, long, conflicts_with_all(["edit", "expression", "filter", "repl", "stdin"]))]
    pub norun: bool,
    /// Build executable `home_dir`/.cargo/bin/`stem` from script `stem`.rs using `cargo build --release`
    #[arg(short = 'x', long)]
    pub executable: bool,
    /// Cargo check script if compiled file is stale. Less thorough than build
    /// Used by integration test to check all demo scripts
    #[arg(short, long, conflicts_with_all(["build", "executable"]))]
    pub check: bool,
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
    /// Optional manifest info for --loop in format ready for Cargo.toml
    //  clap issue 4707 may prevent `requires` from working, as I've experienced.
    #[arg(short = 'T', long, requires = "filter", value_name = "CARGO-TOML")]
    pub toml: Option<String>,
    /// Optional pre-loop logic for --loop, somewhat like awk BEGIN
    //  clap issue 4707 may prevent `requires` from working, as I've experienced.
    #[arg(short = 'B', long, requires = "filter", value_name = "PRE-LOOP")]
    pub begin: Option<String>,
    /// Optional post-loop logic for --loop, somewhat like awk END
    //  clap issue 4707 may prevent `requires` from working, as I've experienced.
    #[arg(short = 'E', long, requires = "filter", value_name = "POST-LOOP")]
    pub end: Option<String>,
    /// Confirm that multiple main methods are valid for this script
    #[arg(short, long)]
    pub multimain: bool,
    /// Display timings
    #[arg(short, long)]
    pub timings: bool,
    /// Set verbose mode
    #[arg(short, long)]
    pub verbose: bool,
    /// Set normal verbosity, only needed to override config value
    #[arg(short = 'N', long = "normal verbosity")]
    pub normal: bool,
    /// Suppress unnecessary output, double up to show only errors
    #[arg(short, long, action = clap::ArgAction::Count, conflicts_with("verbose"))]
    pub quiet: u8,
    /// Strip double quotes from string result of expression (true/false). Default: config value / false.
    #[arg(
        short,
        long,
        // require_equals = true,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",  // Default to true if -u is present but no value is given
        conflicts_with("multimain")
    )]
    pub unquote: Option<bool>,
    // Edit configuration
    #[arg(short = 'C', long, conflicts_with_all(["generate", "build", "executable"]))]
    pub config: bool,
}

// Getter for clap command-line arguments
#[must_use]
pub fn get_args() -> Cli {
    Cli::parse()
}

pub fn validate_args(args: &Cli, proc_flags: &ProcFlags) -> Result<(), Box<dyn Error>> {
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
    // Processing flags for ease of handling command-line options.
    // You can `#[derive]` the `Debug` trait, but implementing it manually
    // can produce output like `A | B` instead of `Flags(A | B)`.
    // #[derive(Debug)]
    #[derive(Clone, Default, PartialEq, Eq)]
    pub struct ProcFlags: u32 {
        const GENERATE = 1;
        const BUILD = 2;
        const FORCE = 4;
        const RUN = 8;
        const NORUN = 16;
        const EXECUTABLE = 32;
        const CHECK = 64;
        const REPL = 128;
        const EXPR = 256;
        const STDIN = 512;
        const EDIT = 1024;
        const LOOP = 2048;
        const MULTI = 4096;
        const TIMINGS = 8192;
        const VERBOSE = 16384;
        const NORMAL = 32768;
        const QUIET = 65536;
        const QUIETER = 131_072;
        const UNQUOTE = 262_144;
        const CONFIG = 524_288;
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

// Set up the processing flags from the command line arguments and pass them back.
// # Errors
//
// Will return `Err` if there is an error parsing the flags to set up and internal
// correctness chack.
// # Panics
//
// Will panic if the internal correctness check fails.
pub fn get_proc_flags(args: &Cli) -> Result<ProcFlags, Box<dyn Error>> {
    let is_expr = args.expression.is_some();
    let is_loop = args.filter.is_some();
    let proc_flags = {
        let mut proc_flags = ProcFlags::empty();
        eprintln!("args={args:#?}");
        proc_flags.set(ProcFlags::GENERATE, args.generate);
        // eprintln!(
        //     "After set(ProcFlags::GENERATE, args.generate), ProcFlags::GENERATE = {:#?}",
        //     ProcFlags::GENERATE
        // );

        proc_flags.set(ProcFlags::BUILD, args.build);
        proc_flags.set(ProcFlags::CHECK, args.check);
        proc_flags.set(ProcFlags::FORCE, args.force);
        proc_flags.set(ProcFlags::QUIET, args.quiet == 1);
        proc_flags.set(ProcFlags::QUIETER, args.quiet >= 2);
        proc_flags.set(ProcFlags::MULTI, args.multimain);
        proc_flags.set(ProcFlags::VERBOSE, args.verbose);
        proc_flags.set(ProcFlags::TIMINGS, args.timings);
        proc_flags.set(ProcFlags::NORUN, args.norun | args.check | args.executable);
        proc_flags.set(ProcFlags::NORMAL, args.normal);
        let gen_build = !args.norun && !args.executable && !args.check;
        eprintln!("gen_build={gen_build}");
        if gen_build {
            proc_flags.set(ProcFlags::GENERATE | ProcFlags::BUILD, true);
        }
        proc_flags.set(ProcFlags::RUN, !proc_flags.contains(ProcFlags::NORUN));
        // eprintln!("After processing ALL, proc_flags={proc_flags:#?}");
        proc_flags.set(ProcFlags::REPL, args.repl);
        proc_flags.set(ProcFlags::EXPR, is_expr);
        proc_flags.set(ProcFlags::STDIN, args.stdin);
        proc_flags.set(ProcFlags::EDIT, args.edit);
        proc_flags.set(ProcFlags::LOOP, is_loop);
        proc_flags.set(ProcFlags::EXECUTABLE, args.executable);
        proc_flags.set(ProcFlags::CONFIG, args.config);

        if !is_loop && (args.toml.is_some() || args.begin.is_some() || args.end.is_some()) {
            if args.toml.is_some() {
                eprintln!("Option {} ({}) requires --loop (-l)", "--toml", "-T");
            }
            if args.begin.is_some() {
                eprintln!("Option {} ({}) requires --loop (-l)", "--begin", "-B");
            }
            if args.end.is_some() {
                eprintln!("Option {} ({}) requires --loop (-l)", "--end", "-E");
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

#[allow(dead_code)]
fn type_of<T>(_x: &T) -> String {
    std::any::type_name::<T>().to_string()
}

#[allow(dead_code)]
fn main() {
    let opt = Cli::parse();

    if opt.verbose {
        vlog!(Verbosity::Normal, "Verbosity enabled");
    }

    if opt.timings {
        vlog!(Verbosity::Normal, "Timings enabled");
    }

    if opt.generate {
        vlog!(Verbosity::Normal, "Generate option selected");
    }

    if opt.build {
        vlog!(Verbosity::Normal, "Build option selected");
    }

    if opt.force {
        vlog!(Verbosity::Normal, "Force option selected");
    }

    if opt.executable {
        vlog!(Verbosity::Normal, "Executable option selected");
    }

    vlog!(Verbosity::Normal, "Unquote={:#?}", opt.unquote);

    if opt.executable {
        vlog!(Verbosity::Normal, "Config option selected");
    }

    vlog!(Verbosity::Normal, "Script to run: {:?}", opt.script);
    if !opt.args.is_empty() {
        vlog!(Verbosity::Normal, "With arguments:");
        for arg in &opt.args {
            vlog!(Verbosity::Normal, "{arg}");
        }
    }

    let proc_flags = get_proc_flags(&opt);
    println!("proc_flags={proc_flags:#?}");
}
