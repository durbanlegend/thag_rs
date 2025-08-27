/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Comprehensive test for all mintty functionality
///
/// This demo script thoroughly tests the mintty theme exporter functionality
/// including exporting themes, validating output format, and checking integration
/// with the theme generation system.
//# Purpose: Comprehensive test of mintty theme functionality
//# Categories: color, styling, terminal, theming, demo
use std::collections::HashMap;
use thag_styling::exporters::{ExportFormat, ThemeExporter};
use thag_styling::Theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐙 Comprehensive Mintty Functionality Test");
    println!("==========================================\n");

    // Test 1: Verify mintty is in available formats
    println!("🔍 Test 1: Checking mintty format availability");
    let all_formats = ExportFormat::all();
    let has_mintty = all_formats
        .iter()
        .any(|f| matches!(f, ExportFormat::Mintty));

    if has_mintty {
        println!("✅ Mintty format found in ExportFormat::all()");
    } else {
        println!("❌ Mintty format not found!");
        return Err("Mintty format not available".into());
    }

    // Test 2: Format properties
    println!("\n🔍 Test 2: Checking format properties");
    println!("   Format name: {}", ExportFormat::Mintty.format_name());
    println!(
        "   File extension: '{}'",
        ExportFormat::Mintty.file_extension()
    );

    assert_eq!(ExportFormat::Mintty.format_name(), "Mintty");
    assert_eq!(ExportFormat::Mintty.file_extension(), "");
    println!("✅ Format properties are correct");

    // Test 3: Export built-in themes
    println!("\n🔍 Test 3: Exporting built-in themes");
    let test_themes = ["dracula", "solarized-dark"];

    for theme_name in &test_themes {
        match Theme::get_builtin(theme_name) {
            Ok(theme) => match ExportFormat::Mintty.export_theme(&theme) {
                Ok(content) => {
                    println!("✅ Successfully exported '{}'", theme_name);
                    validate_mintty_content(&content, theme_name)?;
                }
                Err(e) => {
                    println!("❌ Failed to export '{}': {}", theme_name, e);
                    return Err(format!("Export failed for {}", theme_name).into());
                }
            },
            Err(e) => {
                println!("⚠️  Could not load built-in theme '{}': {}", theme_name, e);
            }
        }
    }

    // Test 4: Content validation
    println!("\n🔍 Test 4: Detailed content validation");
    let theme = Theme::get_builtin("dracula")?;
    let content = ExportFormat::Mintty.export_theme(&theme)?;

    // Parse and analyze content
    let mut config_entries = HashMap::new();
    let mut comment_lines = 0;
    let mut blank_lines = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            blank_lines += 1;
        } else if line.starts_with('#') {
            comment_lines += 1;
        } else if line.contains('=') {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                config_entries.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }
    }

    println!("   Total lines: {}", content.lines().count());
    println!("   Comment lines: {}", comment_lines);
    println!("   Config entries: {}", config_entries.len());
    println!("   Blank lines: {}", blank_lines);

    // Check required entries
    let required_entries = ["BackgroundColour"];
    for entry in &required_entries {
        if config_entries.contains_key(*entry) {
            println!("✅ Found required entry: {}", entry);
        } else {
            println!("⚠️  Missing entry: {}", entry);
        }
    }

    // Test 5: Color format validation
    println!("\n🔍 Test 5: Color format validation");
    for (key, value) in &config_entries {
        if key.contains("Colour") {
            if validate_rgb_format(value) {
                println!("✅ Valid RGB format for {}: {}", key, value);
            } else {
                println!("❌ Invalid RGB format for {}: {}", key, value);
                return Err(format!("Invalid RGB format: {}", value).into());
            }
        }
    }

    // Test 6: Integration with all formats
    println!("\n🔍 Test 6: Integration with other formats");
    let all_format_names: Vec<String> = ExportFormat::all()
        .iter()
        .map(|f| f.format_name().to_string())
        .collect();

    println!("   All available formats: {:?}", all_format_names);

    if all_format_names.contains(&"Mintty".to_string()) {
        println!("✅ Mintty is properly integrated with other formats");
    } else {
        return Err("Mintty not found in integrated formats".into());
    }

    // Test 7: Export all formats including mintty
    println!("\n🔍 Test 7: Testing export with all formats");
    for format in ExportFormat::all() {
        match format.export_theme(&theme) {
            Ok(content) => {
                let lines = content.lines().count();
                println!("✅ {} export: {} lines", format.format_name(), lines);

                if matches!(format, ExportFormat::Mintty) {
                    // Extra validation for mintty
                    if lines < 5 {
                        println!("⚠️  Mintty export seems too short ({} lines)", lines);
                    }
                }
            }
            Err(e) => {
                println!("❌ {} export failed: {}", format.format_name(), e);
                return Err(format!("Export failed for {}", format.format_name()).into());
            }
        }
    }

    println!("\n🎉 All tests passed! Mintty functionality is working correctly.");
    println!("\n📊 Summary:");
    println!("   ✅ Format integration: Working");
    println!("   ✅ Theme export: Working");
    println!("   ✅ Content validation: Working");
    println!("   ✅ RGB format: Working");
    println!("   ✅ Multi-format compatibility: Working");

    Ok(())
}

fn validate_mintty_content(
    content: &str,
    theme_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check header
    if !content.contains("# Mintty Color Scheme:") {
        return Err("Missing mintty header".into());
    }

    // Check that it contains theme name in header
    if !content.contains(theme_name) && !content.contains(&theme_name.replace("-", " ")) {
        println!(
            "⚠️  Theme name '{}' not found in header (may be transformed)",
            theme_name
        );
    }

    // Check basic structure
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 3 {
        return Err("Content too short".into());
    }

    // Check for at least one color configuration
    let has_color_config = content
        .lines()
        .any(|line| line.contains("Colour=") && line.contains(','));

    if !has_color_config {
        return Err("No color configurations found".into());
    }

    Ok(())
}

fn validate_rgb_format(value: &str) -> bool {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() != 3 {
        return false;
    }

    for part in parts {
        if let Ok(num) = part.trim().parse::<u8>() {
            if num > 255 {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}
