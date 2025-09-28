/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Demo script to test all export formats including the new Konsole exporter
//# Purpose: Test all available theme export formats to verify Konsole integration
//# Categories: terminal, themes, export, testing
use std::path::Path;
use thag_styling::{
    exporters::{export_theme_to_file, ExportFormat},
    Theme,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Testing All Export Formats (Including Konsole)\n");

    // Load a sample theme
    let theme_path = Path::new("thag_styling/themes/built_in/catppuccin-mocha.toml");

    if !theme_path.exists() {
        eprintln!("❌ Theme file not found: {}", theme_path.display());
        return Ok(());
    }

    let theme = Theme::load_from_file(theme_path)?;
    println!("📁 Loaded theme: {} ({})", theme.name, theme.description);
    println!("🌈 Background: {:?}\n", theme.term_bg_luma);

    // Test all available formats
    let all_formats = ExportFormat::all();
    println!("🔧 Testing {} export formats:\n", all_formats.len());

    for format in all_formats {
        let format_name = format.format_name();
        let extension = format.file_extension();

        println!("📤 Testing {} format (.{})...", format_name, extension);

        // Generate output filename
        let output_filename = format!(
            "{}.{}",
            theme.name.to_lowercase().replace(' ', "-"),
            extension
        );

        let output_dir = format!("exported_themes/{}", format_name.to_lowercase());
        let output_path = Path::new(&output_dir).join(&output_filename);

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Export theme
        match export_theme_to_file(&theme, *format, &output_path) {
            Ok(()) => {
                println!("  ✅ Successfully exported to: {}", output_path.display());

                // Show file size for verification
                if let Ok(metadata) = std::fs::metadata(&output_path) {
                    println!("  📊 File size: {} bytes", metadata.len());
                }
            }
            Err(e) => {
                println!("  ❌ Export failed: {}", e);
            }
        }

        println!();
    }

    println!("🎯 Export Summary:");
    println!("  • All formats tested with theme: {}", theme.name);
    println!("  • Files exported to: exported_themes/<format>/");
    println!("  • Konsole support: ✅ Available as .colorscheme format");

    println!("\n💡 Usage tips:");
    println!("  • Konsole: Copy .colorscheme files to ~/.local/share/konsole/");
    println!("  • Use `konsoleprofile ColorScheme=<filename>` to apply themes");
    println!("  • Check exported_themes/ directory for all generated files");

    Ok(())
}
