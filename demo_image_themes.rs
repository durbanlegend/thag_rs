#!/usr/bin/env cargo +nightly -Zscript
/*[toml]
[dependencies]
thag_styling = { path = "thag_styling", version = "0.2.0", features = ["image_themes"] }
image = "0.25"
*/

//! Demo script for image-based theme generation
//!
//! This demonstrates the new image theme generation capabilities in thag_styling.
//! It creates sample images and generates color themes from them.
//!
//! Run with: cargo +nightly -Zscript demo_image_themes.rs

use image::{DynamicImage, Rgb, RgbImage};
use thag_styling::{ImageThemeConfig, ImageThemeGenerator, TermBgLuma};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Image Theme Generation Demo");
    println!("===============================\n");

    // Demo 1: Generate theme from a colorful synthetic image
    println!("🌈 Demo 1: Colorful synthetic image");
    let colorful_image = create_colorful_image();
    let generator = ImageThemeGenerator::new();
    let theme1 = generator.generate_from_image(colorful_image, "colorful-demo".to_string())?;

    display_theme_info(&theme1);
    println!();

    // Demo 2: Generate a light theme with custom settings
    println!("☀️ Demo 2: Forced light theme with custom config");
    let config = ImageThemeConfig {
        color_count: 12,
        force_theme_type: Some(TermBgLuma::Light),
        theme_name_prefix: Some("custom".to_string()),
        ..Default::default()
    };

    let bright_image = create_bright_image();
    let generator2 = ImageThemeGenerator::with_config(config);
    let theme2 = generator2.generate_from_image(bright_image, "bright-light".to_string())?;

    display_theme_info(&theme2);
    println!();

    // Demo 3: Generate a dark theme from earth tones
    println!("🌍 Demo 3: Earth tones dark theme");
    let earth_image = create_earth_tones_image();
    let dark_config = ImageThemeConfig {
        force_theme_type: Some(TermBgLuma::Dark),
        color_count: 10,
        ..Default::default()
    };

    let generator3 = ImageThemeGenerator::with_config(dark_config);
    let theme3 = generator3.generate_from_image(earth_image, "earth-tones".to_string())?;

    display_theme_info(&theme3);
    println!();

    // Demo 4: Show TOML export
    println!("💾 Demo 4: TOML export example");
    let toml_content = generate_toml(&theme3)?;
    println!("Generated TOML (first 20 lines):");
    println!("{}", "─".repeat(40));
    for (i, line) in toml_content.lines().enumerate() {
        if i < 20 {
            println!("{}", line);
        } else {
            println!("... (truncated)");
            break;
        }
    }

    println!("\n✨ Demo completed successfully!");
    println!("Image theme generation is working and ready for integration!");

    Ok(())
}

fn create_colorful_image() -> DynamicImage {
    let mut img = RgbImage::new(160, 120);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let color = match ((x / 40) % 4, (y / 30) % 4) {
            (0, 0) => Rgb([220, 20, 60]),   // Crimson
            (1, 0) => Rgb([50, 205, 50]),   // Lime green
            (2, 0) => Rgb([30, 144, 255]),  // Dodger blue
            (3, 0) => Rgb([255, 140, 0]),   // Dark orange
            (0, 1) => Rgb([138, 43, 226]),  // Blue violet
            (1, 1) => Rgb([255, 215, 0]),   // Gold
            (2, 1) => Rgb([0, 191, 255]),   // Deep sky blue
            (3, 1) => Rgb([255, 69, 0]),    // Red orange
            (0, 2) => Rgb([147, 112, 219]), // Medium purple
            (1, 2) => Rgb([60, 179, 113]),  // Medium sea green
            (2, 2) => Rgb([255, 165, 0]),   // Orange
            (3, 2) => Rgb([205, 92, 92]),   // Indian red
            _ => Rgb([240, 248, 255]),      // Alice blue background
        };
        *pixel = color;
    }

    DynamicImage::ImageRgb8(img)
}

fn create_bright_image() -> DynamicImage {
    let mut img = RgbImage::new(120, 120);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let color = match ((x / 24) % 5, (y / 24) % 5) {
            (0, _) => Rgb([255, 182, 193]), // Light pink
            (1, _) => Rgb([173, 216, 230]), // Light blue
            (2, _) => Rgb([144, 238, 144]), // Light green
            (3, _) => Rgb([255, 218, 185]), // Peach puff
            (4, _) => Rgb([221, 160, 221]), // Plum
            _ => Rgb([255, 255, 255]),      // White
        };
        *pixel = color;
    }

    DynamicImage::ImageRgb8(img)
}

