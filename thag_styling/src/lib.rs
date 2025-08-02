//! Terminal styling system with theme support and color detection for `thag_rs` and third party applications.
//!
//! This crate provides a comprehensive styling system for terminal applications, including:
//! - Color detection and terminal capability assessment
//! - Theme-based styling with semantic roles
//! - Inquire UI integration for consistent theming
//! - ANSI color conversion and optimization
//! - Built-in theme collection

#![warn(clippy::pedantic, missing_docs)]

pub mod styling;

use serde::Deserialize;

use std::sync::atomic::AtomicBool;
use std::sync::OnceLock;
use strum::{Display, EnumIter};

// Re-export common types
pub use thag_common::{ColorSupport, TermBgLuma, ThagCommonResult, Verbosity, V};

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
    /// Generic error with message
    #[error("{0}")]
    Generic(String),
}

/// Theme-related error types
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    /// Theme not found
    #[error("Theme not found: {0}")]
    NotFound(String),
    /// Invalid theme configuration
    #[error("Invalid theme configuration: {0}")]
    InvalidConfig(String),
    /// Theme validation failed
    #[error("Theme validation failed: {0}")]
    ValidationFailed(String),
    /// IO error when loading theme
    #[error("IO error loading theme: {0}")]
    Io(#[from] std::io::Error),
    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
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

#[allow(dead_code)]
const THRESHOLD: f64 = 0.5;

/// Enum representing different color value types for terminal styling
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorValue {
    /// Basic 16-color ANSI support
    Basic {
        /// The basic color index (0-15)
        basic: u8,
    },
    /// 256-color support
    Color256 {
        /// The 256-color index (0-255)
        color256: u8,
    },
    /// 24-bit RGB color support
    TrueColor {
        /// RGB values as [r, g, b]
        rgb: [u8; 3],
    },
}

#[allow(dead_code)]
struct StyleConfig {
    color: String,
    style: Option<String>,
}

/// Information about a color including its value, ANSI code, and index
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorInfo {
    /// The color value (RGB, 256-color, or basic)
    pub value: ColorValue,
    /// The ANSI escape sequence for this color
    pub ansi: String,
    /// The color index for palette lookup
    pub index: u8,
}

impl ColorInfo {
    /// Create a basic color info
    #[must_use]
    pub fn basic(index: u8) -> Self {
        Self {
            value: ColorValue::Basic { basic: index },
            ansi: format!("\x1b[38;5;{index}m"),
            index,
        }
    }

    /// Create a 256-color info
    #[must_use]
    pub fn color256(index: u8) -> Self {
        Self {
            value: ColorValue::Color256 { color256: index },
            ansi: format!("\x1b[38;5;{index}m"),
            index,
        }
    }

    /// Create an RGB color info
    #[must_use]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            value: ColorValue::TrueColor { rgb: [r, g, b] },
            ansi: format!("\x1b[38;2;{r};{g};{b}m"),
            index: 0, // RGB colors don't have a meaningful index
        }
    }

    /// Convert this color to the specified color support level
    #[must_use]
    pub fn with_support(self, support: ColorSupport) -> Self {
        #[allow(clippy::match_same_arms)]
        match support {
            ColorSupport::TrueColor => self,
            ColorSupport::Color256 => match self.value {
                ColorValue::TrueColor { rgb } => {
                    let index = find_closest_color(&rgb);
                    Self::color256(index)
                }
                _ => self,
            },
            ColorSupport::Basic => {
                let index = match self.value {
                    ColorValue::TrueColor { rgb } => find_closest_basic_color(&rgb),
                    ColorValue::Color256 { color256 } => {
                        let rgb = get_rgb(color256);
                        find_closest_basic_color(&rgb)
                    }
                    ColorValue::Basic { .. } => self.index,
                };
                Self::basic(index)
            }
            _ => self,
        }
    }
}

/// A styling configuration that includes foreground color and text attributes
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Style {
    /// Foreground color information
    pub foreground: Option<ColorInfo>,
    /// Bold text attribute
    pub bold: bool,
    /// Italic text attribute
    pub italic: bool,
    /// Dim text attribute
    pub dim: bool,
    /// Underline text attribute
    pub underline: bool,
}

