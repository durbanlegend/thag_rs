/*[toml]
[dependencies]
cargo-lookup = "0.1.0"
*/

use cargo_lookup::{Query, Result};

/// Explore querying crates.io information for a crate.
//# Purpose: proof of concept
//# Categories: crates, technique
fn main() -> Result<()> {
    let query: Query = "thag_rs".parse()?;
    let latest_release_info = query.package()?.into_latest().unwrap().features; // .as_json_string();

    println!("{latest_release_info:?}");

    Ok(())
}
