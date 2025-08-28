//! Mintty terminal theme exporter
//!
//! Exports thag themes to Mintty's INI-style theme format used by Git Bash on Windows.
//! Mintty theme files are simple key-value pairs stored without file extensions.

use crate::{
    exporters::{adjust_color_brightness, get_best_dark_color, get_rgb_from_style, ThemeExporter},
    StylingResult, Theme,
};

use std::fmt::Write as _;

/// Mintty theme exporter
pub struct MinttyExporter;

impl ThemeExporter for MinttyExporter {
    #[allow(clippy::too_many_lines)]
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        let mut output = String::new();

        // Add header comment
        let _ = writeln!(
            output,
            "# Mintty Color Scheme: {}\n# Generated from thag theme\n# {}\n",
            theme.name, theme.description
        );

        // Get primary background color
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

        // Basic terminal colors
        let _ = writeln!(
            output,
            "BackgroundColour={},{},{}",
            bg_color.0, bg_color.1, bg_color.2
        );

        if let Some(fg_color) = get_rgb_from_style(&theme.palette.normal) {
            let _ = writeln!(
                output,
                "ForegroundColour={},{},{}",
                fg_color.0, fg_color.1, fg_color.2
            );
        }

        // Cursor colors
        if let Some(cursor_color) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            let _ = writeln!(
                output,
                "CursorColour={},{},{}",
                cursor_color.0, cursor_color.1, cursor_color.2
            );
        }

        // Selection background (slightly brighter than background)
        let selection_bg = adjust_color_brightness(bg_color, 1.4);
        let _ = writeln!(
            output,
            "HighlightBackgroundColour={},{},{}",
            selection_bg.0, selection_bg.1, selection_bg.2
        );

        // Bold text color
        if let Some(bold_color) = get_rgb_from_style(&theme.palette.emphasis)
            .or_else(|| get_rgb_from_style(&theme.palette.normal))
        {
            let _ = writeln!(
                output,
                "BoldColour={},{},{}",
                bold_color.0, bold_color.1, bold_color.2
            );
        }

        // ANSI colors (0-15)
        // Black
        if let Some(black) = get_best_dark_color(theme) {
            let _ = writeln!(output, "Black={},{},{}", black.0, black.1, black.2);
        }

        // Dark Red
        if let Some(red) = get_rgb_from_style(&theme.palette.emphasis) {
            let _ = writeln!(output, "Red={},{},{}", red.0, red.1, red.2);
        }

        // Dark Green
        if let Some(green) = get_rgb_from_style(&theme.palette.success) {
            let _ = writeln!(output, "Green={},{},{}", green.0, green.1, green.2);
        }

        // Dark Yellow
        if let Some(yellow) = get_rgb_from_style(&theme.palette.commentary) {
            let _ = writeln!(output, "Yellow={},{},{}", yellow.0, yellow.1, yellow.2);
        }

        // Dark Blue
        if let Some(blue) = get_rgb_from_style(&theme.palette.info) {
            let _ = writeln!(output, "Blue={},{},{}", blue.0, blue.1, blue.2);
        }

        // Dark Magenta
        if let Some(magenta) = get_rgb_from_style(&theme.palette.heading1) {
            let _ = writeln!(output, "Magenta={},{},{}", magenta.0, magenta.1, magenta.2);
        }

        // Dark Cyan
        if let Some(cyan) = get_rgb_from_style(&theme.palette.code) {
            let _ = writeln!(output, "Cyan={},{},{}", cyan.0, cyan.1, cyan.2);
        }

        // White
        if let Some(white) = get_rgb_from_style(&theme.palette.normal) {
            let _ = writeln!(output, "White={},{},{}", white.0, white.1, white.2);
        }

        // Bright colors (8-15)
        // Bright Black (usually gray)
        if let Some(bright_black) = get_rgb_from_style(&theme.palette.subtle) {
            let _ = writeln!(
                output,
                "BoldBlack={},{},{}",
                bright_black.0, bright_black.1, bright_black.2
            );
        }

        // Bright Red
        if let Some(bright_red) = get_rgb_from_style(&theme.palette.error) {
            let _ = writeln!(
                output,
                "BoldRed={},{},{}",
                bright_red.0, bright_red.1, bright_red.2
            );
        }

        // Bright Green
        if let Some(bright_green) = get_rgb_from_style(&theme.palette.debug) {
            let _ = writeln!(
                output,
                "BoldGreen={},{},{}",
                bright_green.0, bright_green.1, bright_green.2
            );
        }

        // Bright Yellow
        if let Some(bright_yellow) = get_rgb_from_style(&theme.palette.warning) {
            let _ = writeln!(
                output,
                "BoldYellow={},{},{}",
                bright_yellow.0, bright_yellow.1, bright_yellow.2
            );
        }

        // Bright Blue
        if let Some(bright_blue) = get_rgb_from_style(&theme.palette.link) {
            let _ = writeln!(
                output,
                "BoldBlue={},{},{}",
                bright_blue.0, bright_blue.1, bright_blue.2
            );
        }

        // Bright Magenta
        if let Some(bright_magenta) = get_rgb_from_style(&theme.palette.heading2) {
            let _ = writeln!(
                output,
                "BoldMagenta={},{},{}",
                bright_magenta.0, bright_magenta.1, bright_magenta.2
            );
        }

        // Bright Cyan
        if let Some(bright_cyan) = get_rgb_from_style(&theme.palette.hint) {
            let _ = writeln!(
                output,
                "BoldCyan={},{},{}",
                bright_cyan.0, bright_cyan.1, bright_cyan.2
            );
        }

        // Bright White
        if let Some(bright_white) = get_rgb_from_style(&theme.palette.quote) {
            let _ = writeln!(
                output,
                "BoldWhite={},{},{}",
                bright_white.0, bright_white.1, bright_white.2
            );
        }

        Ok(output)
    }

    fn file_extension() -> &'static str {
        "" // Mintty theme files have no extension
    }

    fn format_name() -> &'static str {
        "Mintty"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColorSupport, Palette, TermBgLuma};
    use std::path::PathBuf;

    fn create_test_theme() -> Theme {
        Theme {
            name: "Test Mintty Theme".to_string(),
            filename: PathBuf::from("test.toml"),
            is_builtin: false,
            term_bg_luma: TermBgLuma::Dark,
            min_color_support: ColorSupport::TrueColor,
            palette: Palette::default(),
            backgrounds: vec!["#1e1e2e".to_string()],
            bg_rgbs: vec![(30, 30, 46)],
            description: "A test theme for mintty".to_string(),
        }
    }

    #[test]
    fn test_mintty_export() {
        let theme = create_test_theme();
        let result = MinttyExporter::export_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Check for mintty-specific format - only test what should exist with default palette
        assert!(content.contains("# Mintty Color Scheme"));
        assert!(content.contains("BackgroundColour="));
        assert!(content.contains("HighlightBackgroundColour="));
        assert!(content.contains("Black="));

        // Check that colors are in R,G,B format
        assert!(content.contains("30,30,46")); // Background color from test theme

        // The format should be valid - no syntax errors
        let lines: Vec<&str> = content.lines().collect();
        assert!(lines.len() > 3); // Should have header and some config lines
    }

    #[test]
    fn test_file_extension() {
        assert_eq!(MinttyExporter::file_extension(), "");
    }

    #[test]
    fn test_format_name() {
        assert_eq!(MinttyExporter::format_name(), "Mintty");
    }

    #[test]
    fn test_mintty_debug_output() {
        let theme = create_test_theme();
        let result = MinttyExporter::export_theme(&theme);
        assert!(result.is_ok());
        let content = result.unwrap();
        println!("Debug mintty output:\n{}", content);

        // More lenient test - just check that basic structure exists
        assert!(content.contains("# Mintty Color Scheme"));
        assert!(content.contains("BackgroundColour=30,30,46"));
    }
}
