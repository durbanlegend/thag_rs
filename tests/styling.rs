#[cfg(test)]
mod tests {
    use thag_styling::{Color, ColorSupport, Style, TermAttributes, TermBgLuma, Theme};

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
    fn test_styling_term_attributes_init_default() {
        // Test default initialization using context pattern
        let theme = Theme::get_builtin("basic_dark").unwrap();
        let attrs = TermAttributes::for_testing(ColorSupport::Basic, None, TermBgLuma::Dark, theme);
        attrs.with_context(|| {
            let current_attrs = TermAttributes::current();
            assert!(matches!(current_attrs.color_support, ColorSupport::Basic));
            assert!(matches!(current_attrs.theme.term_bg_luma, TermBgLuma::Dark));
        });
    }

    #[test]
    fn test_styling_term_attributes_init_config() {
        // Test explicit configuration using context pattern
        let theme = Theme::get_builtin("github").unwrap();
        let attrs =
            TermAttributes::for_testing(ColorSupport::Color256, None, TermBgLuma::Light, theme);
        attrs.with_context(|| {
            let current_attrs = TermAttributes::current();
            assert!(matches!(
                current_attrs.color_support,
                ColorSupport::Color256
            ));
            assert!(matches!(
                current_attrs.theme.term_bg_luma,
                TermBgLuma::Light
            ));
        });
    }
}