impl Style {
    /// Create a new style with default values
    #[must_use]
    pub const fn new() -> Self {
        Self {
            foreground: None,
            bold: false,
            italic: false,
            dim: false,
            underline: false,
        }
    }

    /// Create a style from a hex color string
    ///
    /// # Errors
    ///
    /// Any errors from calling `hex_to_rgb` on the hex value string.
    pub fn from_fg_hex(hex: &str) -> StylingResult<Self> {
        let (r, g, b) = hex_to_rgb(hex)?;
        Ok(Self {
            foreground: Some(ColorInfo::rgb(r, g, b)),
            bold: false,
            italic: false,
            dim: false,
            underline: false,
        })
    }

    /// Set the foreground color
    #[must_use]
    pub fn fg(mut self, color: ColorInfo) -> Self {
        self.foreground = Some(color);
        self
    }

    /// Make the style bold
    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Make the style italic
    #[must_use]
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Make the style normal (no bold/italic/dim)
    #[must_use]
    pub const fn normal(mut self) -> Self {
        self.bold = false;
        self.italic = false;
        self.dim = false;
        self
    }

    /// Make the style dim
    #[must_use]
    pub const fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Make the style underlined
    #[must_use]
    pub const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Reset all styling
    #[must_use]
    pub fn reset(&self) -> String {
        String::from("\x1b[0m")
    }

    /// Apply this style to a string and return the styled string
    #[must_use]
    pub fn paint<T: AsRef<str>>(&self, text: T) -> String {
        let text = text.as_ref();
        let mut result = String::new();

        // Add color if present
        if let Some(color) = &self.foreground {
            result.push_str(&color.ansi);
        }

        // Add attributes
        if self.bold {
            result.push_str("\x1b[1m");
        }
        if self.italic {
            result.push_str("\x1b[3m");
        }
        if self.dim {
            result.push_str("\x1b[2m");
        }
        if self.underline {
            result.push_str("\x1b[4m");
        }

        result.push_str(text);
        result.push_str("\x1b[0m"); // Reset

        result
    }

    /// Convert this style to work with a specific color support level
    #[must_use]
    pub fn with_color_index(mut self, support: ColorSupport) -> Self {
        if let Some(color) = self.foreground {
            self.foreground = Some(color.with_support(support));
        }
        self
    }

