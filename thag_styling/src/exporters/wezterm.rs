//! WezTerm terminal theme exporter
//!
//! Exports thag themes to WezTerm's TOML color scheme format.
//! WezTerm uses TOML files in the colors directory, not Lua for color schemes.

use crate::{exporters::ThemeExporter, ColorValue, StylingResult, Theme};

/// WezTerm theme exporter
pub struct WezTermExporter;

impl ThemeExporter for WezTermExporter {
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        let mut output = String::new();

        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

        // Metadata section
        output.push_str("[metadata]\n");
        output.push_str(&format!("name = \"{}\"\n", theme.name));
        output.push_str(&format!("author = \"thag theme generator\"\n"));
        output.push_str(&format!("origin_url = \"Generated from thag theme\"\n"));
        output.push_str(&format!("wezterm_version = \"20240203-110809-5046fc22\"\n"));
        if !theme.name.is_empty() {
            output.push_str(&format!(
                "aliases = [\"{} (thag-generated)\"]\n",
                theme.name
            ));
        }
        output.push_str("\n");

        // Colors section
        output.push_str("[colors]\n");

        // Background and foreground
        output.push_str(&format!(
            "background = \"#{:02x}{:02x}{:02x}\" # Primary background\n",
            bg_color.0, bg_color.1, bg_color.2
        ));

        if let Some(fg_color) = get_rgb_from_style(&theme.palette.normal) {
            output.push_str(&format!(
                "foreground = \"#{:02x}{:02x}{:02x}\" # Primary foreground\n",
                fg_color.0, fg_color.1, fg_color.2
            ));
        }

        output.push_str("\n");

        // ANSI colors array (0-7)
        output.push_str("ansi = [\n");

        let normal_colors = [
            ("Black", get_best_dark_color(theme)),
            ("Red", get_rgb_from_style(&theme.palette.error)),
            ("Green", get_rgb_from_style(&theme.palette.success)),
            ("Yellow", get_rgb_from_style(&theme.palette.warning)),
            ("Blue", get_rgb_from_style(&theme.palette.info)),
            ("Magenta", get_rgb_from_style(&theme.palette.heading1)),
            (
                "Cyan",
                get_rgb_from_style(&theme.palette.heading3).or_else(|| Some((64, 192, 192))),
            ),
            ("White", get_rgb_from_style(&theme.palette.normal)),
        ];

        for (name, rgb_opt) in normal_colors {
            if let Some((r, g, b)) = rgb_opt {
                output.push_str(&format!(
                    "    \"#{:02x}{:02x}{:02x}\", # {}\n",
                    r,
                    g,
                    b,
                    name.to_lowercase()
                ));
            } else {
                // Fallback color
                output.push_str(&format!(
                    "    \"#808080\", # {} (fallback)\n",
                    name.to_lowercase()
                ));
            }
        }

        output.push_str("]\n\n");

        // Bright colors (8-15)
        output.push_str("brights = [\n");

        let bright_colors = [
            (
                "Bright Black",
                get_rgb_from_style(&theme.palette.subtle).or_else(|| Some((64, 64, 64))),
            ),
            (
                "Bright Red",
                get_rgb_from_style(&theme.palette.error).map(brighten_color),
            ),
            (
                "Bright Green",
                get_rgb_from_style(&theme.palette.success).map(brighten_color),
            ),
            (
                "Bright Yellow",
                get_rgb_from_style(&theme.palette.warning).map(brighten_color),
            ),
            (
                "Bright Blue",
                get_rgb_from_style(&theme.palette.info).map(brighten_color),
            ),
            (
                "Bright Magenta",
                get_rgb_from_style(&theme.palette.code).map(brighten_color),
            ),
            (
                "Bright Cyan",
                get_rgb_from_style(&theme.palette.info)
                    .map(brighten_color)
                    .or_else(|| Some((128, 255, 255))),
            ),
            (
                "Bright White",
                get_rgb_from_style(&theme.palette.emphasis)
                    .or_else(|| get_rgb_from_style(&theme.palette.emphasis)), // .map(brighten_color),
            ),
        ];

        for (name, rgb_opt) in bright_colors {
            if let Some((r, g, b)) = rgb_opt {
                output.push_str(&format!(
                    "    \"#{:02x}{:02x}{:02x}\", # {}\n",
                    r,
                    g,
                    b,
                    name.to_lowercase()
                ));
            } else {
                // Fallback color
                output.push_str(&format!(
                    "    \"#c0c0c0\", # {} (fallback)\n",
                    name.to_lowercase()
                ));
            }
        }

        output.push_str("]\n\n");

        // Cursor colors
        if let Some(cursor_color) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            output.push_str(&format!(
                "cursor_bg = \"#{:02x}{:02x}{:02x}\" # Cursor background\n",
                cursor_color.0, cursor_color.1, cursor_color.2
            ));

            output.push_str(&format!(
                "cursor_border = \"#{:02x}{:02x}{:02x}\" # Cursor border\n",
                cursor_color.0, cursor_color.1, cursor_color.2
            ));

            // Cursor text should contrast with cursor color
            let cursor_fg = if is_light_color(cursor_color) {
                bg_color // Use background color for contrast
            } else {
                get_rgb_from_style(&theme.palette.normal).unwrap_or((255, 255, 255))
            };
            output.push_str(&format!(
                "cursor_fg = \"#{:02x}{:02x}{:02x}\" # Cursor text\n",
                cursor_fg.0, cursor_fg.1, cursor_fg.2
            ));
        }

        output.push_str("\n");

        // Selection colors
        let selection_bg = adjust_color_brightness(bg_color, 1.3);
        output.push_str(&format!(
            "selection_bg = \"#{:02x}{:02x}{:02x}\" # Selection background\n",
            selection_bg.0, selection_bg.1, selection_bg.2
        ));

        if let Some(selection_fg) = get_rgb_from_style(&theme.palette.normal) {
            output.push_str(&format!(
                "selection_fg = \"#{:02x}{:02x}{:02x}\" # Selection foreground\n",
                selection_fg.0, selection_fg.1, selection_fg.2
            ));
        }

        output.push_str("\n");

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
                output.push_str(&format!("{} = \"#{:02x}{:02x}{:02x}\"\n", index, r, g, b));
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

/// Check if a color is considered light
fn is_light_color((r, g, b): (u8, u8, u8)) -> bool {
    // Calculate relative luminance
    let r_linear = if r <= 10 {
        r as f32 / 3294.6
    } else {
        ((r as f32 + 14.025) / 269.025).powf(2.4)
    };
    let g_linear = if g <= 10 {
        g as f32 / 3294.6
    } else {
        ((g as f32 + 14.025) / 269.025).powf(2.4)
    };
    let b_linear = if b <= 10 {
        b as f32 / 3294.6
    } else {
        ((b as f32 + 14.025) / 269.025).powf(2.4)
    };

    let luminance = 0.2126 * r_linear + 0.7152 * g_linear + 0.0722 * b_linear;
    luminance > 0.5
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
