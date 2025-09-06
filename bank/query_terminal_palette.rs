/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
crossterm = "0.28"
termbg = "0.6"
*/

//! Query Terminal Palette Colors
//!
//! This tool demonstrates querying the terminal's 16-color ANSI palette using OSC 4
//! escape sequences, similar to how termbg queries background color with OSC 11.
//! It includes both a demonstration of the OSC query concept and practical fallback
//! approaches for palette detection.
//!
//! The tool shows how to construct OSC 4 queries, parse responses, and compare
//! results with current thag theme colors.

//# Purpose: Demo terminal palette querying using OSC 4 sequences
//# Categories: terminal, styling, colors

use std::io;
use std::time::Duration;
use thag_styling::{ColorValue, Style, TermAttributes};

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Calculate color difference (Manhattan distance in RGB space)
    pub fn distance_to(&self, other: &Rgb) -> u16 {
        ((self.r as i16 - other.r as i16).abs()
            + (self.g as i16 - other.g as i16).abs()
            + (self.b as i16 - other.b as i16).abs()) as u16
    }
}

/// Error types for palette querying
#[derive(Debug)]
pub enum PaletteError {
    Io(io::Error),
    Timeout,
    ParseError(String),
    UnsupportedTerminal,
}

impl From<io::Error> for PaletteError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Parse OSC 4 response string to extract RGB values
fn parse_osc4_response(response: &str, expected_index: u8) -> Result<Option<Rgb>, PaletteError> {
    // Expected formats:
    // ESC]4;index;rgb:RRRR/GGGG/BBBB BEL
    // ESC]4;index;#RRGGBB BEL
    // ESC]4;index;rgb:RR/GG/BB BEL

    let index_pattern = format!("4;{};", expected_index);
    if let Some(start) = response.find(&index_pattern) {
        let response_part = &response[start + index_pattern.len()..];

        // Handle rgb: format
        if response_part.starts_with("rgb:") {
            let rgb_part = &response_part[4..];
            if let Some((r_str, rest)) = rgb_part.split_once('/') {
                if let Some((g_str, b_str)) = rest.split_once('/') {
                    // Clean up b_str (might have terminator)
                    let b_clean = b_str
                        .chars()
                        .take_while(|&c| c.is_ascii_hexdigit())
                        .collect::<String>();

                    let r = parse_hex_component(&r_str)?;
                    let g = parse_hex_component(&g_str)?;
                    let b = parse_hex_component(&b_clean)?;

                    return Ok(Some(Rgb::new(r, g, b)));
                }
            }
        }

        // Handle #RRGGBB format
        if let Some(hash_pos) = response_part.find('#') {
            let hex_part = &response_part[hash_pos + 1..];
            if hex_part.len() >= 6 {
                let hex_clean = hex_part
                    .chars()
                    .take(6)
                    .take_while(|&c| c.is_ascii_hexdigit())
                    .collect::<String>();

                if hex_clean.len() == 6 {
                    let r = u8::from_str_radix(&hex_clean[0..2], 16)
                        .map_err(|_| PaletteError::ParseError("Invalid red".to_string()))?;
                    let g = u8::from_str_radix(&hex_clean[2..4], 16)
                        .map_err(|_| PaletteError::ParseError("Invalid green".to_string()))?;
                    let b = u8::from_str_radix(&hex_clean[4..6], 16)
                        .map_err(|_| PaletteError::ParseError("Invalid blue".to_string()))?;

                    return Ok(Some(Rgb::new(r, g, b)));
                }
            }
        }
    }

    Ok(None)
}

/// Parse hex component (handles both 2 and 4 digit hex values)
fn parse_hex_component(hex_str: &str) -> Result<u8, PaletteError> {
    let clean_hex = hex_str
        .chars()
        .take_while(|&c| c.is_ascii_hexdigit())
        .collect::<String>();

    match clean_hex.len() {
        4 => {
            // 16-bit value, take the high byte
            let val = u16::from_str_radix(&clean_hex, 16)
                .map_err(|_| PaletteError::ParseError(format!("Invalid hex: {}", clean_hex)))?;
            Ok((val >> 8) as u8)
        }
        2 => {
            // 8-bit value
            u8::from_str_radix(&clean_hex, 16)
                .map_err(|_| PaletteError::ParseError(format!("Invalid hex: {}", clean_hex)))
        }
        _ => Err(PaletteError::ParseError(format!(
            "Invalid hex length: {}",
            clean_hex
        ))),
    }
}

