/*[toml]
[dependencies]
cargo_toml = "0.20.4"
serde_merge = "0.1.3"
*/

/// Prototype of comprehensive merge of script toml metadata with defaults.
//# Purpose: Develop for inclusion in main project.
//# Categories: crates, prototype, technique
use cargo_toml::{Edition, Manifest};
use serde_merge::omerge;

fn create_default_manifest(source_stem: &str, gen_src_path: &str) -> Manifest {
    let cargo_manifest = format!(
        r##"[package]
name = "{0}"
version = "0.0.1"
edition = "2021"

[dependencies]

[features]

[patch]

[[bin]]
name = "{0}"
path = "{1}"
"##,
        source_stem, gen_src_path
    );

    Manifest::from_str(&cargo_manifest).expect("Failed to parse default manifest")
}

fn merge_manifests(
    default: Manifest,
    user_manifest_str: &str,
) -> Result<Manifest, Box<dyn std::error::Error>> {
    // Parse the user-provided manifest
    let user_manifest: Manifest = Manifest::from_str(user_manifest_str)?;

    // Merge the manifests
    let mut merged_manifest: Manifest = omerge(default, user_manifest)?;

    // Ensure all `[[bin]]` sections have the edition set to E2021
    let bins = &mut merged_manifest.bin;
    for bin in bins {
        // eprintln!("Found bin.edition={:#?}", bin.edition);
        // Don't accept the default of E2015. This is the only way I can think of
        // to stop it defaulting to E2015 and then overriding the template value.
        if matches!(bin.edition, Edition::E2015) {
            bin.edition = cargo_toml::Edition::E2021;
        }
    }

    Ok(merged_manifest)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_stem = "example";
    let gen_src_path = "src/main.rs";

    // Create the default manifest
    let default_manifest = create_default_manifest(source_stem, gen_src_path);

    // Example user TOML block (could be extracted from the script)
    let user_toml_str = r##"
    [profile.release]
    opt-level = 3

    [[bin]]
    name = "custom_name"
    "##;

    // Merge the manifests
    let merged_manifest = merge_manifests(default_manifest, user_toml_str)?;

    println!("{:#?}", merged_manifest);
    Ok(())
}
