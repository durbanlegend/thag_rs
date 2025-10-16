//! `WezTerm` terminal theme exporter
//!
//! Exports thag themes to `WezTerm`'s TOML color scheme format.
//! `WezTerm` uses TOML files in the colors directory, not Lua for color schemes.

use crate::{
    exporters::{adjust_color_brightness, is_light_color, ThemeExporter},
    StylingResult, Theme,
};
use std::fmt::Write as _; // import without risk of name clashing

/// `WezTerm` theme exporter
pub struct WezTermExporter;

impl ThemeExporter for WezTermExporter {
    #[allow(clippy::too_many_lines)]
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        let mut output = String::new();

        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or([0, 0, 0]);

        // Metadata section
        output.push_str("[metadata]\n");
        let _ = writeln!(output, r#"name = "{}""#, theme.name);
        let _ = writeln!(output, r#"author = "thag theme generator""#);
        let _ = writeln!(output, r#"origin_url = "Generated from thag theme""#);
        let _ = writeln!(output, r#"wezterm_version = "20240203-110809-5046fc22""#);
        if !theme.name.is_empty() {
            let _ = writeln!(output, r#"aliases = ["{} (thag-generated)"]"#, theme.name);
        }
        output.push('\n');

        // Colors section
        output.push_str("[colors]\n");

        // Background and foreground
        let [r, g, b] = bg_color;
        let _ = writeln!(
            output,
            r##"background = "#{r:02x}{g:02x}{b:02x}" # Primary background"##
        );

        if let Some([r, g, b]) = &theme.palette.normal.rgb() {
            let _ = writeln!(
                output,
                r##"foreground = "#{r:02x}{g:02x}{b:02x}" # Primary foreground"##
            );
        }

        output.push('\n');

        // ANSI colors array (0-7)
        output.push_str("ansi = [\n");

        let normal_colors: [(&str, Option<[u8; 3]>); 8] = [
            ("Black", Some(theme.bg_rgbs[0])),
            ("Red", theme.palette.emphasis.rgb()),
            ("Green", theme.palette.success.rgb()),
            ("Yellow", theme.palette.commentary.rgb()),
            ("Blue", theme.palette.info.rgb()),
            ("Magenta", theme.palette.heading1.rgb()),
            ("Cyan", theme.palette.code.rgb()),
            ("White", theme.palette.normal.rgb()),
        ];

        for (name, rgb_opt) in normal_colors {
            if let Some([r, g, b]) = rgb_opt {
                let _ = writeln!(
                    output,
                    r##"    "#{:02x}{:02x}{:02x}", # {}"##,
                    r,
                    g,
                    b,
                    name.to_lowercase()
                );
            } else {
                // Fallback color
                let _ = writeln!(
                    output,
                    r##"    "#808080", # {} (fallback)"##,
                    name.to_lowercase()
                );
            }
        }

        output.push_str("]\n\n");

        // Bright colors (8-15)
        output.push_str("brights = [\n");

        let bright_colors: [(&str, Option<[u8; 3]>); 8] = [
            ("Bright Black", theme.palette.subtle.rgb()),
            ("Bright Red", theme.palette.error.rgb()),
            ("Bright Green", theme.palette.debug.rgb()),
            ("Bright Yellow", theme.palette.warning.rgb()),
            ("Bright Blue", theme.palette.link.rgb()),
            ("Bright Magenta", theme.palette.heading2.rgb()),
            ("Bright Cyan", theme.palette.hint.rgb()),
            ("Bright White", theme.palette.quote.rgb()),
        ];

        for (name, rgb_opt) in bright_colors {
            if let Some([r, g, b]) = rgb_opt {
                let _ = writeln!(
                    output,
                    r##"    "#{:02x}{:02x}{:02x}", # {}"##,
                    r,
                    g,
                    b,
                    name.to_lowercase()
                );
            } else {
                // Fallback color
                let _ = writeln!(
                    output,
                    r##"    "#c0c0c0", # {} (fallback)"##,
                    name.to_lowercase()
                );
            }
        }

        output.push_str("]\n\n");

        // Cursor colors
        let cursor_color_opt = theme
            .palette
            .emphasis
            .rgb()
            .or_else(|| theme.palette.normal.rgb());

        if let Some(cursor_color) = cursor_color_opt {
            let [r, g, b] = cursor_color;
            let _ = writeln!(
                output,
                r##"cursor_bg = "#{r:02x}{g:02x}{b:02x}" # Cursor background"##
            );

            let _ = writeln!(
                output,
                r##"cursor_border = "#{r:02x}{g:02x}{b:02x}" # Cursor border"##
            );

            // Cursor text should contrast with cursor color
            let [r, g, b] = if is_light_color(cursor_color) {
                bg_color // Use background color for contrast
            } else {
                theme.palette.normal.rgb().unwrap_or([255, 255, 255])
            };
            let _ = writeln!(
                output,
                r##"cursor_fg = "#{r:02x}{g:02x}{b:02x}" # Cursor text"##
            );
        }

        output.push('\n');

        // Selection colors
        // Use commentary color for better visibility, fallback to brightness adjustment
        let [r, g, b] = theme
            .palette
            .commentary
            .rgb()
            .unwrap_or_else(|| adjust_color_brightness(bg_color, 1.3));
        let _ = writeln!(
            output,
            r##"selection_bg = "#{r:02x}{g:02x}{b:02x}" # Selection background"##,
        );

        if let Some([r, g, b]) = &theme.palette.normal.rgb() {
            let _ = writeln!(
                output,
                r##"selection_fg = "#{r:02x}{g:02x}{b:02x}" # Selection foreground"##
            );
        }

        output.push('\n');

        // Indexed colors for 256-color support (optional but recommended)
        output.push_str("[colors.indexed]\n");

        // Add some common indexed colors based on the theme
        let indexed_colors: [(u8, Option<[u8; 3]>); 6] = [
            (16, theme.palette.warning.rgb()),
            (17, Some(theme.bg_rgbs[0])),
            (18, Some(adjust_color_brightness(bg_color, 1.1))),
            (19, Some(adjust_color_brightness(bg_color, 1.2))),
            (20, theme.palette.subtle.rgb()),
            (21, theme.palette.emphasis.rgb()),
        ];

        for (index, rgb_opt) in indexed_colors {
            if let Some([r, g, b]) = rgb_opt {
                let _ = writeln!(output, r##"{} = "#{:02x}{:02x}{:02x}""##, index, r, g, b);
            }
        }

        Ok(output)
    }

    fn file_extension() -> &'static str {
        "toml"
    }

    fn format_name() -> &'static str {
        "WezTerm"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporters::{
        basic_color_to_rgb, brighten_color, color_256_to_rgb, create_test_theme, is_light_color,
    };

    #[test]
    fn test_wezterm_export() {
        let theme = create_test_theme();
        let result = WezTermExporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Check that the content contains expected TOML structure
        assert!(content.contains("[metadata]"));
        assert!(content.contains("[colors]"));
        assert!(content.contains("background ="));
        assert!(content.contains("foreground ="));
        assert!(content.contains("ansi ="));
        assert!(content.contains("brights ="));
        assert!(content.contains("cursor_bg ="));
        assert!(content.contains("selection_bg ="));
        assert!(content.contains("[colors.indexed]"));
    }

    #[test]
    fn test_color_conversions() {
        assert_eq!(color_256_to_rgb(0), [0, 0, 0]);
        assert_eq!(color_256_to_rgb(15), [255, 255, 255]);
        assert_eq!(basic_color_to_rgb(1), [128, 0, 0]);

        assert_eq!(brighten_color([100, 100, 100]), [130, 130, 130]);
    }

    #[test]
    fn test_color_brightness_detection() {
        assert!(is_light_color([255, 255, 255])); // White should be light
        assert!(!is_light_color([0, 0, 0])); // Black should be dark
        assert!(!is_light_color([64, 64, 64])); // Dark gray should be dark
    }
}
