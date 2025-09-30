/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["inquire_theming"] }
*/

/// Test terminal theme export functionality
///
/// This tool tests the terminal-specific theme exporters to verify that
/// selection backgrounds have sufficient contrast for visibility.
///
/// ## Usage
///
/// ```bash
/// # Test with a specific theme
/// thag test_terminal_export.rs -- --theme dracula
///
/// # Test with multiple themes
/// thag test_terminal_export.rs -- --theme atelier-seaside --theme nord
///
/// # Export to files for testing
/// thag test_terminal_export.rs -- --export --theme dracula
/// ```
use std::error::Error;
use thag_styling::{exporters::ExportFormat, Theme};

use clap::{Arg, Command};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("test_terminal_export")
        .about("Test terminal theme export functionality")
        .arg(
            Arg::new("theme")
                .long("theme")
                .value_name("NAME")
                .action(clap::ArgAction::Append)
                .help("Theme name to test (can be used multiple times)")
                .default_value("dracula"),
        )
        .arg(
            Arg::new("export")
                .long("export")
                .action(clap::ArgAction::SetTrue)
                .help("Export themes to files for manual testing"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_name("FORMAT")
                .help("Terminal format to test (iterm2, alacritty, kitty, wezterm)")
                .default_value("iterm2"),
        )
        .get_matches();

    let theme_names: Vec<_> = matches
        .get_many::<String>("theme")
        .unwrap_or_default()
        .cloned()
        .collect();
    let export_files = matches.get_flag("export");
    let format = matches.get_one::<String>("format").unwrap();

    println!("🧪 Terminal Export Tester");
    println!("═════════════════════════════════════════════════");
    println!("Testing selection background visibility across terminals\n");

    for theme_name in &theme_names {
        println!("🎨 Testing theme: {}", theme_name);
        println!("─────────────────────────────────────────────────");

        let theme = Theme::get_builtin(theme_name)?;
        analyze_theme_colors(&theme);

        match format.as_str() {
            "iterm2" => test_iterm2_export(&theme, export_files)?,
            "alacritty" => test_alacritty_export(&theme, export_files)?,
            "kitty" => test_kitty_export(&theme, export_files)?,
            "wezterm" => test_wezterm_export(&theme, export_files)?,
            _ => {
                println!("❌ Unknown format: {}", format);
                println!("   Supported: iterm2, alacritty, kitty, wezterm");
            }
        }
        println!();
    }

    if export_files {
        println!("💾 Theme files exported to current directory");
        println!("   Import them into your terminals to test selection visibility");
    } else {
        println!("💡 Use --export flag to save theme files for manual testing");
    }

    Ok(())
}

fn analyze_theme_colors(theme: &Theme) {
    let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

    println!("📋 Theme Analysis:");
    println!(
        "   Background: #{:02x}{:02x}{:02x} (RGB: {:?})",
        bg_color.0, bg_color.1, bg_color.2, bg_color
    );

    // Check if theme uses our commentary color (which should have good contrast)
    if let Some(commentary_rgb) = get_rgb_from_style(&theme.palette.commentary) {
        println!(
            "   Commentary: #{:02x}{:02x}{:02x} (RGB: {:?})",
            commentary_rgb.0, commentary_rgb.1, commentary_rgb.2, commentary_rgb
        );

        let contrast = calculate_contrast_ratio(bg_color, commentary_rgb);
        println!(
            "   Selection contrast ratio: {:.2}:1 {}",
            contrast,
            if contrast > 3.0 {
                "✅"
            } else {
                "⚠️  (may be hard to see)"
            }
        );
    } else {
        println!("   Commentary: ❌ Not available");

        // Test the brightness adjustment fallback
        let adjusted = adjust_color_brightness(bg_color, 1.4);
        println!(
            "   Fallback adjustment: #{:02x}{:02x}{:02x} (RGB: {:?})",
            adjusted.0, adjusted.1, adjusted.2, adjusted
        );

        let contrast = calculate_contrast_ratio(bg_color, adjusted);
        println!(
            "   Fallback contrast ratio: {:.2}:1 {}",
            contrast,
            if contrast > 3.0 {
                "✅"
            } else {
                "⚠️  (may be hard to see)"
            }
        );
    }
}

fn test_iterm2_export(theme: &Theme, export: bool) -> Result<(), Box<dyn Error>> {
    println!("🍎 Testing iTerm2 export...");

    let content = ExportFormat::ITerm2.export_theme(theme)?;

    // Look for selection color in the XML
    if content.contains("Selection Color") {
        println!("   ✅ Selection Color found in export");

        // Extract and display the selection color if possible
        if let Some(selection_section) = extract_selection_color_from_iterm(&content) {
            println!("   Selection color section:");
            for line in selection_section.lines().take(10) {
                println!("     {}", line.trim());
            }
        }
    } else {
        println!("   ❌ Selection Color not found in export");
    }

    if export {
        let filename = format!(
            "{}_iterm2.itermcolors",
            theme.name.replace(' ', "_").to_lowercase()
        );
        std::fs::write(&filename, content)?;
        println!("   💾 Exported to: {}", filename);
    }

    Ok(())
}

