/*[toml]
[dependencies]
clap = { version = "4.5.7", features = ["cargo", "derive"] }
*/

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// What mode to run the program in
    #[arg(value_enum)]
    mode: Mode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Run swiftly
    Fast,
    /// Crawl slowly but steadily
    ///
    /// This paragraph is ignored because there is no long help text for possible values.
    Slow,
}

/// Published example from `clap` tutorial (derive), with added displays.
///
/// E.g. rs_script demo/clap_tut_04.rs -ddd -c dummy.cfg test -l
//# Purpose: Demonstrate `clap` CLI using the derive option
fn main() {
    let cli = Cli::parse();

    match cli.mode {
        Mode::Fast => {
            println!("Hare");
        }
        Mode::Slow => {
            println!("Tortoise");
        }
    }

    println!();

    // println!("{}", cli.long_version(crate_version!()));

    // Print summary of enum variants
    println!("Enum Variants:");
    for variant in Mode::value_variants() {
        let poss_val = variant.to_possible_value().unwrap();
        println!("{:#?}", poss_val);
        // Optionally, print doc comments if available
        if let Some(comment) = poss_val.get_help() {
            println!("Help = {}", comment);
        }
    }
    println!();
}