/// Query a single palette color using OSC 4 (simplified demonstration)
pub fn query_palette_color_demo(index: u8) -> Result<Rgb, PaletteError> {
    println!("📤 Would send OSC 4 query: \\x1b]4;{};?\\x07", index);

    // In a real implementation, this would:
    // 1. Send the OSC sequence: format!("\x1b]4;{};?\x07", index)
    // 2. Read from terminal input (not stdin events)
    // 3. Parse response like: "\x1b]4;0;rgb:0000/0000/0000\x07"

    // For demonstration, provide realistic mock responses
    let demo_colors = [
        Rgb::new(40, 44, 52),    // 0: Black (One Dark background)
        Rgb::new(224, 108, 117), // 1: Red
        Rgb::new(152, 195, 121), // 2: Green
        Rgb::new(229, 192, 123), // 3: Yellow
        Rgb::new(97, 175, 239),  // 4: Blue
        Rgb::new(198, 120, 221), // 5: Magenta
        Rgb::new(86, 182, 194),  // 6: Cyan
        Rgb::new(171, 178, 191), // 7: White (foreground)
        Rgb::new(92, 99, 112),   // 8: Bright Black (comments)
        Rgb::new(224, 108, 117), // 9: Bright Red
        Rgb::new(152, 195, 121), // 10: Bright Green
        Rgb::new(229, 192, 123), // 11: Bright Yellow
        Rgb::new(97, 175, 239),  // 12: Bright Blue
        Rgb::new(198, 120, 221), // 13: Bright Magenta
        Rgb::new(86, 182, 194),  // 14: Bright Cyan
        Rgb::new(255, 255, 255), // 15: Bright White
    ];

    if (index as usize) < demo_colors.len() {
        let color = demo_colors[index as usize];
        println!(
            "📥 Mock response: \\x1b]4;{};rgb:{:02x}{:02x}/{:02x}{:02x}/{:02x}{:02x}\\x07",
            index,
            color.r,
            color.r, // Terminal might send 16-bit values
            color.g,
            color.g,
            color.b,
            color.b
        );
        Ok(color)
    } else {
        Err(PaletteError::Timeout)
    }
}

/// Display technical explanation of OSC 4 querying
fn explain_osc4_querying() {
    println!("🔍 Understanding OSC 4 Palette Querying");
    println!("========================================");
    println!();

    println!("OSC (Operating System Command) sequences allow applications to");
    println!("communicate with the terminal emulator. OSC 4 specifically deals");
    println!("with the color palette:");
    println!();

    println!("📤 Query format:  \\x1b]4;<index>;?\\x07");
    println!("   • \\x1b]      - OSC introducer");
    println!("   • 4          - Palette command");
    println!("   • <index>    - Color index (0-15 for ANSI)");
    println!("   • ?          - Query marker");
    println!("   • \\x07       - BEL terminator");
    println!();

    println!("📥 Response format: \\x1b]4;<index>;rgb:<r>/<g>/<b>\\x07");
    println!("   Example: \\x1b]4;1;rgb:ff00/0000/8000\\x07");
    println!("   (Color 1 = RGB(255, 0, 128))");
    println!();

    println!("🔧 Implementation challenges:");
    println!("   • Responses come via terminal input, not stdin events");
    println!("   • Need raw terminal access (not crossterm events)");
    println!("   • Timing sensitive - responses can be delayed");
    println!("   • Format variations between terminal emulators");
    println!("   • Some terminals don't support queries");
    println!();
}

/// Display palette colors with visual representation
fn display_palette_colors(colors: &[(u8, Result<Rgb, PaletteError>)]) {
    println!("🎨 Queried Terminal Palette Colors:");
    println!("===================================");

    let color_names = [
        "Black",
        "Red",
        "Green",
        "Yellow",
        "Blue",
        "Magenta",
        "Cyan",
        "White",
        "Bright Black",
        "Bright Red",
        "Bright Green",
        "Bright Yellow",
        "Bright Blue",
        "Bright Magenta",
        "Bright Cyan",
        "Bright White",
    ];

    // Display in two rows of 8
    for row in 0..2 {
        // Color indices
        print!("   ");
        for col in 0..8 {
            let index = row * 8 + col;
            print!("{:>12}", index);
        }
        println!();

        // Color names
        print!("   ");
        for col in 0..8 {
            let index = row * 8 + col;
            print!("{:>12}", color_names[index]);
        }
        println!();

        // RGB values
        print!("   ");
        for col in 0..8 {
            let index = row * 8 + col;
            let color_result = &colors[index].1;
            match color_result {
                Ok(rgb) => print!("{:>12}", format!("({},{},{})", rgb.r, rgb.g, rgb.b)),
                Err(_) => print!("{:>12}", "N/A"),
            }
        }
        println!();

        // Visual color blocks using ANSI codes
        print!("   ");
        for col in 0..8 {
            let index = row * 8 + col;
            if colors[index].1.is_ok() {
                print!("\x1b[48;5;{}m{:>12}\x1b[0m", index, "");
            } else {
                print!("{:>12}", "❌");
            }
        }
        println!();

        // Sample text in each color
        print!("   ");
        for col in 0..8 {
            let index = row * 8 + col;
            if colors[index].1.is_ok() {
                print!("\x1b[38;5;{}m{:>12}\x1b[0m", index, "Sample");
            } else {
                print!("{:>12}", "");
            }
        }
        println!();
        println!();
    }
}