    /// Create a style for a specific role using the current theme
    #[must_use]
    pub fn for_role(role: Role) -> Self {
        TermAttributes::try_get().map_or_else(
            || {
                // Fallback to basic role-based styling without full theme system
                let color_index = role.color_index();
                Self::new().fg(Color::fixed(color_index))
            },
            |term_attrs| term_attrs.theme.style_for(role),
        )
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

/// Color utility struct providing color constants and methods
pub struct Color;

impl Color {
    // Basic color constants
    const BLACK: u8 = 0;
    const RED: u8 = 1;
    const GREEN: u8 = 2;
    const YELLOW: u8 = 3;
    const BLUE: u8 = 4;
    const MAGENTA: u8 = 5;
    const CYAN: u8 = 6;
    const WHITE: u8 = 7;
    // Bright color constants
    #[allow(dead_code)]
    const DARK_GRAY: u8 = 8;
    #[allow(dead_code)]
    const LIGHT_RED: u8 = 9;
    #[allow(dead_code)]
    const LIGHT_GREEN: u8 = 10;
    const LIGHT_YELLOW: u8 = 11;
    #[allow(dead_code)]
    const LIGHT_BLUE: u8 = 12;
    #[allow(dead_code)]
    const LIGHT_MAGENTA: u8 = 13;
    const LIGHT_CYAN: u8 = 14;
    const LIGHT_GRAY: u8 = 15;

    /// Create a black color
    #[must_use]
    pub fn black() -> ColorInfo {
        ColorInfo::basic(Self::BLACK)
    }

    /// Create a red color
    #[must_use]
    pub fn red() -> ColorInfo {
        ColorInfo::basic(Self::RED)
    }

    /// Create a green color
    #[must_use]
    pub fn green() -> ColorInfo {
        ColorInfo::basic(Self::GREEN)
    }

    /// Create a yellow color
    #[must_use]
    pub fn yellow() -> ColorInfo {
        ColorInfo::basic(Self::YELLOW)
    }

    /// Create a blue color
    #[must_use]
    pub fn blue() -> ColorInfo {
        ColorInfo::basic(Self::BLUE)
    }

    /// Create a magenta color
    #[must_use]
    pub fn magenta() -> ColorInfo {
        ColorInfo::basic(Self::MAGENTA)
    }

    /// Create a cyan color
    #[must_use]
    pub fn cyan() -> ColorInfo {
        ColorInfo::basic(Self::CYAN)
    }

    /// Create a white color
    #[must_use]
    pub fn white() -> ColorInfo {
        ColorInfo::basic(Self::WHITE)
    }

    /// Create a dark gray color
    #[must_use]
    pub fn dark_gray() -> ColorInfo {
        ColorInfo::basic(Self::DARK_GRAY)
    }

    /// Create a light yellow color
    #[must_use]
    pub fn light_yellow() -> ColorInfo {
        ColorInfo::basic(Self::LIGHT_YELLOW)
    }

    /// Create a light cyan color
    #[must_use]
    pub fn light_cyan() -> ColorInfo {
        ColorInfo::basic(Self::LIGHT_CYAN)
    }

    /// Create a light gray color
    #[must_use]
    pub fn light_gray() -> ColorInfo {
        ColorInfo::basic(Self::LIGHT_GRAY)
    }

    /// Create a color from a fixed color index
    #[must_use]
    pub fn fixed(index: u8) -> ColorInfo {
        if index < 16 {
            ColorInfo::basic(index)
        } else {
            ColorInfo::color256(index)
        }
    }
}

/// Type alias for styling level convenience
pub type Lvl = Style;

impl Lvl {
    /// Heading 1 style
    pub const HD1: Self = Self::new();
    /// Heading 2 style
    pub const HD2: Self = Self::new();
    /// Heading 3 style
    pub const HD3: Self = Self::new();
    /// Error style
    pub const ERR: Self = Self::new();
    /// Warning style
    pub const WARN: Self = Self::new();
    /// Success style
    pub const SUCC: Self = Self::new();
    /// Info style
    pub const INFO: Self = Self::new();
    /// Emphasis style
    pub const EMPH: Self = Self::new();
    /// Code style
    pub const CODE: Self = Self::new();
    /// Normal style
    pub const NORM: Self = Self::new();
    /// Subtle style
    pub const SUBT: Self = Self::new();
    /// Hint style
    pub const HINT: Self = Self::new();
    /// Debug style
    pub const DBUG: Self = Self::new();
    /// Trace style
    pub const TRCE: Self = Self::new();
}

/// Semantic roles for styling different types of content
#[derive(
    Clone, Copy, Debug, Deserialize, Display, EnumIter, Eq, Hash, PartialEq, PartialOrd, Ord,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Primary headings
    Heading1,
    /// Secondary headings
    Heading2,
    /// Tertiary headings
    Heading3,
    /// Error messages
    Error,
    /// Warning messages
    Warning,
    /// Success messages
    Success,
    /// Informational messages
    Info,
    /// Emphasis/highlighted text
    Emphasis,
    /// Code/monospace text
    Code,
    /// Normal/default text
    Normal,
    /// Subtle/dimmed text
    Subtle,
    /// Hint/help text
    Hint,
    /// Debug messages
    Debug,
    /// Trace messages
    Trace,
}

impl Role {
    /// Get the color index for this role (for fallback purposes)
    #[must_use]
    pub const fn color_index(&self) -> u8 {
        #[allow(clippy::match_same_arms)]
        match self {
            Self::Heading1 => 4, // Blue
            Self::Heading2 => 6, // Cyan
            Self::Heading3 => 2, // Green
            Self::Error => 1,    // Red
            Self::Warning => 3,  // Yellow
            Self::Success => 2,  // Green
            Self::Info => 6,     // Cyan
            Self::Emphasis => 5, // Magenta
            Self::Code => 11,    // Light Yellow
            Self::Normal => 7,   // White/Default
            Self::Subtle => 8,   // Dark Gray
            Self::Hint => 14,    // Light Cyan
            Self::Debug => 13,   // Light Magenta
            Self::Trace => 8,    // Dark Gray
        }
    }
}

impl From<&Role> for u8 {
    fn from(role: &Role) -> Self {
        #[allow(clippy::match_same_arms)]
        match role {
            Role::Heading1 => 4, // Blue
            Role::Heading2 => 6, // Cyan
            Role::Heading3 => 2, // Green
            Role::Error => 1,    // Red
            Role::Warning => 3,  // Yellow
            Role::Success => 2,  // Green
            Role::Info => 6,     // Cyan
            Role::Emphasis => 5, // Magenta
            Role::Code => 11,    // Light Yellow
            Role::Normal => 7,   // White/Default
            Role::Subtle => 8,   // Dark Gray
            Role::Hint => 14,    // Light Cyan
            Role::Debug => 13,   // Light Magenta
            Role::Trace => 8,    // Dark Gray
        }
    }
}

/// Strategy for initializing color settings
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorInitStrategy {
    /// Use configured values from config file
    Configure,
    /// Use default fallback values
    Default,
    /// Auto-detect terminal capabilities
    Match,
}

impl ColorInitStrategy {
    /// Determine the appropriate color initialization strategy
    #[cfg(feature = "color_detect")]
    pub fn determine<T: StylingConfigProvider>(provider: &T) -> (ColorSupport, TermBgLuma) {
        let config_color_support = provider.color_support();
        let config_term_bg_luma = provider.term_bg_luma();

        let detected_color_support = detect_color_support();
        let detected_term_bg_luma = detect_terminal_background();

        let final_color_support = match config_color_support {
            ColorSupport::Undetermined => detected_color_support,
            _ => config_color_support,
        };

        let final_term_bg_luma = match config_term_bg_luma {
            TermBgLuma::Undetermined => detected_term_bg_luma,
            _ => config_term_bg_luma,
        };

        (final_color_support, final_term_bg_luma)
    }

