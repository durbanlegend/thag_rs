/*[toml]
[dependencies]
thag_styling = { path = "/Users/donf/projects/thag_rs/thag_styling" }
dirs = "5.0"
*/

//! Export the thag-vibrant-dark theme to all supported terminal formats
//!
//! This script loads the actual thag-vibrant-dark theme from the built-in themes
//! and exports it to all supported terminal emulator formats for comparison.

use std::path::Path;
use thag_styling::{
    export_all_formats, export_theme_to_file, generate_installation_instructions, ExportFormat,
    Theme, ThemeExporter,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Exporting thag-vibrant-dark Theme");
    println!("=====================================\n");

    // Load the actual thag-vibrant-dark theme
    let theme = Theme::get_builtin("thag-vibrant-dark")?;

    println!("📋 Theme: {}", theme.name);
    println!("📝 Description: {}", theme.description);
    println!("🌗 Background type: {:?}", theme.term_bg_luma);
    println!("🎯 Color support: {:?}", theme.min_color_support);
    println!("🖼️  Background colors: {:?}", theme.bg_rgbs);
    println!();

    // Show the color palette
    println!("🎨 Color Palette:");
    println!("├─ Normal: {:?}", extract_rgb(&theme.palette.normal));
    println!("├─ Error: {:?}", extract_rgb(&theme.palette.error));
    println!("├─ Warning: {:?}", extract_rgb(&theme.palette.warning));
    println!("├─ Success: {:?}", extract_rgb(&theme.palette.success));
    println!("├─ Info: {:?}", extract_rgb(&theme.palette.info));
    println!("├─ Code: {:?}", extract_rgb(&theme.palette.code));
    println!("├─ Emphasis: {:?}", extract_rgb(&theme.palette.emphasis));
    println!("├─ Subtle: {:?}", extract_rgb(&theme.palette.subtle));
    println!("├─ Heading1: {:?}", extract_rgb(&theme.palette.heading1));
    println!("├─ Heading2: {:?}", extract_rgb(&theme.palette.heading2));
    println!("└─ Heading3: {:?}", extract_rgb(&theme.palette.heading3));
    println!();

    // Create output directory
    let output_dir = "exported_thag_vibrant_dark";
    std::fs::create_dir_all(output_dir)?;

    // Export to all formats
    println!("🚀 Exporting to all terminal formats...");
    let exported_files = export_all_formats(&theme, output_dir, &theme.name.replace('-', "_"))?;

    println!("✅ Successfully exported {} formats:", exported_files.len());
    for file_path in &exported_files {
        let size = std::fs::metadata(file_path)?.len();
        println!("   📄 {} ({} bytes)", file_path.display(), size);
    }
    println!();

    // Show WezTerm specific output for comparison
    println!("🔧 WezTerm Theme Content:");
    println!("{}", "=".repeat(40));
    let wezterm_content = ExportFormat::WezTerm.export_theme(&theme)?;

    // Show first 30 lines of WezTerm theme
    let lines: Vec<&str> = wezterm_content.lines().collect();
    for (i, line) in lines.iter().take(30).enumerate() {
        println!("{:2}: {}", i + 1, line);
    }
    if lines.len() > 30 {
        println!("    ... ({} more lines)", lines.len() - 30);
    }
    println!();

    // Save a copy to your WezTerm colors directory if it exists
    let wezterm_colors_dir = dirs::home_dir()
        .map(|home| home.join(".config/wezterm/colors"))
        .filter(|path| path.exists());

    if let Some(colors_dir) = wezterm_colors_dir {
        let wezterm_file = colors_dir.join("thag_vibrant_dark.toml");
        export_theme_to_file(&theme, ExportFormat::WezTerm, &wezterm_file)?;
        println!("💾 Saved WezTerm theme to: {}", wezterm_file.display());
        println!("   Add this to your wezterm.lua: config.color_scheme = 'thag_vibrant_dark'");
    } else {
        println!("ℹ️  WezTerm colors directory not found at ~/.config/wezterm/colors");
        println!("   You can manually copy the _wezterm.toml file there.");
    }

    println!("\n🎉 Export completed!");
    println!("📁 All files are in the '{}' directory", output_dir);

    Ok(())
}

/// Extract RGB values from a style for display
fn extract_rgb(style: &thag_styling::Style) -> Option<(u8, u8, u8)> {
    style
        .foreground
        .as_ref()
        .and_then(|color_info| match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            _ => None,
        })
}
