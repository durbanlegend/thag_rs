/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["image_themes"] }
*/

/// Light theme tuning previewer - shows fine-tuning effects optimized for light themes
///
/// This script demonstrates fine-tuning controls with parameter ranges
/// optimized for light theme generation. For dark themes, use test_dark_theme_tuning.rs
//# Purpose: Preview and tune light theme generation with optimized parameters
//# Categories: color, styling, terminal, theming, tools
use std::path::Path;
use thag_styling::{ImageThemeConfig, ImageThemeGenerator, StylingResult, TermBgLuma};

fn test_config(name: &str, image_path: &Path, config: ImageThemeConfig) -> StylingResult<()> {
    println!("🎨 {}", name);
    println!("{}", "-".repeat(40));

    let generator = ImageThemeGenerator::with_config(config);

    match generator.generate_from_file(image_path) {
        Ok(theme) => {
            // Show key colors that demonstrate the effects
            println!(
                "Normal:   {}",
                theme
                    .palette
                    .normal
                    .paint(format!("■■■■■ Regular text ({:?})", theme.palette.normal))
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
            println!("{generator:#?}");
            println!();
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    Ok(())
}

fn main() -> StylingResult<()> {
    println!("☀️  Light Theme Fine-Tuning Previewer");
    println!("{}", "=".repeat(50));
    println!();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!(
            "{}: Explore fine-tuning of theme generation from an image",
            args[0]
        );
        eprintln!("Usage: {} <image_file_path>", args[0]);
        std::process::exit(1);
    }

    let image_path = Path::new(&args[1]);

    // Ensure image file exists
    if !image_path.exists() {
        eprintln!("Error: Umage file does not exist: {}", image_path.display());
        std::process::exit(1);
    }

    // Default light theme settings
    test_config(
        "Default Light Settings (baseline)",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            ..Default::default()
        },
    )?;

    // Moderate saturation - light themes look better with less aggressive saturation
    test_config(
        "Moderate Saturation (0.9x) - Refined Colors",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            saturation_multiplier: 0.9,
            ..Default::default()
        },
    )?;

    // Higher saturation - ... or do they?
    test_config(
        "Higher Saturation (1.4x) - Richer Colors",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            saturation_multiplier: 1.4,
            lightness_adjustment: 0.2,
            contrast_multiplier: 1.4,
            ..Default::default()
        },
    )?;

    // Darker theme - darken colors for better contrast against light background
    test_config(
        "Darker Colors (-0.1) - Better Contrast",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            lightness_adjustment: -0.1,
            ..Default::default()
        },
    )?;

    // Higher contrast - more dramatic differences
    test_config(
        "Higher Contrast (1.2x) - More Definition",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            contrast_multiplier: 1.2,
            ..Default::default()
        },
    )?;

    // Professional look - subtle saturation with good contrast
    test_config(
        "Professional (0.85x sat, -0.05 light, 1.1x contrast)",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            saturation_multiplier: 0.85,
            lightness_adjustment: -0.05,
            contrast_multiplier: 1.1,
            ..Default::default()
        },
    )?;

    // Rich but refined - preserve color character while ensuring readability
    test_config(
        "Rich & Refined (1.1x sat, -0.08 light, 1.15x contrast)",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            saturation_multiplier: 1.1,
            lightness_adjustment: -0.08,
            contrast_multiplier: 1.15,
            ..Default::default()
        },
    )?;

    // High contrast accessibility
    test_config(
        "High Contrast Accessibility (0.8x sat, -0.15, 1.4x contrast)",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            saturation_multiplier: 0.8,
            lightness_adjustment: -0.15,
            contrast_multiplier: 1.4,
            ..Default::default()
        },
    )?;

    // Subtle and elegant
    test_config(
        "Subtle & Elegant (0.75x sat, -0.02, 0.9x contrast)",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            saturation_multiplier: 0.75,
            lightness_adjustment: -0.02,
            contrast_multiplier: 0.9,
            ..Default::default()
        },
    )?;

    println!("💡 Light Theme Parameter Guide:");
    println!("• saturation_multiplier: 0.7-1.2 (light themes need restraint)");
    println!("• lightness_adjustment: -0.2 to +0.05 (darken colors for contrast)");
    println!("• contrast_multiplier: 0.8-1.4 (can go higher for accessibility)");
    println!();
    println!("🎯 Recommended Light Theme Combinations:");
    println!("   • Professional: sat=0.85, light=-0.05, contrast=1.1");
    println!("   • Rich Colors: sat=1.1, light=-0.08, contrast=1.15");
    println!("   • Accessibility: sat=0.8, light=-0.15, contrast=1.4");
    println!("   • Elegant: sat=0.75, light=-0.02, contrast=0.9");
    println!();
    println!("🌙 For dark themes, use test_dark_theme_tuning.rs instead!");

    Ok(())
}
