/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["image_themes"] }
*/

/// Simple demo showing the most important fine-tuning effects
///
/// This script demonstrates the three key fine-tuning controls:
/// saturation multiplier, lightness adjustment, and contrast multiplier.
//# Purpose: Simple demo of fine-tuning controls for image themes
//# Categories: color, styling, terminal, theming, tools
use std::path::Path;
use thag_styling::{ImageThemeConfig, ImageThemeGenerator, StylingResult, TermBgLuma};

fn test_config(name: &str, config: ImageThemeConfig) -> StylingResult<()> {
    let image_path = Path::new("assets/thag_morning_coffee_figma.png");

    if !image_path.exists() {
        println!("❌ Test image not found: {}", image_path.display());
        return Ok(());
    }

    println!("🎨 {}", name);
    println!("{}", "-".repeat(40));

    let generator = ImageThemeGenerator::with_config(config);

    match generator.generate_from_file(image_path) {
        Ok(theme) => {
            // Show key colors that demonstrate the effects
            println!(
                "Normal:   {}",
                theme.palette.normal.paint("■■■■■ Regular text")
            );
            println!(
                "Warning:  {}",
                theme.palette.warning.paint("■■■■■ Warning message")
            );
            println!(
                "Code:     {}",
                theme.palette.code.paint("■■■■■ Code snippet")
            );
            println!(
                "Success:  {}",
                theme.palette.success.paint("■■■■■ Success message")
            );
            println!(
                "Quote:    {}",
                theme.palette.quote.paint("■■■■■ Quoted text")
            );
            println!();
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    Ok(())
}

fn main() -> StylingResult<()> {
    println!("🔧 Fine-Tuning Controls - Simple Demo");
    println!("{}", "=".repeat(50));
    println!();

    // Default settings
    test_config(
        "Default Settings (baseline)",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Dark),
            ..Default::default()
        },
    )?;

    // High saturation - more vibrant colors
    test_config(
        "High Saturation (1.5x) - More Vivid",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Dark),
            saturation_multiplier: 1.5,
            ..Default::default()
        },
    )?;

    // Brighter theme - lighter colors
    test_config(
        "Brighter Theme (+0.15) - Softer Look",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Dark),
            lightness_adjustment: 0.15,
            ..Default::default()
        },
    )?;

    // High contrast - more dramatic differences
    test_config(
        "High Contrast (1.3x) - More Dramatic",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Dark),
            contrast_multiplier: 1.3,
            ..Default::default()
        },
    )?;

    // Combined effects
    test_config(
        "Vivid & High Contrast - Maximum Impact",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Dark),
            saturation_multiplier: 1.4,
            lightness_adjustment: 0.05,
            contrast_multiplier: 1.2,
            ..Default::default()
        },
    )?;

    println!("💡 Quick Guide:");
    println!("• saturation_multiplier: 0.5-2.0 (vibrancy)");
    println!("• lightness_adjustment: -0.3 to +0.3 (brightness)");
    println!("• contrast_multiplier: 0.5-1.5 (drama/subtlety)");
    println!();
    println!("🎯 Try these in your ImageThemeConfig!");

    Ok(())
}
