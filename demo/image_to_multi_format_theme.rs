/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect", "image_themes"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config", "image_themes"] }
*/

/// Demo of generating multi-format terminal themes from images
///
/// This example demonstrates the complete workflow:
/// 1. Generate a theme from an image using image analysis
/// 2. Export that theme to all supported terminal emulator formats
/// 3. Provide installation instructions for each format
//# Purpose: Generate multi-format terminal themes from an image
//# Categories: ansi, color, demo, styling, technique, terminal, xterm
use std::path::Path;
use thag_styling::{
    export_all_formats, generate_installation_instructions, generate_theme_from_image_with_config,
    ExportFormat, ImageThemeConfig,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖼️  Image to Multi-Format Theme Demo");
    println!("=====================================\n");

    // Check if image file exists (use a sample image if available)
    let image_paths = [
        "sunset.png",
        "monet-woman-with-parasol.png",
        "demo/sample_image.png",
        "assets/sample.png",
    ];

    let image_path = image_paths
        .iter()
        .find(|path| Path::new(path).exists())
        .ok_or("No sample image found. Please ensure you have an image file (sunset.png, monet-woman-with-parasol.png, etc.) in the project root.")?;

    println!("📷 Using image: {}", image_path);

    // Configure theme generation
    let config = ImageThemeConfig {
        color_count: 16,
        light_threshold: 0.7,
        saturation_threshold: 0.3,
        auto_detect_theme_type: true,
        force_theme_type: None,
        theme_name_prefix: Some("ImageTheme".to_string()),
    };

    // Generate theme from image
    println!("🎨 Analyzing image and generating theme...");
    let theme = generate_theme_from_image_with_config(image_path, config)?;

    println!("✅ Generated theme: {}", theme.name);
    println!("📝 Description: {}", theme.description);
    println!("🌗 Background type: {:?}", theme.term_bg_luma);
    println!("🎯 Color support: {:?}\n", theme.min_color_support);

    // Show the color palette
    println!("🎨 Color Palette:");
    println!("├─ Background: {:?}", theme.bg_rgbs);
    println!(
        "├─ Normal text: {:?}",
        extract_rgb_from_style(&theme.palette.normal)
    );
    println!(
        "├─ Error: {:?}",
        extract_rgb_from_style(&theme.palette.error)
    );
    println!(
        "├─ Warning: {:?}",
        extract_rgb_from_style(&theme.palette.warning)
    );
    println!(
        "├─ Success: {:?}",
        extract_rgb_from_style(&theme.palette.success)
    );
    println!("├─ Info: {:?}", extract_rgb_from_style(&theme.palette.info));
    println!("├─ Code: {:?}", extract_rgb_from_style(&theme.palette.code));
    println!(
        "└─ Emphasis: {:?}\n",
        extract_rgb_from_style(&theme.palette.emphasis)
    );

    // Create output directory
    let output_dir = "exported_image_themes";
    std::fs::create_dir_all(output_dir)?;

    // Export to all formats
    println!("🚀 Exporting to all terminal formats...");
    let theme_filename = theme.name.replace(' ', "_");
    let exported_files = export_all_formats(&theme, output_dir, &theme_filename)?;

    println!("✅ Successfully exported {} formats:", exported_files.len());
    for file_path in &exported_files {
        let size = std::fs::metadata(file_path)?.len();
        println!("   📄 {} ({} bytes)", file_path.display(), size);
    }
    println!();

    // Show installation instructions for each format
    println!("📖 Installation Instructions");
    println!("{}", "=".repeat(50));

    for format in ExportFormat::all() {
        let filename = format!("{}.{}", theme_filename, format.file_extension());

        println!("\n🔧 {} Configuration", format.format_name());
        println!("{}", "-".repeat(30));

        let instructions = generate_installation_instructions(*format, &filename);
        println!("{}", instructions);

        // Show a small preview of the file content
        let file_path = Path::new(output_dir).join(&filename);
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let preview_lines: Vec<&str> = content.lines().take(5).collect();
            println!("📄 Preview:");
            for line in preview_lines {
                println!("   {}", line);
            }
            if content.lines().count() > 5 {
                println!("   ... ({} more lines)", content.lines().count() - 5);
            }
        }
    }

    // Show usage examples
    println!("\n💡 Usage Examples:");
    println!("{}", "-".repeat(20));

    println!("\n🖥️  Alacritty:");
    println!("   # Add to ~/.config/alacritty/alacritty.toml:");
    println!("   import = [\"themes/{}.toml\"]", theme_filename);

    println!("\n🖥️  WezTerm:");
    println!("   -- Add to ~/.config/wezterm/wezterm.lua:");
    println!("   config.color_scheme_dirs = {{ wezterm.config_dir .. '/colors' }}");
    println!("   config.color_scheme = '{}'", theme_filename);

    println!("\n🖥️  Kitty:");
    println!("   # Add to ~/.config/kitty/kitty.conf:");
    println!("   include themes/{}.conf", theme_filename);

    println!("\n🎉 Demo completed!");
    println!(
        "📁 All exported themes are in the '{}' directory",
        output_dir
    );
    println!("🖼️  Original image: {}", image_path);

    Ok(())
}

