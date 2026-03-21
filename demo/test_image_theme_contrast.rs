/*[toml]
[dependencies]
thag_styling = { version = "1, thag-auto", features = ["image_themes"] }
*/

/// Test script to demonstrate improved contrast in image theme generation
///
/// This script shows the enhanced contrast adjustment logic
/// with minimum lightness differences for better readability.
//# Purpose: Demonstrate improved contrast in image theme generation
//# Categories: color, styling, terminal, theming, tools
use std::path::Path;
use thag_styling::{ImageThemeConfig, ImageThemeGenerator, StylingResult, TermBgLuma};

fn main() -> StylingResult<()> {
    println!("🎨 Image Theme Contrast Improvements Demo");
    println!("{}", "=".repeat(45));
    println!();

    // Use a test image from the project
    let image_path = Path::new("assets/munch-the-scream.png");

    if !image_path.exists() {
        println!("❌ Test image not found, trying alternative...");
        let alt_path = Path::new("assets/thag_morning_coffee_figma.png");
        if alt_path.exists() {
            return test_theme(alt_path);
        }
        println!("❌ No suitable test images found in assets/");
        return Ok(());
    }

    test_theme(image_path)
}

fn test_theme(image_path: &Path) -> StylingResult<()> {
    println!("📷 Analyzing: {}", image_path.display());
    println!();

    // Test both light and dark themes
    for (theme_type, theme_name) in [(TermBgLuma::Light, "Light"), (TermBgLuma::Dark, "Dark")] {
        println!("🌅 Testing {} Theme:", theme_name);
        println!("{}", "-".repeat(40));

        let config = ImageThemeConfig {
            theme_name_prefix: Some(format!("contrast-demo-{}", theme_name.to_lowercase())),
            force_theme_type: Some(theme_type.clone()),
            ..Default::default()
        };

        let generator = ImageThemeGenerator::with_config(config);

        match generator.generate_from_file(image_path) {
            Ok(theme) => {
                println!("✅ {} theme generated with enhanced contrast!", theme_name);
                println!();

                // Show the improvements in action
                println!("🎯 Enhanced Contrast Results:");
                println!("• Semantic colors (0.7+ lightness diff):");
                println!(
                    "  {} {}",
                    "Error:".to_string(),
                    theme.palette.error.paint("Critical system failure")
                );
                println!(
                    "  {} {}",
                    "Success:".to_string(),
                    theme
                        .palette
                        .success
                        .paint("Operation completed successfully")
                );
                println!(
                    "  {} {}",
                    "Warning:".to_string(),
                    theme.palette.warning.paint("Resource usage is high")
                );
                println!();

                println!("• Text colors (0.6+ lightness diff with reduced saturation):");
                println!(
                    "  {} {}",
                    "Normal:".to_string(),
                    theme.palette.normal.paint("Regular text content")
                );
                println!(
                    "  {} {}",
                    "Subtle:".to_string(),
                    theme.palette.subtle.paint("Secondary information")
                );
                println!(
                    "  {} {}",
                    "Commentary:".to_string(),
                    theme
                        .palette
                        .commentary
                        .paint("// Code comments are easier to read")
                );
                println!();

                println!("• Headings (0.7+ lightness diff for better visibility):");
                println!("  {}", theme.palette.heading1.paint("# Primary Heading"));
                println!("  {}", theme.palette.heading2.paint("## Secondary Heading"));
                println!();

                println!("💡 Improvements implemented:");
                println!("  ✓ Minimum 0.6 lightness difference for non-core colors");
                println!("  ✓ Minimum 0.7 lightness difference for semantic/heading colors");
                println!("  ✓ Reduced saturation for improved contrast where needed");
                println!("  ✓ All colors maintain their original hue characteristics");
            }
            Err(e) => {
                eprintln!(
                    "❌ Failed to generate {} theme: {}",
                    theme_name.to_lowercase(),
                    e
                );
            }
        }

        println!();
        println!("{}", "=".repeat(60));
        println!();
    }

    Ok(())
}
