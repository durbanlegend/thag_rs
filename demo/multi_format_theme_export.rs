/*[toml]
[dependencies]
thag_styling = { path = "/Users/donf/projects/thag_rs/thag_styling" }
*/

//! Demo of multi-format theme export functionality
//!
//! This example demonstrates how to export a thag theme to multiple terminal emulator formats
//! including Alacritty, WezTerm, iTerm2, Kitty, and Windows Terminal.

use std::path::Path;
use thag_styling::{
    export_all_formats, export_theme_to_file, generate_installation_instructions, ColorSupport,
    ExportFormat, Palette, Style, TermBgLuma, Theme,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Multi-Format Theme Export Demo");
    println!("==================================\n");

    // // Create a sample theme for demonstration
    // let theme = create_sample_theme();

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!(
            "Usage: {} <built-in theme name>, e.g. `dracula_official`",
            args[0]
        );
        std::process::exit(1);
    }

    let theme_name = &args[1];

    // Load the theme
    let theme = Theme::get_builtin(theme_name)?;

    println!("📋 Theme: {}", theme.name);
    println!("📝 Description: {}\n", theme.description);

    // Create output directory
    let output_dir = "exported_themes";
    std::fs::create_dir_all(output_dir)?;

    // Export to all formats at once
    println!("🚀 Exporting to all formats...");
    let exported_files = export_all_formats(&theme, output_dir, &theme.name.replace(' ', "_"))?;

    println!("✅ Successfully exported {} formats:", exported_files.len());
    for file_path in &exported_files {
        println!("   📄 {}", file_path.display());
    }
    println!();

    // Demonstrate individual format exports with installation instructions
    println!("📖 Installation Instructions:");
    println!("{}", "=".repeat(50));

    for format in ExportFormat::all() {
        let filename = format!(
            "{}.{}",
            theme.name.replace(' ', "_"),
            format.file_extension()
        );

        println!("\n🔧 {} ({})", format.format_name(), filename);
        println!("{}", "-".repeat(40));

        // Export to individual format (alternative approach)
        let file_path = Path::new(output_dir).join(&filename);
        export_theme_to_file(&theme, *format, &file_path)?;

        // Show installation instructions
        let instructions = generate_installation_instructions(*format, &filename);
        println!("{}", instructions);

        // Show a preview of the exported content
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let preview_lines: Vec<&str> = content.lines().take(10).collect();
            println!("📄 Preview (first 10 lines):");
            for line in preview_lines {
                println!("   {}", line);
            }
            if content.lines().count() > 10 {
                println!("   ... ({} more lines)", content.lines().count() - 10);
            }
        }
    }

    println!("\n🎉 Demo completed!");
    println!(
        "📁 All exported themes are in the '{}' directory",
        output_dir
    );

    Ok(())
}

// /// Create a sample theme for demonstration purposes
// fn create_sample_theme() -> Theme {
//     use std::path::PathBuf;
//     use thag_styling::ColorInfo;

//     // Create a vibrant color palette
//     let mut palette = Palette::default();

//     // Define colors with RGB values
//     palette.heading1 = Style::fg(ColorInfo::rgb(255, 100, 100)); // Bright red
//     palette.heading2 = Style::fg(ColorInfo::rgb(100, 255, 100)); // Bright green
//     palette.heading3 = Style::fg(ColorInfo::rgb(100, 100, 255)); // Bright blue
//     palette.error = Style::fg(ColorInfo::rgb(255, 50, 50)); // Red
//     palette.warning = Style::fg(ColorInfo::rgb(255, 165, 0)); // Orange
//     palette.success = Style::fg(ColorInfo::rgb(50, 205, 50)); // Green
//     palette.info = Style::fg(ColorInfo::rgb(70, 130, 180)); // Steel blue
//     palette.emphasis = Style::fg(ColorInfo::rgb(255, 20, 147)); // Deep pink
//     palette.code = Style::fg(ColorInfo::rgb(138, 43, 226)); // Blue violet
//     palette.normal = Style::fg(ColorInfo::rgb(220, 220, 220)); // Light gray
//     palette.subtle = Style::fg(ColorInfo::rgb(128, 128, 128)); // Gray
//     palette.hint = Style::fg(ColorInfo::rgb(105, 105, 105)); // Dim gray
//     palette.debug = Style::fg(ColorInfo::rgb(255, 140, 0)); // Dark orange
//     palette.trace = Style::fg(ColorInfo::rgb(192, 192, 192)); // Silver

//     Theme {
//         name: "Vibrant Demo Theme".to_string(),
//         filename: PathBuf::from("vibrant_demo_theme.toml"),
//         is_builtin: false,
//         term_bg_luma: TermBgLuma::Dark,
//         min_color_support: ColorSupport::TrueColor,
//         palette,
//         backgrounds: vec!["#1a1a1a".to_string(), "#2d2d2d".to_string()],
//         bg_rgbs: vec![(26, 26, 26), (45, 45, 45)],
//         description: "A vibrant demonstration theme with bright, saturated colors perfect for showcasing multi-format export capabilities".to_string(),
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_theme_creation() {
        let theme = create_sample_theme();
        assert_eq!(theme.name, "Vibrant Demo Theme");
        assert_eq!(theme.term_bg_luma, TermBgLuma::Dark);
        assert!(!theme.bg_rgbs.is_empty());
        assert!(!theme.backgrounds.is_empty());
    }

    #[test]
    fn test_all_formats_export() {
        let theme = create_sample_theme();
        let temp_dir = tempfile::tempdir().unwrap();

        let result = export_all_formats(&theme, temp_dir.path(), "test_theme");
        assert!(result.is_ok());

        let files = result.unwrap();
        assert_eq!(files.len(), ExportFormat::all().len());

        // Check that all files exist
        for file in files {
            assert!(file.exists());
        }
    }

    #[test]
    fn test_individual_format_export() {
        let theme = create_sample_theme();
        let temp_dir = tempfile::tempdir().unwrap();

        for format in ExportFormat::all() {
            let filename = format!("test_theme.{}", format.file_extension());
            let file_path = temp_dir.path().join(filename);

            let result = export_theme_to_file(&theme, *format, &file_path);
            assert!(
                result.is_ok(),
                "Failed to export {} format",
                format.format_name()
            );
            assert!(
                file_path.exists(),
                "{} file was not created",
                format.format_name()
            );

            // Verify file has content
            let content = std::fs::read_to_string(&file_path).unwrap();
            assert!(
                !content.is_empty(),
                "{} file is empty",
                format.format_name()
            );
        }
    }
}

#[cfg(test)]
mod tempfile {
    pub fn tempdir() -> Result<TempDir, std::io::Error> {
        TempDir::new()
    }

    pub struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new() -> Result<Self, std::io::Error> {
            let path = std::env::temp_dir().join(format!("thag_test_{}", std::process::id()));
            std::fs::create_dir_all(&path)?;
            Ok(TempDir { path })
        }

        pub fn path(&self) -> &std::path::Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}