    /// Fallback determination without color detection
    #[cfg(not(feature = "color_detect"))]
    pub fn determine<T: StylingConfigProvider>(provider: &T) -> (ColorSupport, TermBgLuma) {
        let config_color_support = provider.color_support();
        let config_term_bg_luma = provider.term_bg_luma();

        let final_color_support = match config_color_support {
            ColorSupport::Undetermined => ColorSupport::Basic,
            _ => config_color_support,
        };

        let final_term_bg_luma = match config_term_bg_luma {
            TermBgLuma::Undetermined => TermBgLuma::Dark,
            _ => config_term_bg_luma,
        };

        (final_color_support, final_term_bg_luma)
    }
}

/// Detect color support using supports-color crate
#[cfg(feature = "color_detect")]
fn detect_color_support() -> ColorSupport {
    use supports_color::Stream;

    supports_color::on(Stream::Stdout).map_or(ColorSupport::None, |level| {
        if level.has_16m {
            ColorSupport::TrueColor
        } else if level.has_256 {
            ColorSupport::Color256
        } else if level.has_basic {
            ColorSupport::Basic
        } else {
            ColorSupport::None
        }
    })
}

/// Detect terminal background using termbg crate
#[cfg(feature = "color_detect")]
fn detect_terminal_background() -> TermBgLuma {
    #[allow(clippy::match_same_arms)]
    match termbg::theme(std::time::Duration::from_millis(100)) {
        Ok(termbg::Theme::Light) => TermBgLuma::Light,
        Ok(termbg::Theme::Dark) | Err(_) => TermBgLuma::Dark, // Default to dark on detection failure
    }
}

/// How the terminal attributes were initialized
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HowInitialized {
    /// From configuration file
    Configured,
    /// Using default values
    Defaulted,
    /// Auto-detected from terminal
    Detected,
}

/// Terminal attributes including color support and theme information
#[derive(Clone, Debug)]
pub struct TermAttributes {
    /// How these attributes were initialized
    pub how_initialized: HowInitialized,
    /// Color support level
    pub color_support: ColorSupport,
    /// Terminal background color in hex format
    pub term_bg_hex: Option<String>,
    /// Terminal background RGB values
    pub term_bg_rgb: Option<(u8, u8, u8)>,
    /// Terminal background luminance
    pub term_bg_luma: TermBgLuma,
    /// Current theme
    pub theme: Theme,
}

static INSTANCE: OnceLock<TermAttributes> = OnceLock::new();

/// Global flag for logging enablement
pub static LOGGING_ENABLED: AtomicBool = AtomicBool::new(false);

impl TermAttributes {
    #[allow(dead_code)]
    const fn new() -> Self {
        Self {
            how_initialized: HowInitialized::Defaulted,
            color_support: ColorSupport::Basic,
            term_bg_hex: None,
            term_bg_rgb: None,
            term_bg_luma: TermBgLuma::Dark,
            theme: Theme::default_theme(),
        }
    }

