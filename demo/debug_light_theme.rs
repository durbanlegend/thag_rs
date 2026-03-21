/*[toml]
[dependencies]
thag_styling = { version = "1, thag-auto", features = ["image_themes"] }
*/

/// Minimal debug script to isolate light theme processing issues
///
/// This script tests light theme generation with minimal parameters
/// to identify where the infinite loop or color issues are occurring.
//# Purpose: Debug light theme generation issues
//# Categories: color, styling, terminal, theming, tools
use std::path::Path;
use thag_styling::{ImageThemeConfig, ImageThemeGenerator, StylingResult, TermBgLuma};

fn main() -> StylingResult<()> {
    println!("🐛 Light Theme Debug Script");
    println!("{}", "=".repeat(40));
    println!();

    let image_path = Path::new("assets/thag_morning_coffee_figma.png");

    if !image_path.exists() {
        println!("❌ Test image not found: {}", image_path.display());
        return Ok(());
    }

    println!("Testing basic light theme generation...");

    // Test 1: Absolute minimal configuration
    println!("🔹 Test 1: Minimal light theme");
    let config = ImageThemeConfig {
        force_theme_type: Some(TermBgLuma::Light),
        color_count: 8, // Reduce complexity
        ..Default::default()
    };

    let generator = ImageThemeGenerator::with_config(config);

    match generator.generate_from_file(image_path) {
        Ok(theme) => {
            println!("✅ Basic light theme generated successfully");
            println!("Background: {:?}", theme.bg_rgbs);
            println!("Normal: {}", theme.palette.normal.paint("■■■ Normal text"));
            println!("Error: {}", theme.palette.error.paint("■■■ Error text"));
            println!(
                "Warning: {}",
                theme.palette.warning.paint("■■■ Warning text")
            );
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
            return Err(e);
        }
    }

    println!();
    println!("🔹 Test 2: Light theme with very mild adjustments");

    let config2 = ImageThemeConfig {
        force_theme_type: Some(TermBgLuma::Light),
        color_count: 8,
        saturation_multiplier: 0.95, // Very mild
        lightness_adjustment: -0.02, // Very mild
        contrast_multiplier: 1.05,   // Very mild
        ..Default::default()
    };

    let generator2 = ImageThemeGenerator::with_config(config2);

    match generator2.generate_from_file(image_path) {
        Ok(theme) => {
            println!("✅ Mild adjustment light theme generated");
            println!("Normal: {}", theme.palette.normal.paint("■■■ Normal text"));
            println!("Error: {}", theme.palette.error.paint("■■■ Error text"));
            println!("Debug: {}", theme.palette.debug.paint("■■■ Debug text"));
        }
        Err(e) => {
            println!("❌ Failed at mild adjustments: {}", e);
            return Err(e);
        }
    }

    println!();
    println!("🔹 Test 3: Compare with dark theme");

    let dark_config = ImageThemeConfig {
        force_theme_type: Some(TermBgLuma::Dark),
        color_count: 8,
        saturation_multiplier: 1.2,
        contrast_multiplier: 1.1,
        ..Default::default()
    };

    let dark_generator = ImageThemeGenerator::with_config(dark_config);

    match dark_generator.generate_from_file(image_path) {
        Ok(theme) => {
            println!("✅ Dark theme for comparison");
            println!("Normal: {}", theme.palette.normal.paint("■■■ Normal text"));
            println!("Error: {}", theme.palette.error.paint("■■■ Error text"));
            println!("Debug: {}", theme.palette.debug.paint("■■■ Debug text"));
        }
        Err(e) => {
            println!("❌ Dark theme failed: {}", e);
        }
    }

    println!();
    println!("✨ Debug complete - if this runs, the basic generation works");
    println!("   Issue is likely in more extreme parameter combinations");

    Ok(())
}
