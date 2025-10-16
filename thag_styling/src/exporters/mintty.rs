//! Mintty terminal theme exporter
//!
//! Exports thag themes to Mintty's INI-style theme format used by Git Bash on Windows.
//! Mintty theme files are simple key-value pairs stored without file extensions.

use crate::{
    exporters::{adjust_color_brightness, ThemeExporter},
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
        let bg_color = theme.bg_rgbs.first().copied().unwrap_or([0, 0, 0]);
        let [r, g, b] = bg_color;

        // Basic terminal colors
        let _ = writeln!(output, "BackgroundColour={r},{g},{b}");

        if let Some([r, g, b]) = &theme.palette.normal.rgb() {
            let _ = writeln!(output, "ForegroundColour={r},{g},{b}",);
        }

        // Cursor colors
        let cursor_color = theme
            .palette
            .emphasis
            .rgb()
            .or_else(|| theme.palette.normal.rgb());

        if let Some([r, g, b]) = &cursor_color {
            let _ = writeln!(output, "CursorColour={r},{g},{b}");
        }

        // Selection colors
        // Use commentary color for better visibility, fallback to brightness adjustment
        let [r, g, b] = theme
            .palette
            .commentary
            .rgb()
            // .map(|c| (c[0], c[1], c[2]))
            .unwrap_or_else(|| adjust_color_brightness(bg_color, 1.4));
        let _ = writeln!(output, "SelectionBackgroundColour={r},{g},{b}");

        // Also set legacy HighlightBackgroundColour for compatibility
        let _ = writeln!(output, "HighlightBackgroundColour={r},{g},{b}");

        // Selection foreground (use normal text color)
        if let Some([r, g, b]) = &theme.palette.normal.rgb() {
            let _ = writeln!(output, "SelectionForegroundColour={r},{g},{b}");
        }

        // Bold text color
        let bold_color = theme
            .palette
            .emphasis
            .rgb()
            .or_else(|| theme.palette.normal.rgb());

        if let Some([r, g, b]) = &bold_color {
            let _ = writeln!(output, "BoldColour={r},{g},{b}");
        }

        // ANSI colors (0-15)
        // Black
        if let Some([r, g, b]) = Some(theme.bg_rgbs[0]) {
            let _ = writeln!(output, "Black={r},{g},{b}");
        }

        // Dark Red
        if let Some([r, g, b]) = &theme.palette.emphasis.rgb() {
            let _ = writeln!(output, "Red={r},{g},{b}");
        }

        // Dark Green
        if let Some([r, g, b]) = &theme.palette.success.rgb() {
            let _ = writeln!(output, "Green={r},{g},{b}");
        }

        // Dark Yellow
        if let Some([r, g, b]) = &theme.palette.commentary.rgb() {
            let _ = writeln!(output, "Yellow={r},{g},{b}");
        }

        // Dark Blue
        if let Some([r, g, b]) = &theme.palette.info.rgb() {
            let _ = writeln!(output, "Blue={r},{g},{b}");
        }

        // Dark Magenta
        if let Some([r, g, b]) = &theme.palette.heading1.rgb() {
            let _ = writeln!(output, "Magenta={r},{g},{b}");
        }

        // Dark Cyan
        if let Some([r, g, b]) = &theme.palette.code.rgb() {
            let _ = writeln!(output, "Cyan={r},{g},{b}");
        }

        // White
        if let Some([r, g, b]) = &theme.palette.normal.rgb() {
            let _ = writeln!(output, "White={r},{g},{b}");
        }

        // Bright colors (8-15)
        // Bright Black (usually gray)
        if let Some([r, g, b]) = &theme.palette.subtle.rgb() {
            let _ = writeln!(output, "BoldBlack={r},{g},{b}");
        }

        // Bright Red
        if let Some([r, g, b]) = &theme.palette.error.rgb() {
            let _ = writeln!(output, "BoldRed={r},{g},{b}");
        }

        // Bright Green
        if let Some([r, g, b]) = &theme.palette.debug.rgb() {
            let _ = writeln!(output, "BoldGreen={r},{g},{b}");
        }

        // Bright Yellow
        if let Some([r, g, b]) = &theme.palette.warning.rgb() {
            let _ = writeln!(output, "BoldYellow={r},{g},{b}");
        }

        // Bright Blue
        if let Some([r, g, b]) = &theme.palette.link.rgb() {
            let _ = writeln!(output, "BoldBlue={r},{g},{b}");
        }

        // Bright Magenta
        if let Some([r, g, b]) = &theme.palette.heading2.rgb() {
            let _ = writeln!(output, "BoldMagenta={r},{g},{b}");
        }

        // Bright Cyan
        if let Some([r, g, b]) = &theme.palette.hint.rgb() {
            let _ = writeln!(output, "BoldCyan={r},{g},{b}",);
        }

        // Bright White
        if let Some([r, g, b]) = &theme.palette.quote.rgb() {
            let _ = writeln!(output, "BoldWhite={r},{g},{b}");
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
    use crate::exporters::create_test_theme;

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
        assert!(content.lines().count() > 3); // Should have header and some config lines
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
