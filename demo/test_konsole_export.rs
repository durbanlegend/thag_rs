/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Test script for Konsole theme export functionality
//# Purpose: Test the Konsole colorscheme exporter with Catppuccin Mocha theme
//# Categories: terminal, testing, theming
use std::path::Path;
use thag_styling::{
    exporters::{konsole::KonsoleExporter, ThemeExporter},
    Theme,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the Catppuccin Mocha theme
    let theme_path = Path::new("thag_styling/themes/built_in/catppuccin-mocha.toml");

    if !theme_path.exists() {
        eprintln!("Theme file not found: {}", theme_path.display());
        return Ok(());
    }

    let theme = Theme::load_from_file(theme_path)?;

    println!("Exporting theme: {}", theme.name);
    println!("Description: {}", theme.description);
    println!();

    // Export to Konsole format
    let exported = KonsoleExporter::export_theme(&theme)?;

    println!("Konsole .colorscheme export:");
    println!("============================");
    println!("{}", exported);

    // Test the metadata
    println!("File extension: .{}", KonsoleExporter::file_extension());
    println!("Format name: {}", KonsoleExporter::format_name());

    // Write to file for testing
    let output_path = format!(
        "exported_themes/{}.{}",
        theme.name.to_lowercase().replace(' ', "-"),
        KonsoleExporter::file_extension()
    );

    if let Some(parent) = Path::new(&output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&output_path, &exported)?;
    println!("\nExported theme written to: {}", output_path);

    Ok(())
}
