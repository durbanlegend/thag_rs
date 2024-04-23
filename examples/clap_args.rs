/*[toml]
[dependencies]
clap = { version = "4.5.4", features = ["cargo", "derive"] }
serde = { version = "1.0.198", features = ["derive"] }
strum = { version = "0.26", features = ["derive"] }
*/

use clap::{crate_version, Arg, Command, Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Parser)]
#[command(version, about, long_about = Some("This is the long 'about'"))]
struct Cli {
    /// Which of 3 variants to choose
    #[arg(value_enum)]
    opt: Opt,
}

#[derive(
    ValueEnum,
    EnumIter,
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[serde(rename_all = "kebab-case")]
enum Opt {
    #[default]
    /// Help for variant 1
    VariantNum1,
    /// Help for variant 2
    VariantNum2,
    /// Help for variant 3
    VariantNum3,
}

fn main() {
    let cli = Cli::parse();

    match cli.opt {
        Opt::VariantNum1 => println!("Door number 1"),
        Opt::VariantNum2 => println!("Door number 2"),
        Opt::VariantNum3 => println!("Door number 3"),
    }

    // println!("crate_version={:#?}", cli.get_version());

    for option in Opt::iter() {
        println!("option: {:?}", option);
    }

    // Print summary of enum variants
    println!("Enum Variants:");
    for variant in Opt::value_variants() {
        let poss_val = variant.to_possible_value().unwrap();
        println!("{:#?}", poss_val);
        // Optionally, print doc comments if available
        if let Some(comment) = poss_val.get_help() {
            println!("Help = {}", comment);
        }
    }
}
