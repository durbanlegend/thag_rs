use core::{fmt, str};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "build_run",
    about = "Build and run a given Rust programs, with separate and combined options for stages"
)]
pub(crate) struct Opt {
    #[structopt(subcommand)]
    pub(crate) action: Action,
    #[structopt(
        short = "S",
        long = "check-source",
        help = "Check for changes in source code"
    )]
    #[structopt(short = "v", long = "verbose", help = "Enable verbose output")]
    pub(crate) verbose: bool,
    #[structopt(short = "t", long = "timings", help = "Print timings for each stage")]
    pub(crate) timings: bool,
}

bitflags::bitflags! {
    // You can `#[derive]` the `Debug` trait, but implementing it manually
    // can produce output like `A | B` instead of `Flags(A | B)`.
    // #[derive(Debug)]
    #[derive(PartialEq, Eq)]
    pub struct Flags: u32 {
        const VERBOSE = 1;
        const TIMINGS = 2;
    }
}

impl fmt::Debug for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

impl str::FromStr for Flags {
    type Err = bitflags::parser::ParseError;

    fn from_str(flags: &str) -> Result<Self, Self::Err> {
        bitflags::parser::from_str(flags)
    }
}

#[derive(Debug, PartialEq, StructOpt)]
pub(crate) enum Action {
    #[structopt(
        name = "all",
        about = "Generate, build and run a Rust program from source code"
    )]
    All,
    #[structopt(name = "gen", about = "Generate Cargo.toml and source code")]
    Generate(GenQualifier),
    #[structopt(name = "build", about = "Build the executable from generated code")]
    Build,
    #[structopt(
        name = "gen-and-build",
        about = "Generate Cargo.toml and source code, then build"
    )]
    GenAndBuild,
    #[structopt(name = "run", about = "Run the generated program (if already built)")]
    Run,
    #[structopt(name = "build-and-run", about = "Build and run the generated program")]
    BuildAndRun,
    #[structopt(name = "check-cargo", about = "Check for changes in Cargo.toml")]
    CheckCargo,
    #[structopt(name = "check-source", about = "Check for changes in source code")]
    CheckSource,
}

#[derive(StructOpt, Debug, PartialEq)]
pub enum GenQualifier {
    #[structopt(name = "both", about = "Generate both source and Cargo.toml")]
    Both,
    #[structopt(name = "c", about = "Generate Cargo.toml only")]
    CargoToml,
    #[structopt(name = "s", about = "Generate source only")]
    Source,
}

#[derive(StructOpt, Debug, PartialEq)]
pub enum BuildQualifier {
    #[structopt(
        name = "full",
        about = "Generate source and Cargo.toml before building"
    )]
    Full,
    #[structopt(name = "only", about = "Build only, don't generate first")]
    Only,
}
