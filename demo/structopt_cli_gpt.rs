/*[toml]
[dependencies]
structopt = "0.3.26"
*/

/// Basic demo of GPT-generated CLI using the `structopt` crate. This
/// crate is in maintenance mode, its features having been integrated
/// into `clap`.
//# Purpose: Demonstrate `structopt` CLI.
//# Categories: CLI, crates, technique
//# Sample arguments: `-- -- -Vt dummy.rs 1 2 3`
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "script_runner", about = "A command-line script runner")]
struct Opt {
    /// Show help
    #[structopt(short = "h", long = "help")]
    help: bool,

    /// Show version
    #[structopt(short = "v", long = "version")]
    version: bool,

    /// Enable verbose mode
    #[structopt(short = "V", long = "verbose")]
    verbose: bool,

    /// Show timings
    #[structopt(short = "t", long = "timings")]
    timings: bool,

    /// Generate
    #[structopt(long = "generate")]
    generate: bool,

    /// Build
    #[structopt(long = "build")]
    build: bool,

    /// Script to run
    script: Option<String>,

    /// Arguments for the script
    #[structopt(name = "SCRIPT_ARGS", parse(from_str))]
    script_args: Vec<String>,
}

    let opt = Opt::from_args();

    if opt.help {
        println!("{:?}",Opt::clap().print_help()?);
    } else if opt.version {
        println!("Script Runner v0.1");
    } else if opt.script.is_some() {
        println!("Script to run: {:?}", opt.script);
        println!("Arguments for the script to run: {:?}", opt.script_args);
        println!("Put script run code here");
    } else {
        // Your logic for handling other options and executing the script goes here

        println!("Verbose mode: {}", opt.verbose);
        println!("Show timings: {}", opt.timings);
        println!("Generate: {}", opt.generate);
        println!("Build: {}", opt.build);
    }
