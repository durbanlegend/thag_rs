//! Alacritty terminal theme exporter
//!
//! Exports thag themes to Alacritty's TOML color scheme format.
//! Alacritty uses a specific TOML structure that differs from other terminals.

use crate::{
    exporters::{
        adjust_color_brightness, dim_color, get_best_dark_color, get_rgb_from_style,
        is_light_color, ThemeExporter,
    },
    StylingResult, Theme,
};
use std::fmt::Write as _; // import without risk of name clashing

/// Alacritty theme exporter
pub struct AlacrittyExporter;

impl ThemeExporter for AlacrittyExporter {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::too_many_lines
    )]
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        let mut output = String::new();

        // Add header comment
        let _ = writeln!(
            output,
            "# Alacritty Color Scheme: {}\n# Generated from thag theme\n# {}\n\n",
            theme.name, theme.description
        );

        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

        // Start colors section
        output.push_str("[colors]\n\n");

        // Primary colors section
        output.push_str("[colors.primary]\n");
        let _ = writeln!(
            output,
            "background = \"#{:02x}{:02x}{:02x}\"\n",
            bg_color.0, bg_color.1, bg_color.2
        );

        // Use normal text color for foreground
        if let Some(fg_color) = get_rgb_from_style(&theme.palette.normal) {
            let _ = writeln!(
                output,
                "foreground = \"#{:02x}{:02x}{:02x}\"\n",
                fg_color.0, fg_color.1, fg_color.2
            );
        }

        // Bright foreground (use emphasis or fallback to normal)
        if let Some(bright_fg) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            let _ = writeln!(
                output,
                "bright_foreground = \"#{:02x}{:02x}{:02x}\"\n",
                bright_fg.0, bright_fg.1, bright_fg.2
            );
        }

        // Dim foreground (use subtle or fallback to normal with reduced brightness)
        if let Some(dim_fg) = get_rgb_from_style(&theme.palette.subtle).or_else(|| {
            get_rgb_from_style(&theme.palette.normal).map(|(r, g, b)| {
                // Reduce brightness by 30%
                (
                    (f32::from(r) * 0.7) as u8,
                    (f32::from(g) * 0.7) as u8,
                    (f32::from(b) * 0.7) as u8,
                )
            })
        }) {
            let _ = writeln!(
                output,
                "dim_foreground = \"#{:02x}{:02x}{:02x}\"\n",
                dim_fg.0, dim_fg.1, dim_fg.2
            );
        }

        output.push('\n');

        // Normal colors (0-7)
        output.push_str("[colors.normal]\n");

        // Map semantic colors to ANSI colors
        let normal_colors = [
            ("black", get_best_dark_color(theme)),
            ("red", get_rgb_from_style(&theme.palette.emphasis)),
            ("green", get_rgb_from_style(&theme.palette.success)),
            ("yellow", get_rgb_from_style(&theme.palette.commentary)),
            ("blue", get_rgb_from_style(&theme.palette.info)),
            ("magenta", get_rgb_from_style(&theme.palette.heading1)),
            ("cyan", get_rgb_from_style(&theme.palette.code)),
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

        output.push('\n');

        // Bright colors (8-15) - brighter/more saturated versions
        output.push_str("[colors.bright]\n");

        let bright_colors = [
            ("black", get_rgb_from_style(&theme.palette.subtle)),
            ("red", get_rgb_from_style(&theme.palette.error)),
            ("green", get_rgb_from_style(&theme.palette.debug)),
            ("yellow", get_rgb_from_style(&theme.palette.warning)),
            ("blue", get_rgb_from_style(&theme.palette.link)),
            ("magenta", get_rgb_from_style(&theme.palette.heading2)),
            ("cyan", get_rgb_from_style(&theme.palette.hint)),
            ("white", get_rgb_from_style(&theme.palette.quote)),
        ];

        eprintln!("Bright colors:");
        for (color_name, rgb_opt) in bright_colors {
            if let Some((r, g, b)) = rgb_opt {
                let color = &format!("{} = \"#{:02x}{:02x}{:02x}\"\n", color_name, r, g, b);
                eprintln!("color={color}");
                output.push_str(color);
            }
        }

        output.push('\n');

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
                    .or(Some((32, 96, 96))),
            ),
            (
                "white",
                get_rgb_from_style(&theme.palette.subtle)
                    .or_else(|| get_rgb_from_style(&theme.palette.normal).map(dim_color)),
            ),
        ];

        for (color_name, rgb_opt) in dim_colors {
            if let Some((r, g, b)) = rgb_opt {
                let _ = writeln!(
                    output,
                    "{} = \"#{:02x}{:02x}{:02x}\"\n",
                    color_name, r, g, b
                );
            }
        }

        output.push('\n');

        // Cursor colors
        output.push_str("[colors.cursor]\n");
        if let Some(cursor_color) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            let _ = writeln!(
                output,
                "cursor = \"#{:02x}{:02x}{:02x}\"\n",
                cursor_color.0, cursor_color.1, cursor_color.2
            );
            // Cursor text should contrast with cursor color
            let text_color = if is_light_color(cursor_color) {
                (0, 0, 0) // Black text on light cursor
            } else {
                (255, 255, 255) // White text on dark cursor
            };
            let _ = writeln!(
                output,
                "text = \"#{:02x}{:02x}{:02x}\"\n",
                text_color.0, text_color.1, text_color.2
            );
        }

        output.push('\n');

        // Selection colors
        output.push_str("[colors.selection]\n");

        // Use a slightly modified background color for selection
        let selection_bg = adjust_color_brightness(bg_color, 1.3);
        let _ = writeln!(
            output,
            "background = \"#{:02x}{:02x}{:02x}\"\n",
            selection_bg.0, selection_bg.1, selection_bg.2
        );

        if let Some(selection_fg) = get_rgb_from_style(&theme.palette.normal) {
            let _ = writeln!(
                output,
                "text = \"#{:02x}{:02x}{:02x}\"\n",
                selection_fg.0, selection_fg.1, selection_fg.2
            );
        }

        output.push('\n');

        // Search colors
        output.push_str("[colors.search]\n");
        output.push_str("[colors.search.matches]\n");

        if let Some(search_match) = get_rgb_from_style(&theme.palette.warning) {
            let _ = writeln!(
                output,
                "background = \"#{:02x}{:02x}{:02x}\"\n",
                search_match.0, search_match.1, search_match.2
            );
            let _ = writeln!(
                output,
                "foreground = \"#{:02x}{:02x}{:02x}\"\n",
                bg_color.0, bg_color.1, bg_color.2
            );
        }

        output.push_str("\n[colors.search.focused_match]\n");
        if let Some(focused_match) = get_rgb_from_style(&theme.palette.emphasis) {
            let _ = writeln!(
                output,
                "background = \"#{:02x}{:02x}{:02x}\"\n",
                focused_match.0, focused_match.1, focused_match.2
            );
            let _ = writeln!(
                output,
                "foreground = \"#{:02x}{:02x}{:02x}\"\n",
                bg_color.0, bg_color.1, bg_color.2
            );
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

// /// Extract RGB values from a Style's foreground color
// fn get_rgb_from_style(style: &crate::Style) -> Option<(u8, u8, u8)> {
//     style.foreground.as_ref().and_then(|color_info| {
//         eprintln!("Color value={:?}", color_info.value);
//         match &color_info.value {
//             ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
//             ColorValue::Color256 { color256 } => {
//                 // Convert 256-color index to approximate RGB
//                 Some(color_256_to_rgb(*color256))
//             }
//             ColorValue::Basic { index, .. } => {
//                 // Convert basic color index to RGB
//                 Some(basic_color_to_rgb(*index))
//             }
//         }
//     })
// }

#[cfg(test)]
mod tests {
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
