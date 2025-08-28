//! `WezTerm` terminal theme exporter
//!
//! Exports thag themes to `WezTerm`'s TOML color scheme format.
//! `WezTerm` uses TOML files in the colors directory, not Lua for color schemes.

use crate::{
    exporters::{
        adjust_color_brightness, get_best_dark_color, get_rgb_from_style, is_light_color,
        ThemeExporter,
    },
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
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

        // Metadata section
        output.push_str("[metadata]\n");
        let _ = writeln!(output, "name = \"{}\"\n", theme.name);
        let _ = writeln!(output, "author = \"thag theme generator\"\n");
        let _ = writeln!(output, "origin_url = \"Generated from thag theme\"\n");
        let _ = writeln!(output, "wezterm_version = \"20240203-110809-5046fc22\"\n");
        if !theme.name.is_empty() {
            let _ = writeln!(output, "aliases = [\"{} (thag-generated)\"]\n", theme.name);
        }
        output.push('\n');

        // Colors section
        output.push_str("[colors]\n");

        // Background and foreground
        let _ = writeln!(
            output,
            "background = \"#{:02x}{:02x}{:02x}\" # Primary background\n",
            bg_color.0, bg_color.1, bg_color.2
        );

        if let Some(fg_color) = get_rgb_from_style(&theme.palette.normal) {
            let _ = writeln!(
                output,
                "foreground = \"#{:02x}{:02x}{:02x}\" # Primary foreground\n",
                fg_color.0, fg_color.1, fg_color.2
            );
        }

        output.push('\n');

        // ANSI colors array (0-7)
        output.push('\n');

        let normal_colors = [
            ("Black", get_best_dark_color(theme)),
            ("Red", get_rgb_from_style(&theme.palette.emphasis)),
            ("Green", get_rgb_from_style(&theme.palette.success)),
            ("Yellow", get_rgb_from_style(&theme.palette.commentary)),
            ("Blue", get_rgb_from_style(&theme.palette.info)),
            ("Magenta", get_rgb_from_style(&theme.palette.heading1)),
            ("Cyan", get_rgb_from_style(&theme.palette.code)),
            ("White", get_rgb_from_style(&theme.palette.normal)),
        ];

        for (name, rgb_opt) in normal_colors {
            if let Some((r, g, b)) = rgb_opt {
                let _ = writeln!(
                    output,
                    "    \"#{:02x}{:02x}{:02x}\", # {}\n",
                    r,
                    g,
                    b,
                    name.to_lowercase()
                );
            } else {
                // Fallback color
                let _ = writeln!(
                    output,
                    "    \"#808080\", # {} (fallback)\n",
                    name.to_lowercase()
                );
            }
        }

        output.push_str("]\n\n");

        // Bright colors (8-15)
        output.push_str("brights = [\n");

        let bright_colors = [
            ("Bright Black", get_rgb_from_style(&theme.palette.subtle)),
            ("Bright Red", get_rgb_from_style(&theme.palette.error)),
            ("Bright Green", get_rgb_from_style(&theme.palette.debug)),
            ("Bright Yellow", get_rgb_from_style(&theme.palette.warning)),
            ("Bright Blue", get_rgb_from_style(&theme.palette.link)),
            (
                "Bright Magenta",
                get_rgb_from_style(&theme.palette.heading2),
            ),
            ("Bright Cyan", get_rgb_from_style(&theme.palette.hint)),
            ("Bright White", get_rgb_from_style(&theme.palette.quote)),
        ];

        for (name, rgb_opt) in bright_colors {
            if let Some((r, g, b)) = rgb_opt {
                let _ = writeln!(
                    output,
                    "    \"#{:02x}{:02x}{:02x}\", # {}\n",
                    r,
                    g,
                    b,
                    name.to_lowercase()
                );
            } else {
                // Fallback color
                let _ = writeln!(
                    output,
                    "    \"#c0c0c0\", # {} (fallback)\n",
                    name.to_lowercase()
                );
            }
        }

        output.push_str("]\n\n");

        // Cursor colors
        if let Some(cursor_color) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            let _ = writeln!(
                output,
                "cursor_bg = \"#{:02x}{:02x}{:02x}\" # Cursor background\n",
                cursor_color.0, cursor_color.1, cursor_color.2
            );

            let _ = writeln!(
                output,
                "cursor_border = \"#{:02x}{:02x}{:02x}\" # Cursor border\n",
                cursor_color.0, cursor_color.1, cursor_color.2
            );

            // Cursor text should contrast with cursor color
            let cursor_fg = if is_light_color(cursor_color) {
                bg_color // Use background color for contrast
            } else {
                get_rgb_from_style(&theme.palette.normal).unwrap_or((255, 255, 255))
            };
            let _ = writeln!(
                output,
                "cursor_fg = \"#{:02x}{:02x}{:02x}\" # Cursor text\n",
                cursor_fg.0, cursor_fg.1, cursor_fg.2
            );
        }

        output.push('\n');

        // Selection colors
        let selection_bg = adjust_color_brightness(bg_color, 1.3);
        let _ = writeln!(
            output,
            "selection_bg = \"#{:02x}{:02x}{:02x}\" # Selection background\n",
            selection_bg.0, selection_bg.1, selection_bg.2
        );

        if let Some(selection_fg) = get_rgb_from_style(&theme.palette.normal) {
            let _ = writeln!(
                output,
                "selection_fg = \"#{:02x}{:02x}{:02x}\" # Selection foreground\n",
                selection_fg.0, selection_fg.1, selection_fg.2
            );
        }

        output.push('\n');

        // Indexed colors for 256-color support (optional but recommended)
        output.push_str("[colors.indexed]\n");

        // Add some common indexed colors based on the theme
        let indexed_colors = [
            (16, get_rgb_from_style(&theme.palette.warning)),
            (
                17,
                get_best_dark_color(theme).map(|c| adjust_color_brightness(c, 0.3)),
            ),
            (18, Some(adjust_color_brightness(bg_color, 1.1))),
            (19, Some(adjust_color_brightness(bg_color, 1.2))),
            (20, get_rgb_from_style(&theme.palette.subtle)),
            (21, get_rgb_from_style(&theme.palette.emphasis)),
        ];

        for (index, rgb_opt) in indexed_colors {
            if let Some((r, g, b)) = rgb_opt {
                let _ = writeln!(output, "{} = \"#{:02x}{:02x}{:02x}\"\n", index, r, g, b);
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
    use crate::{
        exporters::{basic_color_to_rgb, brighten_color, color_256_to_rgb, is_light_color},
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
        assert_eq!(color_256_to_rgb(0), (0, 0, 0));
        assert_eq!(color_256_to_rgb(15), (255, 255, 255));
        assert_eq!(basic_color_to_rgb(1), (128, 0, 0));

        assert_eq!(brighten_color((100, 100, 100)), (130, 130, 130));
    }

    #[test]
    fn test_color_brightness_detection() {
        assert!(is_light_color((255, 255, 255))); // White should be light
        assert!(!is_light_color((0, 0, 0))); // Black should be dark
        assert!(!is_light_color((64, 64, 64))); // Dark gray should be dark
    }
}
