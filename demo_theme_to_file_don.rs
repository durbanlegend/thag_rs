/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["image_themes"] }
image = "0.25"
toml = "0.9"
*/

//! Demo script that generates a theme from a synthetic image and saves it to a TOML file
//!
//! This demonstrates the complete workflow of image-based theme generation
//! and shows how the generated TOML files look.
//!
//! Run with: cargo +nightly -Zscript demo_theme_to_file.rs

use image::{DynamicImage, Rgb, RgbImage};
use std::fs;
use thag_styling::{
    save_theme_to_file, theme_to_toml, ImageThemeConfig, ImageThemeGenerator, TermBgLuma,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Theme Generation and File Save Demo");
    println!("=====================================\n");

    // Create a sample image with nice colors
    let sample_image = create_vibrant_image();

    // Generate a dark theme
    println!("📸 Generating dark theme from synthetic image...");
    let dark_config = ImageThemeConfig {
        color_count: 12,
        force_theme_type: Some(TermBgLuma::Dark),
        theme_name_prefix: Some("demo".to_string()),
        ..Default::default()
    };

    let generator = ImageThemeGenerator::with_config(dark_config);
    let dark_theme =
        generator.generate_from_image(sample_image.clone(), "vibrant-dark".to_string())?;

    println!("✅ Generated theme: {}", dark_theme.name);
    println!("   Type: {:?}", dark_theme.term_bg_luma);
    println!("   Colors: {:?}", dark_theme.min_color_support);

    // Save the theme to a file
    let dark_filename = "demo_dark_theme.toml";
    save_theme_to_file(&dark_theme, dark_filename)?;
    println!("💾 Saved dark theme to: {}", dark_filename);

    // Generate a light theme
    println!("\n☀️ Generating light theme from the same image...");
    let light_config = ImageThemeConfig {
        color_count: 14,
        force_theme_type: Some(TermBgLuma::Light),
        theme_name_prefix: Some("demo".to_string()),
        ..Default::default()
    };

    let light_generator = ImageThemeGenerator::with_config(light_config);
    let light_theme =
        light_generator.generate_from_image(sample_image, "vibrant-light".to_string())?;

    println!("✅ Generated theme: {}", light_theme.name);
    println!("   Type: {:?}", light_theme.term_bg_luma);
    println!("   Colors: {:?}", light_theme.min_color_support);

    // Save the light theme
    let light_filename = "demo_light_theme.toml";
    save_theme_to_file(&light_theme, light_filename)?;
    println!("💾 Saved light theme to: {}", light_filename);

    // Show a preview of the dark theme TOML
    println!("\n📄 Preview of dark theme TOML (first 30 lines):");
    println!("{}", "─".repeat(50));

    let toml_content = theme_to_toml(&dark_theme)?;
    for (i, line) in toml_content.lines().enumerate() {
        if i < 30 {
            println!("{}", line);
        } else {
            println!("... (rest saved to file)");
            break;
        }
    }

    // Validate that the generated files are valid TOML
    println!("\n🔍 Validating generated TOML files...");

    let dark_content = fs::read_to_string(dark_filename)?;
    let _dark_parsed: toml::Value = toml::from_str(&dark_content)?;
    println!("✅ {} is valid TOML", dark_filename);

    let light_content = fs::read_to_string(light_filename)?;
    let _light_parsed: toml::Value = toml::from_str(&light_content)?;
    println!("✅ {} is valid TOML", light_filename);

    // Show color statistics
    println!("\n📊 Theme Statistics:");
    println!("Dark theme background: {:?}", dark_theme.backgrounds);
    println!("Light theme background: {:?}", light_theme.backgrounds);

    // Count palette entries
    let dark_toml_lines = toml_content.lines().count();
    println!("Dark theme TOML lines: {}", dark_toml_lines);

    println!("\n✨ Demo completed successfully!");
    println!("Generated files:");
    println!("  - {}", dark_filename);
    println!("  - {}", light_filename);
    println!(
        "\nYou can now use these theme files with thag or any compatible terminal application!"
    );

    Ok(())
}

fn create_vibrant_image() -> DynamicImage {
    let mut img = RgbImage::new(200, 150);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        // Create a gradient pattern with vibrant colors
        let section_x = x / 40;
        let section_y = y / 30;

        let color = match (section_x % 5, section_y % 5) {
            // Top row - cool colors
            (0, 0) => Rgb([41, 128, 185]), // Blue
            (1, 0) => Rgb([142, 68, 173]), // Purple
            (2, 0) => Rgb([26, 188, 156]), // Turquoise
            (3, 0) => Rgb([22, 160, 133]), // Green sea
            (4, 0) => Rgb([52, 152, 219]), // Light blue

            // Second row - warm colors
            (0, 1) => Rgb([231, 76, 60]),  // Red
            (1, 1) => Rgb([230, 126, 34]), // Orange
            (2, 1) => Rgb([241, 196, 15]), // Yellow
            (3, 1) => Rgb([39, 174, 96]),  // Green
            (4, 1) => Rgb([155, 89, 182]), // Violet

            // Third row - earth tones
            (0, 2) => Rgb([192, 57, 43]),  // Dark red
            (1, 2) => Rgb([211, 84, 0]),   // Dark orange
            (2, 2) => Rgb([243, 156, 18]), // Dark yellow
            (3, 2) => Rgb([27, 138, 81]),  // Dark green
            (4, 2) => Rgb([125, 60, 152]), // Dark violet

            // Fourth row - pastels
            (0, 3) => Rgb([255, 182, 193]), // Light pink
            (1, 3) => Rgb([255, 218, 185]), // Peach
            (2, 3) => Rgb([255, 255, 224]), // Light yellow
            (3, 3) => Rgb([144, 238, 144]), // Light green
            (4, 3) => Rgb([221, 160, 221]), // Plum

            // Bottom row - neutral with some accent
            (0, 4) => Rgb([127, 140, 141]), // Gray
            (1, 4) => Rgb([149, 165, 166]), // Light gray
            (2, 4) => Rgb([236, 240, 241]), // Very light gray
            (3, 4) => Rgb([52, 73, 94]),    // Dark gray
            (4, 4) => Rgb([44, 62, 80]),

            _ => Rgb([248, 248, 248]), // Default light background
        };

        *pixel = color;
    }

    DynamicImage::ImageRgb8(img)
}
