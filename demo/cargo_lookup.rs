/*[toml]
[dependencies]
cargo-lookup = "0.1.0"
*/
use cargo_lookup::{Query, Result};
use std::collections::BTreeMap;
use std::env;

/// Explore querying crates.io information for a crate.
///
/// Format: `thag demo/cargo_lookup.rs -- <crate_name>`
//# Purpose: proof of concept
//# Categories: crates, technique
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <crate_name>", args[0]);
        std::process::exit(1);
    }

    let query: Query = args[1].parse()?;
    let latest = query.package()?.into_latest().unwrap();
    println!("version={}", latest.vers);

    let features: BTreeMap<String, Vec<String>> = latest.features;
    let maybe_features2: Option<BTreeMap<String, Vec<String>>> = latest.features2;

    // println!("features={features:?}");
    println!("features={:?}", features.keys());
    if let Some(features2) = maybe_features2 {
        println!("features2={:?}", features2.keys());
    }

    Ok(())
}
