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

// Re-export common types
pub use thag_common::{
    get_verbosity, vprtln, ColorSupport, TermBgLuma, ThagCommonResult, Verbosity, V,
};

pub use styling::{
    display_terminal_attributes, display_theme_details, display_theme_roles, paint_for_role, Color,
    ColorInfo, ColorInitStrategy, HowInitialized, Lvl, PaletteConfig, Role, Style, TermAttributes,
    Theme,
};

/// Result type alias for styling operations
pub type StylingResult<T> = Result<T, StylingError>;

/// Error types for styling operations
#[derive(Debug, thiserror::Error)]
pub enum StylingError {
    /// Theme-related errors
    #[error("Theme error: {0}")]
    Theme(#[from] ThemeError),
    /// Common errors from thag_common
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
    /// FromStr error
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
                ColorValue::Basic { .. } => Some(inquire::ui::Color::AnsiValue(u8::from(&role))),
            },
        )
    }

    /// Create a full render config for inquire with theme support
    #[must_use]
    pub fn create_render_config() -> inquire::ui::RenderConfig<'static> {
        use inquire::ui::{RenderConfig, StyleSheet};

        let term_attrs = TermAttributes::get_or_init();
        let theme = &term_attrs.theme;

        // Helper function to convert thag colors to inquire colors
        let convert_role_to_color = |role: Role| -> inquire::ui::Color {
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
                    ColorValue::Basic { .. } => inquire::ui::Color::AnsiValue(u8::from(&role)),
                },
            )
        };

        RenderConfig::<'_> {
            selected_option: Some(
                StyleSheet::new()
                    .with_fg(convert_role_to_color(Role::Emphasis))
                    .with_attr(inquire::ui::Attributes::BOLD),
            ),
            option: StyleSheet::empty().with_fg(convert_role_to_color(Role::Normal)),
            help_message: StyleSheet::empty().with_fg(convert_role_to_color(Role::Info)),
            error_message: inquire::ui::ErrorMessageRenderConfig::default_colored()
                .with_message(StyleSheet::empty().with_fg(convert_role_to_color(Role::Error))),
            prompt: StyleSheet::empty().with_fg(convert_role_to_color(Role::Normal)),
            answer: StyleSheet::empty().with_fg(convert_role_to_color(Role::Success)),
            placeholder: StyleSheet::empty().with_fg(convert_role_to_color(Role::Subtle)),
            ..Default::default()
        }
    }
}

// Replaced by terminal::detect_term_capabilities
// /// Detect color support using supports-color crate
// #[cfg(feature = "color_detect")]
// fn detect_color_support() -> ColorSupport {
//     use supports_color::Stream;

//     supports_color::on(Stream::Stdout).map_or(ColorSupport::None, |level| {
//         if level.has_16m {
//             ColorSupport::TrueColor
//         } else if level.has_256 {
//             ColorSupport::Color256
//         } else if level.has_basic {
//             ColorSupport::Basic
//         } else {
//             ColorSupport::None
//         }
//     })
// }

// /// Detect terminal background using termbg crate
// #[cfg(feature = "color_detect")]
// fn detect_terminal_background() -> TermBgLuma {
//     #[allow(clippy::match_same_arms)]
//     match termbg::theme(std::time::Duration::from_millis(100)) {
//         Ok(termbg::Theme::Light) => TermBgLuma::Light,
//         Ok(termbg::Theme::Dark) | Err(_) => TermBgLuma::Dark, // Default to dark on detection failure
//     }
// }

