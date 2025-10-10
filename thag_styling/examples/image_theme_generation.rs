//! Example demonstrating image-based theme generation
//!
//! This example shows how to generate terminal color themes from images using
//! the thag_styling crate's image theme generation capabilities.
//!
//! Run with: cargo run --example image_theme_generation --features image_themes

#[cfg(feature = "image_themes")]
use thag_styling::{theme_to_toml, ImageThemeConfig, ImageThemeGenerator, TermBgLuma};

#[cfg(feature = "image_themes")]
use image::{DynamicImage, Rgb, RgbImage};

#[cfg(not(feature = "image_themes"))]
fn main() {
    println!("This example requires the 'image_themes' feature to be enabled.");
    println!("Run with: cargo run --example image_theme_generation --features image_themes");
}

#[cfg(feature = "image_themes")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Image Theme Generation Example\n");

    // Create a sample image for demonstration
    let sample_image = create_sample_image();

    // Example 1: Generate theme with default settings
    println!("📸 Example 1: Default theme generation");
    let generator = ImageThemeGenerator::new();
    let theme1 = generator.generate_from_image(&sample_image, "sample-default".to_string())?;

    println!("Generated theme: {}", theme1.name);
    println!("Description: {}", theme1.description);
    println!("Theme type: {:?}", theme1.term_bg_luma);
    println!("Background colors: {:?}", theme1.backgrounds);
    print_theme_colors(&theme1);
    println!();

    // Example 2: Generate light theme with custom configuration
    println!("☀️ Example 2: Force light theme with custom settings");
    let light_config = ImageThemeConfig {
        color_count: 12,
        light_threshold: 0.6,
        saturation_threshold: 0.4,
        auto_detect_theme_type: false,
        force_theme_type: Some(TermBgLuma::Light),
        theme_name_prefix: Some("custom".to_string()),
        ..Default::default()
    };

    let generator2 = ImageThemeGenerator::with_config(light_config);
    let theme2 =
        generator2.generate_from_image(&create_sample_image(), "light-sample".to_string())?;

    println!("Generated theme: {}", theme2.name);
    println!("Theme type: {:?}", theme2.term_bg_luma);
    print_theme_colors(&theme2);
    println!();

    // Example 3: Generate dark theme
    println!("🌙 Example 3: Force dark theme");
    let dark_config = ImageThemeConfig {
        force_theme_type: Some(TermBgLuma::Dark),
        theme_name_prefix: Some("dark".to_string()),
        ..Default::default()
    };

    let generator3 = ImageThemeGenerator::with_config(dark_config);
    let theme3 =
        generator3.generate_from_image(&create_bright_image(), "dark-sample".to_string())?;

    println!("Generated theme: {}", theme3.name);
    println!("Theme type: {:?}", theme3.term_bg_luma);
    print_theme_colors(&theme3);
    println!();

    // Example 4: Generate theme from a sunset-like image
    println!("🌅 Example 4: Sunset color palette");
    let sunset_image = create_sunset_image();
    let generator4 = ImageThemeGenerator::new();
    let sunset_theme = generator4.generate_from_image(&sunset_image, "sunset-theme".to_string())?;

    println!("Generated theme: {}", sunset_theme.name);
    println!("Theme type: {:?}", sunset_theme.term_bg_luma);
    print_theme_colors(&sunset_theme);
    println!();

    // Example 5: Show how to save theme as TOML
    println!("💾 Example 5: Export theme as TOML");
    let toml_content = theme_to_toml(&sunset_theme)?;
    println!("TOML representation:");
    println!("{}", toml_content);

    // Save to file for inspection
    use std::fs;
    let filename = "example_sunset_theme.toml";
    fs::write(filename, &toml_content)?;
    println!("\n💾 Theme saved to: {}", filename);

    println!("✨ Image theme generation examples completed!");
    Ok(())
}

#[cfg(feature = "image_themes")]
fn create_sample_image() -> DynamicImage {
    let mut img = RgbImage::new(200, 200);

    // Create a gradient-like pattern with various colors
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let color = match ((x / 50) % 4, (y / 50) % 4) {
            (0, 0) => Rgb([70, 130, 180]),  // Steel blue
            (1, 0) => Rgb([60, 179, 113]),  // Medium sea green
            (2, 0) => Rgb([255, 140, 0]),   // Dark orange
            (3, 0) => Rgb([220, 20, 60]),   // Crimson
            (0, 1) => Rgb([147, 112, 219]), // Medium purple
            (1, 1) => Rgb([255, 215, 0]),   // Gold
            (2, 1) => Rgb([32, 178, 170]),  // Light sea green
            (3, 1) => Rgb([255, 69, 0]),    // Red orange
            (0, 2) => Rgb([123, 104, 238]), // Medium slate blue
            (1, 2) => Rgb([34, 139, 34]),   // Forest green
            (2, 2) => Rgb([255, 165, 0]),   // Orange
            (3, 2) => Rgb([205, 92, 92]),   // Indian red
            _ => Rgb([240, 240, 240]),      // Light background
        };
        *pixel = color;
    }

    DynamicImage::ImageRgb8(img)
}

