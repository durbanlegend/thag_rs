//! Hybrid inquire UI theming with multiple strategies
//!
//! This module provides multiple theming approaches for inquire prompts:
//! 1. Full thag_rs integration (when available) - sophisticated base16/base24 themes
//! 2. Lightweight self-contained theming - basic color detection
//! 3. Fallback to default inquire colors
//!
//! The system automatically selects the best available strategy or allows manual selection.

#[cfg(feature = "inquire_theming")]
pub use self::themed::*;

#[cfg(feature = "inquire_theming")]
mod themed {
    pub use inquire::ui::Color;
    use inquire::ui::{Attributes, RenderConfig, StyleSheet};

    /// Available theming strategies
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ThemingStrategy {
        /// Use full thag_rs styling system (requires thag_rs with color_detect)
        FullThagRs,
        /// Use lightweight self-contained theming
        Lightweight,
        /// Use default inquire colors
        Default,
        /// Automatically select best available strategy
        Auto,
    }

    /// Simple color support detection for lightweight theming
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ColorCapability {
        /// No color support
        None,
        /// Basic 16-color support
        Basic,
        /// 256-color support
        Color256,
        /// True color (24-bit RGB) support
        TrueColor,
    }

    /// Terminal background type for lightweight theming
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BackgroundType {
        /// Light background
        Light,
        /// Dark background
        Dark,
        /// Unknown background (default to dark)
        Unknown,
    }