fn test_alacritty_export(theme: &Theme, export: bool) -> Result<(), Box<dyn Error>> {
    println!("⚡ Testing Alacritty export...");

    let content = ExportFormat::Alacritty.export_theme(theme)?;

    if content.contains("[colors.selection]") || content.contains("selection") {
        println!("   ✅ Selection colors found in export");
    } else {
        println!("   ❌ Selection colors not found in export");
    }

    if export {
        let filename = format!(
            "{}_alacritty.yml",
            theme.name.replace(' ', "_").to_lowercase()
        );
        std::fs::write(&filename, content)?;
        println!("   💾 Exported to: {}", filename);
    }

    Ok(())
}

fn test_kitty_export(theme: &Theme, export: bool) -> Result<(), Box<dyn Error>> {
    println!("🐱 Testing Kitty export...");

    let content = ExportFormat::Kitty.export_theme(theme)?;

    if content.contains("selection_background") {
        println!("   ✅ Selection background found in export");
    } else {
        println!("   ❌ Selection background not found in export");
    }

    if export {
        let filename = format!("{}_kitty.conf", theme.name.replace(' ', "_").to_lowercase());
        std::fs::write(&filename, content)?;
        println!("   💾 Exported to: {}", filename);
    }

    Ok(())
}

fn test_wezterm_export(theme: &Theme, export: bool) -> Result<(), Box<dyn Error>> {
    println!("🔧 Testing WezTerm export...");

    let content = ExportFormat::WezTerm.export_theme(theme)?;

    if content.contains("selection_bg") || content.contains("selection_fg") {
        println!("   ✅ Selection colors found in export");
    } else {
        println!("   ❌ Selection colors not found in export");
    }

    if export {
        let filename = format!(
            "{}_wezterm.toml",
            theme.name.replace(' ', "_").to_lowercase()
        );
        std::fs::write(&filename, content)?;
        println!("   💾 Exported to: {}", filename);
    }

    Ok(())
}

fn extract_selection_color_from_iterm(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.contains("Selection Color") {
            // Extract the next several lines that contain the color definition
            let start = i;
            let end = (i + 15).min(lines.len());
            return Some(lines[start..end].join("\n"));
        }
    }

    None
}

// Utility functions
fn get_rgb_from_style(style: &thag_styling::Style) -> Option<(u8, u8, u8)> {
    style
        .foreground
        .as_ref()
        .and_then(|color_info| match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            _ => None,
        })
}

fn adjust_color_brightness((r, g, b): (u8, u8, u8), factor: f32) -> (u8, u8, u8) {
    // For very dark colors, use additive brightening to ensure visibility
    if r < 50 && g < 50 && b < 50 && factor > 1.0 {
        // Add a minimum brightness boost for very dark backgrounds
        let min_boost = 80.0;
        (
            (f32::from(r) * factor + min_boost).clamp(0.0, 255.0) as u8,
            (f32::from(g) * factor + min_boost).clamp(0.0, 255.0) as u8,
            (f32::from(b) * factor + min_boost).clamp(0.0, 255.0) as u8,
        )
    } else {
        // Use multiplicative for normal colors
        (
            (f32::from(r) * factor).clamp(0.0, 255.0) as u8,
            (f32::from(g) * factor).clamp(0.0, 255.0) as u8,
            (f32::from(b) * factor).clamp(0.0, 255.0) as u8,
        )
    }
}

fn calculate_contrast_ratio((r1, g1, b1): (u8, u8, u8), (r2, g2, b2): (u8, u8, u8)) -> f32 {
    // Convert to relative luminance
    let lum1 = relative_luminance(r1, g1, b1);
    let lum2 = relative_luminance(r2, g2, b2);

    // Ensure lighter color is in numerator
    let (light, dark) = if lum1 > lum2 {
        (lum1, lum2)
    } else {
        (lum2, lum1)
    };

    (light + 0.05) / (dark + 0.05)
}

fn relative_luminance(r: u8, g: u8, b: u8) -> f32 {
    let rs = linearize_component(f32::from(r) / 255.0);
    let gs = linearize_component(f32::from(g) / 255.0);
    let bs = linearize_component(f32::from(b) / 255.0);

    0.2126 * rs + 0.7152 * gs + 0.0722 * bs
}

fn linearize_component(c: f32) -> f32 {
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
