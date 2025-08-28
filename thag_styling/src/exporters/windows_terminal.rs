//! Windows Terminal theme exporter
//!
//! Exports thag themes to Windows Terminal's JSON color scheme format.
//! Windows Terminal uses JSON configuration files for color schemes.

use crate::{
    exporters::{adjust_color_brightness, get_best_dark_color, get_rgb_from_style, ThemeExporter},
    StylingResult, Theme,
};

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
            "red": format_color(get_rgb_from_style(&theme.palette.emphasis)),
            "green": format_color(get_rgb_from_style(&theme.palette.success)),
            "yellow": format_color(get_rgb_from_style(&theme.palette.commentary)),
            "blue": format_color(get_rgb_from_style(&theme.palette.info)),
            "purple": format_color(get_rgb_from_style(&theme.palette.heading1)),
            "cyan": format_color(
                get_rgb_from_style(&theme.palette.code)
            ),
            "white": format_color(get_rgb_from_style(&theme.palette.normal)),

            // Bright colors (8-15)
            "brightBlack": format_color(
                get_rgb_from_style(&theme.palette.subtle)
            ),
            "brightRed": format_color(
                get_rgb_from_style(&theme.palette.error)
            ),
            "brightGreen": format_color(
                get_rgb_from_style(&theme.palette.debug)
            ),
            "brightYellow": format_color(
                get_rgb_from_style(&theme.palette.warning)
            ),
            "brightBlue": format_color(
                get_rgb_from_style(&theme.palette.link)
            ),
            "brightPurple": format_color(
                get_rgb_from_style(&theme.palette.heading2)
            ),
            "brightCyan": format_color(
                get_rgb_from_style(&theme.palette.hint)
            ),
            "brightWhite": format_color(
                get_rgb_from_style(&theme.palette.quote)
            )
        });

        // Create the complete schemes array structure that can be merged into settings.json
        let schemes_wrapper = json!(color_scheme);

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

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;
    use crate::{
        exporters::{basic_color_to_rgb, brighten_color, color_256_to_rgb},
        ColorSupport, Palette, TermBgLuma,
    };
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