    /// Detect color support level of the terminal
    fn detect_color_capability() -> ColorCapability {
        // Check environment variables for color support
        if std::env::var("NO_COLOR").is_ok() {
            return ColorCapability::None;
        }

        // Check COLORTERM for truecolor support
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return ColorCapability::TrueColor;
            }
        }

        // Check TERM for color capabilities
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") || term.contains("256") {
                return ColorCapability::Color256;
            } else if term.contains("color") || term == "xterm" || term == "screen" {
                return ColorCapability::Basic;
            }
        }

        // Default to basic color support
        ColorCapability::Basic
    }

    /// Simple terminal background detection
    fn detect_background_type() -> BackgroundType {
        // For now, default to dark since most developer terminals are dark
        // In a full implementation, this would query the terminal background
        // Could be enhanced with environment variable detection
        if let Ok(bg) = std::env::var("TERM_BACKGROUND") {
            match bg.to_lowercase().as_str() {
                "light" => BackgroundType::Light,
                "dark" => BackgroundType::Dark,
                _ => BackgroundType::Unknown,
            }
        } else {
            BackgroundType::Dark // Safe default
        }
    }

    /// Color scheme for inquire UI elements
    struct ColorScheme {
        selected: Color,
        normal: Color,
        help: Color,
        error: Color,
        success: Color,
        subtle: Color,
    }

    /// Create color scheme based on capabilities and background - with improved contrast
    fn create_color_scheme(capability: ColorCapability, background: BackgroundType) -> ColorScheme {
        match (capability, background) {
            // TrueColor with dark background
            (ColorCapability::TrueColor, BackgroundType::Dark) => ColorScheme {
                selected: Color::Rgb {
                    r: 98,
                    g: 209,
                    b: 150,
                }, // Bright green
                normal: Color::Rgb {
                    r: 220,
                    g: 220,
                    b: 220,
                }, // Light gray
                help: Color::Rgb {
                    r: 135,
                    g: 175,
                    b: 255,
                }, // Light blue
                error: Color::Rgb {
                    r: 255,
                    g: 120,
                    b: 120,
                }, // Light red
                success: Color::Rgb {
                    r: 120,
                    g: 255,
                    b: 120,
                }, // Light green
                subtle: Color::Rgb {
                    r: 200,
                    g: 100,
                    b: 255,
                }, // Magenta (better contrast)
            },
            // TrueColor with light background
            (ColorCapability::TrueColor, BackgroundType::Light) => ColorScheme {
                selected: Color::Rgb { r: 0, g: 120, b: 0 }, // Dark green
                normal: Color::Rgb {
                    r: 40,
                    g: 40,
                    b: 40,
                }, // Dark gray
                help: Color::Rgb {
                    r: 0,
                    g: 80,
                    b: 160,
                }, // Dark blue
                error: Color::Rgb { r: 180, g: 0, b: 0 },    // Dark red
                success: Color::Rgb { r: 0, g: 140, b: 0 },  // Dark green
                subtle: Color::Rgb {
                    r: 140,
                    g: 0,
                    b: 140,
                }, // Dark magenta (better contrast)
            },
            // TrueColor with unknown background (default to dark theme colors)
            (ColorCapability::TrueColor, BackgroundType::Unknown) => ColorScheme {
                selected: Color::Rgb {
                    r: 98,
                    g: 209,
                    b: 150,
                }, // Bright green
                normal: Color::Rgb {
                    r: 200,
                    g: 200,
                    b: 200,
                }, // Light gray
                help: Color::Rgb {
                    r: 135,
                    g: 175,
                    b: 255,
                }, // Light blue
                error: Color::Rgb {
                    r: 255,
                    g: 120,
                    b: 120,
                }, // Light red
                success: Color::Rgb {
                    r: 120,
                    g: 255,
                    b: 120,
                }, // Light green
                subtle: Color::Rgb {
                    r: 200,
                    g: 100,
                    b: 255,
                }, // Magenta
            },
            // 256-color with dark background
            (ColorCapability::Color256, BackgroundType::Dark) => ColorScheme {
                selected: Color::AnsiValue(10), // Bright green
                normal: Color::AnsiValue(250),  // Light gray
                help: Color::AnsiValue(75),     // Light blue
                error: Color::AnsiValue(9),     // Bright red
                success: Color::AnsiValue(10),  // Bright green
                subtle: Color::AnsiValue(13),   // Bright magenta (better contrast)
            },
            // 256-color with light background
            (ColorCapability::Color256, BackgroundType::Light) => ColorScheme {
                selected: Color::AnsiValue(2), // Green
                normal: Color::AnsiValue(0),   // Black
                help: Color::AnsiValue(4),     // Blue
                error: Color::AnsiValue(1),    // Red
                success: Color::AnsiValue(2),  // Green
                subtle: Color::AnsiValue(5),   // Magenta (better contrast)
            },
            // 256-color with unknown background (default to dark)
            (ColorCapability::Color256, BackgroundType::Unknown) => ColorScheme {
                selected: Color::AnsiValue(10), // Bright green
                normal: Color::AnsiValue(15),   // White
                help: Color::AnsiValue(14),     // Bright cyan
                error: Color::AnsiValue(9),     // Bright red
                success: Color::AnsiValue(10),  // Bright green
                subtle: Color::AnsiValue(13),   // Bright magenta
            },
            // Basic color support (all backgrounds)
            (ColorCapability::Basic, _) => ColorScheme {
                selected: Color::AnsiValue(10), // Bright green
                normal: Color::AnsiValue(15),   // White
                help: Color::AnsiValue(14),     // Bright cyan
                error: Color::AnsiValue(9),     // Bright red
                success: Color::AnsiValue(10),  // Bright green
                subtle: Color::AnsiValue(13),   // Bright magenta (better than gray)
            },
            // No color support
            (ColorCapability::None, _) => ColorScheme {
                selected: Color::AnsiValue(15), // White
                normal: Color::AnsiValue(15),   // White
                help: Color::AnsiValue(15),     // White
                error: Color::AnsiValue(15),    // White
                success: Color::AnsiValue(15),  // White
                subtle: Color::AnsiValue(15),   // White
            },
        }
    }

    /// Theme color provider trait for dependency injection
    pub trait ThemeColorProvider {
        /// Get color for selected items (emphasis role)
        fn selected_color(&self) -> Color;
        /// Get color for normal text
        fn normal_color(&self) -> Color;
        /// Get color for help messages (info role)
        fn help_color(&self) -> Color;
        /// Get color for error messages
        fn error_color(&self) -> Color;
        /// Get color for success/answer text
        fn success_color(&self) -> Color;
        /// Get color for subtle/placeholder text
        fn subtle_color(&self) -> Color;
    }

    /// Global theme color provider - can be set by applications
    static mut THEME_PROVIDER: Option<Box<dyn ThemeColorProvider + Send + Sync>> = None;

    /// Set a global theme color provider (typically called by thag_rs or other host applications)
    pub fn set_theme_provider(provider: Box<dyn ThemeColorProvider + Send + Sync>) {
        unsafe {
            THEME_PROVIDER = Some(provider);
        }
    }

    /// Try to create render config using provided theme colors
    fn try_theme_provider_styling() -> Option<RenderConfig<'static>> {
        unsafe {
            if let Some(provider) = &THEME_PROVIDER {
                let mut render_config = RenderConfig::default();

                // Use theme provider colors - respects configured theme (Black Metal, etc.)
                render_config.selected_option = Some(
                    StyleSheet::new()
                        .with_fg(provider.selected_color())
                        .with_attr(Attributes::BOLD),
                );

                render_config.option = StyleSheet::empty().with_fg(provider.normal_color());
                render_config.help_message = StyleSheet::empty().with_fg(provider.help_color());
                render_config.error_message =
                    inquire::ui::ErrorMessageRenderConfig::default_colored()
                        .with_message(StyleSheet::empty().with_fg(provider.error_color()));
                render_config.prompt = StyleSheet::empty().with_fg(provider.normal_color());
                render_config.answer = StyleSheet::empty().with_fg(provider.success_color());
                render_config.placeholder = StyleSheet::empty().with_fg(provider.subtle_color());

                Some(render_config)
            } else {
                None
            }
        }
    }

    /// Create render config using lightweight theming (fallback when thag_rs not available)
    fn create_lightweight_render_config() -> RenderConfig<'static> {
        let capability = detect_color_capability();
        let background = detect_background_type();
        let colors = create_color_scheme(capability, background);

        let mut render_config = RenderConfig::default();

        // Configure the selected option with emphasis
        render_config.selected_option = Some(
            StyleSheet::new()
                .with_fg(colors.selected)
                .with_attr(Attributes::BOLD),
        );

        // Configure other UI elements using improved contrast colors
        // Note: This is a fallback - when thag_rs is available, actual theme colors are used
        render_config.option = StyleSheet::empty().with_fg(colors.normal);
        render_config.help_message = StyleSheet::empty().with_fg(colors.help);
        render_config.error_message = inquire::ui::ErrorMessageRenderConfig::default_colored()
            .with_message(StyleSheet::empty().with_fg(colors.error));
        render_config.prompt = StyleSheet::empty().with_fg(colors.normal);
        render_config.answer = StyleSheet::empty().with_fg(colors.success);
        render_config.placeholder = StyleSheet::empty().with_fg(colors.subtle);

        render_config
    }

    /// Convenience function for external tools to provide their theme colors
    ///
    /// This allows tools like thag_demo to inject their theme-aware colors
    /// without creating circular dependencies.
    pub fn apply_external_theme_colors(
        selected: Color,
        normal: Color,
        help: Color,
        error: Color,
        success: Color,
        subtle: Color,
    ) {
        struct ExternalThemeProvider {
            selected: Color,
            normal: Color,
            help: Color,
            error: Color,
            success: Color,
            subtle: Color,
        }

        impl ThemeColorProvider for ExternalThemeProvider {
            fn selected_color(&self) -> Color {
                self.selected
            }
            fn normal_color(&self) -> Color {
                self.normal
            }
            fn help_color(&self) -> Color {
                self.help
            }
            fn error_color(&self) -> Color {
                self.error
            }
            fn success_color(&self) -> Color {
                self.success
            }
            fn subtle_color(&self) -> Color {
                self.subtle
            }
        }

        let provider = Box::new(ExternalThemeProvider {
            selected,
            normal,
            help,
            error,
            success,
            subtle,
        });
        set_theme_provider(provider);
    }

    /// Get a themed RenderConfig using the specified strategy
    ///
    /// # Arguments
    /// * `strategy` - The theming strategy to use
    ///
    /// # Returns
    /// A configured `RenderConfig` based on the selected strategy
    pub fn get_render_config_with_strategy(strategy: ThemingStrategy) -> RenderConfig<'static> {
        match strategy {
            ThemingStrategy::FullThagRs => {
                // Try to use theme provider colors, fall back to lightweight if not available
                try_theme_provider_styling().unwrap_or_else(|| create_lightweight_render_config())
            }
            ThemingStrategy::Lightweight => create_lightweight_render_config(),
            ThemingStrategy::Default => RenderConfig::default(),
            ThemingStrategy::Auto => {
                // PRIORITY: Try theme provider colors first to respect configured themes
                // Only fall back to lightweight hardcoded colors if theme unavailable
                try_theme_provider_styling().unwrap_or_else(|| create_lightweight_render_config())
            }
        }
    }

    /// Get a theme-aware RenderConfig for inquire prompts (auto strategy)
    ///
    /// This function creates an inquire RenderConfig that automatically:
    /// - FIRST: Tries to use configured theme's Role-based colors (respects Black Metal, etc.)
    /// - FALLBACK: Uses lightweight terminal-capability-based theming
    /// - Maps inquire elements to semantic Roles (help → Info, selected → Emphasis, etc.)
    /// - Preserves theme consistency across all UI elements
    ///
    /// # Returns
    /// A configured `RenderConfig` that respects the current theme configuration
    pub fn get_themed_render_config() -> RenderConfig<'static> {
        get_render_config_with_strategy(ThemingStrategy::Auto)
    }

    /// Apply theme-aware styling globally to all inquire prompts
    ///
    /// Uses the auto strategy to select the best available theming approach.
    pub fn apply_global_theming() {
        inquire::set_global_render_config(get_themed_render_config());
    }

    /// Apply specific theming strategy globally to all inquire prompts
    ///
    /// # Arguments
    /// * `strategy` - The theming strategy to use
    pub fn apply_global_theming_with_strategy(strategy: ThemingStrategy) {
        inquire::set_global_render_config(get_render_config_with_strategy(strategy));
    }

    /// Get information about the detected terminal capabilities
    ///
    /// # Returns
    /// A tuple of (ColorCapability, BackgroundType) indicating detected capabilities
    pub fn get_terminal_info() -> (ColorCapability, BackgroundType) {
        (detect_color_capability(), detect_background_type())
    }

    /// Check which theming strategies are available
    ///
    /// # Returns
    /// A vector of available theming strategies
    pub fn get_available_strategies() -> Vec<ThemingStrategy> {
        let mut strategies = vec![
            ThemingStrategy::Lightweight,
            ThemingStrategy::Default,
            ThemingStrategy::Auto,
        ];

        // Check if theme provider is available (respects configured themes)
        if try_theme_provider_styling().is_some() {
            strategies.insert(0, ThemingStrategy::FullThagRs);
        }

        strategies
    }

    /// Get a description of a theming strategy
    pub fn describe_strategy(strategy: ThemingStrategy) -> &'static str {
        match strategy {
            ThemingStrategy::FullThagRs => {
                "Respects configured theme Role colors (Black Metal, etc.)"
            }
            ThemingStrategy::Lightweight => "Basic terminal-aware colors (ignores theme)",
            ThemingStrategy::Default => "Default inquire colors (no theming)",
            ThemingStrategy::Auto => "Use theme colors if available, else basic colors",
        }
    }
}

#[cfg(not(feature = "inquire_theming"))]
mod fallback {
    /// Theming strategy placeholder when feature is disabled
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ThemingStrategy {
        Default,
    }

    /// Fallback function when theming is disabled
    pub fn get_themed_render_config() -> () {
        ()
    }

    /// Fallback function when theming is disabled
    pub fn get_render_config_with_strategy(_strategy: ThemingStrategy) -> () {
        ()
    }

    /// No-op fallback when theming is disabled
    pub fn apply_global_theming() {
        // Do nothing when theming is not available
    }

    /// No-op fallback when theming is disabled
    pub fn apply_global_theming_with_strategy(_strategy: ThemingStrategy) {
        // Do nothing when theming is not available
    }

    /// Fallback terminal info when theming is disabled
    pub fn get_terminal_info() -> ((), ()) {
        ((), ())
    }

    /// Fallback strategies when theming is disabled
    pub fn get_available_strategies() -> Vec<ThemingStrategy> {
        vec![ThemingStrategy::Default]
    }

    /// Fallback description when theming is disabled
    pub fn describe_strategy(_strategy: ThemingStrategy) -> &'static str {
        "Theming disabled"
    }
}

#[cfg(not(feature = "inquire_theming"))]
pub use self::fallback::*;
