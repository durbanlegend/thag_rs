/*[toml]
[dependencies]
thag_styling = { version = "1, thag-auto" }
*/

/// Demo script showing Konsole colorscheme export logic
//# Purpose: Demonstrate exporting thag themes to KDE Konsole .colorscheme format
//# Categories: styling, terminal, theming, tools
use std::path::Path;
use thag_styling::{
    exporters::{export_theme_to_file, generate_installation_instructions, ExportFormat},
    Theme,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Konsole Theme Export Demo\n");

    // Demo themes to export
    let demo_themes = [
        "thag_styling/themes/built_in/catppuccin-mocha.toml",
        "thag_styling/themes/built_in/dracula.toml",
    ];

    for theme_path in demo_themes {
        let theme_path = Path::new(theme_path);

        if !theme_path.exists() {
            println!("⚠️  Theme file not found: {}", theme_path.display());
            continue;
        }

        println!("📁 Loading theme: {}", theme_path.display());
        let theme = Theme::load_from_file(theme_path)?;

        println!("✨ Theme: {} ({})", theme.name, theme.description);
        println!("🌈 Background: {:?}", theme.term_bg_luma);

        // Generate Konsole theme filename
        let output_filename = format!(
            "{}.colorscheme",
            theme.name.to_lowercase().replace(' ', "-")
        );
        let output_path = Path::new("exported_themes").join(&output_filename);

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Export to Konsole format
        println!("📤 Exporting to: {}", output_path.display());
        export_theme_to_file(&theme, ExportFormat::Konsole, &output_path)?;

        // Show installation instructions
        let instructions =
            generate_installation_instructions(ExportFormat::Konsole, &output_filename);
        println!("📋 Installation Instructions:\n{}", instructions);

        println!("{}\n", "─".repeat(70));
    }

    println!("✅ Demo complete! Check the exported_themes/ directory for your .colorscheme files.");
    println!("💡 You can now copy these files to your Konsole themes directory and activate them.");

    Ok(())
}
