/*[toml]
[dependencies]
clap = { version = "4.5.21", features = ["cargo", "derive"] }
*/

/// Published example from `clap` tutorial (derive), with added displays.
///
/// E.g. `thag demo/clap_tut_derive_04_01_enum.rs -- fast`
//# Purpose: Demonstrate `clap` CLI using the derive option
//# Categories: CLI, crates, technique
//# Sample arguments: `-- fast`
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

    // ** The original clap tutorial example ends here. **

    println!();

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
