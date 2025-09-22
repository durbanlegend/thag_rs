use cargo_lookup::{Query, Package, Release, Result};

fn highest_release(pkg: &Package) -> Option<&Release> {
    pkg.releases()
        .iter()
        .filter(|r| !r.yanked)
        .max_by_key(|r| r.vers.clone()) // vers is already semver::Version
}

fn main() -> Result<()> {
    // Create a Query from the crate name
    let query: Query = "inquire".parse()?;
    let pkg: Package = query.package()?;  // fetch the package info

    if let Some(best) = highest_release(&pkg) {
        println!("{} = \"{}\"", pkg.name(), best.vers);
    } else {
        println!("No non-yanked versions found for {}", pkg.name());
    }

    Ok(())
}