    /// Initialize terminal attributes with a config provider
    ///
    /// # Errors
    ///
    /// Any auto-detect errors.
    pub fn initialize<T: StylingConfigProvider>(provider: &T) -> StylingResult<&'static Self> {
        let (color_support, term_bg_luma) = ColorInitStrategy::determine(provider);

        let theme = Theme::auto_detect(color_support, term_bg_luma, provider)?;

        let instance = Self {
            how_initialized: HowInitialized::Detected,
            color_support,
            term_bg_hex: None,
            term_bg_rgb: None,
            term_bg_luma,
            theme,
        };

        Ok(INSTANCE.get_or_init(|| instance))
    }

    /// Check if terminal attributes have been initialized
    #[must_use]
    pub fn is_initialized() -> bool {
        INSTANCE.get().is_some()
    }

    /// Try to get terminal attributes if initialized
    #[must_use]
    pub fn try_get() -> Option<&'static Self> {
        INSTANCE.get()
    }

    /// Get terminal attributes or initialize with default config
    #[must_use]
    pub fn get_or_init() -> &'static Self {
        INSTANCE.get_or_init(|| {
            let provider = NoConfigProvider;
            let (color_support, term_bg_luma) = ColorInitStrategy::determine(&provider);

            // Create theme directly without calling auto_detect to avoid recursion
            let theme = Theme::create_basic_theme(color_support, term_bg_luma);

            Self {
                how_initialized: HowInitialized::Defaulted,
                color_support,
                term_bg_hex: None,
                term_bg_rgb: None,
                term_bg_luma,
                theme,
            }
        })
    }

    /// Create a new instance with a different theme
    #[must_use]
    pub fn with_theme(&self, theme: Theme) -> Self {
        Self {
            theme,
            ..self.clone()
        }
    }

    /// Create a new instance with different color support
    #[must_use]
    pub fn with_color_support(&self, color_support: ColorSupport) -> Self {
        Self {
            color_support,
            how_initialized: HowInitialized::Configured,
            term_bg_hex: self.term_bg_hex.clone(),
            term_bg_rgb: self.term_bg_rgb,
            term_bg_luma: self.term_bg_luma,
            theme: self.theme.clone(),
        }
    }
}

/// Paint text with a specific role using the current theme
#[must_use]
pub fn paint_for_role<T: AsRef<str>>(role: Role, text: T) -> String {
    let term_attrs = TermAttributes::get_or_init();
    let style = term_attrs.theme.style_for(role);
    style.paint(text)
}

/// Get a style for a specific role and theme
#[must_use]
pub fn style_for_theme_and_role(theme: &Theme, role: Role) -> Style {
    theme.style_for(role)
}

/// Theme palette configuration from TOML
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PaletteConfig {
    heading1: Option<String>,
    heading2: Option<String>,
    heading3: Option<String>,
    error: Option<String>,
    warning: Option<String>,
    success: Option<String>,
    info: Option<String>,
    emphasis: Option<String>,
    code: Option<String>,
    normal: Option<String>,
    subtle: Option<String>,
    hint: Option<String>,
    debug: Option<String>,
    trace: Option<String>,
}

/// Runtime theme palette with resolved styles
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Heading 1 style
    pub heading1: Style,
    /// Heading 2 style
    pub heading2: Style,
    /// Heading 3 style
    pub heading3: Style,
    /// Error style
    pub error: Style,
    /// Warning style
    pub warning: Style,
    /// Success style
    pub success: Style,
    /// Info style
    pub info: Style,
    /// Emphasis style
    pub emphasis: Style,
    /// Code style
    pub code: Style,
    /// Normal style
    pub normal: Style,
    /// Subtle style
    pub subtle: Style,
    /// Hint style
    pub hint: Style,
    /// Debug style
    pub debug: Style,
    /// Trace style
    pub trace: Style,
}

