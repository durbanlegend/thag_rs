//! Kitty terminal theme exporter
//!
//! Exports thag themes to Kitty's configuration format.
//! Kitty uses a simple key-value configuration format for color schemes.

use crate::{
    exporters::{adjust_color_brightness, dim_color, is_light_color, ThemeExporter},
    StylingResult, Theme,
};
use std::fmt::Write as _; // import without risk of name clashing

/// Kitty theme exporter
pub struct KittyExporter;

impl ThemeExporter for KittyExporter {
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        let mut output = String::new();

        // Add header comment
        let _ = writeln!(
            output,
            "# Kitty Color Scheme: {}\n# Generated from thag theme\n# {}",
            theme.name, theme.description
        );

        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or([0, 0, 0]);

        // Basic colors
        output.push_str("# Basic colors\n");
        let [r, g, b] = bg_color;
        let _ = writeln!(output, "background #{r:02x}{g:02x}{b:02x}");

        if let Some([r, g, b]) = theme.palette.normal.rgb() {
            let _ = writeln!(output, "foreground #{r:02x}{g:02x}{b:02x}");
        }

        output.push('\n');

        // Selection colors
        output.push_str("# Selection colors\n");
        let selection_bg: [u8; 3] = theme
            .palette
            .commentary
            .rgb()
            // .map(|c| (c[0], c[1], c[2]))
            .unwrap_or_else(|| adjust_color_brightness(bg_color, 1.4));
        let [r, g, b] = selection_bg;
        let _ = writeln!(output, "selection_background #{r:02x}{g:02x}{b:02x}");

        if let Some([r, g, b]) = theme.palette.normal.rgb() {
            let _ = writeln!(output, "selection_foreground #{r:02x}{g:02x}{b:02x}");
        }

        output.push('\n');

        // Cursor colors
        output.push_str("# Cursor colors\n");
        if let Some([r, g, b]) = theme
            .palette
            .emphasis
            .rgb()
            .or_else(|| theme.palette.normal.rgb())
        {
            let _ = writeln!(output, "cursor #{r:02x}{g:02x}{b:02x}");

            // Cursor text should contrast with cursor color
            let cursor_text = if is_light_color([r, g, b]) {
                bg_color // Use background color for contrast
            } else {
                theme
                    .palette
                    .normal
                    .rgb()
                    // .map(|c| (c[0], c[1], c[2]))
                    .unwrap_or([255, 255, 255])
            };
            let [tr, tg, tb] = cursor_text;
            let _ = writeln!(output, "cursor_text_color #{tr:02x}{tg:02x}{tb:02x}");
        }

        output.push('\n');

        // URL colors
        output.push_str("# URL underline color when hovering with mouse\n");
        if let Some([r, g, b]) = theme.palette.info.rgb() {
            let _ = writeln!(output, "url_color #{r:02x}{g:02x}{b:02x}");
        }

        output.push('\n');

        // Visual bell
        output.push_str("# Visual bell color\n");
        if let Some([r, g, b]) = theme.palette.warning.rgb() {
            let _ = writeln!(output, "visual_bell_color #{r:02x}{g:02x}{b:02x}");
        }

        output.push('\n');

        // Active border color
        output.push_str("# Border colors\n");
        if let Some([r, g, b]) = theme.palette.emphasis.rgb() {
            let _ = writeln!(output, "active_border_color #{r:02x}{g:02x}{b:02x}");
        }

        if let Some([r, g, b]) = theme
            .palette
            .subtle
            .rgb()
            // .map(|c| (c[0], c[1], c[2]))
            .or_else(|| Some(adjust_color_brightness(bg_color, 1.3)))
        {
            let _ = writeln!(output, "inactive_border_color #{r:02x}{g:02x}{b:02x}");
        }

        output.push('\n');

        // Tab colors
        output.push_str("# Tab bar colors\n");
        let tab_bg = adjust_color_brightness(bg_color, 0.9);
        let [r, g, b] = tab_bg;
        let _ = writeln!(output, "tab_bar_background #{r:02x}{g:02x}{b:02x}");

        if let Some([r, g, b]) = theme
            .palette
            .emphasis
            .rgb()
            // .map(|c| (c[0], c[1], c[2]))
            .or_else(|| theme.palette.normal.rgb())
        {
            let _ = writeln!(output, "active_tab_foreground #{r:02x}{g:02x}{b:02x}");
        }

        let [r, g, b] = bg_color;
        let _ = writeln!(output, "active_tab_background #{r:02x}{g:02x}{b:02x}");

        if let Some([r, g, b]) = theme
            .palette
            .subtle
            .rgb()
            // .map(|c| (c[0], c[1], c[2]))
            .map(dim_color)
        {
            let _ = writeln!(output, "inactive_tab_foreground #{r:02x}{g:02x}{b:02x}");
        }

        let [r, g, b] = tab_bg;
        let _ = writeln!(output, "inactive_tab_background #{r:02x}{g:02x}{b:02x}");

        output.push('\n');

        // Mark colors (for marks and text search)
        output.push_str("# Mark colors (for text search highlighting)\n");
        if let Some([r, g, b]) = theme.palette.warning.rgb() {
            let _ = writeln!(output, "mark1_background #{r:02x}{g:02x}{b:02x}");
            let [fr, fg, fb] = bg_color;
            let _ = writeln!(output, "mark1_foreground #{fr:02x}{fg:02x}{fb:02x}");
        }