/// Compare queried colors with current thag theme
fn compare_with_thag_theme(colors: &[(u8, Result<Rgb, PaletteError>)]) {
    println!("🔍 Comparison with Current Thag Theme:");
    println!("======================================");

    let term_attrs = TermAttributes::get_or_init();
    let theme = &term_attrs.theme;

    println!("Current theme: {}", theme.name);
    println!("Description: {}", theme.description);
    println!();

    // Map thag roles to ANSI colors (approximation)
    let role_mappings = [
        (
            0,
            "Background",
            theme.bg_rgbs.first().copied(),
            "Terminal background",
        ),
        (
            1,
            "Error",
            extract_rgb_from_style(&theme.palette.error),
            "Error messages",
        ),
        (
            2,
            "Success",
            extract_rgb_from_style(&theme.palette.success),
            "Success messages",
        ),
        (
            3,
            "Warning",
            extract_rgb_from_style(&theme.palette.warning),
            "Warnings",
        ),
        (
            4,
            "Info",
            extract_rgb_from_style(&theme.palette.info),
            "Information",
        ),
        (
            7,
            "Normal",
            extract_rgb_from_style(&theme.palette.normal),
            "Normal text",
        ),
        (
            8,
            "Subtle",
            extract_rgb_from_style(&theme.palette.subtle),
            "Subtle text",
        ),
    ];

    for (ansi_index, role_name, theme_rgb, description) in role_mappings {
        if let Some(queried_result) = colors.get(ansi_index).map(|(_, result)| result) {
            if let (Ok(queried_rgb), Some(theme_rgb)) = (queried_result, theme_rgb) {
                let theme_rgb_struct = Rgb::new(theme_rgb.0, theme_rgb.1, theme_rgb.2);
                let distance = queried_rgb.distance_to(&theme_rgb_struct);

                println!("Color {}: {} ({})", ansi_index, role_name, description);
                println!(
                    "  Queried: RGB({}, {}, {}) vs Theme: RGB({}, {}, {})",
                    queried_rgb.r,
                    queried_rgb.g,
                    queried_rgb.b,
                    theme_rgb.0,
                    theme_rgb.1,
                    theme_rgb.2
                );

                let status = if distance < 30 {
                    "✅ Very similar"
                } else if distance < 100 {
                    "⚠️  Somewhat different"
                } else {
                    "❌ Quite different"
                };
                println!("  {} (distance: {})", status, distance);
                println!();
            }
        }
    }
}

/// Extract RGB from a thag Style
fn extract_rgb_from_style(style: &Style) -> Option<(u8, u8, u8)> {
    style.foreground.as_ref().and_then(|color_info| {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            _ => None, // For simplicity, only handle true color
        }
    })
}

/// Demonstrate practical palette detection alternatives
fn demonstrate_alternatives() {
    println!("🛠️  Practical Alternatives for Palette Detection:");
    println!("================================================");
    println!();

    println!("1. Environment Variables:");
    if let Ok(term) = std::env::var("TERM") {
        println!("   TERM = {}", term);
    }
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        println!("   COLORTERM = {}", colorterm);
    }
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        println!("   TERM_PROGRAM = {}", term_program);
    }
    println!();

    println!("2. Background Detection (using termbg):");
    match termbg::theme(Duration::from_millis(100)) {
        Ok(theme) => println!("   Terminal theme: {:?}", theme),
        Err(e) => println!("   Could not detect theme: {:?}", e),
    }

    match termbg::rgb(Duration::from_millis(100)) {
        Ok(rgb) => println!("   Background RGB: ({}, {}, {})", rgb.r, rgb.g, rgb.b),
        Err(e) => println!("   Could not detect background: {:?}", e),
    }
    println!();

    println!("3. Color Support Detection:");
    let term_attrs = TermAttributes::get_or_init();
    println!("   Detected support: {:?}", term_attrs.color_support);
    println!("   Background luma: {:?}", term_attrs.term_bg_luma);
    println!();

    println!("4. Manual Terminal Detection:");
    let terminal_hints = [
        ("WEZTERM_EXECUTABLE", "WezTerm"),
        ("ALACRITTY_SOCKET", "Alacritty"),
        ("KITTY_WINDOW_ID", "Kitty"),
        ("ITERM_SESSION_ID", "iTerm2"),
        ("WT_SESSION", "Windows Terminal"),
    ];

    for (env_var, terminal_name) in terminal_hints {
        if std::env::var(env_var).is_ok() {
            println!("   Detected: {} ({})", terminal_name, env_var);
        }
    }
}

