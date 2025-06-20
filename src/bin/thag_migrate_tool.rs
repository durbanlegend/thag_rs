/*[toml]
[dependencies]
inquire = "0.7.5"
thag_rs = { path = "../..", default-features = false, features = ["core"] }
*/

/// Tool to help migrate existing tools from tools/ to src/bin/ with auto-help integration.
///
/// This utility helps migrate tools by:
/// - Moving files from tools/ to src/bin/
/// - Adding the auto_help! macro integration
/// - Updating Cargo.toml entries if needed
/// - Preserving all existing functionality
//# Purpose: Migrate tools from tools/ directory to src/bin/ with auto-help integration
//# Categories: tools
//# Usage: thag_migrate_tool [--help|-h]
use inquire::{Confirm, Select};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thag_rs::{auto_help, help_system::check_help_and_exit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for help first
    let help = auto_help!("thag_migrate_tool");
    check_help_and_exit(&help);

    println!("🔧 Tool Migration Helper");
    println!("This tool helps migrate existing tools from tools/ to src/bin/\n");

    // Find tools in the tools/ directory
    let tools_dir = Path::new("tools");
    if !tools_dir.exists() {
        println!("❌ tools/ directory not found");
        return Ok(());
    }

    let tools = find_rust_files(tools_dir)?;
    if tools.is_empty() {
        println!("❌ No .rs files found in tools/ directory");
        return Ok(());
    }

    println!("📁 Found {} tool(s) in tools/:", tools.len());
    for tool in &tools {
        println!("  • {}", tool.display());
    }

    // Select which tool to migrate
    let tool_names: Vec<String> = tools
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    let selected = Select::new("Select tool to migrate:", tool_names).prompt()?;

    let source_path = tools_dir.join(&selected);
    let dest_path = Path::new("src/bin").join(&selected);

    println!("\n📋 Migration Plan:");
    println!("  Source: {}", source_path.display());
    println!("  Destination: {}", dest_path.display());

    // Check if destination already exists
    if dest_path.exists() {
        println!("⚠️  Destination file already exists!");
        if !Confirm::new("Overwrite existing file?")
            .with_default(false)
            .prompt()?
        {
            println!("❌ Migration cancelled");
            return Ok(());
        }
    }

    // Confirm migration
    if !Confirm::new("Proceed with migration?")
        .with_default(true)
        .prompt()?
    {
        println!("❌ Migration cancelled");
        return Ok(());
    }

    // Check if we're in a git repository
    let is_git_repo = check_git_repo();

    // Perform the migration
    migrate_tool(&source_path, &dest_path, &selected, is_git_repo)?;

    // Update Cargo.toml
    let cargo_updated = update_cargo_toml(&selected)?;

    println!("\n✅ Migration completed successfully!");
    if cargo_updated {
        println!("✅ Cargo.toml updated with new [[bin]] entry");
    }
    println!(
        "  1. Test the migrated tool: cargo build --bin {} --features tools",
        selected.trim_end_matches(".rs")
    );
    println!(
        "  2. Test help: ./target/debug/{} --help",
        selected.trim_end_matches(".rs")
    );
    if is_git_repo {
        println!("  3. The file has been moved with git to preserve history");
        println!(
            "  4. Commit the changes: git commit -m 'Migrate {} to src/bin with auto_help'",
            selected
        );
    } else {
        println!("  3. Remove the original file from tools/ when satisfied");
        println!("     (Note: Not in a git repo, so history wasn't preserved)");
    }

    Ok(())
}

fn find_rust_files(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut rust_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            rust_files.push(path);
        }
    }

    rust_files.sort();
    Ok(rust_files)
}

fn migrate_tool(
    source: &Path,
    dest: &Path,
    tool_name: &str,
    use_git: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Starting migration...");

    if use_git {
        // Use git mv to preserve history
        println!("📜 Using git mv to preserve file history...");

        // Ensure destination directory exists
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        // First, git mv the file
        let git_mv_result = Command::new("git")
            .args(["mv", &source.to_string_lossy(), &dest.to_string_lossy()])
            .output()?;

        if !git_mv_result.status.success() {
            let error = String::from_utf8_lossy(&git_mv_result.stderr);
            return Err(format!("Git mv failed: {}", error).into());
        }

        println!("✅ File moved with git mv");

        // Now read and transform the content at the new location
        let content = fs::read_to_string(dest)?;
        let transformed = transform_tool_content(&content, tool_name)?;
        fs::write(dest, transformed)?;

        println!("✅ File transformed with auto-help integration");

        // Stage the changes
        let git_add_result = Command::new("git")
            .args(["add", &dest.to_string_lossy()])
            .output()?;

        if !git_add_result.status.success() {
            println!("⚠️  Warning: Could not stage changes automatically");
        } else {
            println!("✅ Changes staged for commit");
        }
    } else {
        // Fallback to regular file operations
        println!("📁 Using regular file copy (not in git repo)...");

        // Read the source file
        let content = fs::read_to_string(source)?;

        // Transform the content
        let transformed = transform_tool_content(&content, tool_name)?;

        // Ensure destination directory exists
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write the transformed content
        fs::write(dest, transformed)?;

        println!("✅ File copied and transformed");
    }

    Ok(())
}