fn create_earth_tones_image() -> DynamicImage {
    let mut img = RgbImage::new(150, 100);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let progress_x = x as f32 / 150.0;
        let progress_y = y as f32 / 100.0;

        let color = if progress_y < 0.3 {
            // Sky area - muted blues and grays
            let r = (120.0 + 60.0 * progress_x) as u8;
            let g = (130.0 + 50.0 * progress_x) as u8;
            let b = (140.0 + 80.0 * progress_x) as u8;
            Rgb([r, g, b])
        } else if progress_y < 0.7 {
            // Middle area - earth tones
            let r = (139.0 - 40.0 * progress_x) as u8;
            let g = (100.0 - 20.0 * progress_x) as u8;
            let b = (50.0 - 30.0 * progress_x) as u8;
            Rgb([r, g, b])
        } else {
            // Ground area - darker earth
            let r = (101.0 - 50.0 * progress_x) as u8;
            let g = (67.0 - 30.0 * progress_x) as u8;
            let b = (33.0 - 20.0 * progress_x) as u8;
            Rgb([r, g, b])
        };

        *pixel = color;
    }

    DynamicImage::ImageRgb8(img)
}

fn display_theme_info(theme: &thag_styling::Theme) {
    println!("Generated theme: {}", theme.name);
    println!("Description: {}", theme.description);
    println!("Theme type: {:?}", theme.term_bg_luma);
    println!("Background: {:?}", theme.backgrounds);

    println!("Color palette:");
    let palette_items = [
        ("Normal", &theme.palette.normal),
        ("Subtle", &theme.palette.subtle),
        ("Emphasis", &theme.palette.emphasis),
        ("Error", &theme.palette.error),
        ("Warning", &theme.palette.warning),
        ("Success", &theme.palette.success),
        ("Info", &theme.palette.info),
        ("Code", &theme.palette.code),
        ("Heading1", &theme.palette.heading1),
        ("Heading2", &theme.palette.heading2),
    ];

    for (name, style) in palette_items.iter().take(6) {
        // Show first 6 for brevity
        if let Some(color_info) = &style.foreground {
            let rgb = match &color_info.value {
                thag_styling::ColorValue::TrueColor { rgb } => *rgb,
                _ => [128, 128, 128], // Fallback
            };

            let attrs = format_attrs(style);
            println!(
                "  {:>10}: #{:02x}{:02x}{:02x}{}",
                name, rgb[0], rgb[1], rgb[2], attrs
            );
        }
    }
}

fn format_attrs(style: &thag_styling::Style) -> String {
    let mut attrs = Vec::new();
    if style.bold {
        attrs.push("bold");
    }
    if style.italic {
        attrs.push("italic");
    }
    if style.dim {
        attrs.push("dim");
    }

    if attrs.is_empty() {
        String::new()
    } else {
        format!(" ({})", attrs.join(", "))
    }
}

fn generate_toml(theme: &thag_styling::Theme) -> Result<String, Box<dyn std::error::Error>> {
    let mut toml = String::new();

    toml.push_str(&format!("name = {:?}\n", theme.name));
    toml.push_str(&format!("description = {:?}\n", theme.description));
    toml.push_str(&format!(
        "term_bg_luma = {:?}\n",
        format!("{:?}", theme.term_bg_luma).to_lowercase()
    ));
    toml.push_str(&format!(
        "min_color_support = {:?}\n",
        format!("{:?}", theme.min_color_support).to_lowercase()
    ));
    toml.push_str(&format!("backgrounds = {:?}\n", theme.backgrounds));
    toml.push_str("bg_rgbs = [\n");
    for rgb in &theme.bg_rgbs {
        toml.push_str(&format!("    [{}, {}, {}],\n", rgb.0, rgb.1, rgb.2));
    }
    toml.push_str("]\n\n");

    // Add a few palette entries as examples
    let entries = [
        ("normal", &theme.palette.normal),
        ("error", &theme.palette.error),
        ("success", &theme.palette.success),
        ("heading1", &theme.palette.heading1),
    ];

    for (name, style) in entries {
        toml.push_str(&format!("[palette.{}]\n", name));
        if let Some(color_info) = &style.foreground {
            if let thag_styling::ColorValue::TrueColor { rgb } = &color_info.value {
                toml.push_str(&format!("rgb = [{}, {}, {}]\n", rgb[0], rgb[1], rgb[2]));
            }
        }
        if style.bold {
            toml.push_str("style = [\"bold\"]\n");
        }
        toml.push('\n');
    }

    Ok(toml)
}
