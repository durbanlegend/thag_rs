//! Alacritty terminal theme exporter
//!
//! Exports thag themes to Alacritty's TOML color scheme format.
//! Alacritty uses a specific TOML structure that differs from other terminals.

use crate::{exporters::ThemeExporter, ColorValue, StylingResult, Theme};

/// Alacritty theme exporter
pub struct AlacrittyExporter;

impl ThemeExporter for AlacrittyExporter {
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!(
            "# Alacritty Color Scheme: {}\n# Generated from thag theme\n# {}\n\n",
            theme.name, theme.description
        ));

        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

        // Start colors section
        output.push_str("[colors]\n\n");

        // Primary colors section
        output.push_str("[colors.primary]\n");
        output.push_str(&format!(
            "background = \"#{:02x}{:02x}{:02x}\"\n",
            bg_color.0, bg_color.1, bg_color.2
        ));

        // Use normal text color for foreground
        if let Some(fg_color) = get_rgb_from_style(&theme.palette.normal) {
            output.push_str(&format!(
                "foreground = \"#{:02x}{:02x}{:02x}\"\n",
                fg_color.0, fg_color.1, fg_color.2
            ));
        }

        // Bright foreground (use emphasis or fallback to normal)
        if let Some(bright_fg) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            output.push_str(&format!(
                "bright_foreground = \"#{:02x}{:02x}{:02x}\"\n",
                bright_fg.0, bright_fg.1, bright_fg.2
            ));
        }

        // Dim foreground (use subtle or fallback to normal with reduced brightness)
        if let Some(dim_fg) = get_rgb_from_style(&theme.palette.subtle).or_else(|| {
            get_rgb_from_style(&theme.palette.normal).map(|(r, g, b)| {
                // Reduce brightness by 30%
                (
                    (r as f32 * 0.7) as u8,
                    (g as f32 * 0.7) as u8,
                    (b as f32 * 0.7) as u8,
                )
            })
        }) {
            output.push_str(&format!(
                "dim_foreground = \"#{:02x}{:02x}{:02x}\"\n",
                dim_fg.0, dim_fg.1, dim_fg.2
            ));
        }

        output.push_str("\n");

        // Normal colors (0-7)
        output.push_str("[colors.normal]\n");

        // Map semantic colors to ANSI colors
        let normal_colors = [
            ("black", get_best_dark_color(theme)),
            ("red", get_rgb_from_style(&theme.palette.error)),
            ("green", get_rgb_from_style(&theme.palette.success)),
            ("yellow", get_rgb_from_style(&theme.palette.warning)),
            ("blue", get_rgb_from_style(&theme.palette.info)),
            ("magenta", get_rgb_from_style(&theme.palette.heading1)),
            (
                "cyan",
                get_rgb_from_style(&theme.palette.heading3).or_else(|| Some((64, 192, 192))),
            ),
            ("white", get_rgb_from_style(&theme.palette.normal)),
        ];

        eprintln!("Normal colors:");
        for (color_name, rgb_opt) in normal_colors {
            if let Some((r, g, b)) = rgb_opt {
                let color = &format!("{} = \"#{:02x}{:02x}{:02x}\"\n", color_name, r, g, b);
                eprintln!("color={color}");
                output.push_str(color);
            }
        }

        output.push_str("\n");

        // Bright colors (8-15) - brighter/more saturated versions
        output.push_str("[colors.bright]\n");

        let bright_colors = [
            (
                "black",
                get_rgb_from_style(&theme.palette.subtle).or_else(|| Some((64, 64, 64))),
            ),
            (
                "red",
                get_rgb_from_style(&theme.palette.error).map(brighten_color),
            ),
            (
                "green",
                get_rgb_from_style(&theme.palette.success).map(brighten_color),
            ),
            (
                "yellow",
                get_rgb_from_style(&theme.palette.warning).map(brighten_color),
            ),
            (
                "blue",
                get_rgb_from_style(&theme.palette.info).map(brighten_color),
            ),
            (
                "magenta",
                get_rgb_from_style(&theme.palette.code).map(brighten_color),
            ),
            (
                "cyan",
                get_rgb_from_style(&theme.palette.info)
                    .map(brighten_color)
                    .or_else(|| Some((128, 255, 255))),
            ),
            (
                "white",
                get_rgb_from_style(&theme.palette.emphasis)
                    .or_else(|| get_rgb_from_style(&theme.palette.emphasis)), // .map(brighten_color),
            ),
        ];

        eprintln!("Bright colors:");
        for (color_name, rgb_opt) in bright_colors {
            if let Some((r, g, b)) = rgb_opt {
                let color = &format!("{} = \"#{:02x}{:02x}{:02x}\"\n", color_name, r, g, b);
                eprintln!("color={color}");
                output.push_str(color);
            }
        }

        output.push_str("\n");

        // Dim colors (optional, for compatibility)
        output.push_str("[colors.dim]\n");

        let dim_colors = [
            ("black", Some((16, 16, 16))),
            (
                "red",
                get_rgb_from_style(&theme.palette.error).map(dim_color),
            ),
            (
                "green",
                get_rgb_from_style(&theme.palette.success).map(dim_color),
            ),
            (
                "yellow",
                get_rgb_from_style(&theme.palette.warning).map(dim_color),
            ),
            (
                "blue",
                get_rgb_from_style(&theme.palette.info).map(dim_color),
            ),
            (
                "magenta",
                get_rgb_from_style(&theme.palette.code).map(dim_color),
            ),
            (
                "cyan",
                get_rgb_from_style(&theme.palette.info)
                    .map(dim_color)
                    .or_else(|| Some((32, 96, 96))),
            ),
            (
                "white",
                get_rgb_from_style(&theme.palette.subtle)
                    .or_else(|| get_rgb_from_style(&theme.palette.normal).map(dim_color)),
            ),
        ];

        for (color_name, rgb_opt) in dim_colors {
            if let Some((r, g, b)) = rgb_opt {
                output.push_str(&format!(
                    "{} = \"#{:02x}{:02x}{:02x}\"\n",
                    color_name, r, g, b
                ));
            }
        }

        output.push_str("\n");

        // Cursor colors
        output.push_str("[colors.cursor]\n");
        if let Some(cursor_color) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            output.push_str(&format!(
                "cursor = \"#{:02x}{:02x}{:02x}\"\n",
                cursor_color.0, cursor_color.1, cursor_color.2
            ));
            // Cursor text should contrast with cursor color
            let text_color = if is_light_color(cursor_color) {
                (0, 0, 0) // Black text on light cursor
            } else {
                (255, 255, 255) // White text on dark cursor
            };
            output.push_str(&format!(
                "text = \"#{:02x}{:02x}{:02x}\"\n",
                text_color.0, text_color.1, text_color.2
            ));
        }

        output.push_str("\n");

        // Selection colors
        output.push_str("[colors.selection]\n");

        // Use a slightly modified background color for selection
        let selection_bg = adjust_color_brightness(bg_color, 1.3);
        output.push_str(&format!(
            "background = \"#{:02x}{:02x}{:02x}\"\n",
            selection_bg.0, selection_bg.1, selection_bg.2
        ));

        if let Some(selection_fg) = get_rgb_from_style(&theme.palette.normal) {
            output.push_str(&format!(
                "text = \"#{:02x}{:02x}{:02x}\"\n",
                selection_fg.0, selection_fg.1, selection_fg.2
            ));
        }

        output.push_str("\n");

        // Search colors
        output.push_str("[colors.search]\n");
        output.push_str("[colors.search.matches]\n");

        if let Some(search_match) = get_rgb_from_style(&theme.palette.warning) {
            output.push_str(&format!(
                "background = \"#{:02x}{:02x}{:02x}\"\n",
                search_match.0, search_match.1, search_match.2
            ));
            output.push_str(&format!(
                "foreground = \"#{:02x}{:02x}{:02x}\"\n",
                bg_color.0, bg_color.1, bg_color.2
            ));
        }

        output.push_str("\n[colors.search.focused_match]\n");
        if let Some(focused_match) = get_rgb_from_style(&theme.palette.emphasis) {
            output.push_str(&format!(
                "background = \"#{:02x}{:02x}{:02x}\"\n",
                focused_match.0, focused_match.1, focused_match.2
            ));
            output.push_str(&format!(
                "foreground = \"#{:02x}{:02x}{:02x}\"\n",
                bg_color.0, bg_color.1, bg_color.2
            ));
        }

        Ok(output)
    }

    fn file_extension() -> &'static str {
        "toml"
    }

    fn format_name() -> &'static str {
        "Alacritty"
    }
}

