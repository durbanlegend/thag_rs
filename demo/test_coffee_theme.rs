/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["image_themes"] }
*/

/// Test script to debug color selection for the morning coffee image
///
/// This script specifically tests the hue range assignments and fallback logic
/// to understand why colors aren't matching their intended hue ranges.
//# Purpose: Debug hue range assignments in morning coffee theme generation
//# Categories: color, styling, terminal, theming, tools
use std::path::Path;
use thag_styling::{ImageThemeConfig, ImageThemeGenerator, StylingResult, TermBgLuma};

fn main() -> StylingResult<()> {
    println!("☕ Testing Morning Coffee Image Theme Generation");
    println!("{}", "=".repeat(60));
    println!();

    let image_path = Path::new("assets/thag_morning_coffee_figma.png");

    if !image_path.exists() {
        println!(
            "❌ Morning coffee image not found: {}",
            image_path.display()
        );
        return Ok(());
    }

    println!("📷 Analyzing: {}", image_path.display());
    println!();

    // Test dark theme generation with debug output
    let config = ImageThemeConfig {
        theme_name_prefix: Some("coffee-debug".to_string()),
        force_theme_type: Some(TermBgLuma::Dark),
        ..Default::default()
    };

    let generator = ImageThemeGenerator::with_config(config);

    match generator.generate_from_file(image_path) {
        Ok(theme) => {
            println!("✅ Dark theme generated successfully!");
            println!();

            println!("🎨 Generated Colors:");
            if let Some([r, g, b]) = theme.bg_rgbs.first() {
                println!("Background: RGB({}, {}, {})", r, g, b);
            }
            println!();

            // Display all palette colors
            let colors = [
                ("Normal", &theme.palette.normal),
                ("Subtle", &theme.palette.subtle),
                ("Hint", &theme.palette.hint),
                ("Error", &theme.palette.error),
                ("Warning", &theme.palette.warning),
                ("Success", &theme.palette.success),
                ("Info", &theme.palette.info),
                ("Code", &theme.palette.code),
                ("Emphasis", &theme.palette.emphasis),
                ("Heading1", &theme.palette.heading1),
                ("Heading2", &theme.palette.heading2),
                ("Heading3", &theme.palette.heading3),
                ("Debug", &theme.palette.debug),
                ("Link", &theme.palette.link),
                ("Quote", &theme.palette.quote),
                ("Commentary", &theme.palette.commentary),
            ];

            for (name, style) in colors {
                println!(
                    "{:12} {}",
                    format!("{}:", name),
                    style.paint("■■■■■■■■■■ Sample text")
                );
            }

            println!();
            println!("🔍 Focus on Code vs Emphasis:");
            println!(
                "Code:     {} - Should be blue/magenta (240-300°)",
                theme.palette.code.paint("■■■■■■■■■■ Code sample text")
            );
            println!(
                "Emphasis: {} - Should be brown/orange (15-45°)",
                theme
                    .palette
                    .emphasis
                    .paint("■■■■■■■■■■ Emphasis sample text")
            );
            println!();

            println!("📊 Sample content with new colors:");
            println!("{}", theme.palette.heading1.paint("# Main Heading"));
            println!(
                "{}",
                theme
                    .palette
                    .normal
                    .paint("Regular text content in the theme.")
            );
            println!(
                "{}",
                theme
                    .palette
                    .code
                    .paint("function example() { return 42; }")
            );
            println!(
                "{}",
                theme
                    .palette
                    .emphasis
                    .paint("This text should be emphasized in brown/orange.")
            );
            println!(
                "{}",
                theme.palette.error.paint("Error: Something went wrong!")
            );
            println!(
                "{}",
                theme.palette.warning.paint("Warning: Check this setting.")
            );
            println!(
                "{}",
                theme.palette.success.paint("Success: Operation completed.")
            );
            println!(
                "{}",
                theme.palette.info.paint("Info: Additional details here.")
            );
        }
        Err(e) => {
            println!("❌ Failed to generate theme: {}", e);
        }
    }

    println!();
    println!("💡 Expected hue ranges:");
    println!("  Error:    0-60°   (red to orange/yellow)");
    println!("  Warning:  30-90°  (yellow to yellow-green)");
    println!("  Success:  90-150° (green to cyan)");
    println!("  Info:     180-240° (cyan to blue)");
    println!("  Code:     240-300° (blue to magenta)");
    println!("  Emphasis: 15-45°  (orange to brown)");
    println!();
    println!("If colors don't match their intended hue ranges,");
    println!("check the debug output above for fallback reasons.");

    Ok(())
}
