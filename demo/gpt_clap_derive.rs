/*[toml]
[dependencies]
clap = { version = "4.5.7", features = ["derive"] }
*/

use clap::Parser;

/// GPT-generated CLI using the `clap` crate.
//# Purpose: Demonstrate `clap` CLI using the derive option.
// Script Runner
#[derive(Debug, Parser)]
#[clap(version = "1.0", author = "Your Name")]
struct Opt {
    /// Sets the script to run
    script: String,
    /// Sets the arguments for the script
    #[clap(last = true)]
    args: Vec<String>,
    /// Sets the level of verbosity
    #[clap(short, long)]
    verbose: bool,
    /// Displays timings
    #[clap(short, long)]
    timings: bool,
    /// Generates something
    #[clap(short, long)]
    generate: bool,
    /// Builds something
    #[clap(short, long)]
    build: bool,
}

fn main() {
    let opt = Opt::parse();

    if opt.verbose {
        println!("Verbosity enabled");
    }

    if opt.timings {
        println!("Timings enabled");
    }

    if opt.generate {
        println!("Generating something");
    }

    if opt.build {
        println!("Building something");
    }

    println!("Running script: {}", opt.script);
    if !opt.args.is_empty() {
        println!("With arguments:");
        for arg in &opt.args {
            println!("{}", arg);
        }
    }
}