/// Extract RGB values from a style for display purposes
fn extract_rgb_from_style(style: &thag_styling::Style) -> Option<(u8, u8, u8)> {
    style.foreground.as_ref().and_then(|color_info| {
        match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            thag_styling::ColorValue::Color256 { color256 } => {
                // Simple 256-color to RGB approximation for display
                let index = *color256;
                match index {
                    0..=15 => {
                        // Basic colors
                        let colors = [
                            (0, 0, 0),
                            (128, 0, 0),
                            (0, 128, 0),
                            (128, 128, 0),
                            (0, 0, 128),
                            (128, 0, 128),
                            (0, 128, 128),
                            (192, 192, 192),
                            (128, 128, 128),
                            (255, 0, 0),
                            (0, 255, 0),
                            (255, 255, 0),
                            (0, 0, 255),
                            (255, 0, 255),
                            (0, 255, 255),
                            (255, 255, 255),
                        ];
                        colors.get(index as usize).copied()
                    }
                    16..=231 => {
                        // 216 color cube
                        let n = index - 16;
                        let r = (n / 36) * 51;
                        let g = ((n % 36) / 6) * 51;
                        let b = (n % 6) * 51;
                        Some((r, g, b))
                    }
                    232..=255 => {
                        // Grayscale
                        let gray = 8 + (index - 232) * 10;
                        Some((gray, gray, gray))
                    }
                }
            }
            thag_styling::ColorValue::Basic { index, .. } => {
                // Basic ANSI colors
                let colors = [
                    (0, 0, 0),
                    (128, 0, 0),
                    (0, 128, 0),
                    (128, 128, 0),
                    (0, 0, 128),
                    (128, 0, 128),
                    (0, 128, 128),
                    (192, 192, 192),
                    (128, 128, 128),
                    (255, 0, 0),
                    (0, 255, 0),
                    (255, 255, 0),
                    (0, 0, 255),
                    (255, 0, 255),
                    (0, 255, 255),
                    (255, 255, 255),
                ];
                colors.get(*index as usize).copied()
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use thag_styling::{ColorSupport, Palette, Style, TermBgLuma, Theme};

    fn create_test_theme() -> Theme {
        Theme {
            name: "Test Image Theme".to_string(),
            filename: PathBuf::from("test_image_theme.toml"),
            is_builtin: false,
            term_bg_luma: TermBgLuma::Dark,
            min_color_support: ColorSupport::TrueColor,
            palette: Palette::default(),
            backgrounds: vec!["#2a2a2a".to_string()],
            bg_rgbs: vec![(42, 42, 42)],
            description: "Test theme generated from image".to_string(),
        }
    }

    #[test]
    fn test_extract_rgb_from_style() {
        use thag_styling::{ColorInfo, ColorValue};

        let style = Style::fg(ColorInfo::rgb(255, 128, 64));
        let rgb = extract_rgb_from_style(&style);
        assert_eq!(rgb, Some((255, 128, 64)));
    }

    #[test]
    fn test_theme_export_workflow() {
        let theme = create_test_theme();

        // Test that we can create the output directory
        let temp_dir = std::env::temp_dir().join(format!("thag_test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Test export
        let result = export_all_formats(&theme, &temp_dir, "test_theme");
        assert!(result.is_ok());

        let files = result.unwrap();
        assert_eq!(files.len(), ExportFormat::all().len());

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_installation_instructions() {
        for format in ExportFormat::all() {
            let instructions = generate_installation_instructions(*format, "test_theme.ext");
            assert!(!instructions.is_empty());
            assert!(instructions.contains(format.format_name()));
        }
    }
}
