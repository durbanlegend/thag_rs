/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

//! Alacritty Theme Installation Script
//!
//! This script properly installs the thag-vibrant-dark theme for Alacritty:
//! 1. Creates the correct theme file with fixed color mappings
//! 2. Installs it in the right location
//! 3. Updates or creates the Alacritty config with proper import syntax
//! 4. Provides verification steps

use std::fs;
use std::path::{Path, PathBuf};
use thag_styling::{ExportFormat, Theme, ThemeExporter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛠️  Alacritty Theme Installation Script");
    println!("========================================\n");

    // Load the theme
    let theme = Theme::get_builtin("thag-vibrant-dark")?;
    println!("📋 Installing theme: {}", theme.name);
    println!("📝 Description: {}", theme.description);

    // Get Alacritty config directory
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let alacritty_dir = home_dir.join(".config/alacritty");
    let themes_dir = alacritty_dir.join("themes");
    let config_file = alacritty_dir.join("alacritty.toml");

    println!("📁 Alacritty directory: {}", alacritty_dir.display());

    // Create directories if they don't exist
    fs::create_dir_all(&themes_dir)?;
    println!("✅ Created themes directory: {}", themes_dir.display());

    // Generate the corrected Alacritty theme with proper color mappings
    let theme_content = generate_corrected_alacritty_theme(&theme)?;

    // Save theme file
    let theme_filename = "thag_vibrant_dark.toml";
    let theme_path = themes_dir.join(theme_filename);
    fs::write(&theme_path, &theme_content)?;
    println!("✅ Saved theme file: {}", theme_path.display());

    // Update or create Alacritty config
    update_alacritty_config(&config_file, theme_filename)?;

    // Verification steps
    println!("\n🔍 Verification Steps:");
    println!("======================");
    println!("1. Restart Alacritty");
    println!("2. Check that the background is dark gray (#202020)");
    println!("3. Check that normal text is light green (#90e090)");
    println!("4. Run: alacritty --print-events (to check for config errors)");

    // Show the color mapping
    println!("\n🎨 Expected Color Mapping:");
    println!("==========================");
    show_color_mapping(&theme);

    println!("\n🎉 Installation complete!");
    println!("📄 Theme file: {}", theme_path.display());
    println!("⚙️  Config file: {}", config_file.display());

    Ok(())
}

/// Generate corrected Alacritty theme with proper color mappings
fn generate_corrected_alacritty_theme(theme: &Theme) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "# Alacritty Color Scheme: {}\n# Generated from thag theme with corrected mappings\n# {}\n\n",
        theme.name, theme.description
    ));

    let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

    // Start colors section
    output.push_str("[colors]\n\n");

    // Primary colors
    output.push_str("[colors.primary]\n");
    output.push_str(&format!(
        "background = \"#{:02x}{:02x}{:02x}\"\n",
        bg_color.0, bg_color.1, bg_color.2
    ));

    if let Some(fg_color) = extract_rgb(&theme.palette.normal) {
        output.push_str(&format!(
            "foreground = \"#{:02x}{:02x}{:02x}\"\n",
            fg_color.0, fg_color.1, fg_color.2
        ));
    }

    if let Some(bright_fg) = extract_rgb(&theme.palette.emphasis) {
        output.push_str(&format!(
            "bright_foreground = \"#{:02x}{:02x}{:02x}\"\n",
            bright_fg.0, bright_fg.1, bright_fg.2
        ));
    }

    if let Some(dim_fg) = extract_rgb(&theme.palette.subtle) {
        output.push_str(&format!(
            "dim_foreground = \"#{:02x}{:02x}{:02x}\"\n",
            dim_fg.0, dim_fg.1, dim_fg.2
        ));
    }

    output.push_str("\n");

    // Normal colors (0-7) - CORRECTED MAPPINGS
    output.push_str("[colors.normal]\n");

    // Fixed color mappings based on thag_sync_palette behavior
    let normal_colors = [
        ("black", Some(bg_color)),                         // 0: background
        ("red", extract_rgb(&theme.palette.error)),        // 1: error
        ("green", extract_rgb(&theme.palette.success)),    // 2: success
        ("yellow", extract_rgb(&theme.palette.warning)),   // 3: warning
        ("blue", extract_rgb(&theme.palette.info)),        // 4: info
        ("magenta", extract_rgb(&theme.palette.heading1)), // 5: heading1
        ("cyan", extract_rgb(&theme.palette.heading3)),    // 6: heading3
        ("white", extract_rgb(&theme.palette.normal)),     // 7: normal text
    ];

    for (color_name, rgb_opt) in normal_colors {
        if let Some((r, g, b)) = rgb_opt {
            output.push_str(&format!(
                "{} = \"#{:02x}{:02x}{:02x}\"\n",
                color_name, r, g, b
            ));
        }
    }

    output.push_str("\n");

    // Bright colors (8-15) - CORRECTED MAPPINGS
    output.push_str("[colors.bright]\n");

    let bright_colors = [
        ("black", extract_rgb(&theme.palette.subtle)), // 8: subtle
        ("red", extract_rgb(&theme.palette.trace)),    // 9: trace (bright red)
        ("green", extract_rgb(&theme.palette.debug)),  // 10: debug (bright green)
        ("yellow", extract_rgb(&theme.palette.heading3)), // 11: heading3 (bright yellow)
        ("blue", extract_rgb(&theme.palette.heading2)), // 12: heading2 (bright blue)
        ("magenta", extract_rgb(&theme.palette.heading1)), // 13: heading1 (bright magenta)
        ("cyan", extract_rgb(&theme.palette.hint)),    // 14: hint (bright cyan)
        (
            "white",
            // extract_rgb(&theme.palette.normal).map(brighten_color),
            extract_rgb(&theme.palette.emphasis),
        ), // 15: emphasis
    ];

    for (color_name, rgb_opt) in bright_colors {
        if let Some((r, g, b)) = rgb_opt {
            output.push_str(&format!(
                "{} = \"#{:02x}{:02x}{:02x}\"\n",
                color_name, r, g, b
            ));
        }
    }

    output.push_str("\n");

    // Cursor colors
    output.push_str("[colors.cursor]\n");
    if let Some(cursor_color) = extract_rgb(&theme.palette.emphasis) {
        output.push_str(&format!(
            "cursor = \"#{:02x}{:02x}{:02x}\"\n",
            cursor_color.0, cursor_color.1, cursor_color.2
        ));
        let cursor_text = if is_light_color(cursor_color) {
            (0, 0, 0)
        } else {
            (255, 255, 255)
        };
        output.push_str(&format!(
            "text = \"#{:02x}{:02x}{:02x}\"\n",
            cursor_text.0, cursor_text.1, cursor_text.2
        ));
    }

    output.push_str("\n");

    // Selection colors
    output.push_str("[colors.selection]\n");
    let selection_bg = adjust_brightness(bg_color, 1.4);
    output.push_str(&format!(
        "background = \"#{:02x}{:02x}{:02x}\"\n",
        selection_bg.0, selection_bg.1, selection_bg.2
    ));

    if let Some(selection_fg) = extract_rgb(&theme.palette.normal) {
        output.push_str(&format!(
            "text = \"#{:02x}{:02x}{:02x}\"\n",
            selection_fg.0, selection_fg.1, selection_fg.2
        ));
    }

    Ok(output)
}

