/*[toml]
[dependencies]
clap = { version = "4.5.4", features = ["cargo", "derive"] }
convert_case = "0.6.0"
serde = { version = "1.0.198", features = ["derive"] }
strum = { version = "0.26.2", features = ["derive"] }
*/

use clap::{Parser, ValueEnum};
use convert_case::{Case, Casing};
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
    ValueEnum,
    EnumIter,
    EnumProperty,
    IntoStaticStr
    // Copy,
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
// #[serde(rename_all = "kebab-case")]
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

// #[derive(Deserialize, Debug)]
// pub(crate) struct Check {
//     // #[serde(flatten)]
//     pub condition: Option<Opt>,
// }

fn main() {
    let cli = Cli::parse();

    match cli.opt {
        Opt::VariantNum1 => println!("Door number 1"),
        Opt::VariantNum2 => println!("Door number 2"),
        Opt::VariantNum3 => println!("Door number 3"),
    }

    // println!("crate_version={:#?}", cli.get_version());
    assert_eq!("MY VARIABLE NAME", "My variable NAME".to_case(Case::Upper));

    // println!(
    //     "Check={:?}",
    //     Check {
    //         condition: Some(cli.opt)
    //     }
    // );

    // Using strum and convert_case, but note that the latter's kebab case
    // doesn't match serde's version when it comes to numbers :(
    for option in Opt::iter() {
        println!(
            "option: {:?}, {:?}, {}",
            option,
            option.get_str("key").unwrap(),
            <Opt as Into<&'static str>>::into(option).to_case(Case::Kebab)
        );
    }

    // Print summary of enum variants
    println!("Enum Variants:");
    for variant in Opt::value_variants() {
        let poss_val = variant.to_possible_value().unwrap();
        println!("{}={poss_val:#?}", poss_val.get_name());
        // Optionally, print doc comments if available
        if let Some(comment) = poss_val.get_help() {
            println!("Help = {}", comment);
        }
    }
}
