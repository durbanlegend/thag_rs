/*[toml]
[dependencies]
thag_styling = { version = "0.2", thag-auto = true }
*/

//! Terminal Palette Display Tool
//!
//! This tool displays the current terminal's color palette, including:
//! - All 16 ANSI colors (0-15)
//! - Extended 256-color palette samples
//! - True color capability test
//! - Terminal background detection
//! - Current thag theme colors for comparison

use std::fmt::Write;
use thag_styling::{ColorSupport, Style, TermAttributes, TermBgLuma, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Terminal Palette Display Tool");
    println!("================================\n");

    // Display terminal information
    display_terminal_info();

    // Display ANSI color palette
    display_ansi_colors();

    // Display 256-color palette samples
    display_256_color_samples();

    // Test true color capability
    display_true_color_test();

    // Display current thag theme colors if available
    display_thag_theme_colors();

    println!("\n🎉 Palette display complete!");
    println!("💡 Use this to compare with your terminal emulator's color settings.");

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
    println!("🎨 ANSI Color Palette (0-15):");
    println!("==============================");

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

/// Display samples from the 256-color palette
fn display_256_color_samples() {
    println!("🌈 256-Color Palette Samples:");
    println!("==============================");

    // Color cube (16-231) - show a representative sample
    println!("Color Cube (16-231) - Representative Sample:");
    for row in 0..6 {
        print!("   ");
        for col in 0..6 {
            let index = 16 + row * 36 + col * 6;
            print!("\x1b[48;5;{}m {:3} \x1b[0m", index, index);
        }
        println!();
    }

    println!();

    // Grayscale (232-255)
    println!("Grayscale Ramp (232-255):");
    print!("   ");
    for i in 232..=255 {
        if (i - 232) % 12 == 0 && i > 232 {
            println!();
            print!("   ");
        }
        print!("\x1b[48;5;{}m {:3} \x1b[0m", i, i);
    }
    println!();
    println!();
}

/// Test true color capability with a gradient
fn display_true_color_test() {
    println!("🌟 True Color (24-bit) Test:");
    println!("=============================");

    // Red to blue gradient
    println!("Red → Blue Gradient:");
    print!("   ");
    for i in 0..32 {
        let red = 255 - (i * 8);
        let blue = i * 8;
        print!("\x1b[48;2;{};0;{}m \x1b[0m", red, blue);
    }
    println!();

    // Green gradient
    println!("Green Gradient:");
    print!("   ");
    for i in 0..32 {
        let green = i * 8;
        print!("\x1b[48;2;0;{};0m \x1b[0m", green);
    }
    println!();

    // Rainbow spectrum
    println!("Rainbow Spectrum:");
    print!("   ");
    for i in 0..32 {
        let hue = (i as f32 / 32.0) * 360.0;
        let (r, g, b) = hsl_to_rgb(hue, 1.0, 0.5);
        print!("\x1b[48;2;{};{};{}m \x1b[0m", r, g, b);
    }
    println!();
    println!();
}

/// Display current thag theme colors
fn display_thag_theme_colors() {
    println!("🎯 Current Thag Theme Colors:");
    println!("==============================");

    let term_attrs = TermAttributes::get_or_init();
    let theme = &term_attrs.theme;

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
}

/// Extract RGB information from a style for display
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

/// Convert HSL to RGB (simple implementation)
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r_prime + m) * 255.0) as u8,
        ((g_prime + m) * 255.0) as u8,
        ((b_prime + m) * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsl_to_rgb() {
        // Test pure red (hue = 0)
        let (r, g, b) = hsl_to_rgb(0.0, 1.0, 0.5);
        assert_eq!((r, g, b), (255, 0, 0));

        // Test pure green (hue = 120)
        let (r, g, b) = hsl_to_rgb(120.0, 1.0, 0.5);
        assert_eq!((r, g, b), (0, 255, 0));

        // Test pure blue (hue = 240)
        let (r, g, b) = hsl_to_rgb(240.0, 1.0, 0.5);
        assert_eq!((r, g, b), (0, 0, 255));
    }

    #[test]
    fn test_extract_rgb_info() {
        let style = Style::fg(thag_styling::ColorInfo::rgb(255, 128, 64));
        let info = extract_rgb_info(&style);
        assert_eq!(info, "RGB(255, 128, 64)");
    }
}