impl Palette {
    /// Get style for a specific role
    #[must_use]
    pub fn style_for_role(&self, role: Role) -> Style {
        match role {
            Role::Heading1 => self.heading1.clone(),
            Role::Heading2 => self.heading2.clone(),
            Role::Heading3 => self.heading3.clone(),
            Role::Error => self.error.clone(),
            Role::Warning => self.warning.clone(),
            Role::Success => self.success.clone(),
            Role::Info => self.info.clone(),
            Role::Emphasis => self.emphasis.clone(),
            Role::Code => self.code.clone(),
            Role::Normal => self.normal.clone(),
            Role::Subtle => self.subtle.clone(),
            Role::Hint => self.hint.clone(),
            Role::Debug => self.debug.clone(),
            Role::Trace => self.trace.clone(),
        }
    }
}

/// Theme definition loaded from TOML
#[derive(Clone, Debug, Deserialize)]
pub struct ThemeDefinition {
    #[allow(dead_code)]
    name: String,
    #[serde(default)]
    /// Path to the theme file (e.g., "`themes/built_in/dracula.toml`")
    pub filename: Option<String>,
    /// Whether this is a built-in theme or a custom theme
    #[serde(default)]
    pub is_builtin: bool, // true for built-in themes, false for custom    pub term_bg_luma: TermBgLuma,
    /// Light or dark background requirement
    pub term_bg_luma: String,
    /// Minimum color support required
    pub min_color_support: String,
    /// All possible Hex RGB values for theme background
    pub backgrounds: Vec<String>, // Keep as hex strings in TOML
    /// Theme description
    pub description: String,
    /// Color palette configuration
    pub palette: PaletteConfig,
}

impl ThemeDefinition {
    /// Get the background luminance requirement
    #[must_use]
    pub fn term_bg_luma(&self) -> &str {
        &self.term_bg_luma
    }

    /// Get the minimum color support requirement
    #[must_use]
    pub fn min_color_support(&self) -> &str {
        &self.min_color_support
    }

    /// Get the background colors
    #[must_use]
    pub fn backgrounds(&self) -> &[String] {
        &self.backgrounds
    }
}

/// Runtime theme with resolved styles and metadata
#[derive(Clone, Debug)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Optional filename if loaded from file
    pub filename: Option<String>,
    /// Whether this is a built-in theme
    pub is_builtin: bool,
    /// Terminal background luminance requirement
    pub term_bg_luma: TermBgLuma,
    /// Minimum color support required
    pub min_color_support: ColorSupport,
    /// Color palette with resolved styles
    pub palette: Palette,
    /// Background colors as hex strings
    pub backgrounds: Vec<String>,
    /// Background RGB values
    pub bg_rgbs: Vec<(u8, u8, u8)>,
    /// Theme description
    pub description: String,
}

impl Theme {
    /// Create a default theme for fallback
    #[must_use]
    pub const fn default_theme() -> Self {
        Self {
            name: String::new(),
            filename: None,
            is_builtin: true,
            term_bg_luma: TermBgLuma::Dark,
            min_color_support: ColorSupport::Basic,
            palette: Palette {
                heading1: Style::new(),
                heading2: Style::new(),
                heading3: Style::new(),
                error: Style::new(),
                warning: Style::new(),
                success: Style::new(),
                info: Style::new(),
                emphasis: Style::new(),
                code: Style::new(),
                normal: Style::new(),
                subtle: Style::new(),
                hint: Style::new(),
                debug: Style::new(),
                trace: Style::new(),
            },
            backgrounds: Vec::new(),
            bg_rgbs: Vec::new(),
            description: String::new(),
        }
    }

    /// Auto-detect and load appropriate theme
    ///
    /// # Errors
    ///
    /// TODO: Supposedly will bubble up any errors encountered.
    pub fn auto_detect<T: StylingConfigProvider>(
        color_support: ColorSupport,
        term_bg_luma: TermBgLuma,
        _provider: &T,
    ) -> StylingResult<Self> {
        // For now, return a basic theme
        // TODO: Implement full theme loading logic
        Ok(Self::create_basic_theme(color_support, term_bg_luma))
    }

