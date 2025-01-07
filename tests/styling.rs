#[cfg(test)]
mod tests {
    use thag_rs::styling::{ColorInitStrategy, TermAttributes};
    use thag_rs::{Color, ColorSupport, Style, TermTheme};

    // Helper function to make ANSI sequences visible in test output
    fn debug_ansi(s: &str) -> String {
        s.replace('\x1b', "\\x1b")
    }

    #[test]
    fn test_styling_basic_style_creation() {
        let style = Style::default();
        assert_eq!(style.paint("test"), "test");

        let red_style = Color::red().bold();
        let output = red_style.paint("test");
        assert!(
            output.contains("\x1b[31m"),
            "Expected red color, got: {}",
            debug_ansi(&output)
        );
        assert!(
            output.contains("\x1b[1m"),
            "Expected bold, got: {}",
            debug_ansi(&output)
        );
    }

    #[test]
    fn test_term_attributes_init() {
        // Test default initialization
        let attrs = TermAttributes::initialize(&ColorInitStrategy::Default);
        assert!(matches!(attrs.color_support, ColorSupport::Ansi16));
        assert!(matches!(attrs.theme, TermTheme::Dark));

        // Test explicit configuration
        let attrs = TermAttributes::initialize(&ColorInitStrategy::Configure(
            &ColorSupport::Xterm256,
            &TermTheme::Light,
        ));
        assert!(matches!(attrs.color_support, ColorSupport::Xterm256));
        assert!(matches!(attrs.theme, TermTheme::Light));
    }
}
