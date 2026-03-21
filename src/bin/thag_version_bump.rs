/*[toml]
[dependencies]
thag_styling = { version = "1, thag-auto", features = ["inquire_theming"] }
toml_edit = "0.22"
regex = "1.11"
*/

/// Tool to bump version numbers across the thag_rs workspace for major releases.
///
/// This utility automates the process of updating version numbers in:
/// - All Cargo.toml files (workspace members and dependencies)
/// - All demo scripts with thag-auto dependencies
/// - All tool binaries with thag-auto dependencies
///
/// It's designed to handle the coordinated version bumps needed when releasing
/// a new major version across all workspace crates.
//# Purpose: Automate version bumping across workspace for major releases
//# Categories: tools
//# Usage: thag_version_bump [--dry-run] [--version VERSION]
use inquire::{set_global_render_config, Confirm, Text};
use regex::Regex;

use std::fs;
use std::path::{Path, PathBuf};
use thag_styling::{auto_help, help_system::check_help_and_exit, themed_inquire_config};
use toml_edit::{DocumentMut, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for help first
    let help = auto_help!();
    check_help_and_exit(&help);

    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|arg| arg == "--dry-run" || arg == "-n");

    // Check for --version argument
    let version_arg = args
        .iter()
        .position(|arg| arg == "--version" || arg == "-v")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.to_string());

    if dry_run {
        println!("🔍 DRY RUN MODE - No files will be modified\n");
    }

    println!("🚀 thag_rs Version Bump Tool");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Define the workspace structure
    let workspace_crates = vec![
        ("thag_rs", "0.2.2"),
        ("thag_common", "0.2.1"),
        ("thag_demo", "0.2.1"),
        ("thag_proc_macros", "0.2.1"),
        ("thag_profiler", "0.1.1"),
        ("thag_styling", "0.2.1"),
    ];

    println!("📦 Current versions:");
    for (name, version) in &workspace_crates {
        println!("  {} = {}", name, version);
    }
    println!();

    // Get target version from args or prompt
    let target_version = if let Some(ref v) = version_arg {
        v.clone()
    } else {
        set_global_render_config(themed_inquire_config());
        Text::new("Enter target version (e.g., 1.0.0):")
            .with_default("1.0.0")
            .prompt()?
    };

    // Validate semver format
    if !is_valid_semver(&target_version) {
        eprintln!("❌ Invalid semver format: {}", target_version);
        return Ok(());
    }

    let target_version_short = extract_major_minor(&target_version);

    println!("\n📋 Plan:");
    println!(
        "  • Update all workspace crate versions to: {}",
        target_version
    );
    let dep_version_note = if target_version.starts_with("0.") {
        format!("{} (major.minor for 0.x)", target_version_short)
    } else {
        format!("{} (major only for 1.x+)", target_version_short)
    };

    println!(
        "  • Update all dependency references to: {}",
        dep_version_note
    );
    println!(
        "  • Update all thag-auto scripts to: {}, thag-auto",
        target_version_short
    );

    if !dry_run && version_arg.is_none() {
        set_global_render_config(themed_inquire_config());
        let confirm = Confirm::new("Proceed with version bump?")
            .with_default(false)
            .prompt()?;

        if !confirm {
            println!("❌ Aborted");
            return Ok(());
        }
    }

    println!("\n🔧 Processing files...\n");

    let mut stats = Stats::new();

    // Step 1: Update workspace Cargo.toml files
    println!("1️⃣  Updating workspace Cargo.toml files...");
    for (crate_name, _) in &workspace_crates {
        let cargo_toml_path = if *crate_name == "thag_rs" {
            PathBuf::from("Cargo.toml")
        } else {
            PathBuf::from(format!("{}/Cargo.toml", crate_name))
        };

        if cargo_toml_path.exists() {
            update_cargo_toml(
                &cargo_toml_path,
                &target_version,
                &target_version_short,
                &workspace_crates,
                dry_run,
                &mut stats,
            )?;
        }
    }

    // Step 2: Update demo scripts
    println!("\n2️⃣  Updating demo scripts...");
    let demo_dir = Path::new("demo");
    if demo_dir.exists() {
        update_scripts_in_directory(demo_dir, &target_version_short, dry_run, &mut stats)?;
    }

    // Step 3: Update tool binaries
    println!("\n3️⃣  Updating tool binaries...");
    let bin_dir = Path::new("src/bin");
    if bin_dir.exists() {
        update_scripts_in_directory(bin_dir, &target_version_short, dry_run, &mut stats)?;
    }

    // Print summary
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Summary:");
    println!("  Cargo.toml files updated: {}", stats.cargo_tomls);
    println!("  Demo scripts updated: {}", stats.demo_scripts);
    println!("  Tool binaries updated: {}", stats.tool_bins);
    println!("  Total files processed: {}", stats.total());

    if dry_run {
        println!("\n⚠️  DRY RUN - No changes were made");
        println!("   Run without --dry-run to apply changes");
    } else {
        println!("\n✅ Version bump complete!");
        println!("\n📝 Next steps:");
        println!("  1. Review changes: git diff");
        println!("  2. Test locally: THAG_DEV_PATH=$PWD cargo test");
        println!(
            "  3. Commit: git commit -am 'chore: bump version to {}'",
            target_version
        );
        println!("  4. Publish subcrates in order (see release checklist)");
    }

    Ok(())
}

struct Stats {
    cargo_tomls: usize,
    demo_scripts: usize,
    tool_bins: usize,
}

