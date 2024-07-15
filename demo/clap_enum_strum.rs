/*[toml]
[dependencies]
clap = { version = "4.5.7", features = ["cargo", "derive"] }
serde = { version = "1.0.204", features = ["derive"] }
strum = { version = "0.26.2", features = ["derive"] }
*/

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumProperty, IntoEnumIterator, IntoStaticStr};

#[derive(Parser)]
#[command(version, about, long_about = Some("This is the long 'about'"))]
struct Cli {
    /// Which of 3 variants to choose
    #[arg(value_enum)]
    opt: Opt,
}

#[derive(
    ValueEnum, EnumIter, EnumProperty, IntoStaticStr, Clone, Debug, Default, Serialize, Deserialize,
)]
#[strum(serialize_all = "kebab-case")]
enum Opt {
    #[default]
    /// Help for variant 1
    #[strum(props(key = "var1"))]
    VariantNum1,
    /// Help for variant 2
    #[strum(props(key = "var2"))]
    VariantNum2,
    /// Help for variant 3
    #[strum(props(key = "var3"))]
    VariantNum3,
}

/// Exploring using clap with an enum, in conjunction with strum.
/// E.g. `rs_script demo/clap_enum_strum.rs -- variant-num2`
//# Purpose: Simple demo of featured crates, contrasting how they expose variants.
fn main() {
    let cli = Cli::parse();

    println!("Chosen option:");
    match cli.opt {
        Opt::VariantNum1 => println!("Door number 1"),
        Opt::VariantNum2 => println!("Door number 2"),
        Opt::VariantNum3 => println!("Door number 3"),
    }

    // Using strum
    println!("\nEnum properties and text, using strum:");
    for option in Opt::iter() {
        println!(
            "strum associated property 'key'={}, variant={:#?}",
            option.get_str("key").unwrap(),
            <Opt as Into<&'static str>>::into(option),
        );
    }

    // Print summary of enum variants using clap
    println!("\nEnum Variants, using clap:");
    for variant in Opt::value_variants() {
        let poss_val = variant.to_possible_value().unwrap();
        println!("name={}\nvariant={poss_val:#?}", poss_val.get_name());
        // Optionally, print doc comments if available
        if let Some(comment) = poss_val.get_help() {
            println!("Help (from doc comment) = {}", comment);
        }
        println!();
    }
}