#[cfg(feature = "image_themes")]
fn create_bright_image() -> DynamicImage {
    let mut img = RgbImage::new(150, 150);

    // Create a bright, vibrant image
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let color = match ((x / 30) % 5, (y / 30) % 5) {
            (0, _) => Rgb([255, 0, 255]), // Magenta
            (1, _) => Rgb([0, 255, 255]), // Cyan
            (2, _) => Rgb([255, 255, 0]), // Yellow
            (3, _) => Rgb([255, 0, 0]),   // Red
            (4, _) => Rgb([0, 255, 0]),   // Green
            _ => Rgb([255, 255, 255]),    // White
        };
        *pixel = color;
    }

    DynamicImage::ImageRgb8(img)
}

#[cfg(feature = "image_themes")]
fn create_sunset_image() -> DynamicImage {
    let mut img = RgbImage::new(300, 200);

    // Create a sunset gradient
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let progress_y = y as f32 / 200.0;
        let progress_x = x as f32 / 300.0;

        let color = if progress_y < 0.3 {
            // Sky - gradient from dark blue to orange
            let t = progress_y / 0.3;
            let r = (30.0 + (255.0 - 30.0) * t * progress_x) as u8;
            let g = (30.0 + (140.0 - 30.0) * t) as u8;
            let b = (100.0 * (1.0 - t)) as u8;
            Rgb([r, g, b])
        } else if progress_y < 0.6 {
            // Horizon - warm colors
            let t = (progress_y - 0.3) / 0.3;
            let r = (255.0 - 50.0 * t) as u8;
            let g = (100.0 + 55.0 * t) as u8;
            let b = (20.0 + 30.0 * t) as u8;
            Rgb([r, g, b])
        } else {
            // Ground - darker earth tones
            let t = (progress_y - 0.6) / 0.4;
            let r = (139.0 - 90.0 * t) as u8;
            let g = (69.0 - 40.0 * t) as u8;
            let b = (19.0 - 10.0 * t) as u8;
            Rgb([r, g, b])
        };

        *pixel = color;
    }

    // let _ = img.save("sunset.png");
    DynamicImage::ImageRgb8(img)
}

#[cfg(feature = "image_themes")]
fn print_theme_colors(theme: &thag_styling::Theme) {
    println!("Color palette:");

    // Access palette fields directly
    let palette_items = [
        ("Normal", &theme.palette.normal),
        ("Subtle", &theme.palette.subtle),
        ("Emphasis", &theme.palette.emphasis),
        ("Heading1", &theme.palette.heading1),
        ("Heading2", &theme.palette.heading2),
        ("Heading3", &theme.palette.heading3),
        ("Error", &theme.palette.error),
        ("Warning", &theme.palette.warning),
        ("Success", &theme.palette.success),
        ("Info", &theme.palette.info),
        ("Code", &theme.palette.code),
        ("Hint", &theme.palette.hint),
        ("Debug", &theme.palette.debug),
        ("Link", &theme.palette.link),
        ("Quote", &theme.palette.quote),
        ("Commentary", &theme.palette.commentary),
    ];

    for (name, style) in palette_items {
        if let Some(color_info) = &style.foreground {
            let rgb = match &color_info.value {
                thag_styling::ColorValue::TrueColor { rgb } => *rgb,
                thag_styling::ColorValue::Color256 { color256 } => {
                    // Convert 256-color to approximate RGB
                    color_256_to_rgb(*color256)
                }
                thag_styling::ColorValue::Basic { .. } => [128, 128, 128], // Gray fallback
            };

            let style_attrs = if style.bold && style.italic {
                " (bold, italic)"
            } else if style.bold {
                " (bold)"
            } else if style.italic {
                " (italic)"
            } else if style.dim {
                " (dim)"
            } else {
                ""
            };

            println!(
                "  {:>12}: #{:02x}{:02x}{:02x}{}",
                name, rgb[0], rgb[1], rgb[2], style_attrs
            );
        }
    }
}

#[cfg(feature = "image_themes")]
fn color_256_to_rgb(color: u8) -> [u8; 3] {
    match color {
        0..=15 => {
            // Standard 16 colors
            let colors = [
                [0, 0, 0],
                [128, 0, 0],
                [0, 128, 0],
                [128, 128, 0],
                [0, 0, 128],
                [128, 0, 128],
                [0, 128, 128],
                [192, 192, 192],
                [128, 128, 128],
                [255, 0, 0],
                [0, 255, 0],
                [255, 255, 0],
                [0, 0, 255],
                [255, 0, 255],
                [0, 255, 255],
                [255, 255, 255],
            ];
            colors[color as usize]
        }
        16..=231 => {
            // 216 color cube
            let n = color - 16;
            let r = (n / 36) * 51;
            let g = ((n % 36) / 6) * 51;
            let b = (n % 6) * 51;
            [r, g, b]
        }
        232..=255 => {
            // Grayscale
            let gray = 8 + (color - 232) * 10;
            [gray, gray, gray]
        }
    }
}

// Use the theme_to_toml function from the library
// (function removed since it's now available in the library)
