//! Windows Terminal theme exporter
//!
//! Exports thag themes to Windows Terminal's JSON color scheme format.
//! Windows Terminal uses JSON configuration files for color schemes.

use crate::{exporters::ThemeExporter, ColorValue, StylingResult, Theme};
use serde_json::json;

/// Windows Terminal theme exporter
pub struct WindowsTerminalExporter;

impl ThemeExporter for WindowsTerminalExporter {
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

        // Build the Windows Terminal color scheme JSON
        let color_scheme = json!({
            "name": theme.name,
            "background": format_color(Some(bg_color)),
            "foreground": format_color(get_rgb_from_style(&theme.palette.normal)),

            // Cursor colors
            "cursorColor": format_color(
                get_rgb_from_style(&theme.palette.emphasis)
                    .or_else(|| get_rgb_from_style(&theme.palette.normal))
            ),

            // Selection colors
            "selectionBackground": format_color(Some(adjust_color_brightness(bg_color, 1.4))),

            // ANSI colors (0-7: normal, 8-15: bright)
            "black": format_color(get_best_dark_color(theme)),
            "red": format_color(get_rgb_from_style(&theme.palette.error)),
            "green": format_color(get_rgb_from_style(&theme.palette.success)),
            "yellow": format_color(get_rgb_from_style(&theme.palette.warning)),
            "blue": format_color(get_rgb_from_style(&theme.palette.info)),
            "purple": format_color(get_rgb_from_style(&theme.palette.heading1)),
            "cyan": format_color(
                get_rgb_from_style(&theme.palette.heading3).or_else(|| Some((64, 192, 192)))
            ),
            "white": format_color(get_rgb_from_style(&theme.palette.normal)),

            // Bright colors (8-15)
            "brightBlack": format_color(
                get_rgb_from_style(&theme.palette.subtle).or_else(|| Some((64, 64, 64)))
            ),
            "brightRed": format_color(
                get_rgb_from_style(&theme.palette.error).map(brighten_color)
            ),
            "brightGreen": format_color(
                get_rgb_from_style(&theme.palette.success).map(brighten_color)
            ),
            "brightYellow": format_color(
                get_rgb_from_style(&theme.palette.warning).map(brighten_color)
            ),
            "brightBlue": format_color(
                get_rgb_from_style(&theme.palette.info).map(brighten_color)
            ),
            "brightPurple": format_color(
                get_rgb_from_style(&theme.palette.code).map(brighten_color)
            ),
            "brightCyan": format_color(
                get_rgb_from_style(&theme.palette.info)
                    .map(brighten_color)
                    .or_else(|| Some((128, 255, 255)))
            ),
            "brightWhite": format_color(
                get_rgb_from_style(&theme.palette.emphasis)
                    .or_else(|| get_rgb_from_style(&theme.palette.emphasis))
                    // .map(brighten_color)
            )
        });

        // Create the complete schemes array structure that can be merged into settings.json
        let schemes_wrapper = json!({
            "schemes": [color_scheme]
        });

        // Convert to pretty-printed JSON
        serde_json::to_string_pretty(&schemes_wrapper)
            .map_err(|e| crate::StylingError::Generic(format!("JSON serialization error: {}", e)))
    }

    fn file_extension() -> &'static str {
        "json"
    }

    fn format_name() -> &'static str {
        "Windows Terminal"
    }
}

/// Format RGB color as hex string for Windows Terminal
fn format_color(rgb_opt: Option<(u8, u8, u8)>) -> String {
    let (r, g, b) = rgb_opt.unwrap_or((128, 128, 128));
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// Extract RGB values from a Style's foreground color
fn get_rgb_from_style(style: &crate::Style) -> Option<(u8, u8, u8)> {
    style.foreground.as_ref().and_then(|color_info| {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            ColorValue::Color256 { color256 } => {
                // Convert 256-color index to approximate RGB
                Some(color_256_to_rgb(*color256))
            }
            ColorValue::Basic { index, .. } => {
                // Convert basic color index to RGB
                Some(basic_color_to_rgb(*index))
            }
        }
    })
}