impl Stats {
    fn new() -> Self {
        Self {
            cargo_tomls: 0,
            demo_scripts: 0,
            tool_bins: 0,
        }
    }

    fn total(&self) -> usize {
        self.cargo_tomls + self.demo_scripts + self.tool_bins
    }
}

fn is_valid_semver(version: &str) -> bool {
    let re = Regex::new(r"^\d+\.\d+\.\d+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$").unwrap();
    re.is_match(version)
}

fn extract_major_minor(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.is_empty() {
        return version.to_string();
    }

    let major = parts[0];

    // For 0.x versions, use major.minor (since minor can have breaking changes)
    // For 1.x and above, use just major (follows SemVer properly)
    if major == "0" && parts.len() > 1 {
        format!("{}.{}", parts[0], parts[1])
    } else {
        major.to_string()
    }
}

fn update_cargo_toml(
    path: &Path,
    full_version: &str,
    short_version: &str,
    workspace_crates: &[(&str, &str)],
    dry_run: bool,
    stats: &mut Stats,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut doc = content.parse::<DocumentMut>()?;
    let mut modified = false;

    // Update package version
    if let Some(package) = doc.get_mut("package").and_then(|p| p.as_table_mut()) {
        if let Some(version_item) = package.get_mut("version") {
            if let Some(version_val) = version_item.as_value_mut() {
                *version_val = Value::from(full_version);
                modified = true;
                println!(
                    "  ✓ {} [package.version = {}]",
                    path.display(),
                    full_version
                );
            }
        }
    }

    // Update workspace member dependencies
    let dependency_sections = vec!["dependencies", "dev-dependencies", "build-dependencies"];

    for section in dependency_sections {
        if let Some(deps) = doc.get_mut(section).and_then(|d| d.as_table_mut()) {
            for (crate_name, _) in workspace_crates {
                if let Some(dep_item) = deps.get_mut(*crate_name) {
                    if let Some(dep_table) = dep_item.as_inline_table_mut() {
                        if dep_table.contains_key("path") {
                            // This is a workspace dependency
                            if let Some(version_val) = dep_table.get_mut("version") {
                                *version_val = Value::from(short_version);
                                modified = true;
                                println!(
                                    "  ✓ {} [{}.{}.version = {}]",
                                    path.display(),
                                    section,
                                    crate_name,
                                    short_version
                                );
                            }
                        }
                    } else if let Some(dep_table) = dep_item.as_table_mut() {
                        if dep_table.contains_key("path") {
                            if let Some(version_item) = dep_table.get_mut("version") {
                                if let Some(version_val) = version_item.as_value_mut() {
                                    *version_val = Value::from(short_version);
                                    modified = true;
                                    println!(
                                        "  ✓ {} [{}.{}.version = {}]",
                                        path.display(),
                                        section,
                                        crate_name,
                                        short_version
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if modified && !dry_run {
        fs::write(path, doc.to_string())?;
        stats.cargo_tomls += 1;
    } else if modified {
        stats.cargo_tomls += 1;
    }

    Ok(())
}

fn update_scripts_in_directory(
    dir: &Path,
    version: &str,
    dry_run: bool,
    stats: &mut Stats,
) -> Result<(), Box<dyn std::error::Error>> {
    let is_demo = dir.ends_with("demo");

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if update_script_toml_block(&path, version, dry_run)? {
                if is_demo {
                    stats.demo_scripts += 1;
                } else {
                    stats.tool_bins += 1;
                }
            }
        }
    }

    Ok(())
}

fn update_script_toml_block(
    path: &Path,
    version: &str,
    dry_run: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;

    // Check if file has thag-auto dependencies
    if !content.contains("thag-auto") {
        return Ok(false);
    }

    // Pattern to match thag crate dependencies with thag-auto
    let patterns = vec![
        (
            r#"thag_rs\s*=\s*\{\s*version\s*=\s*"(\d+\.\d+),\s*thag-auto""#,
            "thag_rs",
        ),
        (
            r#"thag_common\s*=\s*\{\s*version\s*=\s*"(\d+\.\d+),\s*thag-auto""#,
            "thag_common",
        ),
        (
            r#"thag_proc_macros\s*=\s*\{\s*version\s*=\s*"(\d+\.\d+),\s*thag-auto""#,
            "thag_proc_macros",
        ),
        (
            r#"thag_profiler\s*=\s*\{\s*version\s*=\s*"(\d+\.\d+),\s*thag-auto""#,
            "thag_profiler",
        ),
        (
            r#"thag_styling\s*=\s*\{\s*version\s*=\s*"(\d+\.\d+),\s*thag-auto""#,
            "thag_styling",
        ),
        (
            r#"thag_demo\s*=\s*\{\s*version\s*=\s*"(\d+\.\d+),\s*thag-auto""#,
            "thag_demo",
        ),
    ];

    let mut new_content = content.clone();
    let mut modified = false;

    for (pattern, crate_name) in patterns {
        let re = Regex::new(pattern)?;

        if re.is_match(&new_content) {
            let replacement = format!(r#"{} = {{ version = "{}, thag-auto""#, crate_name, version);
            let new = re.replace_all(&new_content, replacement);

            if new != new_content {
                new_content = new.to_string();
                modified = true;
            }
        }
    }

    if modified {
        println!("  ✓ {}", path.display());
        if !dry_run {
            fs::write(path, new_content)?;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}
