/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }

[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect", "image_themes"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config", "image_themes"] }
*/
#![allow(clippy::uninlined_format_args)]
/// Generate `thag_styling` themes from image files using file navigator.
///
/// This tool analyzes image files to extract dominant colors and generates
/// `thag_styling`-compatible theme files. Supports auto-detection of theme type
/// (light/dark) and customizable color extraction parameters.
//# Purpose: Generate custom `thag_styling` themes from images
//# Categories: color, styling, terminal, theming, tools
use std::error::Error;
use std::fs;
use thag_proc_macros::file_navigator;
use thag_styling::{
    cprtln, theme_to_toml, ImageThemeConfig, ImageThemeGenerator, Role, Styleable, StylingError,
    StylingResult, TermBgLuma, Theme,
};

file_navigator! {}

fn main() -> Result<(), Box<dyn Error>> {
    println!(
        "🖼️  {} - Image to Theme Generator",
        "thag_image_to_theme".info()
    );
    println!("{}", "=".repeat(60));
    println!();

    // Initialize file navigator
    let mut navigator = FileNavigator::new();

    // Configure supported image extensions
    let _image_extensions = vec![
        "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "webp", "PNG", "JPG", "JPEG", "GIF",
        "BMP", "TIFF", "TIF", "WEBP",
    ];

    println!("📁 Select an image file to analyze:");
    println!("   Supported formats: PNG, JPEG, GIF, BMP, TIFF, WebP");
    println!();

    // Use file navigator to select image file
    let Ok(image_path) = select_file(
        &mut navigator,
        Some("png,jpg,jpeg,gif,bmp,tiff,tif,webp"),
        false,
    ) else {
        println!("\n❌ No image file selected. Exiting.");
        return Ok(());
    };

    println!(
        "📷 Selected image: {}",
        image_path.display().to_string().success()
    );
    println!();

    // Get configuration from user
    let config = get_theme_config()?;

    // Generate theme from image
    println!("🎨 Analyzing image and generating theme...");
    let generator = ImageThemeGenerator::with_config(config.clone());

    let mut theme = match generator.generate_from_file(&image_path) {
        Ok(theme) => theme,
        Err(e) => {
            cprtln!(Role::Error, "❌ Failed to generate theme: {}", e);
            return Err(e.into());
        }
    };

    let image_name = image_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let theme_type_suffix = match theme.term_bg_luma {
        TermBgLuma::Light => "light",
        TermBgLuma::Dark => "dark",
        TermBgLuma::Undetermined => "",
    };

    // Set the theme name
    let default_name = format!(
        "thag-{}-{theme_type_suffix}",
        image_name.to_lowercase().replace(' ', "-"),
    );
    theme.name = Text::new("Enter theme name:")
        .with_default(&default_name)
        .prompt()?;

    // // Set the theme name
    // theme.name = config.theme_name_prefix.unwrap_or_else(|| {
    //     let base_name = image_path
    //         .file_stem()
    //         .and_then(|s| s.to_str())
    //         .unwrap_or("image");

    //     let theme_type_suffix = match theme.term_bg_luma {
    //         TermBgLuma::Light => "-light",
    //         TermBgLuma::Dark => "-dark",
    //         TermBgLuma::Undetermined => "",
    //     };

    //     format!(
    //         "thag-{}{}",
    //         base_name.to_lowercase().replace(' ', "-"),
    //         theme_type_suffix
    //     )
    // });

    // Display theme information
    display_theme_info(&theme);

    // Save theme to file
    save_theme_file(&theme, &mut navigator)?;

    println!("\n🎉 Theme generation completed successfully!");
    Ok(())
}

/// Get theme configuration from user input
fn get_theme_config() -> Result<ImageThemeConfig, Box<dyn Error>> {
    use inquire::{Confirm, CustomType, Select};

    // Theme type detection
    let theme_type_options = vec![
        "Auto-detect from image brightness",
        "Force light theme",
        "Force dark theme",
    ];

    let theme_type_choice = Select::new("Theme type:", theme_type_options).prompt()?;

    let (auto_detect, force_type) = match theme_type_choice {
        "Force light theme" => (false, Some(TermBgLuma::Light)),
        "Force dark theme" => (false, Some(TermBgLuma::Dark)),
        _ => (true, None),
    };

    // Color count
    let color_count: usize = CustomType::new("Number of colors to extract:")
        .with_default(16)
        .with_help_message("Recommended: 8-32 colors")
        .prompt()?;

    let color_count = color_count.clamp(8, 64);

    // Advanced options
    let show_advanced = Confirm::new("Configure advanced options?")
        .with_default(false)
        .prompt()?;

    let (light_threshold, saturation_threshold) = if show_advanced {
        let light_thresh: f32 = CustomType::new("Light threshold (0.0-1.0):")
            .with_default(0.7)
            .with_help_message("Higher values = more strict light theme detection")
            .prompt()?;

        let sat_thresh: f32 = CustomType::new("Saturation threshold (0.0-1.0):")
            .with_default(0.3)
            .with_help_message("Higher values = more saturated colors required")
            .prompt()?;

        (light_thresh.clamp(0.0, 1.0), sat_thresh.clamp(0.0, 1.0))
    } else {
        (0.7, 0.3)
    };

    Ok(ImageThemeConfig {
        color_count,
        light_threshold,
        saturation_threshold,
        auto_detect_theme_type: auto_detect,
        force_theme_type: force_type,
        theme_name_prefix: None,
    })
}

