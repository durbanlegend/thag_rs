use cargo_lookup::{Package, Query, Release, Result};

/// Updated prototype of getting the highest valid release of a crate via `cargo-lookup`.
/// The crate in its raw state only gets the latest. `thag_rs` was picking up `inquire`
/// 8.1 because it was released after 9.1 to fix the same issue on the previous version.
//# Purpose: Originally used to debug and then prototype crate lookup, now brought up to date.
//# Categories: debugging, prototype
fn highest_release(pkg: &Package) -> Option<&Release> {
    pkg.releases()
        .iter()
        .filter(|r| !r.yanked)
        .filter(|r| r.vers.pre.is_empty())
        .max_by_key(|r| r.vers.clone()) // vers is already semver::Version
}

fn main() -> Result<()> {
    // Create a Query from the crate name
    let query: Query = "inquire".parse()?;
    let pkg: Package = query.package()?; // fetch the package info

    if let Some(best) = highest_release(&pkg) {
        println!("{} = \"{}\"", pkg.name(), best.vers);
    } else {
        println!("No non-yanked versions found for {}", pkg.name());
    }

    Ok(())
}