/// Update or create Alacritty config file
fn update_alacritty_config(
    config_path: &Path,
    theme_filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let import_line = format!("import = [\"themes/{}\"]\n", theme_filename);

    if config_path.exists() {
        // Read existing config
        let existing_config = fs::read_to_string(config_path)?;

        // Check if import already exists
        if existing_config.contains("import = [") {
            println!("⚠️  Config already has import statements. Please manually add:");
            println!("   [general]");
            println!("   {}", import_line.trim());
        } else {
            // Add import to existing config
            let mut new_config = String::new();
            new_config.push_str("[general]\n");
            new_config.push_str(&import_line);
            new_config.push_str("\n");
            new_config.push_str(&existing_config);

            fs::write(config_path, new_config)?;
            println!("✅ Updated existing config file");
        }
    } else {
        // Create new config
        let new_config = format!(
            "# Alacritty Configuration\n# Generated by thag theme installer\n\n[general]\n{}\n",
            import_line
        );

        fs::write(config_path, new_config)?;
        println!("✅ Created new config file");
    }

    Ok(())
}

/// Show expected color mapping
fn show_color_mapping(theme: &Theme) {
    let mappings = [
        (
            "ANSI 0 (Black)",
            "Background",
            theme.bg_rgbs.first().copied(),
        ),
        ("ANSI 1 (Red)", "Error", extract_rgb(&theme.palette.error)),
        (
            "ANSI 2 (Green)",
            "Success",
            extract_rgb(&theme.palette.success),
        ),
        (
            "ANSI 3 (Yellow)",
            "Warning",
            extract_rgb(&theme.palette.warning),
        ),
        ("ANSI 4 (Blue)", "Code", extract_rgb(&theme.palette.code)),
        (
            "ANSI 5 (Magenta)",
            "Emphasis",
            extract_rgb(&theme.palette.emphasis),
        ),
        ("ANSI 6 (Cyan)", "Info", extract_rgb(&theme.palette.info)),
        (
            "ANSI 7 (White)",
            "Normal",
            extract_rgb(&theme.palette.normal),
        ),
        (
            "ANSI 8 (Bright Black)",
            "Subtle",
            extract_rgb(&theme.palette.subtle),
        ),
        (
            "ANSI 15 (Bright White)",
            "Emphasis",
            extract_rgb(&theme.palette.emphasis),
        ),
    ];

    for (ansi_name, semantic_name, rgb_opt) in mappings {
        if let Some((r, g, b)) = rgb_opt {
            println!(
                "   {} ← {} #{:02x}{:02x}{:02x}",
                ansi_name, semantic_name, r, g, b
            );
        }
    }
}

/// Extract RGB from style
fn extract_rgb(style: &thag_styling::Style) -> Option<(u8, u8, u8)> {
    style
        .foreground
        .as_ref()
        .and_then(|color_info| match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            _ => None,
        })
}

/// Brighten a color
fn brighten_color((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    let factor = 1.3;
    (
        ((r as f32 * factor).min(255.0)) as u8,
        ((g as f32 * factor).min(255.0)) as u8,
        ((b as f32 * factor).min(255.0)) as u8,
    )
}

/// Adjust brightness
fn adjust_brightness((r, g, b): (u8, u8, u8), factor: f32) -> (u8, u8, u8) {
    (
        ((r as f32 * factor).min(255.0).max(0.0)) as u8,
        ((g as f32 * factor).min(255.0).max(0.0)) as u8,
        ((b as f32 * factor).min(255.0).max(0.0)) as u8,
    )
}

/// Check if color is light
fn is_light_color((r, g, b): (u8, u8, u8)) -> bool {
    let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    luminance > 128.0
}