fn transform_tool_content(
    content: &str,
    tool_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut transformed = Vec::new();
    // let mut in_main = false;
    let mut help_added = false;
    let tool_name_without_ext = tool_name.trim_end_matches(".rs");

    for (i, line) in lines.iter().enumerate() {
        // Add the line as-is first
        transformed.push(line.to_string());

        // Look for patterns to add help system integration
        if line.contains("fn main(") && !help_added {
            // in_main = true;
            // Check if thag_rs is already imported
            let has_thag_import = lines
                .iter()
                .any(|l| l.contains(r#"use thag_rs::{{auto_help"#));

            if !has_thag_import {
                // Add thag_rs import in the dependencies section
                if let Some(toml_end) = find_toml_block_end(&lines) {
                    // Insert thag_rs dependency
                    let deps_line =
                        format!(r#"thag_rs = {{ path = "../..", default-features = false"#);
                    // This is a simplified approach - in practice you'd want more sophisticated TOML parsing
                    transformed.insert(toml_end, deps_line);
                }

                // Add the import after the last use statement
                if let Some(last_use_idx) = find_last_use_statement(&lines) {
                    transformed.insert(
                        last_use_idx + 2,
                        "use thag_rs::{auto_help, help_system::check_help_and_exit};".to_string(),
                    );
                }
            }

            // Add help system code at the beginning of main
            if i + 1 < lines.len() {
                transformed.push(
                    "    // Check for help first - automatically extracts from source comments"
                        .to_string(),
                );
                transformed.push(format!(
                    r#"    let help = auto_help!("{}");"#,
                    tool_name_without_ext
                ));
                transformed.push("    check_help_and_exit(&help);".to_string());
                transformed.push("".to_string()); // Empty line for spacing
            }

            help_added = true;
        }
    }

    Ok(transformed.join("\n"))
}

fn find_toml_block_end(lines: &[&str]) -> Option<usize> {
    let mut in_toml = false;
    for (i, line) in lines.iter().enumerate() {
        if line.contains("/*[toml]") {
            in_toml = true;
        } else if in_toml && line.contains("*/") {
            return Some(i);
        }
    }
    None
}

fn find_last_use_statement(lines: &[&str]) -> Option<usize> {
    let mut last_use = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("fn ") {
            break;
        }
        if line.trim().starts_with("use ") {
            last_use = Some(i);
        }
    }
    last_use
}

fn check_git_repo() -> bool {
    // Check if we're in a git repository by running git status
    let result = Command::new("git").args(["status", "--porcelain"]).output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn update_cargo_toml(tool_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let cargo_path = Path::new("Cargo.toml");
    if !cargo_path.exists() {
        println!("⚠️  Cargo.toml not found, skipping update");
        return Ok(false);
    }

    let content = fs::read_to_string(cargo_path)?;
    let tool_name_without_ext = tool_name.trim_end_matches(".rs");

    // Check if this bin entry already exists
    let bin_entry = format!(r#"name = "{}""#, tool_name_without_ext);
    if content.contains(&bin_entry) {
        println!(
            "ℹ️  Cargo.toml already contains entry for {}",
            tool_name_without_ext
        );
        return Ok(false);
    }

    // Find the last [[bin]] entry to insert after it
    let lines: Vec<&str> = content.lines().collect();
    let mut insert_index = None;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("[[bin]]") {
            // Look for the end of this bin entry (next [[bin]] or end of file)
            for j in (i + 1)..lines.len() {
                if lines[j].starts_with("[[bin]]") || lines[j].starts_with("[") {
                    insert_index = Some(j);
                    break;
                } else if j == lines.len() - 1 {
                    insert_index = Some(lines.len());
                    break;
                }
            }
        }
    }

    if let Some(index) = insert_index {
        let mut new_lines = lines[..index].to_vec();

        // Add the new bin entry
        new_lines.push("");
        new_lines.push("[[bin]]");
        let var_name = format!(r#"name = "{}""#, tool_name_without_ext);
        new_lines.push(&var_name);
        let var_name = format!(r#"path = "src/bin/{}""#, tool_name);
        new_lines.push(&var_name);
        new_lines.push(r#"required-features = ["tools"]"#);

        // Add remaining lines
        new_lines.extend(&lines[index..]);

        let new_content = new_lines.join("\n");
        fs::write(cargo_path, new_content)?;

        println!(
            "📝 Added [[bin]] entry for {} to Cargo.toml",
            tool_name_without_ext
        );
        return Ok(true);
    }

    println!("⚠️  Could not find appropriate location to insert [[bin]] entry");
    Ok(false)
}
