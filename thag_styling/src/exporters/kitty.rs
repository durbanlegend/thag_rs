//! Kitty terminal theme exporter
//!
//! Exports thag themes to Kitty's configuration format.
//! Kitty uses a simple key-value configuration format for color schemes.

use crate::{
    exporters::{
        adjust_color_brightness, dim_color, get_best_dark_color, get_rgb_from_style,
        is_light_color, ThemeExporter,
    },
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
            "# Kitty Color Scheme: {}\n# Generated from thag theme\n# {}\n\n",
            theme.name, theme.description
        );

        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

        // Basic colors
        output.push_str("# Basic colors\n");
        let _ = writeln!(
            output,
            "background #{:02x}{:02x}{:02x}\n",
            bg_color.0, bg_color.1, bg_color.2
        );

        if let Some(fg_color) = get_rgb_from_style(&theme.palette.normal) {
            let _ = writeln!(
                output,
                "foreground #{:02x}{:02x}{:02x}\n",
                fg_color.0, fg_color.1, fg_color.2
            );
        }

        output.push('\n');

        // Selection colors
        output.push_str("# Selection colors\n");
        let selection_bg = adjust_color_brightness(bg_color, 1.4);
        let _ = writeln!(
            output,
            "selection_background #{:02x}{:02x}{:02x}\n",
            selection_bg.0, selection_bg.1, selection_bg.2
        );

        if let Some(selection_fg) = get_rgb_from_style(&theme.palette.normal) {
            let _ = writeln!(
                output,
                "selection_foreground #{:02x}{:02x}{:02x}\n",
                selection_fg.0, selection_fg.1, selection_fg.2
            );
        }

        output.push('\n');

        // Cursor colors
        output.push_str("# Cursor colors\n");
        if let Some(cursor_color) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            let _ = writeln!(
                output,
                "cursor #{:02x}{:02x}{:02x}\n",
                cursor_color.0, cursor_color.1, cursor_color.2
            );

            // Cursor text should contrast with cursor color
            let cursor_text = if is_light_color(cursor_color) {
                bg_color // Use background color for contrast
            } else {
                get_rgb_from_style(&theme.palette.normal).unwrap_or((255, 255, 255))
            };
            let _ = writeln!(
                output,
                "cursor_text_color #{:02x}{:02x}{:02x}\n",
                cursor_text.0, cursor_text.1, cursor_text.2
            );
        }

        output.push('\n');

        // URL colors
        output.push_str("# URL underline color when hovering with mouse\n");
        if let Some(url_color) = get_rgb_from_style(&theme.palette.info) {
            let _ = writeln!(
                output,
                "url_color #{:02x}{:02x}{:02x}\n",
                url_color.0, url_color.1, url_color.2
            );
        }

        output.push('\n');

        // Visual bell
        output.push_str("# Visual bell color\n");
        if let Some(bell_color) = get_rgb_from_style(&theme.palette.warning) {
            let _ = writeln!(
                output,
                "visual_bell_color #{:02x}{:02x}{:02x}\n",
                bell_color.0, bell_color.1, bell_color.2
            );
        }

        output.push('\n');

        // Active border color
        output.push_str("# Border colors\n");
        if let Some(active_border) = get_rgb_from_style(&theme.palette.emphasis) {
            let _ = writeln!(
                output,
                "active_border_color #{:02x}{:02x}{:02x}\n",
                active_border.0, active_border.1, active_border.2
            );
        }

        if let Some(inactive_border) = get_rgb_from_style(&theme.palette.subtle)
            .or_else(|| Some(adjust_color_brightness(bg_color, 1.3)))
        {
            let _ = writeln!(
                output,
                "inactive_border_color #{:02x}{:02x}{:02x}\n",
                inactive_border.0, inactive_border.1, inactive_border.2
            );
        }

        output.push('\n');

        // Tab colors
        output.push_str("# Tab bar colors\n");
        let tab_bg = adjust_color_brightness(bg_color, 0.9);
        let _ = writeln!(
            output,
            "tab_bar_background #{:02x}{:02x}{:02x}\n",
            tab_bg.0, tab_bg.1, tab_bg.2
        );

        if let Some(active_tab_fg) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            let _ = writeln!(
                output,
                "active_tab_foreground #{:02x}{:02x}{:02x}\n",
                active_tab_fg.0, active_tab_fg.1, active_tab_fg.2
            );
        }

        let _ = writeln!(
            output,
            "active_tab_background #{:02x}{:02x}{:02x}\n",
            bg_color.0, bg_color.1, bg_color.2
        );

        if let Some(inactive_tab_fg) = get_rgb_from_style(&theme.palette.subtle)
            .or_else(|| get_rgb_from_style(&theme.palette.normal).map(dim_color))
        {
            let _ = writeln!(
                output,
                "inactive_tab_foreground #{:02x}{:02x}{:02x}\n",
                inactive_tab_fg.0, inactive_tab_fg.1, inactive_tab_fg.2
            );
        }

        let _ = writeln!(
            output,
            "inactive_tab_background #{:02x}{:02x}{:02x}\n",
            tab_bg.0, tab_bg.1, tab_bg.2
        );

        output.push('\n');

        // Mark colors (for marks and text search)
        output.push_str("# Mark colors (for text search highlighting)\n");
        if let Some(mark1_bg) = get_rgb_from_style(&theme.palette.warning) {
            let _ = writeln!(
                output,
                "mark1_background #{:02x}{:02x}{:02x}\n",
                mark1_bg.0, mark1_bg.1, mark1_bg.2
            );
            let _ = writeln!(
                output,
                "mark1_foreground #{:02x}{:02x}{:02x}\n",
                bg_color.0, bg_color.1, bg_color.2
            );
        }

        if let Some(mark2_bg) = get_rgb_from_style(&theme.palette.info) {
            let _ = writeln!(
                output,
                "mark2_background #{:02x}{:02x}{:02x}\n",
                mark2_bg.0, mark2_bg.1, mark2_bg.2
            );
            let _ = writeln!(
                output,
                "mark2_foreground #{:02x}{:02x}{:02x}\n",
                bg_color.0, bg_color.1, bg_color.2
            );
        }

        if let Some(mark3_bg) = get_rgb_from_style(&theme.palette.success) {
            let _ = writeln!(
                output,
                "mark3_background #{:02x}{:02x}{:02x}\n",
                mark3_bg.0, mark3_bg.1, mark3_bg.2
            );
            let _ = writeln!(
                output,
                "mark3_foreground #{:02x}{:02x}{:02x}\n",
                bg_color.0, bg_color.1, bg_color.2
            );
        }

        output.push('\n');

        // Black (0-7: normal colors)
        output.push_str("# The color table\n");
        output.push_str("#\n");
        output.push_str("# black\n");
        if let Some((r, g, b)) = get_best_dark_color(theme) {
            let _ = writeln!(output, "color0 #{:02x}{:02x}{:02x}\n", r, g, b);
        }
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.subtle).or(Some((64, 64, 64))) {
            let _ = writeln!(output, "color8 #{:02x}{:02x}{:02x}\n", r, g, b);
        }

        output.push_str("\n# red\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.error) {
            let _ = writeln!(output, "color1 #{:02x}{:02x}{:02x}\n", r, g, b);
        }
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.link).or(Some((64, 64, 64))) {
            let _ = writeln!(output, "color8 #{:02x}{:02x}{:02x}\n", r, g, b);
        }

        output.push_str("\n# green\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.success) {
            let _ = writeln!(output, "color2 #{:02x}{:02x}{:02x}\n", r, g, b);
        }
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.debug).or(Some((64, 64, 64))) {
            let _ = writeln!(output, "color8 #{:02x}{:02x}{:02x}\n", r, g, b);
        }

        output.push_str("\n# yellow\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.warning) {
            let _ = writeln!(output, "color3 #{:02x}{:02x}{:02x}\n", r, g, b);
        }
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.emphasis).or(Some((64, 64, 64)))
        {
            let _ = writeln!(output, "color8 #{:02x}{:02x}{:02x}\n", r, g, b);
        }

        output.push_str("\n# blue\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.info) {
            let _ = writeln!(output, "color4 #{:02x}{:02x}{:02x}\n", r, g, b);
        }
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.code) {
            let _ = writeln!(output, "color12 #{:02x}{:02x}{:02x}\n", r, g, b);
        }

        output.push_str("\n# magenta\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.heading1) {
            let _ = writeln!(output, "color5 #{:02x}{:02x}{:02x}\n", r, g, b);
        }
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.heading2) {
            let _ = writeln!(output, "color13 #{:02x}{:02x}{:02x}\n", r, g, b);
        }

        output.push_str("\n# cyan\n");
        let cyan_normal = get_rgb_from_style(&theme.palette.heading3).unwrap_or((64, 192, 192));
        let _ = writeln!(
            output,
            "color6 #{:02x}{:02x}{:02x}\n",
            cyan_normal.0, cyan_normal.1, cyan_normal.2
        );
        let cyan_bright = get_rgb_from_style(&theme.palette.hint).unwrap_or((64, 192, 192));
        let _ = writeln!(
            output,
            "color14 #{:02x}{:02x}{:02x}\n",
            cyan_bright.0, cyan_bright.1, cyan_bright.2
        );

        output.push_str("\n# white\n");
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.normal) {
            let _ = writeln!(output, "color7 #{:02x}{:02x}{:02x}\n", r, g, b);
        }
        if let Some((r, g, b)) = get_rgb_from_style(&theme.palette.quote) {
            let _ = writeln!(output, "color15 #{:02x}{:02x}{:02x}\n", r, g, b);
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
    fn test_kitty_export() {
        let theme = create_test_theme();
        let result = KittyExporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();

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
        assert_eq!(color_256_to_rgb(0), (0, 0, 0));
        assert_eq!(color_256_to_rgb(15), (255, 255, 255));
        assert_eq!(basic_color_to_rgb(1), (128, 0, 0));

        assert_eq!(brighten_color((100, 100, 100)), (130, 130, 130));
        assert_eq!(dim_color((100, 100, 100)), (60, 60, 60));
    }

    #[test]
    fn test_color_brightness_detection() {
        assert!(is_light_color((255, 255, 255))); // White should be light
        assert!(!is_light_color((0, 0, 0))); // Black should be dark
        assert!(!is_light_color((64, 64, 64))); // Dark gray should be dark
    }
}
