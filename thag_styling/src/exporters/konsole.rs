//! Konsole terminal theme exporter
//!
//! Exports thag themes to KDE Konsole's .colorscheme format.
//! Konsole uses an INI-style configuration format with RGB color values.

use crate::{
    exporters::{adjust_color_brightness, dim_color, ThemeExporter},
    StylingResult, Theme,
};
use std::fmt::Write as _; // import without risk of name clashing

/// Konsole theme exporter
pub struct KonsoleExporter;

impl ThemeExporter for KonsoleExporter {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::too_many_lines
    )]
    #[allow(clippy::similar_names)]
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        let mut output = String::new();

        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

        // General section
        let _ = writeln!(output, "[General]");
        let _ = writeln!(output, "Description={}", theme.name);
        let _ = writeln!(output, "Opacity=1");
        output.push('\n');

        // Background colors
        let _ = writeln!(output, "[Background]");
        let _ = writeln!(output, "Color={},{},{}", bg_color.0, bg_color.1, bg_color.2);
        output.push('\n');

        let _ = writeln!(output, "[BackgroundIntense]");
        let intense_bg = adjust_color_brightness(bg_color, 1.2);
        let _ = writeln!(
            output,
            "Color={},{},{}",
            intense_bg.0, intense_bg.1, intense_bg.2
        );
        output.push('\n');

        let _ = writeln!(output, "[BackgroundFaint]");
        let faint_bg = adjust_color_brightness(bg_color, 0.8);
        let _ = writeln!(output, "Color={},{},{}", faint_bg.0, faint_bg.1, faint_bg.2);
        output.push('\n');

        // Foreground colors
        let fg_color = theme.palette.normal.rgb().unwrap_or([255, 255, 255]);

        let _ = writeln!(output, "[Foreground]");
        let _ = writeln!(
            output,
            "Color={},{},{}",
            fg_color[0], fg_color[1], fg_color[2]
        );
        output.push('\n');

        let _ = writeln!(output, "[ForegroundIntense]");
        let intense_fg = theme.palette.emphasis.rgb().unwrap_or(fg_color);
        let _ = writeln!(output, "Bold=true");
        let _ = writeln!(
            output,
            "Color={},{},{}",
            intense_fg[0], intense_fg[1], intense_fg[2]
        );
        output.push('\n');

        let _ = writeln!(output, "[ForegroundFaint]");
        let faint_fg = theme
            .palette
            .subtle
            .rgb()
            .unwrap_or_else(|| dim_color((fg_color[0], fg_color[1], fg_color[2])).into());
        let _ = writeln!(
            output,
            "Color={},{},{}",
            faint_fg[0], faint_fg[1], faint_fg[2]
        );
        output.push('\n');

        // ANSI Colors 0-7 (normal colors)
        let normal_colors = [
            // Color 0 (black) - use background
            bg_color,
            // Color 1 (red) - use error or emphasis
            theme
                .palette
                .error
                .rgb()
                .or_else(|| theme.palette.emphasis.rgb())
                .map(|arr| (arr[0], arr[1], arr[2]))
                .unwrap_or((220, 50, 47)),
            // Color 2 (green) - use success
            theme
                .palette
                .success
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2]))
                .unwrap_or((133, 153, 0)),
            // Color 3 (yellow) - use warning or commentary
            theme
                .palette
                .warning
                .rgb()
                .or_else(|| theme.palette.commentary.rgb())
                .map(|arr| (arr[0], arr[1], arr[2]))
                .unwrap_or((181, 137, 0)),
            // Color 4 (blue) - use info
            theme
                .palette
                .info
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2]))
                .unwrap_or((38, 139, 210)),
            // Color 5 (magenta) - use heading1
            theme
                .palette
                .heading1
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2]))
                .unwrap_or((211, 54, 130)),
            // Color 6 (cyan) - use code or hint
            theme
                .palette
                .code
                .rgb()
                .or_else(|| theme.palette.hint.rgb())
                .map(|arr| (arr[0], arr[1], arr[2]))
                .unwrap_or((42, 161, 152)),
            // Color 7 (white) - use normal foreground
            (fg_color[0], fg_color[1], fg_color[2]),
        ];

        for (i, (r, g, b)) in normal_colors.iter().enumerate() {
            let _ = writeln!(output, "[Color{}]", i);
            let _ = writeln!(output, "Color={},{},{}", r, g, b);
            output.push('\n');
        }

        // ANSI Colors 0-7 Intense (bright colors 8-15)
        let bright_colors = [
            // Color 0 intense (bright black) - use subtle
            theme
                .palette
                .subtle
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2]))
                .unwrap_or_else(|| adjust_color_brightness(bg_color, 2.0)),
            // Color 1 intense (bright red) - use brighter error
            theme
                .palette
                .error
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2]))
                .map_or((255, 84, 84), |c| adjust_color_brightness(c, 1.3)),
            // Color 2 intense (bright green) - use debug or brighter success
            theme
                .palette
                .debug
                .rgb()
                .or_else(|| {
                    theme
                        .palette
                        .success
                        .rgb()
                        .map(|c| adjust_color_brightness((c[0], c[1], c[2]), 1.3).into())
                })
                .map(|arr| (arr[0], arr[1], arr[2]))
                .unwrap_or((84, 255, 84)),
            // Color 3 intense (bright yellow) - use brighter warning
            theme
                .palette
                .warning
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2]))
                .map_or((255, 255, 84), |c| adjust_color_brightness(c, 1.3)),
            // Color 4 intense (bright blue) - use link or brighter info
            theme
                .palette
                .link
                .rgb()
                .or_else(|| {
                    theme
                        .palette
                        .info
                        .rgb()
                        .map(|c| adjust_color_brightness((c[0], c[1], c[2]), 1.3).into())
                })
                .map(|arr| (arr[0], arr[1], arr[2]))
                .unwrap_or((84, 84, 255)),
            // Color 5 intense (bright magenta) - use heading2 or brighter heading1
            theme
                .palette
                .heading2
                .rgb()
                .or_else(|| {
                    theme
                        .palette
                        .heading1
                        .rgb()
                        .map(|c| adjust_color_brightness((c[0], c[1], c[2]), 1.3).into())
                })
                .map(|arr| (arr[0], arr[1], arr[2]))
                .unwrap_or((255, 84, 255)),
            // Color 6 intense (bright cyan) - use brighter code/hint
            theme
                .palette
                .hint
                .rgb()
                .or_else(|| {
                    theme
                        .palette
                        .code
                        .rgb()
                        .map(|c| adjust_color_brightness((c[0], c[1], c[2]), 1.3).into())
                })
                .map(|arr| (arr[0], arr[1], arr[2]))
                .unwrap_or((84, 255, 255)),
            // Color 7 intense (bright white) - use quote or brighter normal
            theme
                .palette
                .quote
                .rgb()
                .map(|arr| (arr[0], arr[1], arr[2]))
                .or_else(|| {
                    Some(adjust_color_brightness(
                        (fg_color[0], fg_color[1], fg_color[2]),
                        1.3,
                    ))
                })
                .unwrap_or((255, 255, 255)),
        ];

        for (i, (r, g, b)) in bright_colors.iter().enumerate() {
            let _ = writeln!(output, "[Color{}Intense]", i);
            let _ = writeln!(output, "Color={},{},{}", r, g, b);
            output.push('\n');
        }

        // ANSI Colors 0-7 Faint (dim colors)
        for (i, (r, g, b)) in normal_colors.iter().enumerate() {
            let faint_color = dim_color((*r, *g, *b));
            let _ = writeln!(output, "[Color{}Faint]", i);
            let _ = writeln!(
                output,
                "Color={},{},{}",
                faint_color.0, faint_color.1, faint_color.2
            );
            output.push('\n');
        }

        Ok(output)
    }

    fn file_extension() -> &'static str {
        "colorscheme"
    }

    fn format_name() -> &'static str {
        "Konsole"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporters::create_test_theme;

    #[test]
    fn test_konsole_export() {
        let theme = create_test_theme();
        let result = KonsoleExporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Check that the content contains expected sections
        assert!(content.contains("[General]"));
        assert!(content.contains("[Background]"));
        assert!(content.contains("[Foreground]"));
        assert!(content.contains("[Color0]"));
        assert!(content.contains("[Color0Intense]"));
        assert!(content.contains("[Color0Faint]"));
        assert!(content.contains("Description="));
        assert!(content.contains("Color="));
    }

    #[test]
    fn test_konsole_file_extension() {
        assert_eq!(KonsoleExporter::file_extension(), "colorscheme");
    }

    #[test]
    fn test_konsole_format_name() {
        assert_eq!(KonsoleExporter::format_name(), "Konsole");
    }
}
