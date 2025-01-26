#[cfg(test)]
mod tests {
    use thag_rs::styling::{ColorInitStrategy, TermAttributes};
    use thag_rs::{Color, ColorSupport, Style, TermBgLuma};

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

    #[ignore = "Fails as runs second and you can only initialise once"]
    #[test]
    fn test_styling_term_attributes_init_default() {
        // Test default initialization
        let attrs = TermAttributes::initialize(&ColorInitStrategy::Default);
        assert!(matches!(attrs.color_support, ColorSupport::Basic));
        assert!(matches!(attrs.theme.term_bg_luma, TermBgLuma::Dark));
    }

    #[test]
    fn test_styling_term_attributes_init_config() {
        // Test explicit configuration
        let attrs = TermAttributes::initialize(&ColorInitStrategy::Configure(
            ColorSupport::Color256,
            TermBgLuma::Light,
            None,
        ));
        eprintln!("attrs.color_support={0:#?}", attrs.color_support);
        assert!(matches!(attrs.color_support, ColorSupport::Color256));
        assert!(matches!(attrs.theme.term_bg_luma, TermBgLuma::Light));
    }
}
