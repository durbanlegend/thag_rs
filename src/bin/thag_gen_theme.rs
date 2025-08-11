/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["tools"] }
thag_styling = { version = "0.2", features = ["image_themes"] }
*/

use std::env;
use std::fs;
use std::path::Path;
use thag_rs::{auto_help, cprtln, Role, ThagResult};
use thag_styling::{generate_theme_from_image, ImageThemeConfig, ImageThemeGenerator, TermBgLuma};

/// Generate terminal color themes from images
///
/// This tool analyzes images and extracts dominant colors to create terminal color themes.
/// The generated themes can be saved as TOML files compatible with thag's theming system.
//# Purpose: Generate custom color themes from images using color analysis
//# Categories: theming, colors, tools, customization

fn print_usage() {
    println!("Usage:");
    println!("  thag_gen_theme <image_path>                      Generate theme from image (auto-detect light/dark)");
    println!("  thag_gen_theme <image_path> --light              Force light theme generation");
    println!("  thag_gen_theme <image_path> --dark               Force dark theme generation");
    println!("  thag_gen_theme <image_path> --name <theme_name>  Custom theme name");
    println!("  thag_gen_theme <image_path> --colors <count>     Number of colors to extract (default: 16)");
    println!("  thag_gen_theme <image_path> --output <file>      Save theme to file");
    println!("  thag_gen_theme help                              Show this help message");
    println!();
    println!("Examples:");
    println!("  thag_gen_theme sunset.jpg");
    println!("  thag_gen_theme nature.png --light --name forest-light");
    println!("  thag_gen_theme artwork.png --colors 20 --output my-theme.toml");
    println!();
    println!("Supported image formats: PNG, JPEG, GIF, BMP, TIFF, WebP");
}

fn main() -> ThagResult<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.contains(&"help".to_string()) || args.contains(&"--help".to_string())
    {
        auto_help();
        print_usage();
        return Ok(());
    }

    let image_path = &args[1];

    // Parse command line arguments
    let mut config = ImageThemeConfig::default();
    let mut theme_name: Option<String> = None;
    let mut output_file: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--light" => {
                config.force_theme_type = Some(TermBgLuma::Light);
                config.auto_detect_theme_type = false;
            }
            "--dark" => {
                config.force_theme_type = Some(TermBgLuma::Dark);
                config.auto_detect_theme_type = false;
            }
            "--name" => {
                if i + 1 < args.len() {
                    theme_name = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    cprtln!(Role::Error, "Missing theme name after --name");
                    return Ok(());
                }
            }
            "--colors" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<usize>() {
                        Ok(count) => {
                            config.color_count = count.max(8).min(64); // Reasonable bounds
                        }
                        Err(_) => {
                            cprtln!(Role::Error, "Invalid color count: {}", args[i + 1]);
                            return Ok(());
                        }
                    }
                    i += 1;
                } else {
                    cprtln!(Role::Error, "Missing color count after --colors");
                    return Ok(());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    cprtln!(Role::Error, "Missing output file after --output");
                    return Ok(());
                }
            }
            _ => {
                cprtln!(Role::Warning, "Unknown option: {}", args[i]);
            }
        }
        i += 1;
    }

    // Check if image file exists
    if !Path::new(image_path).exists() {
        cprtln!(Role::Error, "Image file not found: {}", image_path);
        return Ok(());
    }

    cprtln!(Role::Info, "🎨 Analyzing image: {}", image_path);

    // Generate theme name if not provided
    let final_theme_name = theme_name.unwrap_or_else(|| {
        let path = Path::new(image_path);
        let base_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("generated");

        let theme_type_suffix = match config.force_theme_type {
            Some(TermBgLuma::Light) => "-light",
            Some(TermBgLuma::Dark) => "-dark",
            None => "",
        };

        format!("image-{}{}", base_name, theme_type_suffix)
    });

    // Generate the theme
    let generator = ImageThemeGenerator::with_config(config);
    let theme = match generator.generate_from_file(image_path) {
        Ok(mut theme) => {
            theme.name = final_theme_name;
            theme
        }
        Err(e) => {
            cprtln!(Role::Error, "Failed to generate theme: {}", e);
            return Ok(());
        }
    };

    // Display theme information
    cprtln!(Role::Success, "✅ Generated theme: {}", theme.name);
    cprtln!(Role::Normal, "Description: {}", theme.description);
    cprtln!(Role::Normal, "Theme type: {:?}", theme.term_bg_luma);
    cprtln!(Role::Normal, "Color support: {:?}", theme.min_color_support);
    cprtln!(Role::Normal, "Background colors: {:?}", theme.backgrounds);

    println!();
    cprtln!(Role::Heading2, "Color palette:");
    display_palette(&theme);

    // Generate TOML content
    let toml_content = theme_to_toml(&theme)?;

    // Save to file if specified, otherwise print to stdout
    if let Some(output_path) = output_file {
        match fs::write(&output_path, &toml_content) {
            Ok(()) => {
                cprtln!(Role::Success, "💾 Theme saved to: {}", output_path);
            }
            Err(e) => {
                cprtln!(Role::Error, "Failed to write theme file: {}", e);
            }
        }
    } else {
        println!();
        cprtln!(Role::Heading2, "TOML representation:");
        println!("{}", toml_content);
    }

    Ok(())
}