    /// Create a basic theme with default colors
    #[must_use]
    pub fn create_basic_theme(color_support: ColorSupport, term_bg_luma: TermBgLuma) -> Self {
        let palette = Palette {
            heading1: Style::new()
                .fg(Color::blue().with_support(color_support))
                .bold(),
            heading2: Style::new()
                .fg(Color::cyan().with_support(color_support))
                .bold(),
            heading3: Style::new()
                .fg(Color::green().with_support(color_support))
                .bold(),
            error: Style::new()
                .fg(Color::red().with_support(color_support))
                .bold(),
            warning: Style::new()
                .fg(Color::yellow().with_support(color_support))
                .bold(),
            success: Style::new()
                .fg(Color::green().with_support(color_support))
                .bold(),
            info: Style::new().fg(Color::cyan().with_support(color_support)),
            emphasis: Style::new()
                .fg(Color::magenta().with_support(color_support))
                .bold(),
            code: Style::new().fg(Color::light_yellow().with_support(color_support)),
            normal: Style::new().fg(Color::white().with_support(color_support)),
            subtle: Style::new().fg(Color::dark_gray().with_support(color_support)),
            hint: Style::new().fg(Color::light_cyan().with_support(color_support)),
            debug: Style::new().fg(Color::light_gray().with_support(color_support)),
            trace: Style::new().fg(Color::dark_gray().with_support(color_support)),
        };

        Self {
            name: "default".to_string(),
            filename: None,
            is_builtin: true,
            term_bg_luma,
            min_color_support: color_support,
            palette,
            backgrounds: Vec::new(),
            bg_rgbs: Vec::new(),
            description: "Default built-in theme".to_string(),
        }
    }

    /// Get style for a specific role
    #[must_use]
    pub fn style_for(&self, role: Role) -> Style {
        self.palette.style_for_role(role)
    }
}

/// Convert hex color string to RGB tuple
fn hex_to_rgb(hex: &str) -> StylingResult<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(StylingError::Generic(
            "Invalid hex color format".to_string(),
        ));
    }

    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| StylingError::Generic("Invalid hex color format".to_string()))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| StylingError::Generic("Invalid hex color format".to_string()))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| StylingError::Generic("Invalid hex color format".to_string()))?;

    Ok((r, g, b))
}

/// Find the closest color in the 256-color palette
#[must_use]
pub const fn find_closest_color(rgb: &[u8; 3]) -> u8 {
    // Simple implementation - return a reasonable default
    // TODO: Implement proper color distance calculation
    match (rgb[0], rgb[1], rgb[2]) {
        (r, g, b) if r > 128 && g < 128 && b < 128 => 1, // Red
        (r, g, b) if r < 128 && g > 128 && b < 128 => 2, // Green
        (r, g, b) if r > 128 && g > 128 && b < 128 => 3, // Yellow
        (r, g, b) if r < 128 && g < 128 && b > 128 => 4, // Blue
        (r, g, b) if r > 128 && g < 128 && b > 128 => 5, // Magenta
        (r, g, b) if r < 128 && g > 128 && b > 128 => 6, // Cyan
        (r, g, b) if r > 200 && g > 200 && b > 200 => 7, // White
        _ => 0,                                          // Black
    }
}

/// Find the closest basic color (0-15)
#[must_use]
pub fn find_closest_basic_color(rgb: &[u8; 3]) -> u8 {
    find_closest_color(rgb).min(15)
}

/// Get RGB values for a 256-color index
#[allow(clippy::match_same_arms)]
#[must_use]
pub const fn get_rgb(color256: u8) -> [u8; 3] {
    // Basic colors (0-15)
    if color256 < 16 {
        match color256 {
            0 => [0, 0, 0],        // Black
            1 => [128, 0, 0],      // Dark Red
            2 => [0, 128, 0],      // Dark Green
            3 => [128, 128, 0],    // Dark Yellow
            4 => [0, 0, 128],      // Dark Blue
            5 => [128, 0, 128],    // Dark Magenta
            6 => [0, 128, 128],    // Dark Cyan
            7 => [192, 192, 192],  // Light Gray
            8 => [128, 128, 128],  // Dark Gray
            9 => [255, 0, 0],      // Red
            10 => [0, 255, 0],     // Green
            11 => [255, 255, 0],   // Yellow
            12 => [0, 0, 255],     // Blue
            13 => [255, 0, 255],   // Magenta
            14 => [0, 255, 255],   // Cyan
            15 => [255, 255, 255], // White
            _ => [0, 0, 0],        // Fallback
        }
    } else {
        // For 256-color palette, this is a simplified implementation
        // TODO: Implement proper 256-color to RGB conversion
        [128, 128, 128] // Gray fallback
    }
}
