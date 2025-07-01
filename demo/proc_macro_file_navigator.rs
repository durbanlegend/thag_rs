/*[toml]
[dependencies]
thag_proc_macros = { version = "0.1, thag-auto" }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["core", "simplelog"] }
inquire = "0.7"
edit = "0.1"
*/

/// Enhanced file navigator demo with editing and saving capabilities.
///
/// This demo showcases the file_navigator proc macro by:
/// 1. Selecting a file using an interactive file browser
/// 2. Reading and displaying the file content
/// 3. Opening the file in an external editor for modification
/// 4. Saving the modified content to a new file
/// 5. Demonstrating all generated methods from the file_navigator macro
//# Purpose: Comprehensive demo of file_navigator macro with full workflow
//# Categories: technique, proc_macros, file_handling, interactive
use inquire::Confirm;
use std::fs;
use std::path::PathBuf;
use thag_demo_proc_macros::file_navigator;

file_navigator! {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗂️  Enhanced File Navigator Demo");
    println!("================================\n");

    let mut navigator = FileNavigator::new();

    // Step 1: Select a file
    println!("Step 1: Select a file to work with");
    let selected_file = match select_file(&mut navigator, Some("rs"), false) {
        Ok(path) => path,
        Err(_) => {
            println!("No file selected. Exiting.");
            return Ok(());
        }
    };

    println!("Selected file: {}\n", selected_file.display());

    // Step 2: Read and display current content
    println!("Step 2: Reading current file content");
    let content = fs::read_to_string(&selected_file)?;
    let line_count = content.lines().count();
    let char_count = content.chars().count();

    println!("File stats:");
    println!("  - Lines: {}", line_count);
    println!("  - Characters: {}", char_count);
    println!("  - Size: {} bytes", content.len());

    // Show first few lines as preview
    let preview_lines: Vec<&str> = content.lines().take(5).collect();
    println!("\nPreview (first 5 lines):");
    for (i, line) in preview_lines.iter().enumerate() {
        println!("  {:2}: {}", i + 1, line);
    }
    if line_count > 5 {
        println!("  ... ({} more lines)", line_count - 5);
    }

    // Step 3: Ask if user wants to edit
    println!("\nStep 3: File editing");
    let should_edit = Confirm::new("Would you like to edit this file?")
        .with_default(true)
        .prompt()?;

    let modified_content = if should_edit {
        // Open in external editor
        println!("Opening file in external editor...");
        let edited_content = edit::edit(&content)?;

        if edited_content != content {
            println!("✅ File content was modified!");
            let modified_lines = edited_content.lines().count();
            let modified_chars = edited_content.chars().count();
            println!("New stats:");
            println!(
                "  - Lines: {} ({})",
                modified_lines,
                if modified_lines > line_count {
                    format!("+{}", modified_lines - line_count)
                } else if modified_lines < line_count {
                    format!("-{}", line_count - modified_lines)
                } else {
                    "no change".to_string()
                }
            );
            println!(
                "  - Characters: {} ({})",
                modified_chars,
                if modified_chars > char_count {
                    format!("+{}", modified_chars - char_count)
                } else if modified_chars < char_count {
                    format!("-{}", char_count - modified_chars)
                } else {
                    "no change".to_string()
                }
            );
            edited_content
        } else {
            println!("No changes made to the file.");
            content
        }
    } else {
        // Just add a comment as a simple transformation
        println!("Adding a timestamp comment to demonstrate transformation...");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        format!("// File processed at timestamp: {}\n{}", timestamp, content)
    };

    // Step 4: Save to new file
    println!("\nStep 4: Saving modified content");
    let default_output = format!(
        "{}_modified.rs",
        selected_file.file_stem().unwrap().to_string_lossy()
    );

    let output_filename = Text::new("Enter output filename:")
        .with_default(&default_output)
        .prompt()?;

    let output_path = selected_file.parent().unwrap().join(&output_filename);

    // Use the generated save_to_file method
    save_to_file(
        modified_content,
        &output_path.display().to_string(),
        None,
        true,
    )?;

    println!("✅ File saved successfully to: {}", output_path.display());

    // Step 5: Demonstrate other generated methods
    println!("\nStep 5: Demonstrating other file navigator capabilities");

    // Show directory listing
    if let Some(parent_dir) = selected_file.parent() {
        println!("\nFiles in the same directory:");
        if let Ok(entries) = fs::read_dir(parent_dir) {
            let mut rust_files = Vec::new();
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        rust_files.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }
            rust_files.sort();
            for (i, file) in rust_files.iter().take(10).enumerate() {
                println!("  {}: {}", i + 1, file);
            }
            if rust_files.len() > 10 {
                println!("  ... and {} more .rs files", rust_files.len() - 10);
            }
        }
    }

    println!("\n🎉 File navigator demo completed successfully!");
    println!("Generated methods used:");
    println!("  - select_file()    : Interactive file selection");
    println!("  - save_to_file()   : Save content to specified path");
    println!("  - FileNavigator    : Main navigator struct");

    Ok(())
}
