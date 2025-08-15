//! iTerm2 terminal theme exporter
//!
//! Exports thag themes to iTerm2's JSON color scheme format.
//! iTerm2 uses JSON files for color presets that can be imported through the UI.

use crate::{exporters::ThemeExporter, ColorValue, StylingResult, Theme};
use serde_json::{json, Value};

/// iTerm2 theme exporter
pub struct ITerm2Exporter;

impl ThemeExporter for ITerm2Exporter {
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

        // Build the iTerm2 color scheme JSON
        let color_scheme = json!({
            "Ansi 0 Color": create_color_dict(get_best_dark_color(theme)),
            "Ansi 1 Color": create_color_dict(get_rgb_from_style(&theme.palette.error)),
            "Ansi 2 Color": create_color_dict(get_rgb_from_style(&theme.palette.success)),
            "Ansi 3 Color": create_color_dict(get_rgb_from_style(&theme.palette.warning)),
            "Ansi 4 Color": create_color_dict(get_rgb_from_style(&theme.palette.info)),
            "Ansi 5 Color": create_color_dict(get_rgb_from_style(&theme.palette.code)),
            "Ansi 6 Color": create_color_dict(
                get_rgb_from_style(&theme.palette.info).or_else(|| Some((64, 192, 192)))
            ),
            "Ansi 7 Color": create_color_dict(get_rgb_from_style(&theme.palette.normal)),
            "Ansi 8 Color": create_color_dict(
                get_rgb_from_style(&theme.palette.subtle).or_else(|| Some((64, 64, 64)))
            ),
            "Ansi 9 Color": create_color_dict(
                get_rgb_from_style(&theme.palette.error).map(brighten_color)
            ),
            "Ansi 10 Color": create_color_dict(
                get_rgb_from_style(&theme.palette.success).map(brighten_color)
            ),
            "Ansi 11 Color": create_color_dict(
                get_rgb_from_style(&theme.palette.warning).map(brighten_color)
            ),
            "Ansi 12 Color": create_color_dict(
                get_rgb_from_style(&theme.palette.info).map(brighten_color)
            ),
            "Ansi 13 Color": create_color_dict(
                get_rgb_from_style(&theme.palette.code).map(brighten_color)
            ),
            "Ansi 14 Color": create_color_dict(
                get_rgb_from_style(&theme.palette.info)
                    .map(brighten_color)
                    .or_else(|| Some((128, 255, 255)))
            ),
            "Ansi 15 Color": create_color_dict(
                get_rgb_from_style(&theme.palette.emphasis)
                    .or_else(|| get_rgb_from_style(&theme.palette.normal))
                    .map(brighten_color)
            ),

            // Background and foreground
            "Background Color": create_color_dict(Some(bg_color)),
            "Foreground Color": create_color_dict(get_rgb_from_style(&theme.palette.normal)),

            // Bold colors
            "Bold Color": create_color_dict(
                get_rgb_from_style(&theme.palette.emphasis)
                    .or_else(|| get_rgb_from_style(&theme.palette.normal))
            ),

            // Cursor colors
            "Cursor Color": create_color_dict(
                get_rgb_from_style(&theme.palette.emphasis)
                    .or_else(|| get_rgb_from_style(&theme.palette.normal))
            ),
            "Cursor Text Color": create_color_dict(Some(bg_color)),

            // Selection colors
            "Selection Color": create_color_dict(Some(adjust_color_brightness(bg_color, 1.4))),
            "Selected Text Color": create_color_dict(get_rgb_from_style(&theme.palette.normal)),

            // Link color
            "Link Color": create_color_dict(
                get_rgb_from_style(&theme.palette.info)
                    .or_else(|| Some((0, 122, 255)))
            ),

            // Badge color
            "Badge Color": create_color_dict(
                get_rgb_from_style(&theme.palette.warning)
                    .or_else(|| Some((255, 193, 7)))
            ),

            // Tab colors
            "Tab Color": create_color_dict(Some(adjust_color_brightness(bg_color, 0.9))),

            // Underline color
            "Underline Color": create_color_dict(
                get_rgb_from_style(&theme.palette.emphasis)
                    .or_else(|| get_rgb_from_style(&theme.palette.normal))
            ),

            // Guide color (for rulers, etc.)
            "Guide Color": create_color_dict(Some(adjust_color_brightness(bg_color, 1.2))),

            // Session name colors
            "Session Name Color": create_color_dict(get_rgb_from_style(&theme.palette.normal)),
        });

        // Convert to pretty-printed JSON
        serde_json::to_string_pretty(&color_scheme)
            .map_err(|e| crate::StylingError::Generic(format!("JSON serialization error: {}", e)))
    }

    fn file_extension() -> &'static str {
        "json"
    }

    fn format_name() -> &'static str {
        "iTerm2"
    }
}

/// Create an iTerm2 color dictionary from RGB values
fn create_color_dict(rgb_opt: Option<(u8, u8, u8)>) -> Value {
    let (r, g, b) = rgb_opt.unwrap_or((128, 128, 128));

    // iTerm2 uses normalized float values (0.0 - 1.0)
    json!({
        "Alpha Component": 1.0,
        "Blue Component": b as f64 / 255.0,
        "Color Space": "sRGB",
        "Green Component": g as f64 / 255.0,
        "Red Component": r as f64 / 255.0
    })
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
    fn test_iterm2_export() {
        let theme = create_test_theme();
        let result = ITerm2Exporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Check that the content is valid JSON
        let parsed: Value = serde_json::from_str(&content).unwrap();

        // Check for required iTerm2 color keys
        assert!(parsed.get("Background Color").is_some());
        assert!(parsed.get("Foreground Color").is_some());
        assert!(parsed.get("Ansi 0 Color").is_some());
        assert!(parsed.get("Ansi 15 Color").is_some());
        assert!(parsed.get("Cursor Color").is_some());
        assert!(parsed.get("Selection Color").is_some());
    }

    #[test]
    fn test_color_dict_creation() {
        let color_dict = create_color_dict(Some((255, 128, 64)));

        assert_eq!(color_dict["Alpha Component"], 1.0);
        assert_eq!(color_dict["Red Component"], 1.0);
        assert!(
            (color_dict["Green Component"].as_f64().unwrap() - 0.5019607843137255).abs() < 0.001
        );
        assert!(
            (color_dict["Blue Component"].as_f64().unwrap() - 0.25098039215686274).abs() < 0.001
        );
        assert_eq!(color_dict["Color Space"], "sRGB");
    }

    #[test]
    fn test_color_conversions() {
        assert_eq!(color_256_to_rgb(0), (0, 0, 0));
        assert_eq!(color_256_to_rgb(15), (255, 255, 255));
        assert_eq!(basic_color_to_rgb(1), (128, 0, 0));

        assert_eq!(brighten_color((100, 100, 100)), (130, 130, 130));
    }
}
