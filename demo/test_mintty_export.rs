/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Test script for mintty theme exporter
///
/// This demo script tests the mintty theme exporter logic
/// by loading a built-in theme and exporting it to mintty format.
//# Purpose: Test mintty theme export logic
//# Categories: color, styling, terminal, theming, demo
use thag_styling::exporters::{ExportFormat, ThemeExporter};
use thag_styling::Theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐙 Testing Mintty Theme Export");
    println!("===============================\n");

    // Load a built-in theme
    let theme_name = "dracula";
    println!("📋 Loading theme: {}", theme_name);

    let theme = Theme::get_builtin(theme_name)
        .map_err(|e| format!("Failed to load theme '{}': {}", theme_name, e))?;

    println!("✅ Successfully loaded theme: {}", theme.name);
    println!("   Description: {}", theme.description);
    println!();

    // Export to mintty format
    println!("🔄 Exporting to mintty format...");
    let mintty_content = ExportFormat::Mintty
        .export_theme(&theme)
        .map_err(|e| format!("Failed to export theme: {}", e))?;

    println!("✅ Successfully exported to mintty format\n");

    // Display the exported content
    println!("📄 Exported Mintty Theme Content:");
    println!("{}", "=".repeat(50));
    println!("{}", mintty_content);
    println!("{}", "=".repeat(50));
    println!();

    // Show some statistics
    let lines = mintty_content.lines().count();
    let non_comment_lines = mintty_content
        .lines()
        .filter(|line| !line.trim().starts_with('#') && !line.trim().is_empty())
        .count();

    println!("📊 Export Statistics:");
    println!("   Total lines: {}", lines);
    println!("   Configuration lines: {}", non_comment_lines);
    println!("   Format: Mintty INI-style");
    println!(
        "   File extension: {} (no extension)",
        ExportFormat::Mintty.file_extension()
    );

    Ok(())
}
