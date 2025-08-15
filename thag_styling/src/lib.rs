//! Terminal styling system with theme support and color detection for `thag_rs` and third party applications.
//!
//! This crate provides a comprehensive styling system for terminal applications, including:
//! - Color detection and terminal capability assessment
//! - Theme-based styling with semantic roles
//! - Inquire UI integration for consistent theming
//! - ANSI color conversion and optimization
//! - Built-in theme collection

#![warn(clippy::pedantic, missing_docs)]

/// Message styling
pub mod styling;

/// Third-party crate integrations
pub mod integrations;

/// Runtime terminal palette synchronization using OSC sequences
pub mod palette_sync;

/// Image-based theme generation
#[cfg(feature = "image_themes")]
pub mod image_themes;

/// Multi-format theme exporters for various terminal emulators
pub mod exporters;

// Re-export common types
pub use thag_common::{
    debug_log, get_verbosity, prtln, re, vprtln, ColorSupport, TermBgLuma, ThagCommonResult,
    Verbosity, OUTPUT_MANAGER, V,
};

pub use styling::{
    display_terminal_attributes, display_theme_details, display_theme_roles, find_closest_color,
    get_rgb, paint_for_role, AnsiStyleExt, Color, ColorInfo, ColorInitStrategy, ColorValue,
    HowInitialized, Palette, PaletteConfig, Role, Style, StyleLike, Styled, TermAttributes, Theme,
};

// Re-export integration traits and types
pub use integrations::ThemedStyle;

// Re-export palette sync functionality
pub use palette_sync::PaletteSync;

#[cfg(feature = "ratatui_support")]
pub use integrations::ratatui_integration::RatatuiStyleExt;

#[cfg(feature = "nu_ansi_term_support")]
pub use integrations::nu_ansi_term_integration::NuAnsiTermStyleExt;

#[cfg(feature = "crossterm_support")]
pub use integrations::crossterm_integration::{CrosstermStyleExt, ThemedStylize};

pub use thag_proc_macros::styled;

// Re-export image theme generation types
#[cfg(feature = "image_themes")]
pub use image_themes::{
    generate_and_save_theme, generate_theme_from_image, generate_theme_from_image_with_config,
    save_theme_to_file, theme_to_toml, ImageThemeConfig, ImageThemeGenerator,
};

// Re-export theme exporter types
pub use exporters::{
    export_all_formats, export_theme_to_file, generate_installation_instructions, ExportFormat,
    ThemeExporter,
};

/// Result type alias for styling operations
pub type StylingResult<T> = Result<T, StylingError>;

/// Error types for styling operations
#[derive(Debug, thiserror::Error)]
pub enum StylingError {
    /// Theme-related errors
    #[error("Theme error: {0}")]
    Theme(#[from] ThemeError),
    /// Common errors from `thag_common`
    #[error("Common error: {0}")]
    Common(#[from] thag_common::ThagCommonError),
    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// TOML parsing errors
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    /// Parse integer error
    #[error("Parse error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    /// String parse error
    #[error("Parse error: {0}")]
    ParseStr(#[from] strum::ParseError),
    /// Parse error
    #[error("Parse error")]
    Parse,
    /// `FromStr` error
    #[error("FromStr error: {0}")]
    FromStr(String),
    /// Generic error with message
    #[error("{0}")]
    Generic(String),
}

/// Theme-related error types
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    /// Failed to detect terminal background color
    #[error("Background RGB not detected or configured for terminal")]
    BackgroundDetectionFailed,
    /// Color support mismatch between theme requirements and terminal capabilities
    #[error("Theme requires {required:?} colors but terminal only supports {available:?}")]
    ColorSupportMismatch {
        /// The color support level required by the theme
        required: ColorSupport,
        /// The color support level available in the terminal
        available: ColorSupport,
    },
    /// Attempted to use a dark theme with a light terminal background
    #[error("Only light themes may be selected for a light terminal background")]
    DarkThemeLightTerm,
    /// Terminal does not support sufficient colors for the requested operation
    #[error(
        "Configured or detected level of terminal colour support is insufficient for this theme"
    )]
    InsufficientColorSupport,
    /// Invalid ANSI escape code format
    #[error("{0}")]
    InvalidAnsiCode(String),
    /// Invalid color support specification
    #[error("Invalid color support: {0}")]
    InvalidColorSupport(String),
    /// Invalid color value format or specification
    #[error("Invalid color value: {0}")]
    InvalidColorValue(String),
    /// Invalid style attribute specification
    #[error("Invalid style attribute: {0}")]
    InvalidStyle(String),
    /// Invalid terminal background luminance value
    #[error("Unknown value: must be `light` or `dark`: {0}")]
    InvalidTermBgLuma(String),
    /// Attempted to use a light theme with a dark terminal background
    #[error("Only dark themes may be selected for a dark terminal background")]
    LightThemeDarkTerm,
    /// No valid background color found for the specified theme
    #[error("No valid background found for theme {0}")]
    NoValidBackground(String),
    /// Terminal background luminance mismatch with theme requirements
    #[error("Theme requires {theme:?} background but terminal is {terminal:?}")]
    TermBgLumaMismatch {
        /// The background luminance required by the theme
        theme: TermBgLuma,
        /// The actual background luminance of the terminal
        terminal: TermBgLuma,
    },
    /// Unknown or unrecognized theme name
    #[error("Unknown theme: {0}")]
    UnknownTheme(String),
}

