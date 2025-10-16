/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["image_themes"] }
*/

/// Extreme parameter test for light themes to verify fine-tuning is working
///
/// This script tests light theme generation with extreme parameter differences
/// to see if the fine-tuning system is actually responsive.
//# Purpose: Test extreme light theme parameter differences
//# Categories: color, styling, terminal, theming, tools
use std::path::Path;
use thag_styling::{ImageThemeConfig, ImageThemeGenerator, StylingResult, TermBgLuma};

fn test_extreme_config(name: &str, config: ImageThemeConfig) -> StylingResult<()> {
    let image_path = Path::new("assets/thag_morning_coffee_figma.png");

    if !image_path.exists() {
        println!("❌ Test image not found: {}", image_path.display());
        return Ok(());
    }

    println!("🎨 {}", name);
    println!("{}", "-".repeat(50));
    println!("   Sat: {:.2}x, Light: {:+.2}, Contrast: {:.2}x",
        config.saturation_multiplier,
        config.lightness_adjustment,
        config.contrast_multiplier);
    println!();

    let generator = ImageThemeGenerator::with_config(config);

    match generator.generate_from_file(image_path) {
        Ok(theme) => {
            println!("Normal:   {}", theme.palette.normal.paint("■■■■■ Normal text"));
            println!("Warning:  {}", theme.palette.warning.paint("■■■■■ Warning"));
            println!("Error:    {}", theme.palette.error.paint("■■■■■ Error"));
            println!("Code:     {}", theme.palette.code.paint("■■■■■ Code"));
            println!("Quote:    {}", theme.palette.quote.paint("■■■■■ Quote"));
            println!();
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    Ok(())
}

fn main() -> StylingResult<()> {
    println!("🔬 Extreme Light Theme Parameter Test");
    println!("{}", "=".repeat(60));
    println!();

    // Baseline
    test_extreme_config(
        "BASELINE - Default Settings",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            ..Default::default()
        }
    )?;

    // Extreme low saturation
    test_extreme_config(
        "EXTREME LOW SATURATION",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            saturation_multiplier: 0.3,  // Very low
            ..Default::default()
        }
    )?;

    // Extreme high saturation
    test_extreme_config(
        "EXTREME HIGH SATURATION",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            saturation_multiplier: 2.0,  // Very high
            ..Default::default()
        }
    )?;

    // Extreme dark adjustment
    test_extreme_config(
        "EXTREME DARK COLORS",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            lightness_adjustment: -0.3,  // Very dark
            ..Default::default()
        }
    )?;

    // Extreme light adjustment
    test_extreme_config(
        "EXTREME LIGHT COLORS",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            lightness_adjustment: 0.15,   // Brighter
            ..Default::default()
        }
    )?;

    // Extreme low contrast
    test_extreme_config(
        "EXTREME LOW CONTRAST",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            contrast_multiplier: 0.3,     // Very low
            ..Default::default()
        }
    )?;

    // Extreme high contrast
    test_extreme_config(
        "EXTREME HIGH CONTRAST",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            contrast_multiplier: 2.0,     // Very high
            ..Default::default()
        }
    )?;

    // Combined extreme
    test_extreme_config(
        "COMBINED EXTREME",
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            saturation_multiplier: 0.5,
            lightness_adjustment: -0.2,
            contrast_multiplier: 1.8,
            ..Default::default()
        }
    )?;

    println!("🎯 Results Analysis:");
    println!("If fine-tuning is working correctly, you should see:");
    println!("• Low saturation: More muted/gray colors");
    println!("• High saturation: More vivid colors");
    println!("• Dark adjustment: Darker overall colors");
    println!("• Light adjustment: Lighter overall colors");
    println!("• Low contrast: Colors closer to background");
    println!("• High contrast: Colors very different from background");
    println!();
    println!("If all results look similar, fine-tuning is not working!");

    Ok(())
}