/// Test the parsing functions with sample data
fn test_parsing_functions() {
    println!("🧪 Testing OSC 4 Response Parsing:");
    println!("===================================");
    println!();

    let test_cases = [
        ("\x1b]4;1;rgb:ff00/8000/4000\x07", 1, "16-bit RGB format"),
        ("\x1b]4;0;rgb:00/00/00\x07", 0, "8-bit RGB format"),
        ("\x1b]4;15;#ffffff\x07", 15, "Hex format"),
        ("\x1b]4;8;rgb:8080/8080/8080\x07", 8, "Gray color"),
    ];

    for (response, expected_index, description) in test_cases {
        print!("Testing {}: ", description);
        match parse_osc4_response(response, expected_index) {
            Ok(Some(rgb)) => {
                println!("✅ RGB({}, {}, {})", rgb.r, rgb.g, rgb.b);
            }
            Ok(None) => println!("❌ No match found"),
            Err(e) => println!("❌ Parse error: {:?}", e),
        }
    }
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Terminal Palette Query Tool");
    println!("==============================");
    println!("This tool demonstrates OSC 4 palette querying concepts");
    println!("and provides practical alternatives for palette detection.");
    println!();

    // Explain the technical concept
    explain_osc4_querying();

    // Test parsing functions
    test_parsing_functions();

    // Demonstrate query process with mock data
    println!("🔄 Demonstrating Palette Query Process:");
    println!("=======================================");

    let mut colors = Vec::new();
    for i in 0..16 {
        let result = query_palette_color_demo(i);
        colors.push((i, result));
        std::thread::sleep(Duration::from_millis(50)); // Simulate timing
    }

    // Display results
    let successful = colors.iter().filter(|(_, result)| result.is_ok()).count();
    println!();
    println!(
        "Query simulation complete: {}/16 colors retrieved",
        successful
    );
    println!();

    if successful > 0 {
        display_palette_colors(&colors);
        compare_with_thag_theme(&colors);
    }

    // Show practical alternatives
    demonstrate_alternatives();

    println!("💡 Summary:");
    println!("   • OSC 4 querying is possible but technically challenging");
    println!("   • Requires raw terminal I/O, not event-based input");
    println!("   • Terminal support varies (most modern ones support it)");
    println!("   • Alternative detection methods are often more practical");
    println!("   • This demo shows the parsing logic for when responses are received");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_component() {
        assert_eq!(parse_hex_component("ff").unwrap(), 255);
        assert_eq!(parse_hex_component("00").unwrap(), 0);
        assert_eq!(parse_hex_component("80").unwrap(), 128);
        assert_eq!(parse_hex_component("ff00").unwrap(), 255); // 16-bit
        assert_eq!(parse_hex_component("8000").unwrap(), 128); // 16-bit
    }

    #[test]
    fn test_parse_osc4_response_rgb_format() {
        let response = "\x1b]4;1;rgb:ff00/8000/0000";
        let result = parse_osc4_response(response, 1).unwrap();
        assert_eq!(result, Some(Rgb::new(255, 128, 0)));
    }

    #[test]
    fn test_parse_osc4_response_hex_format() {
        let response = "\x1b]4;1;#ff8000";
        let result = parse_osc4_response(response, 1).unwrap();
        assert_eq!(result, Some(Rgb::new(255, 128, 0)));
    }

    #[test]
    fn test_rgb_distance() {
        let rgb1 = Rgb::new(255, 0, 0);
        let rgb2 = Rgb::new(255, 0, 0);
        assert_eq!(rgb1.distance_to(&rgb2), 0);

        let rgb3 = Rgb::new(0, 255, 0);
        assert_eq!(rgb1.distance_to(&rgb3), 510); // 255 + 255
    }
}