        if let Some([r, g, b]) = theme.palette.info.rgb() {
            let _ = writeln!(output, "mark2_background #{r:02x}{g:02x}{b:02x}");
            let [fr, fg, fb] = bg_color;
            let _ = writeln!(output, "mark2_foreground #{fr:02x}{fg:02x}{fb:02x}");
        }

        if let Some([r, g, b]) = theme.palette.success.rgb() {
            let _ = writeln!(output, "mark3_background #{r:02x}{g:02x}{b:02x}");
            let [fr, fg, fb] = bg_color;
            let _ = writeln!(output, "mark3_foreground #{fr:02x}{fg:02x}{fb:02x}");
        }

        output.push('\n');

        // Black (0-7: normal colors)
        output.push_str("# The color table\n");
        output.push_str("#\n");
        output.push_str("# black\n");
        if let Some([r, g, b]) = Some(theme.bg_rgbs[0]) {
            let _ = writeln!(output, "color0 #{:02x}{:02x}{:02x}", r, g, b);
        }
        if let Some([r, g, b]) = theme
            .palette
            .subtle
            .rgb()
            // .map(|c| (c[0], c[1], c[2]))
            .or(Some([64, 64, 64]))
        {
            let _ = writeln!(output, "color8 #{:02x}{:02x}{:02x}", r, g, b);
        }

        output.push_str("\n# red\n");
        // ANSI color codes
        if let Some([r, g, b]) = theme.palette.error.rgb() {
            let _ = writeln!(output, "color1 #{r:02x}{g:02x}{b:02x}");
        }
        if let Some([r, g, b]) = theme.palette.error.rgb() {
            let _ = writeln!(output, "color9 #{r:02x}{g:02x}{b:02x}");
        }

        if let Some([r, g, b]) = theme.palette.success.rgb() {
            let _ = writeln!(output, "color2 #{r:02x}{g:02x}{b:02x}");
        }
        if let Some([r, g, b]) = theme.palette.debug.rgb() {
            let _ = writeln!(output, "color10 #{r:02x}{g:02x}{b:02x}");
        }

        if let Some([r, g, b]) = theme.palette.warning.rgb() {
            let _ = writeln!(output, "color3 #{r:02x}{g:02x}{b:02x}");
        }
        if let Some([r, g, b]) = theme.palette.warning.rgb() {
            let _ = writeln!(output, "color11 #{r:02x}{g:02x}{b:02x}");
        }

        if let Some([r, g, b]) = theme.palette.info.rgb() {
            let _ = writeln!(output, "color4 #{r:02x}{g:02x}{b:02x}");
        }
        if let Some([r, g, b]) = theme.palette.link.rgb() {
            let _ = writeln!(output, "color12 #{r:02x}{g:02x}{b:02x}");
        }

        if let Some([r, g, b]) = theme.palette.code.rgb() {
            let _ = writeln!(output, "color5 #{r:02x}{g:02x}{b:02x}");
        }
        if let Some([r, g, b]) = theme.palette.heading2.rgb() {
            let _ = writeln!(output, "color13 #{r:02x}{g:02x}{b:02x}");
        }

        if let Some([r, g, b]) = theme.palette.code.rgb() {
            let _ = writeln!(output, "color6 #{r:02x}{g:02x}{b:02x}");
        }
        if let Some([r, g, b]) = theme.palette.hint.rgb() {
            let _ = writeln!(output, "color14 #{r:02x}{g:02x}{b:02x}");
        }

        if let Some([r, g, b]) = theme.palette.normal.rgb() {
            let _ = writeln!(output, "color7 #{r:02x}{g:02x}{b:02x}");
        }
        if let Some([r, g, b]) = theme.palette.quote.rgb() {
            let _ = writeln!(output, "color15 #{r:02x}{g:02x}{b:02x}");
        }

        output.push('\n');

        Ok(output)
    }

    fn file_extension() -> &'static str {
        "conf"
    }

    fn format_name() -> &'static str {
        "Kitty"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporters::{
        basic_color_to_rgb, brighten_color, color_256_to_rgb, create_test_theme,
    };

    #[test]
    fn test_kitty_export() {
        let theme = create_test_theme();
        let result = KittyExporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();
        // eprintln!("content={content}");

        // Check that the content contains expected configuration keys
        assert!(content.contains("background"));
        assert!(content.contains("foreground"));
        assert!(content.contains("selection_background"));
        assert!(content.contains("cursor"));
        assert!(content.contains("color0"));
        assert!(content.contains("color15"));
        assert!(content.contains("active_tab_foreground"));
        assert!(content.contains("mark1_background"));
    }

    #[test]
    fn test_color_conversions() {
        assert_eq!(color_256_to_rgb(0), [0, 0, 0]);
        assert_eq!(color_256_to_rgb(15), [255, 255, 255]);
        assert_eq!(basic_color_to_rgb(1), [128, 0, 0]);

        assert_eq!(brighten_color([100, 100, 100]), [130, 130, 130]);
        assert_eq!(dim_color([100, 100, 100]), [60, 60, 60]);
    }

    #[test]
    fn test_color_brightness_detection() {
        assert!(is_light_color([255, 255, 255])); // White should be light
        assert!(!is_light_color([0, 0, 0])); // Black should be dark
        assert!(!is_light_color([64, 64, 64])); // Dark gray should be dark
    }
}
