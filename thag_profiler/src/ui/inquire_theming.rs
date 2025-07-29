//! Inquire UI theming integration for thag_profiler
//!
//! This module provides theme-aware styling for inquire prompts with lightweight
//! color detection that doesn't depend on the full thag_rs styling system.

#[cfg(feature = "inquire_theming")]
pub use self::themed::*;

#[cfg(feature = "inquire_theming")]
mod themed {
    use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet};

    /// Simple color support detection
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ColorSupport {
        /// No color support
        None,
        /// Basic 16-color support
        Basic,
        /// 256-color support
        Color256,
        /// True color (24-bit RGB) support
        TrueColor,
    }

    /// Simple terminal background detection
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TerminalBackground {
        /// Light background
        Light,
        /// Dark background
        Dark,
        /// Unknown background
        Unknown,
    }

    /// Detect color support level of the terminal
    fn detect_color_support() -> ColorSupport {
        // Check environment variables for color support
        if std::env::var("NO_COLOR").is_ok() {
            return ColorSupport::None;
        }

        // Check COLORTERM for truecolor support
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return ColorSupport::TrueColor;
            }
        }

        // Check TERM for color capabilities
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") || term.contains("256") {
                return ColorSupport::Color256;
            } else if term.contains("color") || term == "xterm" || term == "screen" {
                return ColorSupport::Basic;
            }
        }

        // Default to basic color support
        ColorSupport::Basic
    }

    /// Simple terminal background detection
    fn detect_terminal_background() -> TerminalBackground {
        // For now, default to dark since most developer terminals are dark
        // In a full implementation, this would query the terminal background
        TerminalBackground::Dark
    }

    /// Create theme-appropriate colors based on detected capabilities
    fn create_color_scheme(support: ColorSupport, background: TerminalBackground) -> ColorScheme {
        match (support, background) {
            (ColorSupport::TrueColor, TerminalBackground::Dark) => ColorScheme {
                selected: Color::Rgb {
                    r: 98,
                    g: 209,
                    b: 150,
                }, // Green
                normal: Color::Rgb {
                    r: 200,
                    g: 200,
                    b: 200,
                }, // Light gray
                help: Color::Rgb {
                    r: 150,
                    g: 150,
                    b: 150,
                }, // Medium gray
                error: Color::Rgb {
                    r: 255,
                    g: 100,
                    b: 100,
                }, // Red
                success: Color::Rgb {
                    r: 100,
                    g: 255,
                    b: 100,
                }, // Green
                subtle: Color::Rgb {
                    r: 120,
                    g: 120,
                    b: 120,
                }, // Dark gray
            },
            (ColorSupport::TrueColor, TerminalBackground::Light) => ColorScheme {
                selected: Color::Rgb { r: 0, g: 128, b: 0 }, // Dark green
                normal: Color::Rgb {
                    r: 50,
                    g: 50,
                    b: 50,
                }, // Dark gray
                help: Color::Rgb {
                    r: 100,
                    g: 100,
                    b: 100,
                }, // Medium gray
                error: Color::Rgb { r: 200, g: 0, b: 0 },    // Dark red
                success: Color::Rgb { r: 0, g: 150, b: 0 },  // Dark green
                subtle: Color::Rgb {
                    r: 150,
                    g: 150,
                    b: 150,
                }, // Light gray
            },
            (ColorSupport::Color256, TerminalBackground::Dark) => ColorScheme {
                selected: Color::AnsiValue(10), // Bright green
                normal: Color::AnsiValue(15),   // White
                help: Color::AnsiValue(8),      // Bright black (gray)
                error: Color::AnsiValue(9),     // Bright red
                success: Color::AnsiValue(10),  // Bright green
                subtle: Color::AnsiValue(8),    // Bright black (gray)
            },
            (ColorSupport::Color256, TerminalBackground::Light) => ColorScheme {
                selected: Color::AnsiValue(2), // Green
                normal: Color::AnsiValue(0),   // Black
                help: Color::AnsiValue(8),     // Bright black (gray)
                error: Color::AnsiValue(1),    // Red
                success: Color::AnsiValue(2),  // Green
                subtle: Color::AnsiValue(8),   // Bright black (gray)
            },
            (ColorSupport::Color256, TerminalBackground::Unknown) => ColorScheme {
                selected: Color::AnsiValue(10), // Bright green (default to dark theme)
                normal: Color::AnsiValue(15),   // White
                help: Color::AnsiValue(8),      // Bright black (gray)
                error: Color::AnsiValue(9),     // Bright red
                success: Color::AnsiValue(10),  // Bright green
                subtle: Color::AnsiValue(8),    // Bright black (gray)
            },
            (ColorSupport::TrueColor, TerminalBackground::Unknown) => ColorScheme {
                selected: Color::Rgb {
                    r: 98,
                    g: 209,
                    b: 150,
                }, // Green (default to dark theme)
                normal: Color::Rgb {
                    r: 200,
                    g: 200,
                    b: 200,
                }, // Light gray
                help: Color::Rgb {
                    r: 150,
                    g: 150,
                    b: 150,
                }, // Medium gray
                error: Color::Rgb {
                    r: 255,
                    g: 100,
                    b: 100,
                }, // Red
                success: Color::Rgb {
                    r: 100,
                    g: 255,
                    b: 100,
                }, // Green
                subtle: Color::Rgb {
                    r: 120,
                    g: 120,
                    b: 120,
                }, // Dark gray
            },
            (ColorSupport::Basic, _) => ColorScheme {
                selected: Color::AnsiValue(10), // Bright green
                normal: Color::AnsiValue(15),   // White
                help: Color::AnsiValue(8),      // Bright black (gray)
                error: Color::AnsiValue(9),     // Bright red
                success: Color::AnsiValue(10),  // Bright green
                subtle: Color::AnsiValue(8),    // Bright black (gray)
            },
            (ColorSupport::None, _) => ColorScheme {
                selected: Color::AnsiValue(15), // White
                normal: Color::AnsiValue(15),   // White
                help: Color::AnsiValue(15),     // White
                error: Color::AnsiValue(15),    // White
                success: Color::AnsiValue(15),  // White
                subtle: Color::AnsiValue(15),   // White
            },
        }
    }

    /// Color scheme for the UI
    struct ColorScheme {
        selected: Color,
        normal: Color,
        help: Color,
        error: Color,
        success: Color,
        subtle: Color,
    }

    /// Get a theme-aware RenderConfig for inquire prompts
    ///
    /// This function creates an inquire RenderConfig that automatically:
    /// - Detects terminal capabilities (TrueColor, 256-color, or Basic)
    /// - Applies appropriate colors based on terminal background
    /// - Falls back gracefully on limited terminals
    ///
    /// # Returns
    ///
    /// A configured `RenderConfig` that matches the current terminal capabilities
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use thag_profiler::ui::inquire_theming::get_themed_render_config;
    /// use inquire::Select;
    ///
    /// // Apply theme-aware styling to all inquire prompts
    /// inquire::set_global_render_config(get_themed_render_config());
    ///
    /// // Now use inquire normally - it will use the themed config
    /// let options = vec!["Option 1", "Option 2", "Option 3"];
    /// let selection = Select::new("Choose an option:", options).prompt();
    /// ```
    pub fn get_themed_render_config() -> RenderConfig<'static> {
        let color_support = detect_color_support();
        let background = detect_terminal_background();
        let colors = create_color_scheme(color_support, background);

        let mut render_config = RenderConfig::default();

        // Configure the selected option with emphasis
        render_config.selected_option = Some(
            StyleSheet::new()
                .with_fg(colors.selected)
                .with_attr(Attributes::BOLD),
        );

        // Configure other UI elements
        render_config.option = StyleSheet::empty().with_fg(colors.normal);
        render_config.help_message = StyleSheet::empty().with_fg(colors.help);
        render_config.error_message = inquire::ui::ErrorMessageRenderConfig::default_colored()
            .with_message(StyleSheet::empty().with_fg(colors.error));
        render_config.prompt = StyleSheet::empty().with_fg(colors.normal);
        render_config.answer = StyleSheet::empty().with_fg(colors.success);
        render_config.placeholder = StyleSheet::empty().with_fg(colors.subtle);
        render_config.placeholder = StyleSheet::empty().with_fg(colors.subtle);

        render_config
    }

    /// Apply theme-aware styling globally to all inquire prompts
    ///
    /// This is a convenience function that automatically configures inquire
    /// to use theme-aware styling for all subsequent prompts in your application.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use thag_profiler::ui::inquire_theming::apply_global_theming;
    ///
    /// // Call once at the start of your application
    /// apply_global_theming();
    ///
    /// // All inquire prompts will now use theme-aware styling
    /// ```
    pub fn apply_global_theming() {
        inquire::set_global_render_config(get_themed_render_config());
    }

    /// Get information about the detected terminal capabilities
    ///
    /// This function returns information about what was detected about the
    /// terminal's color capabilities and background.
    ///
    /// # Returns
    ///
    /// A tuple of (ColorSupport, TerminalBackground) indicating the detected capabilities
    pub fn get_terminal_info() -> (ColorSupport, TerminalBackground) {
        (detect_color_support(), detect_terminal_background())
    }
}

#[cfg(not(feature = "inquire_theming"))]
mod fallback {
    use inquire::ui::RenderConfig;

    /// Fallback function when theming is disabled
    ///
    /// Returns the default inquire RenderConfig when the `inquire_theming`
    /// feature is not enabled.
    pub fn get_themed_render_config() -> RenderConfig<'static> {
        RenderConfig::default()
    }

    /// No-op fallback when theming is disabled
    pub fn apply_global_theming() {
        // Do nothing when theming is not available
    }

    /// Fallback terminal info when theming is disabled
    pub fn get_terminal_info() -> ((), ()) {
        ((), ())
    }
}

#[cfg(not(feature = "inquire_theming"))]
pub use self::fallback::*;