/// Convert 256-color index to RGB
fn color_256_to_rgb(index: u8) -> (u8, u8, u8) {
    match index {
        // Standard colors (0-15)
        0 => (0, 0, 0),        // Black
        1 => (128, 0, 0),      // Red
        2 => (0, 128, 0),      // Green
        3 => (128, 128, 0),    // Yellow
        4 => (0, 0, 128),      // Blue
        5 => (128, 0, 128),    // Magenta
        6 => (0, 128, 128),    // Cyan
        7 => (192, 192, 192),  // White
        8 => (128, 128, 128),  // Bright Black
        9 => (255, 0, 0),      // Bright Red
        10 => (0, 255, 0),     // Bright Green
        11 => (255, 255, 0),   // Bright Yellow
        12 => (0, 0, 255),     // Bright Blue
        13 => (255, 0, 255),   // Bright Magenta
        14 => (0, 255, 255),   // Bright Cyan
        15 => (255, 255, 255), // Bright White

        // 216 color cube (16-231)
        16..=231 => {
            let n = index - 16;
            let r = (n / 36) * 51;
            let g = ((n % 36) / 6) * 51;
            let b = (n % 6) * 51;
            (r, g, b)
        }

        // Grayscale (232-255)
        232..=255 => {
            let gray = 8 + (index - 232) * 10;
            (gray, gray, gray)
        }
    }
}

/// Convert basic color index to RGB
fn basic_color_to_rgb(index: u8) -> (u8, u8, u8) {
    match index {
        0 => (0, 0, 0),        // Black
        1 => (128, 0, 0),      // Red
        2 => (0, 128, 0),      // Green
        3 => (128, 128, 0),    // Yellow
        4 => (0, 0, 128),      // Blue
        5 => (128, 0, 128),    // Magenta
        6 => (0, 128, 128),    // Cyan
        7 => (192, 192, 192),  // White
        8 => (128, 128, 128),  // Bright Black
        9 => (255, 0, 0),      // Bright Red
        10 => (0, 255, 0),     // Bright Green
        11 => (255, 255, 0),   // Bright Yellow
        12 => (0, 0, 255),     // Bright Blue
        13 => (255, 0, 255),   // Bright Magenta
        14 => (0, 255, 255),   // Bright Cyan
        15 => (255, 255, 255), // Bright White
        _ => (128, 128, 128),  // Default gray
    }
}

/// Brighten a color by increasing its components
fn brighten_color((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    let factor = 1.3;
    (
        ((r as f32 * factor).min(255.0)) as u8,
        ((g as f32 * factor).min(255.0)) as u8,
        ((b as f32 * factor).min(255.0)) as u8,
    )
}

/// Adjust color brightness by a factor
fn adjust_color_brightness((r, g, b): (u8, u8, u8), factor: f32) -> (u8, u8, u8) {
    (
        ((r as f32 * factor).min(255.0).max(0.0)) as u8,
        ((g as f32 * factor).min(255.0).max(0.0)) as u8,
        ((b as f32 * factor).min(255.0).max(0.0)) as u8,
    )
}

/// Get the best dark color from the theme for black mapping
fn get_best_dark_color(theme: &Theme) -> Option<(u8, u8, u8)> {
    // Try background first, then subtle, then create a dark color
    theme
        .bg_rgbs
        .first()
        .copied()
        .or_else(|| get_rgb_from_style(&theme.palette.subtle))
        .or_else(|| Some((16, 16, 16)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColorSupport, Palette, TermBgLuma};
    use std::path::PathBuf;

    fn create_test_theme() -> Theme {
        Theme {
            name: "Test Theme".to_string(),
            filename: PathBuf::from("test.toml"),
            is_builtin: false,
            term_bg_luma: TermBgLuma::Dark,
            min_color_support: ColorSupport::TrueColor,
            palette: Palette::default(),
            backgrounds: vec!["#1e1e2e".to_string()],
            bg_rgbs: vec![(30, 30, 46)],
            description: "A test theme".to_string(),
        }
    }

    #[test]
    fn test_windows_terminal_export() {
        let theme = create_test_theme();
        let result = WindowsTerminalExporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Check that the content is valid JSON
        let parsed: Value = serde_json::from_str(&content).unwrap();

        // Check for required Windows Terminal structure
        assert!(parsed.get("schemes").is_some());
        let schemes = parsed["schemes"].as_array().unwrap();
        assert!(!schemes.is_empty());

        let scheme = &schemes[0];
        assert!(scheme.get("name").is_some());
        assert!(scheme.get("background").is_some());
        assert!(scheme.get("foreground").is_some());
        assert!(scheme.get("black").is_some());
        assert!(scheme.get("brightWhite").is_some());
        assert!(scheme.get("cursorColor").is_some());
    }

    #[test]
    fn test_color_formatting() {
        assert_eq!(format_color(Some((255, 128, 64))), "#FF8040");
        assert_eq!(format_color(Some((0, 0, 0))), "#000000");
        assert_eq!(format_color(None), "#808080");
    }

    #[test]
    fn test_color_conversions() {
        assert_eq!(color_256_to_rgb(0), (0, 0, 0));
        assert_eq!(color_256_to_rgb(15), (255, 255, 255));
        assert_eq!(basic_color_to_rgb(1), (128, 0, 0));

        assert_eq!(brighten_color((100, 100, 100)), (130, 130, 130));
    }
}
