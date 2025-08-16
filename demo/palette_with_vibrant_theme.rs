/*[toml]
[dependencies]
thag_styling = { path = "/Users/donf/projects/thag_rs/thag_styling" }
*/

//! Terminal Palette Display Tool with thag-vibrant-dark Theme
//!
//! This tool displays the current terminal's color palette alongside
//! the thag-vibrant-dark theme colors for direct comparison.

use std::fmt::Write;
use thag_styling::{ColorSupport, Style, TermAttributes, TermBgLuma, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Terminal Palette with thag-vibrant-dark Theme");
    println!("=================================================\n");

    // Load the thag-vibrant-dark theme explicitly
    let vibrant_theme = Theme::get_builtin("thag-vibrant-dark")?;

    // Display terminal information
    display_terminal_info();

    // Display ANSI color palette
    display_ansi_colors();

    // Display thag-vibrant-dark theme colors
    display_vibrant_theme_colors(&vibrant_theme);

    // Side-by-side comparison
    display_color_comparison(&vibrant_theme);

    println!("\n🎉 Palette display complete!");
    println!("💡 Use this to compare your terminal's colors with the thag theme.");

    Ok(())
}

/// Display basic terminal information
fn display_terminal_info() {
    println!("📟 Terminal Information:");
    println!("========================");

    // Try to get terminal attributes
    let term_attrs = TermAttributes::get_or_init();

    println!("🔍 Color Support: {:?}", term_attrs.color_support);
    println!("🌓 Background Luma: {:?}", term_attrs.term_bg_luma);

    // Display environment variables that affect colors
    if let Ok(term) = std::env::var("TERM") {
        println!("🖥️  TERM: {}", term);
    }
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        println!("🌈 COLORTERM: {}", colorterm);
    }

    println!();
}

/// Display the 16 basic ANSI colors
fn display_ansi_colors() {
    println!("🎨 Current Terminal ANSI Colors (0-15):");
    println!("========================================");

    // Basic colors (0-7)
    println!("Standard Colors (0-7):");
    display_color_row(&[
        (0, "Black"),
        (1, "Red"),
        (2, "Green"),
        (3, "Yellow"),
        (4, "Blue"),
        (5, "Magenta"),
        (6, "Cyan"),
        (7, "White"),
    ]);

    println!();

    // Bright colors (8-15)
    println!("Bright Colors (8-15):");
    display_color_row(&[
        (8, "Bright Black"),
        (9, "Bright Red"),
        (10, "Bright Green"),
        (11, "Bright Yellow"),
        (12, "Bright Blue"),
        (13, "Bright Magenta"),
        (14, "Bright Cyan"),
        (15, "Bright White"),
    ]);

    println!();
}

/// Display a row of colors with their indices and names
fn display_color_row(colors: &[(u8, &str)]) {
    // Print color indices
    print!("   ");
    for (index, _) in colors {
        print!("{:>12}", index);
    }
    println!();

    // Print color names
    print!("   ");
    for (_, name) in colors {
        print!("{:>12}", name);
    }
    println!();

    // Print colored blocks using ANSI escape codes
    print!("   ");
    for (index, _) in colors {
        print!("\x1b[48;5;{}m{:>12}\x1b[0m", index, "");
    }
    println!();

    // Print sample text in each color
    print!("   ");
    for (index, _) in colors {
        print!("\x1b[38;5;{}m{:>12}\x1b[0m", index, "Sample");
    }
    println!();
}

/// Display thag-vibrant-dark theme colors
fn display_vibrant_theme_colors(theme: &Theme) {
    println!("🌟 thag-vibrant-dark Theme Colors:");
    println!("===================================");

    println!("Theme: {}", theme.name);
    println!("Description: {}", theme.description);
    println!("Background: {:?}", theme.bg_rgbs);
    println!();

    // Display semantic colors
    let semantic_colors = [
        ("Normal", &theme.palette.normal),
        ("Error", &theme.palette.error),
        ("Warning", &theme.palette.warning),
        ("Success", &theme.palette.success),
        ("Info", &theme.palette.info),
        ("Code", &theme.palette.code),
        ("Emphasis", &theme.palette.emphasis),
        ("Subtle", &theme.palette.subtle),
        ("Heading1", &theme.palette.heading1),
        ("Heading2", &theme.palette.heading2),
        ("Heading3", &theme.palette.heading3),
    ];

    println!("Semantic Colors:");
    for (name, style) in semantic_colors {
        let colored_text = style.paint(format!("{:>12}", name));
        let rgb_info = extract_rgb_info(style);
        println!("   {} - {}", colored_text, rgb_info);
    }

    println!();

    // Show background color if available
    if let Some((r, g, b)) = theme.bg_rgbs.first() {
        println!("Background Color Preview:");
        print!("   ");
        for _ in 0..20 {
            print!("\x1b[48;2;{};{};{}m \x1b[0m", r, g, b);
        }
        println!(" RGB({}, {}, {})", r, g, b);
    }

    println!();
}

