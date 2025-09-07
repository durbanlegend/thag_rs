/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["image_themes"] }
*/

/// Test script to demonstrate fine-tuning controls for theme generation
///
/// This script shows how to use the saturation multiplier, lightness adjustment,
/// and contrast multiplier to fine-tune generated themes.
//# Purpose: Demonstrate fine-tuning controls for image theme generation
//# Categories: color, styling, terminal, theming, tools
use std::path::Path;
use thag_styling::{ImageThemeConfig, ImageThemeGenerator, StylingResult, TermBgLuma};

fn test_theme_with_settings(
    image_path: &Path,
    name: &str,
    saturation_mult: f32,
    lightness_adj: f32,
    contrast_mult: f32,
) -> StylingResult<()> {
    println!("🎨 Testing: {}", name);
    println!(
        "   Saturation: {:.1}x, Lightness: {:+.1}, Contrast: {:.1}x",
        saturation_mult, lightness_adj, contrast_mult
    );
    println!("{}", "-".repeat(50));

    let config = ImageThemeConfig {
        theme_name_prefix: Some(format!(
            "fine-tune-{}",
            name.to_lowercase().replace(' ', "-")
        )),
        force_theme_type: Some(TermBgLuma::Dark),
        saturation_multiplier: saturation_mult,
        lightness_adjustment: lightness_adj,
        contrast_multiplier: contrast_mult,
        ..Default::default()
    };

    let generator = ImageThemeGenerator::with_config(config);

    match generator.generate_from_file(image_path) {
        Ok(theme) => {
            // Display sample colors
            let sample_colors = [
                ("Normal", &theme.palette.normal),
                ("Error", &theme.palette.error),
                ("Warning", &theme.palette.warning),
                ("Success", &theme.palette.success),
                ("Info", &theme.palette.info),
                ("Code", &theme.palette.code),
                ("Emphasis", &theme.palette.emphasis),
                ("Quote", &theme.palette.quote),
            ];

            for (name, style) in sample_colors {
                println!(
                    "{:9} {}",
                    format!("{}:", name),
                    style.paint("■■■■■■ Sample text")
                );
            }

            println!();
            println!("📝 Sample content:");
            println!(
                "{}",
                theme
                    .palette
                    .normal
                    .paint("Regular text that should be easily readable.")
            );
            println!(
                "{}",
                theme
                    .palette
                    .code
                    .paint("function example() { return 'code'; }")
            );
            println!(
                "{}",
                theme
                    .palette
                    .emphasis
                    .paint("Important emphasized text stands out.")
            );
            println!(
                "{}",
                theme
                    .palette
                    .error
                    .paint("❌ Error messages are clearly visible.")
            );
            println!(
                "{}",
                theme
                    .palette
                    .success
                    .paint("✅ Success messages look positive.")
            );
        }
        Err(e) => {
            println!("❌ Failed to generate theme: {}", e);
        }
    }

    println!();
    Ok(())
}

fn main() -> StylingResult<()> {
    println!("🔧 Fine-Tuning Controls Demo");
    println!("{}", "=".repeat(60));
    println!();

    let image_path = Path::new("assets/thag_morning_coffee_figma.png");

    if !image_path.exists() {
        println!("❌ Test image not found: {}", image_path.display());
        return Ok(());
    }

    println!("📷 Using: {}", image_path.display());
    println!();

    // Test different fine-tuning combinations
    let test_cases = [
        ("Default Settings", 1.0, 0.0, 1.0),
        ("High Saturation", 1.5, 0.0, 1.0),
        ("Low Saturation", 0.7, 0.0, 1.0),
        ("Brighter Theme", 1.0, 0.15, 1.0),
        ("Darker Theme", 1.0, -0.15, 1.0),
        ("High Contrast", 1.0, 0.0, 1.3),
        ("Low Contrast", 1.0, 0.0, 0.7),
        ("Vivid & Bright", 1.4, 0.1, 1.1),
        ("Muted & Dark", 0.8, -0.1, 0.8),
        ("Maximum Saturation", 2.0, 0.0, 1.0),
        ("Minimal Saturation", 0.5, 0.0, 1.0),
    ];

    for (name, sat_mult, light_adj, contrast_mult) in test_cases {
        test_theme_with_settings(image_path, name, sat_mult, light_adj, contrast_mult)?;
        println!("{}", "=".repeat(60));
        println!();
    }

    println!("💡 Fine-tuning Parameter Guide:");
    println!();
    println!("🎨 Saturation Multiplier (0.5 - 2.0):");
    println!("   • 0.5-0.8: Muted, professional look");
    println!("   • 1.0: Natural saturation (default)");
    println!("   • 1.2-1.5: Vivid, energetic colors");
    println!("   • 1.6-2.0: Maximum vibrancy");
    println!();
    println!("💡 Lightness Adjustment (-0.3 to +0.3):");
    println!("   • -0.3 to -0.1: Darker, more dramatic");
    println!("   • 0.0: Natural lightness (default)");
    println!("   • +0.1 to +0.3: Brighter, softer appearance");
    println!();
    println!("🔍 Contrast Multiplier (0.5 - 1.5):");
    println!("   • 0.5-0.7: Subtle, low contrast");
    println!("   • 1.0: Balanced contrast (default)");
    println!("   • 1.2-1.5: High contrast for accessibility");
    println!();
    println!("🎯 Recommended Combinations:");
    println!("   • Professional: sat=0.8, light=0.0, contrast=1.1");
    println!("   • Vibrant: sat=1.4, light=0.1, contrast=1.0");
    println!("   • Accessibility: sat=1.0, light=0.0, contrast=1.3");
    println!("   • Artistic: sat=1.6, light=-0.05, contrast=0.9");

    Ok(())
}