/// Display comprehensive theme information
fn display_theme_info(theme: &Theme) {
    println!("✅ {} generated successfully!", "Theme".success());
    println!();

    println!("📋 {} Information:", "Theme".info());
    println!("   Name: {}", theme.name.info());
    println!("   Description: {}", theme.description);
    println!("   Type: {:?}", theme.term_bg_luma);
    println!("   Color Support: {:?}", theme.min_color_support);
    println!("   Backgrounds: {:?}", theme.bg_rgbs);
    println!();

    println!("🎨 {} Preview:", "Color Palette".info());
    display_color_palette(theme);
}

/// Display color palette with visual preview
fn display_color_palette(theme: &Theme) {
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
        let styled_name = style.paint(format!("{:>12}", name));
        let rgb_info = extract_rgb_info(style);
        println!("   {styled_name} {rgb_info}");
    }

    // Show background colors
    if let Some((r, g, b)) = theme.bg_rgbs.first() {
        println!();
        print!("   Background: ");
        // Create background color preview
        for _ in 0..12 {
            print!("\x1b[48;2;{};{};{}m \x1b[0m", r, g, b);
        }
        println!(" RGB({}, {}, {})", r, g, b);
    }
    println!();
}

/// Extract RGB information from a style for display
fn extract_rgb_info(style: &thag_styling::Style) -> String {
    style.foreground.as_ref().map_or_else(
        || "No color".to_string(),
        |color_info| match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => {
                format!(
                    "#{:02x}{:02x}{:02x} RGB({}, {}, {})",
                    rgb[0], rgb[1], rgb[2], rgb[0], rgb[1], rgb[2]
                )
            }
            thag_styling::ColorValue::Color256 { color256 } => {
                format!("Color256({})", color256)
            }
            thag_styling::ColorValue::Basic { index, .. } => {
                format!("ANSI({})", index)
            }
        },
    )
}

/// Save theme to a TOML file using file navigator
fn save_theme_file(theme: &Theme, navigator: &mut FileNavigator) -> StylingResult<()> {
    use inquire::{Confirm, Text};

    // Ask if user wants to save the theme
    let should_save = Confirm::new("Save theme to file?")
        .with_default(true)
        .prompt()
        .map_err(|e| StylingError::FromStr(format!("Input error: {}", e)))?;

    if !should_save {
        println!("Theme not saved.");
        return Ok(());
    }

    // Get output directory
    println!("\n📁 Select directory to save theme file:");
    let Ok(output_dir) = select_directory(navigator, true) else {
        println!("❌ No directory selected. Theme not saved.");
        return Ok(());
    };

    // Get filename
    let default_filename = format!("{}.toml", theme.name);
    let filename = Text::new("Theme filename:")
        .with_default(&default_filename)
        .with_help_message("Will be saved with .toml extension")
        .prompt()
        .map_err(|e| StylingError::FromStr(format!("Input error: {}", e)))?;

    let filename = if std::path::Path::new(&filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
    {
        filename
    } else {
        format!("{}.toml", filename)
    };

    let output_path = output_dir.join(&filename);

    // Check if file exists
    if output_path.exists() {
        let overwrite = Confirm::new(&format!("File '{}' already exists. Overwrite?", filename))
            .with_default(false)
            .prompt()
            .map_err(|e| StylingError::FromStr(format!("Input error: {}", e)))?;

        if !overwrite {
            println!("Theme not saved.");
            return Ok(());
        }
    }

    // Generate TOML content
    let toml_content = theme_to_toml(theme)
        .map_err(|e| StylingError::FromStr(format!("TOML generation failed: {}", e)))?;

    // Write to file
    fs::write(&output_path, &toml_content)
        .map_err(|e| StylingError::FromStr(format!("Failed to write theme file: {}", e)))?;

    println!(
        "💾 Theme saved to: {}",
        output_path.display().to_string().success()
    );

    // Ask if user wants to view the TOML content
    let show_toml = Confirm::new("Display TOML content?")
        .with_default(false)
        .prompt()
        .map_err(|e| StylingError::FromStr(format!("Input error: {}", e)))?;

    if show_toml {
        println!();
        println!("📄 {} Content:", "TOML".info());
        println!("{}", "─".repeat(60));
        println!("{}", toml_content);
    }

    println!();
    println!("💡 {} To use this theme:", "Tip:".warning());
    println!("   • Copy to your thag themes directory");
    println!("   • Use with thag_gen_terminal_themes to export to terminal formats");
    println!("   • Reference in your thag configuration");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use thag_styling::{ColorSupport, Palette, TermBgLuma};

    fn create_test_theme() -> Theme {
        Theme {
            name: "Test Theme".to_string(),
            filename: PathBuf::from("test_theme.toml"),
            is_builtin: false,
            term_bg_luma: TermBgLuma::Dark,
            min_color_support: ColorSupport::TrueColor,
            palette: Palette::default(),
            backgrounds: vec!["#2a2a2a".to_string()],
            bg_rgbs: vec![(42, 42, 42)],
            description: "Test theme for unit tests".to_string(),
        }
    }

    #[test]
    fn test_extract_rgb_info() {
        use thag_styling::{ColorInfo, Style};

        let style = Style::fg(ColorInfo::rgb(255, 128, 64));
        let info = extract_rgb_info(&style);
        assert!(info.contains("ff8040"));
        assert!(info.contains("255, 128, 64"));
    }

    #[test]
    fn test_theme_creation() {
        let theme = create_test_theme();
        assert_eq!(theme.name, "Test Theme");
        assert!(!theme.bg_rgbs.is_empty());
    }

    #[test]
    fn test_image_theme_config_defaults() {
        let config = ImageThemeConfig::default();
        assert_eq!(config.color_count, 16);
        assert!(config.auto_detect_theme_type);
        assert!(config.force_theme_type.is_none());
    }
}