/// Extract RGB values from a Style's foreground color
fn get_rgb_from_style(style: &crate::Style) -> Option<(u8, u8, u8)> {
    style.foreground.as_ref().and_then(|color_info| {
        eprintln!("Color value={:?}", color_info.value);
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

/// Dim a color by reducing its components
fn dim_color((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    let factor = 0.6;
    (
        (r as f32 * factor) as u8,
        (g as f32 * factor) as u8,
        (b as f32 * factor) as u8,
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
    fn test_alacritty_export() {
        let theme = create_test_theme();
        let result = AlacrittyExporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Check that the content contains expected sections
        assert!(content.contains("[colors]"));
        assert!(content.contains("[colors.primary]"));
        assert!(content.contains("[colors.normal]"));
        assert!(content.contains("[colors.bright]"));
        assert!(content.contains("background ="));
        assert!(content.contains("foreground ="));
    }

    #[test]
    fn test_color_conversions() {
        assert_eq!(color_256_to_rgb(0), (0, 0, 0));
        assert_eq!(color_256_to_rgb(15), (255, 255, 255));
        assert_eq!(basic_color_to_rgb(1), (128, 0, 0));

        assert_eq!(brighten_color((100, 100, 100)), (130, 130, 130));
        assert_eq!(dim_color((100, 100, 100)), (60, 60, 60));
    }
}