/// Trait for providing styling configuration to break circular dependency
pub trait StylingConfigProvider {
    /// Get color support setting
    fn color_support(&self) -> ColorSupport;
    /// Get terminal background luminance setting
    fn term_bg_luma(&self) -> TermBgLuma;
    /// Get terminal background RGB setting
    fn term_bg_rgb(&self) -> Option<(u8, u8, u8)>;
    /// Get background color list
    fn backgrounds(&self) -> Vec<String>;
    /// Get preferred light themes
    fn preferred_light(&self) -> Vec<String>;
    /// Get preferred dark themes
    fn preferred_dark(&self) -> Vec<String>;
}

/// Default implementation that uses no configuration
pub struct NoConfigProvider;

impl StylingConfigProvider for NoConfigProvider {
    fn color_support(&self) -> ColorSupport {
        ColorSupport::Undetermined
    }

    fn term_bg_luma(&self) -> TermBgLuma {
        TermBgLuma::default()
    }

    fn term_bg_rgb(&self) -> Option<(u8, u8, u8)> {
        None
    }

    fn backgrounds(&self) -> Vec<String> {
        Vec::new()
    }

    fn preferred_light(&self) -> Vec<String> {
        Vec::new()
    }

    fn preferred_dark(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Create an inquire `RenderConfig` that respects the current theme
#[cfg(feature = "inquire_theming")]
#[must_use]
pub fn themed_inquire_config() -> inquire::ui::RenderConfig<'static> {
    inquire_theming::create_render_config()
}

/// Helper functions for inquire UI theming integration
#[cfg(feature = "inquire_theming")]
pub mod inquire_theming {
    use super::{ColorValue, Role, TermAttributes};

    /// Convert a thag Role to an inquire Color using the current theme
    #[must_use]
    pub fn role_to_inquire_color(role: Role) -> Option<inquire::ui::Color> {
        let term_attrs = TermAttributes::get_or_init();
        let theme = &term_attrs.theme;
        let style = theme.style_for(role);

        style.foreground.as_ref().map_or_else(
            || Some(inquire::ui::Color::AnsiValue(u8::from(&role))),
            |color_info| match &color_info.value {
                ColorValue::TrueColor { rgb } => Some(inquire::ui::Color::Rgb {
                    r: rgb[0],
                    g: rgb[1],
                    b: rgb[2],
                }),
                ColorValue::Color256 { color256 } => Some(inquire::ui::Color::AnsiValue(*color256)),
                ColorValue::Basic { .. } => {
                    // Use thag's existing color mapping for basic terminals
                    Some(inquire::ui::Color::AnsiValue(u8::from(&role)))
                }
            },
        )
    }

    /// Create a theme-aware `RenderConfig` for inquire prompts
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn create_render_config() -> inquire::ui::RenderConfig<'static> {
        let mut render_config = inquire::ui::RenderConfig::default();

        // Get terminal attributes and current theme from thag's color system
        let term_attrs = TermAttributes::get_or_init();
        let theme = &term_attrs.theme;

        // Helper function to convert thag colors to inquire colors
        let convert_color = |role: Role| -> inquire::ui::Color {
            let style = theme.style_for(role);
            style.foreground.as_ref().map_or_else(
                || inquire::ui::Color::AnsiValue(u8::from(&role)),
                |color_info| match &color_info.value {
                    ColorValue::TrueColor { rgb } => inquire::ui::Color::Rgb {
                        r: rgb[0],
                        g: rgb[1],
                        b: rgb[2],
                    },
                    ColorValue::Color256 { color256 } => inquire::ui::Color::AnsiValue(*color256),
                    ColorValue::Basic { .. } => {
                        // Use thag's existing color mapping for basic terminals
                        inquire::ui::Color::AnsiValue(u8::from(&role))
                    }
                },
            )
        };

