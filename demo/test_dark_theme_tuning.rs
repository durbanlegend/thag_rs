/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["image_themes"] }
*/

/// Dark theme tuning previewer - shows fine-tuning effects optimized for dark themes
///
/// This script demonstrates fine-tuning controls with parameter ranges
/// optimized for dark theme generation. For light themes, use test_light_theme_tuning.rs
//# Purpose: Preview and tune dark theme generation with optimized parameters
//# Categories: color, styling, terminal, theming, tools
use std::path::Path;
use thag_styling::{
    styling::rgb_to_hex, ImageThemeConfig, ImageThemeGenerator, StylingResult, TermBgLuma,
};

fn test_config(name: &str, image_path: &Path, config: ImageThemeConfig) -> StylingResult<()> {
    println!("🎨 {}", name);
    println!("{}", "-".repeat(40));

    let generator = ImageThemeGenerator::with_config(config);

    match generator.generate_from_file(image_path) {
        Ok(theme) => {
            // Show the palette colours
            theme.palette.iter().for_each(|(style_name, style)| {
                if let Some([r, g, b]) = style.rgb() {
                    println!(
                        "{}",
                        style.paint(format!(
                            "{style_name:<12} ■■■■■ {} = ({r:>3},{g:>3},{b:>3})",
                            rgb_to_hex(&(r, g, b))
                        ))
                    );
                }
            });

            println!();
            println!("{generator:#?}");
            println!();
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    Ok(())
}

fn main() -> StylingResult<()> {
    println!("🌙 Dark Theme Fine-Tuning Previewer");
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

    // Default dark theme settings
    test_config(
        "Default Dark Settings (baseline)",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Dark),
            ..Default::default()
        },
    )?;

    // High saturation - more vibrant colors
    test_config(
        "High Saturation (1.5x) - More Vivid",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Dark),
            saturation_multiplier: 1.5,
            ..Default::default()
        },
    )?;

    // Brighter theme - lighter colors
    test_config(
        "Brighter Theme (+0.15) - Softer Look",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Dark),
            lightness_adjustment: 0.15,
            ..Default::default()
        },
    )?;

    // High contrast - more dramatic differences
    test_config(
        "High Contrast (1.3x) - More Dramatic",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Dark),
            contrast_multiplier: 1.3,
            ..Default::default()
        },
    )?;

    // Combined effects
    test_config(
        "Vivid & High Contrast - Maximum Impact",
        image_path,
        ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Dark),
            saturation_multiplier: 1.4,
            lightness_adjustment: 0.05,
            contrast_multiplier: 1.2,
            ..Default::default()
        },
    )?;

    println!("💡 Dark Theme Parameter Guide:");
    println!("• saturation_multiplier: 0.7-2.0 (dark themes can handle higher saturation)");
    println!("• lightness_adjustment: -0.1 to +0.3 (brighten colors for dark backgrounds)");
    println!("• contrast_multiplier: 0.8-1.5 (dramatic contrast works well)");
    println!();
    println!("🎯 For light themes, use test_light_theme_tuning.rs instead!");

    Ok(())
}
