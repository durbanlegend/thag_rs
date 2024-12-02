/*[toml]
[dependencies]
bitflags = "2.5.0"
clap = { version = "4.5.21", features = ["derive"] }
*/

/// Basic CLI example using clap.
///
/// E.g. `thag -t demo/cmd_args_clap.rs -- -atv hello.sh`
//# Purpose: For comparison with `demo/cmd_args_bpaf_gpt.rs`.
//# Categories: CLI, crates, technique
use bitflags::bitflags;
use clap::Parser;
use core::{fmt, str};

#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Your Name")]
pub(crate) struct Opt {
    /// Sets the script to run
    pub(crate) script: String,
    /// Sets the arguments for the script
    #[clap(last = true)]
    pub(crate) args: Vec<String>,
    /// Sets the level of verbosity
    #[clap(short, long)]
    pub(crate) verbose: bool,
    /// Displays timings
    #[clap(short, long)]
    pub(crate) timings: bool,
    /// Generates Rust source and individual cargo .toml
    #[clap(short = 'g', long = "gen")]
    pub(crate) generate: bool,
    /// Builds script
    #[clap(short, long)]
    pub(crate) build: bool,
    /// Generates, builds and runs script
    #[clap(short, long)]
    pub(crate) all: bool,
    /// Runs compiled script
    #[clap(short, long)]
    pub(crate) run: bool,
}

pub(crate) fn get_opt() -> Opt {
    Opt::parse()
}

#[allow(dead_code)]
fn main() {
    println!("In {}", env!("CARGO_PKG_NAME"));

    let opt = get_opt();

    if opt.verbose {
        println!("Verbosity enabled");
    }

    if opt.timings {
        println!("Timings enabled");
    }

    if opt.generate {
        println!("Generating source and cargo .toml file");
    }

    if opt.build {
        println!("Building something");
    }

    if opt.all {
        println!("Option 'all' selected");
    }

    if opt.run {
        println!("Running script");
    }

    println!("Running script: {}", opt.script);
    if !opt.args.is_empty() {
        println!("With arguments:");
        for arg in &opt.args {
            println!("{arg}");
        }
    }
}

bitflags! {
    // You can `#[derive]` the `Debug` trait, but implementing it manually
    // can produce output like `A | B` instead of `Flags(A | B)`.
    // #[derive(Debug)]
    #[derive(PartialEq, Eq)]
    pub struct ProcFlags: u32 {
        const GENERATE = 1;
        const BUILD = 2;
        const RUN = 4;
        const ALL = 8;
        const VERBOSE = 16;
        const TIMINGS = 32;
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
