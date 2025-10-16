//! Alacritty terminal theme exporter
//!
//! Exports thag themes to Alacritty's TOML color scheme format.
//! Alacritty uses a specific TOML structure that differs from other terminals.

use crate::{
    exporters::{adjust_color_brightness, dim_color, is_light_color, ThemeExporter},
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
            "# Alacritty Color Scheme: {}\n# Generated from thag theme\n# {}",
            theme.name, theme.description
        );

        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or([0, 0, 0]);

        // Start colors section
        output.push_str("[colors]\n\n");

        // Primary colors section
        output.push_str("[colors.primary]\n");
        let [r, g, b] = bg_color;
        let _ = writeln!(output, r##"background = "#{r:02x}{g:02x}{b:02x}""##);

        // Use normal text color for foreground
        if let Some([r, g, b]) = &theme.palette.normal.rgb() {
            let _ = writeln!(output, r##"foreground = "#{:02x}{:02x}{:02x}""##, r, g, b);
        }

        // Bright foreground (use emphasis or fallback to normal)
        if let Some([r, g, b]) = &theme
            .palette
            .emphasis
            .rgb()
            .or_else(|| theme.palette.normal.rgb())
        {
            let _ = writeln!(
                output,
                r##"bright_foreground = "#{:02x}{:02x}{:02x}""##,
                r, g, b
            );
        }

        // Dim foreground (use subtle or fallback to normal with reduced brightness)
        if let Some([r, g, b]) = &theme.palette.subtle.rgb().or_else(|| {
            theme.palette.normal.rgb().map(|[r, g, b]| {
                // Reduce brightness by 30%
                [
                    (f32::from(r) * 0.7) as u8,
                    (f32::from(g) * 0.7) as u8,
                    (f32::from(b) * 0.7) as u8,
                ]
            })
        }) {
            let _ = writeln!(output, r##"dim_foreground = "#{r:02x}{g:02x}{b:02x}""##);
        }

        output.push('\n');

        // Normal colors (0-7)
        output.push_str("[colors.normal]\n");

        // Map semantic colors to ANSI colors
        let normal_colors = [
            ("black", Some(theme.bg_rgbs[0])),
            ("red", theme.palette.error.rgb()),
            ("green", theme.palette.success.rgb()),
            ("yellow", theme.palette.warning.rgb()),
            ("blue", theme.palette.info.rgb()),
            ("magenta", theme.palette.error.rgb()),
            ("cyan", theme.palette.code.rgb()),
            ("white", theme.palette.normal.rgb()),
        ];

        eprintln!("Normal colors:");
        for (color_name, rgb_opt) in normal_colors {
            if let Some([r, g, b]) = rgb_opt {
                let color = &format!(r##"{color_name} = "#{r:02x}{g:02x}{b:02x}""##);
                eprintln!("color={color}");
                output.push_str(color);
            }
        }

        output.push('\n');

        // Bright colors (8-15) - brighter/more saturated versions
        output.push_str("[colors.bright]\n");

        let bright_colors = [
            ("black", theme.palette.subtle.rgb()),
            ("red", theme.palette.error.rgb()),
            ("green", theme.palette.debug.rgb()),
            ("yellow", theme.palette.warning.rgb()),
            ("blue", theme.palette.link.rgb()),
            ("magenta", theme.palette.heading2.rgb()),
            ("cyan", theme.palette.hint.rgb()),
            ("white", theme.palette.quote.rgb()),
        ];

        eprintln!("Bright colors:");
        for (color_name, rgb_opt) in bright_colors {
            if let Some([r, g, b]) = rgb_opt {
                let color = &format!(r##"{color_name} = "#{r:02x}{g:02x}{b:02x}""##);
                eprintln!("color={color}");
                output.push_str(color);
            }
        }

        output.push('\n');

        // Dim colors (optional, for compatibility)
        output.push_str("[colors.dim]\n");

        let dim_colors = [
            ("black", Some([16, 16, 16])),
            ("red", theme.palette.error.rgb().map(dim_color)),
            ("green", theme.palette.success.rgb().map(dim_color)),
            ("yellow", theme.palette.warning.rgb().map(dim_color)),
            ("blue", theme.palette.info.rgb().map(dim_color)),
            ("magenta", theme.palette.error.rgb().map(dim_color)),
            ("cyan", theme.palette.code.rgb().map(dim_color)),
            (
                "white",
                theme
                    .palette
                    .subtle
                    .rgb()
                    .or_else(|| theme.palette.normal.rgb())
                    // .map(|[r, g, b]| (r, g, b))
                    .map(dim_color),
            ),
        ];

        for (color_name, rgb_opt) in dim_colors {
            if let Some([r, g, b]) = rgb_opt {
                let _ = writeln!(output, r##"{color_name} = "#{r:02x}{g:02x}{b:02x}""##);
            }
        }

        output.push('\n');

        // Cursor colors
        output.push_str("[colors.cursor]\n");
        if let Some(cursor_color) = theme
            .palette
            .emphasis
            .rgb()
            .or_else(|| theme.palette.normal.rgb())
        {
            let _ = writeln!(
                output,
                r##"cursor = "#{:02x}{:02x}{:02x}""##,
                cursor_color[0], cursor_color[1], cursor_color[2]
            );
            // Cursor text should contrast with cursor color
            let text_color = if is_light_color(cursor_color) {
                [0, 0, 0] // Black text on light cursor
            } else {
                [255, 255, 255] // White text on dark cursor
            };
            let _ = writeln!(
                output,
                r##"text = "#{:02x}{:02x}{:02x}""##,
                text_color[0], text_color[1], text_color[2]
            );
        }

        output.push('\n');

        // Selection colors
        output.push_str("[colors.selection]\n");

        // Use commentary color for better selection visibility
        let selection_bg: [u8; 3] = theme
            .palette
            .commentary
            .rgb()
            // .map(|[r, g, b]| (r, g, b))
            .unwrap_or_else(|| adjust_color_brightness(bg_color, 1.3));
        let _ = writeln!(
            output,
            "    background: \"#{:02x}{:02x}{:02x}\"",
            selection_bg[0], selection_bg[1], selection_bg[2]
        );

        if let Some([r, g, b]) = theme.palette.normal.rgb() {
            let _ = writeln!(output, r##"text = "#{r:02x}{g:02x}{b:02x}""##);
        }

        output.push('\n');

        // Search colors
        output.push_str("[colors.search]\n");
        output.push_str("[colors.search.matches]\n");

        if let Some([r, g, b]) = theme.palette.warning.rgb() {
            let _ = writeln!(output, r##"background = "#{r:02x}{g:02x}{b:02x}""##);
            let [fr, fg, fb] = bg_color;
            let _ = writeln!(output, r##"foreground = "#{fr:02x}{fg:02x}{fb:02x}""##);
        }

        output.push_str("\n[colors.search.focused_match]\n");
        if let Some([r, g, b]) = theme.palette.emphasis.rgb() {
            let _ = writeln!(output, r##"background = "#{r:02x}{g:02x}{b:02x}""##);
            let [fr, fg, fb] = bg_color;
            let _ = writeln!(output, r##"foreground = "#{fr:02x}{fg:02x}{fb:02x}""##);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporters::{
        basic_color_to_rgb, brighten_color, color_256_to_rgb, create_test_theme,
    };

    #[test]
    fn test_alacritty_export() {
        let theme = create_test_theme();
        let result = AlacrittyExporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();

        dbg!(&content);

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
        assert_eq!(color_256_to_rgb(0), [0, 0, 0]);
        assert_eq!(color_256_to_rgb(15), [255, 255, 255]);
        assert_eq!(basic_color_to_rgb(1), [128, 0, 0]);

        assert_eq!(brighten_color([100, 100, 100]), [130, 130, 130]);
        assert_eq!(dim_color([100, 100, 100]), [60, 60, 60]);
    }
}