/// Display side-by-side color comparison showing current terminal vs expected thag colors
fn display_color_comparison(theme: &Theme) {
    println!("🔄 ANSI Color Mapping Comparison:");
    println!("==================================");

    // CORRECTED mappings that should match thag_sync_palette behavior
    let color_mappings = [
        ("Black (0)", 0, "Background", get_best_dark_color(theme)),
        ("Red (1)", 1, "Error", extract_rgb(&theme.palette.error)),
        (
            "Green (2)",
            2,
            "Success",
            extract_rgb(&theme.palette.success),
        ),
        (
            "Yellow (3)",
            3,
            "Warning",
            extract_rgb(&theme.palette.warning),
        ),
        ("Blue (4)", 4, "Info", extract_rgb(&theme.palette.info)),
        (
            "Magenta (5)",
            5,
            "Heading1",
            extract_rgb(&theme.palette.heading1),
        ), // CORRECTED: should be heading1 (purple)
        (
            "Cyan (6)",
            6,
            "Heading3",
            extract_rgb(&theme.palette.heading3),
        ), // CORRECTED: should be heading3 (cyan-blue)
        ("White (7)", 7, "Normal", extract_rgb(&theme.palette.normal)),
        (
            "Bright Black (8)",
            8,
            "Subtle",
            extract_rgb(&theme.palette.subtle),
        ),
        (
            "Bright White (15)",
            15,
            "Emphasis",
            extract_rgb(&theme.palette.emphasis),
        ),
    ];

    println!(
        "ANSI Color           Current Terminal      Expected (thag theme)           Semantic Role"
    );
    println!(
        "──────────────────────────────────────────────────────────────────────────────────────"
    );

    for (name, ansi_index, semantic_role, thag_rgb) in color_mappings {
        // Current terminal color (visual sample only)
        let terminal_sample = format!("\x1b[38;5;{}m████\x1b[0m", ansi_index);

        // Expected thag color with RGB info in both hex and decimal
        let thag_display = if let Some((r, g, b)) = thag_rgb {
            format!(
                "\x1b[38;2;{};{};{}m████\x1b[0m #{:02x}{:02x}{:02x} ({:3},{:3},{:3})",
                r, g, b, r, g, b, r, g, b
            )
        } else {
            "N/A".to_string()
        };

        println!(
            "{:16} {:17} {:31} {}",
            name, terminal_sample, thag_display, semantic_role
        );
    }

    println!();
    println!("💡 Notes:");
    println!("• Current Terminal shows what your terminal currently displays");
    println!("• Expected shows what it should look like with correct thag theme");
    println!(
        "• If ANSI 5 (Magenta) is wrong, it should be purple #{:02x}{:02x}{:02x} (Heading1)",
        extract_rgb(&theme.palette.heading1)
            .unwrap_or((172, 106, 205))
            .0,
        extract_rgb(&theme.palette.heading1)
            .unwrap_or((172, 106, 205))
            .1,
        extract_rgb(&theme.palette.heading1)
            .unwrap_or((172, 106, 205))
            .2
    );
    println!("• Use thag_sync_palette --apply thag-vibrant-dark to see correct colors");
}

/// Extract RGB values from a style for display
fn extract_rgb_info(style: &Style) -> String {
    match &style.foreground {
        Some(color_info) => match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => {
                format!("RGB({}, {}, {})", rgb[0], rgb[1], rgb[2])
            }
            thag_styling::ColorValue::Color256 { color256 } => {
                format!("256-Color({})", color256)
            }
            thag_styling::ColorValue::Basic { index, .. } => {
                format!("Basic({})", index)
            }
        },
        None => "No color".to_string(),
    }
}

/// Extract RGB tuple from a style
fn extract_rgb(style: &Style) -> Option<(u8, u8, u8)> {
    style
        .foreground
        .as_ref()
        .and_then(|color_info| match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            _ => None,
        })
}

/// Get the best dark color from the theme for black mapping
fn get_best_dark_color(theme: &Theme) -> Option<(u8, u8, u8)> {
    // Try background first, then subtle, then create a dark color
    theme
        .bg_rgbs
        .first()
        .copied()
        .or_else(|| extract_rgb(&theme.palette.subtle))
        .or_else(|| Some((16, 16, 16)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rgb_info() {
        let style = Style::fg(thag_styling::ColorInfo::rgb(255, 128, 64));
        let info = extract_rgb_info(&style);
        assert_eq!(info, "RGB(255, 128, 64)");
    }

    #[test]
    fn test_extract_rgb() {
        let style = Style::fg(thag_styling::ColorInfo::rgb(255, 128, 64));
        let rgb = extract_rgb(&style);
        assert_eq!(rgb, Some((255, 128, 64)));
    }
}