        // Helper function to extract RGB values from a role for color distance calculation
        #[allow(clippy::cast_possible_truncation)]
        let get_rgb = |role: Role| -> Option<(u8, u8, u8)> {
            let style = theme.style_for(role);
            style
                .foreground
                .as_ref()
                .and_then(|color_info| match &color_info.value {
                    ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
                    ColorValue::Color256 { color256 } => {
                        // Convert 256-color to RGB for distance calculation
                        let index = *color256 as usize;
                        if index < 16 {
                            // Standard colors
                            let colors = [
                                (0, 0, 0),       // Black
                                (128, 0, 0),     // Red
                                (0, 128, 0),     // Green
                                (128, 128, 0),   // Yellow
                                (0, 0, 128),     // Blue
                                (128, 0, 128),   // Magenta
                                (0, 128, 128),   // Cyan
                                (192, 192, 192), // White
                                (128, 128, 128), // Bright Black
                                (255, 0, 0),     // Bright Red
                                (0, 255, 0),     // Bright Green
                                (255, 255, 0),   // Bright Yellow
                                (0, 0, 255),     // Bright Blue
                                (255, 0, 255),   // Bright Magenta
                                (0, 255, 255),   // Bright Cyan
                                (255, 255, 255), // Bright White
                            ];
                            colors.get(index).copied()
                        } else if index < 232 {
                            // 216 color cube
                            let n = index - 16;
                            let r = (n / 36) * 51;
                            let g = ((n % 36) / 6) * 51;
                            let b = (n % 6) * 51;
                            Some((r as u8, g as u8, b as u8))
                        } else {
                            // Grayscale
                            let gray = 8 + (index - 232) * 10;
                            Some((gray as u8, gray as u8, gray as u8))
                        }
                    }
                    ColorValue::Basic { .. } => {
                        // Convert basic role to approximate RGB for distance calculation
                        match role {
                            Role::Error => Some((255, 0, 0)),
                            Role::Success => Some((0, 255, 0)),
                            Role::Warning => Some((255, 255, 0)),
                            Role::Info => Some((0, 255, 255)),
                            Role::Code => Some((255, 0, 255)),
                            Role::Emphasis => Some((255, 128, 0)),
                            Role::Heading3 => Some((128, 255, 128)),
                            _ => Some((192, 192, 192)),
                        }
                    }
                })
        };

        // Color distance function (same as in styling.rs)
        let color_distance = |c1: (u8, u8, u8), c2: (u8, u8, u8)| -> f32 {
            let dr = (f32::from(c1.0) - f32::from(c2.0)).powi(2);
            let dg = (f32::from(c1.1) - f32::from(c2.1)).powi(2);
            let db = (f32::from(c1.2) - f32::from(c2.2)).powi(2);
            (dr + dg + db).sqrt()
        };

        // Choose the best selected_option color based on color distance from Normal
        let prompt_rgb = get_rgb(Role::Normal);
        let candidate_roles = [
            Role::Emphasis,
            Role::Heading2,
            Role::Heading3,
            Role::Info,
            Role::Success,
        ];

        let best_role = prompt_rgb.map_or(Role::Code, |normal_color| {
            candidate_roles
                .iter()
                .filter_map(|&role| {
                    get_rgb(role).map(|rgb| (role, color_distance(normal_color, rgb)))
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map_or(Role::Code, |(role, _)| role)
        });
        // Map inquire UI elements to appropriate thag roles
        render_config.selected_option = Some(
            inquire::ui::StyleSheet::new()
                .with_fg(convert_color(best_role))
                .with_attr(inquire::ui::Attributes::BOLD),
        );

        // Set regular option styling to Normal role
        render_config.option =
            inquire::ui::StyleSheet::empty().with_fg(convert_color(Role::Normal));
        render_config.help_message =
            inquire::ui::StyleSheet::empty().with_fg(convert_color(Role::Info));
        render_config.error_message = inquire::ui::ErrorMessageRenderConfig::default_colored()
            .with_message(inquire::ui::StyleSheet::empty().with_fg(convert_color(Role::Error)));
        render_config.prompt =
            inquire::ui::StyleSheet::empty().with_fg(convert_color(Role::Normal));
        render_config.answer =
            inquire::ui::StyleSheet::empty().with_fg(convert_color(Role::Success));
        render_config.placeholder =
            inquire::ui::StyleSheet::empty().with_fg(convert_color(Role::Subtle));

        render_config
    }
}