// /// How the terminal attributes were initialized
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub enum HowInitialized {
//     /// From configuration file
//     Configured,
//     /// Using default values
//     Defaulted,
//     /// Auto-detected from terminal
//     Detected,
// }

// /// Terminal attributes including color support and theme information
// #[derive(Clone, Debug)]
// pub struct TermAttributes {
//     /// How these attributes were initialized
//     pub how_initialized: HowInitialized,
//     /// Color support level
//     pub color_support: ColorSupport,
//     /// Terminal background color in hex format
//     pub term_bg_hex: Option<String>,
//     /// Terminal background RGB values
//     pub term_bg_rgb: Option<(u8, u8, u8)>,
//     /// Terminal background luminance
//     pub term_bg_luma: TermBgLuma,
//     /// Current theme
//     pub theme: Theme,
// }

// static INSTANCE: OnceLock<TermAttributes> = OnceLock::new();

// /// Global flag for logging enablement
// pub static LOGGING_ENABLED: AtomicBool = AtomicBool::new(false);

// impl TermAttributes {
//     #[allow(dead_code)]
//     const fn new() -> Self {
//         Self {
//             how_initialized: HowInitialized::Defaulted,
//             color_support: ColorSupport::Basic,
//             term_bg_hex: None,
//             term_bg_rgb: None,
//             term_bg_luma: TermBgLuma::Dark,
//             theme: Theme::default_theme(),
//         }
//     }

//     /// Initialize terminal attributes with a config provider
//     ///
//     /// # Errors
//     ///
//     /// Any auto-detect errors.
//     pub fn initialize<T: StylingConfigProvider>(provider: &T) -> StylingResult<&'static Self> {
//         let (color_support, term_bg_luma) = ColorInitStrategy::determine(provider);

//         let theme = Theme::auto_detect(color_support, term_bg_luma, provider)?;

//         let instance = Self {
//             how_initialized: HowInitialized::Detected,
//             color_support,
//             term_bg_hex: None,
//             term_bg_rgb: None,
//             term_bg_luma,
//             theme,
//         };

//         Ok(INSTANCE.get_or_init(|| instance))
//     }

//     /// Check if terminal attributes have been initialized
//     #[must_use]
//     pub fn is_initialized() -> bool {
//         INSTANCE.get().is_some()
//     }

//     /// Try to get terminal attributes if initialized
//     #[must_use]
//     pub fn try_get() -> Option<&'static Self> {
//         INSTANCE.get()
//     }

//     /// Get terminal attributes or initialize with default config
//     #[must_use]
//     pub fn get_or_init() -> &'static Self {
//         INSTANCE.get_or_init(|| {
//             let provider = NoConfigProvider;
//             let (color_support, term_bg_luma) = ColorInitStrategy::determine(&provider);

//             // Create theme directly without calling auto_detect to avoid recursion
//             let theme = Theme::create_basic_theme(color_support, term_bg_luma);

//             Self {
//                 how_initialized: HowInitialized::Defaulted,
//                 color_support,
//                 term_bg_hex: None,
//                 term_bg_rgb: None,
//                 term_bg_luma,
//                 theme,
//             }
//         })
//     }

//     /// Create a new instance with a different theme
//     #[must_use]
//     pub fn with_theme(&self, theme: Theme) -> Self {
//         Self {
//             theme,
//             ..self.clone()
//         }
//     }

//     /// Create a new instance with different color support
//     #[must_use]
//     pub fn with_color_support(&self, color_support: ColorSupport) -> Self {
//         Self {
//             color_support,
//             how_initialized: HowInitialized::Configured,
//             term_bg_hex: self.term_bg_hex.clone(),
//             term_bg_rgb: self.term_bg_rgb,
//             term_bg_luma: self.term_bg_luma,
//             theme: self.theme.clone(),
//         }
//     }
// }

// /// Paint text with a specific role using the current theme
// #[must_use]
// pub fn paint_for_role<T: AsRef<str>>(role: Role, text: T) -> String {
//     let term_attrs = TermAttributes::get_or_init();
//     let style = term_attrs.theme.style_for(role);
//     style.paint(text)
// }

// /// Get a style for a specific role and theme
// #[must_use]
// pub fn style_for_theme_and_role(theme: &Theme, role: Role) -> Style {
//     theme.style_for(role)
// }

// /// Theme palette configuration from TOML
// #[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
// pub struct PaletteConfig {
//     heading1: Option<String>,
//     heading2: Option<String>,
//     heading3: Option<String>,
//     error: Option<String>,
//     warning: Option<String>,
//     success: Option<String>,
//     info: Option<String>,
//     emphasis: Option<String>,
//     code: Option<String>,
//     normal: Option<String>,
//     subtle: Option<String>,
//     hint: Option<String>,
//     debug: Option<String>,
//     trace: Option<String>,
// }

// /// Runtime theme palette with resolved styles
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct Palette {
//     /// Heading 1 style
//     pub heading1: Style,
//     /// Heading 2 style
//     pub heading2: Style,
//     /// Heading 3 style
//     pub heading3: Style,
//     /// Error style
//     pub error: Style,
//     /// Warning style
//     pub warning: Style,
//     /// Success style
//     pub success: Style,
//     /// Info style
//     pub info: Style,
//     /// Emphasis style
//     pub emphasis: Style,
//     /// Code style
//     pub code: Style,
//     /// Normal style
//     pub normal: Style,
//     /// Subtle style
//     pub subtle: Style,
//     /// Hint style
//     pub hint: Style,
//     /// Debug style
//     pub debug: Style,
//     /// Trace style
//     pub trace: Style,
// }

// impl Palette {
//     /// Get style for a specific role
//     #[must_use]
//     pub fn style_for_role(&self, role: Role) -> Style {
//         match role {
//             Role::Heading1 => self.heading1.clone(),
//             Role::Heading2 => self.heading2.clone(),
//             Role::Heading3 => self.heading3.clone(),
//             Role::Error => self.error.clone(),
//             Role::Warning => self.warning.clone(),
//             Role::Success => self.success.clone(),
//             Role::Info => self.info.clone(),
//             Role::Emphasis => self.emphasis.clone(),
//             Role::Code => self.code.clone(),
//             Role::Normal => self.normal.clone(),
//             Role::Subtle => self.subtle.clone(),
//             Role::Hint => self.hint.clone(),
//             Role::Debug => self.debug.clone(),
//             Role::Trace => self.trace.clone(),
//         }
//     }
// }

// /// Theme definition loaded from TOML
// #[derive(Clone, Debug, Deserialize)]
// pub struct ThemeDefinition {
//     #[allow(dead_code)]
//     name: String,
//     #[serde(default)]
//     /// Path to the theme file (e.g., "`themes/built_in/dracula.toml`")
//     pub filename: Option<String>,
//     /// Whether this is a built-in theme or a custom theme
//     #[serde(default)]
//     pub is_builtin: bool, // true for built-in themes, false for custom    pub term_bg_luma: TermBgLuma,
//     /// Light or dark background requirement
//     pub term_bg_luma: String,
//     /// Minimum color support required
//     pub min_color_support: String,
//     /// All possible Hex RGB values for theme background
//     pub backgrounds: Vec<String>, // Keep as hex strings in TOML
//     /// Theme description
//     pub description: String,
//     /// Color palette configuration
//     pub palette: PaletteConfig,
// }

// impl ThemeDefinition {
//     /// Get the background luminance requirement
//     #[must_use]
//     pub fn term_bg_luma(&self) -> &str {
//         &self.term_bg_luma
//     }

//     /// Get the minimum color support requirement
//     #[must_use]
//     pub fn min_color_support(&self) -> &str {
//         &self.min_color_support
//     }

//     /// Get the background colors
//     #[must_use]
//     pub fn backgrounds(&self) -> &[String] {
//         &self.backgrounds
//     }
// }

// /// Runtime theme with resolved styles and metadata
// #[derive(Clone, Debug)]
// pub struct Theme {
//     /// Theme name
//     pub name: String,
//     /// Optional filename if loaded from file
//     pub filename: Option<String>,
//     /// Whether this is a built-in theme
//     pub is_builtin: bool,
//     /// Terminal background luminance requirement
//     pub term_bg_luma: TermBgLuma,
//     /// Minimum color support required
//     pub min_color_support: ColorSupport,
//     /// Color palette with resolved styles
//     pub palette: Palette,
//     /// Background colors as hex strings
//     pub backgrounds: Vec<String>,
//     /// Background RGB values
//     pub bg_rgbs: Vec<(u8, u8, u8)>,
//     /// Theme description
//     pub description: String,
// }

// impl Theme {
//     /// Create a default theme for fallback
//     #[must_use]
//     pub const fn default_theme() -> Self {
//         Self {
//             name: String::new(),
//             filename: None,
//             is_builtin: true,
//             term_bg_luma: TermBgLuma::Dark,
//             min_color_support: ColorSupport::Basic,
//             palette: Palette {
//                 heading1: Style::new(),
//                 heading2: Style::new(),
//                 heading3: Style::new(),
//                 error: Style::new(),
//                 warning: Style::new(),
//                 success: Style::new(),
//                 info: Style::new(),
//                 emphasis: Style::new(),
//                 code: Style::new(),
//                 normal: Style::new(),
//                 subtle: Style::new(),
//                 hint: Style::new(),
//                 debug: Style::new(),
//                 trace: Style::new(),
//             },
//             backgrounds: Vec::new(),
//             bg_rgbs: Vec::new(),
//             description: String::new(),
//         }
//     }

//     /// Auto-detect and load appropriate theme
//     ///
//     /// # Errors
//     ///
//     /// TODO: Supposedly will bubble up any errors encountered.
//     pub fn auto_detect<T: StylingConfigProvider>(
//         color_support: ColorSupport,
//         term_bg_luma: TermBgLuma,
//         _provider: &T,
//     ) -> StylingResult<Self> {
//         // For now, return a basic theme
//         // TODO: Implement full theme loading logic
//         Ok(Self::create_basic_theme(color_support, term_bg_luma))
//     }

//     /// Create a basic theme with default colors
//     #[must_use]
//     pub fn create_basic_theme(color_support: ColorSupport, term_bg_luma: TermBgLuma) -> Self {
//         let palette = Palette {
//             heading1: Style::new()
//                 .fg(Color::blue().with_support(color_support))
//                 .bold(),
//             heading2: Style::new()
//                 .fg(Color::cyan().with_support(color_support))
//                 .bold(),
//             heading3: Style::new()
//                 .fg(Color::green().with_support(color_support))
//                 .bold(),
//             error: Style::new()
//                 .fg(Color::red().with_support(color_support))
//                 .bold(),
//             warning: Style::new()
//                 .fg(Color::yellow().with_support(color_support))
//                 .bold(),
//             success: Style::new()
//                 .fg(Color::green().with_support(color_support))
//                 .bold(),
//             info: Style::new().fg(Color::cyan().with_support(color_support)),
//             emphasis: Style::new()
//                 .fg(Color::magenta().with_support(color_support))
//                 .bold(),
//             code: Style::new().fg(Color::light_yellow().with_support(color_support)),
//             normal: Style::new().fg(Color::white().with_support(color_support)),
//             subtle: Style::new().fg(Color::dark_gray().with_support(color_support)),
//             hint: Style::new().fg(Color::light_cyan().with_support(color_support)),
//             debug: Style::new().fg(Color::light_gray().with_support(color_support)),
//             trace: Style::new().fg(Color::dark_gray().with_support(color_support)),
//         };

//         Self {
//             name: "default".to_string(),
//             filename: None,
//             is_builtin: true,
//             term_bg_luma,
//             min_color_support: color_support,
//             palette,
//             backgrounds: Vec::new(),
//             bg_rgbs: Vec::new(),
//             description: "Default built-in theme".to_string(),
//         }
//     }

//     /// Get style for a specific role
//     #[must_use]
//     pub fn style_for(&self, role: Role) -> Style {
//         self.palette.style_for_role(role)
//     }
// }

// /// Convert hex color string to RGB tuple
// fn hex_to_rgb(hex: &str) -> StylingResult<(u8, u8, u8)> {
//     let hex = hex.trim_start_matches('#');
//     if hex.len() != 6 {
//         return Err(StylingError::Generic(
//             "Invalid hex color format".to_string(),
//         ));
//     }

//     let r = u8::from_str_radix(&hex[0..2], 16)
//         .map_err(|_| StylingError::Generic("Invalid hex color format".to_string()))?;
//     let g = u8::from_str_radix(&hex[2..4], 16)
//         .map_err(|_| StylingError::Generic("Invalid hex color format".to_string()))?;
//     let b = u8::from_str_radix(&hex[4..6], 16)
//         .map_err(|_| StylingError::Generic("Invalid hex color format".to_string()))?;

//     Ok((r, g, b))
// }

// /// Find the closest color in the 256-color palette
// #[must_use]
// pub const fn find_closest_color(rgb: &[u8; 3]) -> u8 {
//     // Simple implementation - return a reasonable default
//     // TODO: Implement proper color distance calculation
//     match (rgb[0], rgb[1], rgb[2]) {
//         (r, g, b) if r > 128 && g < 128 && b < 128 => 1, // Red
//         (r, g, b) if r < 128 && g > 128 && b < 128 => 2, // Green
//         (r, g, b) if r > 128 && g > 128 && b < 128 => 3, // Yellow
//         (r, g, b) if r < 128 && g < 128 && b > 128 => 4, // Blue
//         (r, g, b) if r > 128 && g < 128 && b > 128 => 5, // Magenta
//         (r, g, b) if r < 128 && g > 128 && b > 128 => 6, // Cyan
//         (r, g, b) if r > 200 && g > 200 && b > 200 => 7, // White
//         _ => 0,                                          // Black
//     }
// }

// /// Find the closest basic color (0-15)
// #[must_use]
// pub fn find_closest_basic_color(rgb: &[u8; 3]) -> u8 {
//     find_closest_color(rgb).min(15)
// }

// /// Get RGB values for a 256-color index
// #[allow(clippy::match_same_arms)]
// #[must_use]
// pub const fn get_rgb(color256: u8) -> [u8; 3] {
//     // Basic colors (0-15)
//     if color256 < 16 {
//         match color256 {
//             0 => [0, 0, 0],        // Black
//             1 => [128, 0, 0],      // Dark Red
//             2 => [0, 128, 0],      // Dark Green
//             3 => [128, 128, 0],    // Dark Yellow
//             4 => [0, 0, 128],      // Dark Blue
//             5 => [128, 0, 128],    // Dark Magenta
//             6 => [0, 128, 128],    // Dark Cyan
//             7 => [192, 192, 192],  // Light Gray
//             8 => [128, 128, 128],  // Dark Gray
//             9 => [255, 0, 0],      // Red
//             10 => [0, 255, 0],     // Green
//             11 => [255, 255, 0],   // Yellow
//             12 => [0, 0, 255],     // Blue
//             13 => [255, 0, 255],   // Magenta
//             14 => [0, 255, 255],   // Cyan
//             15 => [255, 255, 255], // White
//             _ => [0, 0, 0],        // Fallback
//         }
//     } else {
//         // For 256-color palette, this is a simplified implementation
//         // TODO: Implement proper 256-color to RGB conversion
//         [128, 128, 128] // Gray fallback
//     }
// }