fn display_palette(theme: &thag_styling::Theme) {
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
        ("Trace", &theme.palette.trace),
    ];

    for (name, style) in palette_items {
        if let Some(color_info) = &style.foreground {
            let rgb = match &color_info.value {
                thag_styling::ColorValue::TrueColor { rgb } => *rgb,
                thag_styling::ColorValue::Color256 { color256 } => color_256_to_rgb(*color256),
                thag_styling::ColorValue::Basic { .. } => [128, 128, 128],
            };

            let style_attrs = format_style_attributes(style);

            // Use the actual style to display the color name
            let styled_name = style.paint(name);

            println!(
                "  {:>12}: #{:02x}{:02x}{:02x}{}",
                styled_name, rgb[0], rgb[1], rgb[2], style_attrs
            );
        }
    }
}

fn format_style_attributes(style: &thag_styling::Style) -> String {
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
    if style.underline {
        attrs.push("underline");
    }

    if attrs.is_empty() {
        String::new()
    } else {
        format!(" ({})", attrs.join(", "))
    }
}

fn color_256_to_rgb(color: u8) -> [u8; 3] {
    match color {
        0..=15 => {
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
            let n = color - 16;
            let r = (n / 36) * 51;
            let g = ((n % 36) / 6) * 51;
            let b = (n % 6) * 51;
            [r, g, b]
        }
        232..=255 => {
            let gray = 8 + (color - 232) * 10;
            [gray, gray, gray]
        }
    }
}

fn theme_to_toml(theme: &thag_styling::Theme) -> ThagResult<String> {
    let mut toml = String::new();

    // Header information
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

    // Palette section
    let palette_items = [
        ("normal", &theme.palette.normal),
        ("subtle", &theme.palette.subtle),
        ("emphasis", &theme.palette.emphasis),
        ("heading1", &theme.palette.heading1),
        ("heading2", &theme.palette.heading2),
        ("heading3", &theme.palette.heading3),
        ("error", &theme.palette.error),
        ("warning", &theme.palette.warning),
        ("success", &theme.palette.success),
        ("info", &theme.palette.info),
        ("code", &theme.palette.code),
        ("hint", &theme.palette.hint),
        ("debug", &theme.palette.debug),
        ("trace", &theme.palette.trace),
    ];

    for (role_name, style) in palette_items {
        toml.push_str(&format!("[palette.{}]\n", role_name));

        if let Some(color_info) = &style.foreground {
            match &color_info.value {
                thag_styling::ColorValue::TrueColor { rgb } => {
                    toml.push_str(&format!("rgb = [{}, {}, {}]\n", rgb[0], rgb[1], rgb[2]));
                }
                thag_styling::ColorValue::Color256 { color256 } => {
                    let rgb = color_256_to_rgb(*color256);
                    toml.push_str(&format!("rgb = [{}, {}, {}]\n", rgb[0], rgb[1], rgb[2]));
                }
                thag_styling::ColorValue::Basic { .. } => {
                    toml.push_str("rgb = [128, 128, 128]  # Basic color fallback\n");
                }
            }
        }

        // Add style attributes
        let mut style_attrs = Vec::new();
        if style.bold {
            style_attrs.push("\"bold\"");
        }
        if style.italic {
            style_attrs.push("\"italic\"");
        }
        if style.dim {
            style_attrs.push("\"dim\"");
        }
        if style.underline {
            style_attrs.push("\"underline\"");
        }

        if !style_attrs.is_empty() {
            toml.push_str(&format!("style = [{}]\n", style_attrs.join(", ")));
        }

        toml.push('\n');
    }

    Ok(toml)
}
