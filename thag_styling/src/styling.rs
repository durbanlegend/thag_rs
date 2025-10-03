use crate::{StylingError, StylingResult, ThemeError};

// Type alias for compatibility with PaletteMethods proc macro
type ThagResult<T> = StylingResult<T>;

use serde::Deserialize;
use std::clone::Clone;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::OnceLock;
use strum::{Display, EnumIter, IntoEnumIterator};
use thag_common::{vprtln, ColorSupport, TermBgLuma, V};
use thag_proc_macros::{preload_themes, PaletteMethods};

#[cfg(feature = "color_detect")]
use thag_common::terminal::{self, is_light_color};

#[cfg(feature = "config")]
use crate::StylingConfigProvider;

#[cfg(feature = "config")]
use thag_common::config::maybe_config;

// #[cfg(feature = "ratatui_support")]
#[cfg(feature = "config")]
/// Implementation that uses the actual config
pub struct ConfigProvider;

#[cfg(feature = "config")]
impl StylingConfigProvider for ConfigProvider {
    fn color_support(&self) -> ColorSupport {
        maybe_config()
            .map(|c| c.styling.color_support)
            .unwrap_or_default()
    }

    fn term_bg_luma(&self) -> TermBgLuma {
        maybe_config()
            .map(|c| c.styling.term_bg_luma)
            .unwrap_or_default()
    }

    fn term_bg_rgb(&self) -> Option<[u8; 3]> {
        maybe_config().and_then(|c| c.styling.term_bg_rgb)
    }

    fn backgrounds(&self) -> Vec<String> {
        maybe_config()
            .map(|c| c.styling.backgrounds)
            .unwrap_or_default()
    }

    fn preferred_light(&self) -> Vec<String> {
        maybe_config()
            .map(|c| c.styling.preferred_light)
            .unwrap_or_default()
    }

    fn preferred_dark(&self) -> Vec<String> {
        maybe_config()
            .map(|c| c.styling.preferred_dark)
            .unwrap_or_default()
    }
}

#[allow(unused_imports)]
#[cfg(debug_assertions)]
use thag_common::debug_log;

// Include the generated theme data
// include!(concat!(env!("OUT_DIR"), "/theme_data.rs"));

// #[cfg(feature = "color_detect")]
#[cfg(feature = "config")]
const THRESHOLD: f32 = 30.0; // Adjust this value as needed

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
/// Represents different color value formats for terminal styling
pub enum ColorValue {
    /// Basic ANSI color with code and index
    Basic {
        /// Basic color index
        index: u8,
    },
    /// 256-color palette index
    Color256 {
        /// Color index in the 256-color palette
        color256: u8,
    }, // 256-color index
    /// True color RGB values
    TrueColor {
        /// RGB color values as [red, green, blue]
        rgb: [u8; 3],
    }, // RGB values
}

#[derive(Clone, Debug, Deserialize)]
struct StyleConfig {
    #[serde(flatten)]
    color: ColorValue,
    #[serde(default)]
    style: Vec<String>, // ["bold".to_string(), "italic".to_string(), etc.]
}

/// Contains color information including the color value, ANSI escape sequence, and palette index
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorInfo {
    /// The color value in one of the supported formats (`Basic`, `Color256`, or `TrueColor`)
    pub value: ColorValue,
    /// The color palette index (0-255 for indexed colors, or closest match for RGB)
    pub index: u8,
}

impl ColorInfo {
    /// Creates a new `ColorInfo` with basic ANSI color format
    ///
    /// # Arguments
    /// * `ansi` - The ANSI escape sequence for this color (unused, kept for compatibility)
    /// * `index` - The color palette index (0-15 for basic colors)
    #[must_use]
    pub const fn basic(_ansi: &str, index: u8) -> Self {
        Self {
            value: ColorValue::Basic { index },
            index,
        }
    }

    /// Creates a new `ColorInfo` with 256-color palette format
    ///
    /// # Arguments
    /// * `index` - The color index in the 256-color palette (0-255)
    #[must_use]
    pub const fn color256(index: u8) -> Self {
        Self {
            value: ColorValue::Color256 { color256: index },
            index,
        }
    }

    /// Creates a new `ColorInfo` with true color RGB format
    ///
    /// # Arguments
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    #[must_use]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            value: ColorValue::TrueColor { rgb: [r, g, b] },
            index: find_closest_color([r, g, b]),
        }
    }

    /// Creates appropriate `ColorInfo` based on terminal color support level
    ///
    /// # Arguments
    /// * `rgb` - RGB color values as an array [r, g, b]
    /// * `support` - The color support level of the terminal
    #[must_use]
    pub fn with_support(rgb: [u8; 3], support: ColorSupport) -> Self {
        match support {
            ColorSupport::TrueColor => Self::rgb(rgb[0], rgb[1], rgb[2]),
            ColorSupport::Color256 => Self::color256(find_closest_color(rgb)),
            _ => Self::color256(find_closest_basic_color(rgb)),
        }
    }

    /// Generates the appropriate ANSI escape sequence for this color based on terminal support
    ///
    /// # Arguments
    /// * `support` - The color support level of the terminal
    #[must_use]
    pub fn to_ansi_for_support(&self, support: ColorSupport) -> String {
        match (&self.value, support) {
            // TrueColor support - use RGB values directly
            (ColorValue::TrueColor { rgb }, ColorSupport::TrueColor) => {
                format!("\x1b[38;2;{};{};{}m", rgb[0], rgb[1], rgb[2])
            }
            // TrueColor color but limited support - use closest color index
            (ColorValue::TrueColor { .. }, ColorSupport::Color256) => {
                format!("\x1b[38;5;{}m", self.index)
            }
            (ColorValue::TrueColor { .. }, ColorSupport::Basic | ColorSupport::Undetermined) => {
                let basic_index = if self.index > 15 {
                    self.index % 16
                } else {
                    self.index
                };
                let code = if basic_index <= 7 {
                    basic_index + 30
                } else {
                    basic_index + 90 - 8
                };
                format!("\x1b[{}m", code)
            }
            // 256-color support
            (
                ColorValue::Color256 { color256 },
                ColorSupport::TrueColor | ColorSupport::Color256,
            ) => {
                format!("\x1b[38;5;{}m", color256)
            }
            (
                ColorValue::Color256 { color256 },
                ColorSupport::Basic | ColorSupport::Undetermined,
            ) => {
                let basic_index = if *color256 > 15 {
                    color256 % 16
                } else {
                    *color256
                };
                let code = if basic_index <= 7 {
                    basic_index + 30
                } else {
                    basic_index + 90 - 8
                };
                format!("\x1b[{}m", code)
            }
            // Basic color support
            (ColorValue::Basic { index, .. }, _) => {
                let code = if *index <= 7 {
                    index + 30
                } else {
                    index + 90 - 8
                };
                format!("\x1b[{}m", code)
            }
            // No color support
            (_, ColorSupport::None) => String::new(),
        }
    }
}

/// A foreground style for message styling.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Style {
    /// Optional foreground color information for this style
    pub foreground: Option<ColorInfo>,
    /// Whether this style should be rendered in bold
    pub bold: bool,
    /// Whether this style should be rendered in italic
    pub italic: bool,
    /// Whether this style should be rendered dimmed/faint
    pub dim: bool,
    /// Whether this style should be rendered with underline
    pub underline: bool,
}

impl Style {
    /// Creates a new Style with default values (no formatting)
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

    // Used by proc macro palette_methods.
    fn from_config(config: &StyleConfig) -> StylingResult<Self> {
        let mut style = match &config.color {
            ColorValue::Basic { index, .. } => {
                let index = *index;
                let ansi = basic_index_to_ansi(index);
                Self::fg(ColorInfo::basic(&ansi, index))
            }
            ColorValue::Color256 { color256 } => Self::fg(ColorInfo::color256(*color256)),
            ColorValue::TrueColor { rgb } => {
                let mut color_info = ColorInfo::rgb(rgb[0], rgb[1], rgb[2]);
                color_info.index = find_closest_color(*rgb);
                Self::fg(color_info)
            }
        };

        // Apply additional styles
        for s in &config.style {
            match s.as_str() {
                "bold" => style = style.bold(),
                "italic" => style = style.italic(),
                "dim" => style = style.dim(),
                "underline" => style = style.underline(),
                _ => return Err(ThemeError::InvalidStyle(s.clone()).into()),
            }
        }

        Ok(style)
    }

    /// Initializes and returns a `Style` from a foreground color expressed as a hex RGB value.
    ///
    /// # Errors
    ///
    /// This function will return an error if it encounters an invalid hex RGB value.
    #[cfg(feature = "config")]
    pub fn from_fg_hex(hex: &str) -> StylingResult<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                let mut color_info = ColorInfo::rgb(r, g, b);
                color_info.index = find_closest_color([r, g, b]);
                Ok(Self::fg(color_info))
            } else {
                Err(StylingError::Parse)
            }
        } else {
            Err(StylingError::Parse)
        }
    }

    /// Creates a new Style with the specified foreground color
    #[must_use]
    pub fn fg(color_info: ColorInfo) -> Self {
        Self {
            foreground: Some(color_info),
            ..Default::default()
        }
    }

    /// Returns the Style with RGB foreground color
    #[must_use]
    pub fn with_rgb(rgb: [u8; 3]) -> Self {
        let mut color_info = ColorInfo::rgb(rgb[0], rgb[1], rgb[2]);
        color_info.index = find_closest_color(rgb);
        Self {
            foreground: Some(color_info),
            ..Default::default()
        }
    }

    /// Returns the foreground RGB value as a wrapped u8 array if it exists.
    #[must_use]
    pub fn rgb(&self) -> Option<[u8; 3]> {
        let Some(fg) = &self.foreground else {
            return None;
        };
        match &fg.value {
            ColorValue::Basic { index, .. } => {
                // Return nominal value unless we interrogate terminal
                let index_to_rgb = index_to_rgb(*index);
                // index_to_rgb.map(|(r, g, b)| [r, g, b])
                Some(index_to_rgb)
            }
            ColorValue::Color256 { color256 } => Some(index_to_rgb(*color256)), // .map(|(r, g, b)| [r, g, b]),
            ColorValue::TrueColor { rgb } => Some(*rgb),
        }
    }

    /// Returns the Style with bold formatting enabled
    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Returns the Style with italic formatting enabled
    #[must_use]
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Returns the Style unchanged (used for method chaining)
    #[must_use]
    pub const fn normal(self) -> Self {
        self
    }

    /// Returns the Style with dim/faint formatting enabled
    #[must_use]
    pub const fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Returns the Style with underline formatting enabled
    #[must_use]
    pub const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Resets all text formatting flags to their default (false) state
    #[allow(clippy::missing_const_for_fn)]
    pub fn reset(&mut self) {
        self.bold = false;
        self.italic = false;
        self.dim = false;
        self.underline = false;
    }

    /// Applies this style's formatting to the given displayable value and returns the formatted string
    pub fn paint<D>(&self, val: D) -> String
    where
        D: std::fmt::Display,
    {
        // vprtln!(V::VV, "self.foreground={:#?}", self.foreground);
        if self.foreground.is_none() {
            return val.to_string();
        }

        if !self.bold && !self.italic && !self.underline && !self.dim {
            if let Some(ref color_info) = self.foreground {
                if color_info.index == 0 {
                    return val.to_string();
                }
            }
        }

        let mut result = String::new();
        let mut needs_reset = false;
        let mut full_reset = false;
        let mut reset_string: String = String::new();

        if let Some(color_info) = &self.foreground {
            let ansi = color_info.to_ansi_for_support(TermAttributes::current().color_support);
            result.push_str(&ansi);
            needs_reset = true;
            full_reset = true;
            reset_string.push_str("\x1b[0m");
        }
        if self.bold {
            result.push_str("\x1b[1m");
            needs_reset = true;
            if !full_reset {
                reset_string.push_str("\x1b[22m");
            }
        }
        if self.italic {
            result.push_str("\x1b[3m");
            needs_reset = true;
            if !full_reset {
                reset_string.push_str("\x1b[23m");
            }
        }
        if self.dim {
            result.push_str("\x1b[2m");
            needs_reset = true;
            if !full_reset {
                reset_string.push_str("\x1b[22m");
            }
        }
        if self.underline {
            result.push_str("\x1b[4m");
            needs_reset = true;
            if !full_reset {
                reset_string.push_str("\x1b[24m");
            }
        }

        result.push_str(&val.to_string());

        if needs_reset {
            result.push_str(&reset_string);
        }

        result
    }

    /// Creates a new Style with the specified 256-color palette index
    #[must_use]
    pub fn with_color_index(index: u8) -> Self {
        Self {
            foreground: Some(ColorInfo::color256(index)),
            ..Default::default()
        }
    }

    #[must_use]
    /// Get the `Style` for a `Role` from the currently loaded theme.
    pub fn for_role(role: Role) -> Self {
        TermAttributes::current().theme.style_for(role)
    }

    /// Generate only the ANSI codes for this style without any content or reset
    /// This is used for restoring outer styles after embedded content
    #[must_use]
    pub fn ansi_codes(&self) -> String {
        let mut result = String::new();

        if let Some(color_info) = &self.foreground {
            let ansi = color_info.to_ansi_for_support(TermAttributes::get_or_init().color_support);
            result.push_str(&ansi);
        }
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

        result
    }
}

/// Use the index directly to get the `AnsiCode`
#[must_use]
pub fn basic_index_to_ansi(index: u8) -> String {
    let code = if index <= 7 {
        index + 30
    } else {
        index + 90 - 8
    };
    format!("\x1b[{code}m")
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Role> for Style {
    fn from(role: Role) -> Self {
        TermAttributes::get_or_init().theme.style_for(role)
    }
}

/// Provides static methods for creating basic ANSI color styles
pub struct Color;

#[allow(dead_code)]
impl Color {
    // Basic ANSI 16 colors (indices 0-15)
    const BLACK: &'static str = "\x1b[30m"; // index 0
    const RED: &'static str = "\x1b[31m"; // index 1
    const GREEN: &'static str = "\x1b[32m"; // index 2
    const YELLOW: &'static str = "\x1b[33m"; // index 3
    const BLUE: &'static str = "\x1b[34m"; // index 4
    const MAGENTA: &'static str = "\x1b[35m"; // index 5
    const CYAN: &'static str = "\x1b[36m"; // index 6
    const WHITE: &'static str = "\x1b[37m"; // index 7

    // Bright colors (indices 8-15)
    const DARK_GRAY: &'static str = "\x1b[90m"; // index 8
    const LIGHT_RED: &'static str = "\x1b[91m"; // index 9
    const LIGHT_GREEN: &'static str = "\x1b[92m"; // index 10
    const LIGHT_YELLOW: &'static str = "\x1b[93m"; // index 11
    const LIGHT_BLUE: &'static str = "\x1b[94m"; // index 12
    const LIGHT_MAGENTA: &'static str = "\x1b[95m"; // index 13
    const LIGHT_CYAN: &'static str = "\x1b[96m"; // index 14
    const LIGHT_GRAY: &'static str = "\x1b[97m"; // index 15

    /// Creates a new Style with black foreground color
    #[must_use]
    pub fn black() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::BLACK, 0)),
            ..Default::default()
        }
    }

    /// Creates a new Style with red foreground color
    #[must_use]
    pub fn red() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::RED, 1)),
            ..Default::default()
        }
    }

    /// Creates a new Style with green foreground color
    #[must_use]
    pub fn green() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::GREEN, 2)),
            ..Default::default()
        }
    }

    /// Creates a new Style with yellow foreground color
    #[must_use]
    pub fn yellow() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::YELLOW, 3)),
            ..Default::default()
        }
    }

    /// Creates a new Style with blue foreground color
    #[must_use]
    pub fn blue() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::BLUE, 4)),
            ..Default::default()
        }
    }

    /// Creates a new Style with magenta foreground color
    #[must_use]
    pub fn magenta() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::MAGENTA, 5)),
            ..Default::default()
        }
    }

    /// Creates a new Style with cyan foreground color
    #[must_use]
    pub fn cyan() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::CYAN, 6)),
            ..Default::default()
        }
    }

    /// Creates a new Style with white foreground color
    #[must_use]
    pub fn white() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::WHITE, 7)),
            ..Default::default()
        }
    }

    /// Creates a new Style with dark gray foreground color
    #[must_use]
    pub fn dark_gray() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::DARK_GRAY, 8)),
            ..Default::default()
        }
    }

    /// Creates a new Style with light yellow foreground color
    #[must_use]
    pub fn light_yellow() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::LIGHT_YELLOW, 11)),
            ..Default::default()
        }
    }

    /// Creates a new Style with light cyan foreground color
    #[must_use]
    pub fn light_cyan() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::LIGHT_CYAN, 14)),
            ..Default::default()
        }
    }

    /// Creates a new Style with light gray foreground color
    #[must_use]
    pub fn light_gray() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::LIGHT_GRAY, 15)),
            ..Default::default()
        }
    }

    /// Creates a new Style with the specified color index
    ///
    /// # Arguments
    /// * `index` - Color index (0-15 for basic colors, 16-255 for extended colors)
    #[must_use]
    pub fn fixed(index: u8) -> Style {
        if index < 16 {
            // Basic colours
            // Use the index directly to get the AnsiCode
            let code = if index <= 7 {
                index + 30
            } else {
                index + 90 - 8
            };
            let ansi = format!("\x1b[{code}m");
            Style::fg(ColorInfo::basic(&ansi, index))
        } else {
            Style::fg(ColorInfo::color256(index))
        }
    }
}

impl Role {
    /// Short alias for `Role::Heading1`
    pub const HD1: Self = Self::Heading1;
    /// Short alias for `Role::Heading2`
    pub const HD2: Self = Self::Heading2;
    /// Short alias for `Role::Heading3`
    pub const HD3: Self = Self::Heading3;
    /// Short alias for `Role::Error`
    pub const ERR: Self = Self::Error;
    /// Short alias for `Role::Warning`
    pub const WARN: Self = Self::Warning;
    /// Short alias for `Role::Success`
    pub const SUCC: Self = Self::Success;
    /// Short alias for `Role::Info`
    pub const INFO: Self = Self::Info;
    /// Short alias for `Role::Emphasis`
    pub const EMPH: Self = Self::Emphasis;
    /// Short alias for `Role::Code`
    pub const CODE: Self = Self::Code;
    /// Short alias for `Role::Normal`
    pub const NORM: Self = Self::Normal;
    /// Short alias for `Role::Subtle`
    pub const SUBT: Self = Self::Subtle;
    /// Short alias for `Role::Hint`
    pub const HINT: Self = Self::Hint;
    /// Short alias for `Role::Debug`
    pub const DBUG: Self = Self::Debug;
    /// Short alias for `Role::Link`
    pub const LINK: Self = Self::Link;
    /// Short alias for `Role::Quote`
    pub const QUOT: Self = Self::Quote;
    /// Short alias for `Role::Commentary`
    pub const COMM: Self = Self::Commentary;
}

impl Role {
    /// Returns the color index for this role from the current theme
    ///
    /// Gets the style for this role from the currently loaded theme and returns
    /// the color index from its foreground color. If no foreground color is set,
    /// returns 7 (white) as a fallback.
    ///
    /// # Returns
    /// The color palette index (0-255) for this role's foreground color
    #[must_use]
    pub fn color_index(&self) -> u8 {
        let style = Style::for_role(*self);
        style.foreground.map_or(7, |color_info| color_info.index) // 7 = white as fallback
    }
}

// We can implement conversions to u8 directly here
impl From<&Role> for u8 {
    fn from(role: &Role) -> Self {
        role.color_index()
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
/// Strategies for initializing terminal color attributes and theme selection
///
/// This enum defines different approaches for determining the appropriate color support
/// and theme based on terminal capabilities, configuration, or defaults.
pub enum ColorInitStrategy {
    /// Use explicitly configured color support, background luminance, and optional background RGB
    ///
    /// Parameters:
    /// - `ColorSupport`: The configured color support level
    /// - `TermBgLuma`: The configured background luminance (light/dark)
    /// - `Option<[u8; 3]>`: Optional RGB values for the background color
    Configure(ColorSupport, TermBgLuma, Option<[u8; 3]>),
    /// Use safe default values without detection or configuration
    ///
    /// Falls back to basic color support with dark background theme
    Default,
    // #[cfg(feature = "color_detect")]
    /// Automatically detect terminal capabilities and match appropriate theme
    ///
    /// Uses terminal detection to determine color support and background luminance,
    /// then selects the best matching theme
    Match,
}

impl ColorInitStrategy {
    /// Determines the appropriate color initialization strategy based on available features and configuration.
    ///
    /// This method evaluates the current environment and available features to select the most
    /// appropriate strategy for initializing terminal color attributes:
    ///
    /// - **With `color_detect` feature**: Uses auto-detection unless in test environment
    /// - **With `config` feature only**: Uses configuration values or falls back to defaults
    /// - **Neither feature**: Always uses safe defaults
    ///
    /// # Returns
    /// A static reference to the determined `ColorInitStrategy`
    ///
    /// # Behavior
    /// - In test environments (`TEST_ENV` set): Always returns `Default` strategy
    /// - On Windows with `color_detect`: Uses `Configure` strategy with detected values
    /// - Other platforms with `color_detect`: Uses `Match` strategy for auto-detection
    /// - With config only: Uses `Configure` or `Match` based on configuration completeness
    /// - No features: Uses `Default` strategy
    #[must_use]
    pub fn determine() -> Self {
        // `color_detect` feature overrides configured colour support.
        #[cfg(feature = "color_detect")]
        let strategy = if std::env::var("TEST_ENV").is_ok() {
            #[cfg(debug_assertions)]
            debug_log!("Avoiding colour detection for testing");
            Self::Default
        } else {
            // Use unified auto-detection for all platforms including Windows
            Self::Match
        };

        #[cfg(all(not(feature = "color_detect"), feature = "config"))]
        let strategy = if std::env::var("TEST_ENV").is_ok() {
            #[cfg(debug_assertions)]
            debug_log!("Avoiding colour detection for testing");
            Self::Default
        } else if let Some(config) = maybe_config() {
            match (
                config.styling.color_support,
                config.styling.term_bg_luma,
                config.styling.term_bg_rgb,
            ) {
                (ColorSupport::Undetermined, _, _)
                | (_, TermBgLuma::Undetermined, _)
                | (_, _, None) => Self::Configure(
                    config.styling.color_support,
                    config.styling.term_bg_luma,
                    config.styling.term_bg_rgb,
                ),
                _ => Self::Match,
            }
        } else {
            Self::Default
        };

        #[cfg(all(not(feature = "color_detect"), not(feature = "config")))]
        let strategy = Self::Default;

        strategy
    }
}

#[derive(Clone, Debug, Display)]
/// Indicates how terminal attributes were initialized
pub enum HowInitialized {
    /// Attributes were explicitly configured by the user
    Configured,
    /// Attributes were set to safe default values
    Defaulted,
    /// Attributes were automatically detected from the terminal
    Detected,
}

/// Manages terminal color attributes and styling based on terminal capabilities and theme
#[derive(Debug, Clone)]
/// The complete styling attributes for terminal output
pub struct TermAttributes {
    /// The initialization strategy used
    pub init_strategy: ColorInitStrategy,
    /// Indicates how the terminal attributes were initialized (configured, defaulted, or detected)
    pub how_initialized: HowInitialized,
    /// The level of color support available in the terminal
    pub color_support: ColorSupport,
    /// The terminal background color as a hex string (e.g., "#1e1e1e")
    pub term_bg_hex: Option<String>,
    /// The terminal background color as RGB values (red, green, blue)
    pub term_bg_rgb: Option<[u8; 3]>,
    /// The luminance (light/dark) of the terminal background
    pub term_bg_luma: TermBgLuma,
    /// The theme used for styling text and interface elements
    pub theme: Theme,
}

/// Global instance of `TermAttributes`
static INSTANCE: OnceLock<TermAttributes> = OnceLock::new();
/// Global flag to enable/disable logging
pub static LOGGING_ENABLED: AtomicBool = AtomicBool::new(true);

thread_local! {
    /// Thread-local storage for theme context override
    static THEME_CONTEXT: std::cell::RefCell<Option<Theme>> = const { std::cell::RefCell::new(None) };
}

thread_local! {
    /// Thread-local storage for `TermAttributes` context override
    static TERM_ATTRIBUTES_CONTEXT: std::cell::RefCell<Option<TermAttributes>> = const { std::cell::RefCell::new(None) };
}

impl TermAttributes {
    /// Creates a new `TermAttributes` instance with specified support and theme
    #[allow(dead_code)]
    const fn new(
        color_support: ColorSupport,
        term_bg: Option<[u8; 3]>,
        term_bg_luma: TermBgLuma,
        theme: Theme,
    ) -> Self {
        Self {
            init_strategy: ColorInitStrategy::Default,
            how_initialized: HowInitialized::Defaulted,
            color_support,
            term_bg_hex: None,
            term_bg_rgb: term_bg,
            term_bg_luma,
            theme,
        }
    }

    /// Internal initialization method - use `get_or_init()` or `get_or_init_with_strategy()` instead
    ///
    /// This function initializes the terminal attributes singleton with color support
    /// and theme settings according to the specified strategy.
    ///
    /// # Arguments
    /// * `strategy` - The color initialization strategy to use
    ///
    /// # Returns
    /// A reference to the initialized `TermAttributes` instance
    ///
    /// # Panics
    /// * Built-in theme loading fails (which should never happen with correct installation)
    /// * Theme conversion fails during initialization
    #[allow(clippy::too_many_lines)]
    fn initialize(strategy: &ColorInitStrategy) -> &'static Self {
        let get_or_init = INSTANCE.get_or_init(|| -> Self {
            #[cfg(feature = "config")]
            let Some(_config) = maybe_config() else {
                panic!("Error initializing configuration")
            };

            match *strategy {
                ColorInitStrategy::Configure(support, bg_luma, bg_rgb) => {
                    let theme_name = match (support, bg_luma, bg_rgb) {
                        (_, TermBgLuma::Light, _) => "github",
                        (
                            ColorSupport::Basic | ColorSupport::Undetermined | ColorSupport::None,
                            _,
                            _,
                        ) => "basic_dark",
                        (_, TermBgLuma::Dark | TermBgLuma::Undetermined, _) => "espresso",
                    };
                    let theme = Theme::get_theme_runtime_or_builtin_with_color_support(theme_name, support)
                        .expect("Failed to load theme");
                    Self {
                        init_strategy: strategy.clone(),
                        how_initialized: HowInitialized::Configured,
                        color_support: support,
                        theme,
                        term_bg_hex: None,
                        term_bg_rgb: bg_rgb,
                        term_bg_luma: match bg_luma {
                            TermBgLuma::Light => TermBgLuma::Light,
                            TermBgLuma::Dark | TermBgLuma::Undetermined => TermBgLuma::Dark,
                        },
                    }
                }
                ColorInitStrategy::Default => {
                    let theme =
                        Theme::get_theme_runtime_or_builtin_with_color_support("basic_dark", ColorSupport::Basic)
                            .expect("Failed to load basic dark theme");
                    Self {
                        init_strategy: strategy.clone(),
                        how_initialized: HowInitialized::Defaulted,
                        color_support: ColorSupport::Basic,
                        term_bg_hex: None,
                        term_bg_rgb: None,
                        term_bg_luma: TermBgLuma::Dark,
                        theme,
                    }
                }
                ColorInitStrategy::Match => {
                    #[cfg(feature = "color_detect")]
                    {
                        // Check for THAG_THEME environment variable first
                        if let Ok(theme_name) = std::env::var("THAG_THEME") {
                            vprtln!(V::V, "Using THAG_THEME environment variable: {}", theme_name);

                            let (color_support, term_bg_rgb_ref) =
                                thag_common::terminal::detect_term_capabilities();
                            let term_bg_rgb = Some(*term_bg_rgb_ref);
                            let term_bg_hex = Some(rgb_to_hex(term_bg_rgb_ref));

                            // Load the specified theme directly (try runtime first, then builtin)
                            let theme = Theme::get_theme_runtime_or_builtin(&theme_name)
                                .map_or_else(|_| {
                                    vprtln!(V::V, "Warning: THAG_THEME '{}' not found, falling back to auto-detection", theme_name);
                                    Theme::auto_detect(*color_support, TermBgLuma::Dark, Some(term_bg_rgb_ref))
                                        .expect("Failed to auto-detect fallback theme")
                                }, |mut theme| {
                                    if *color_support != ColorSupport::TrueColor {
                                        theme.convert_to_color_support(*color_support);
                                    }
                                    theme
                                });

                            // Determine theme's background luma from the theme itself
                            let term_bg_luma = if let Some(&[r, g, b]) = theme.bg_rgbs.first() {
                                if is_light_color([r, g, b]) {
                                    TermBgLuma::Light
                                } else {
                                    TermBgLuma::Dark
                                }
                            } else if is_light_color(*term_bg_rgb_ref) {
                                TermBgLuma::Light
                            } else {
                                TermBgLuma::Dark
                            };

                            return Self {
                                init_strategy: strategy.clone(),
                                how_initialized: HowInitialized::Detected,
                                color_support: *color_support,
                                term_bg_hex,
                                term_bg_rgb,
                                term_bg_luma,
                                theme,
                            };
                        }

                        // Original auto-detection logic when THAG_THEME is not set
                        let (color_support, term_bg_rgb_ref) =
                            thag_common::terminal::detect_term_capabilities();
                        // let term_bg_rgb_ref = terminal::get_term_bg_rgb().ok();
                        let term_bg_rgb = Some(*term_bg_rgb_ref);
                        let term_bg_hex = Some(rgb_to_hex(term_bg_rgb_ref));
                        let term_bg_luma = if is_light_color(*term_bg_rgb_ref) {
                            TermBgLuma::Light
                        } else {
                            TermBgLuma::Dark
                        };
                        let theme =
                            Theme::auto_detect(*color_support, term_bg_luma, Some(term_bg_rgb_ref))
                                .expect("Failed to auto-detect theme");
                        Self {
                            init_strategy: strategy.clone(),
                            how_initialized: HowInitialized::Detected,
                            color_support: *color_support,
                            term_bg_hex,
                            term_bg_rgb,
                            term_bg_luma,
                            theme,
                        }
                    }
                    #[cfg(all(not(feature = "color_detect"), feature = "config"))]
                    {
                        if let Some(config) = maybe_config() {
                            let term_bg_rgb = config
                                .styling
                                .term_bg_rgb
                                .unwrap_or_else(|| panic!("Attempted to unwrap term_bg_rgb: None"));
                            let color_support = config.styling.color_support;
                            let term_bg_luma = config.styling.term_bg_luma;
                            let theme = if color_support == ColorSupport::None {
                                Theme::get_theme_runtime_or_builtin("none").expect("Failed to load `none` theme")
                            } else {
                                Theme::auto_detect(color_support, term_bg_luma, Some(&term_bg_rgb))
                                    .expect("Failed to auto-detect theme")
                            };
                            Self {
                                init_strategy: strategy.clone(),
                                how_initialized: HowInitialized::Configured,
                                color_support,
                                term_bg_hex: Some(rgb_to_hex(&term_bg_rgb)),
                                term_bg_rgb: Some(term_bg_rgb),
                                term_bg_luma,
                                theme,
                            }
                        } else {
                            let theme = Theme::get_theme_runtime_or_builtin_with_color_support(
                                "basic_dark",
                                ColorSupport::Basic,
                            )
                            .expect("Failed to load basic dark theme");
                            Self {
                                init_strategy: strategy.clone(),
                                how_initialized: HowInitialized::Defaulted,
                                color_support: ColorSupport::Basic,
                                term_bg_hex: None,
                                term_bg_rgb: None,
                                term_bg_luma: TermBgLuma::Dark,
                                theme,
                            }
                        }
                    }
                    #[cfg(all(not(feature = "config"), not(feature = "color_detect")))]
                    {
                        let theme =
                            Theme::get_theme_runtime_or_builtin_with_color_support("basic_dark", ColorSupport::Basic)
                                .expect("Failed to load basic dark theme");
                        Self {
                            init_strategy: strategy.clone(),
                            how_initialized: HowInitialized::Defaulted,
                            color_support: ColorSupport::Basic,
                            term_bg_hex: None,
                            term_bg_rgb: None,
                            term_bg_luma: TermBgLuma::Dark,
                            theme,
                        }
                    }
                }
            }
        });
        get_or_init
    }

    /// Checks if `TermAttributes` has been initialized
    pub fn is_initialized() -> bool {
        INSTANCE.get().is_some()
    }

    /// Attempts to get the `TermAttributes` instance, returning None if not initialized
    pub fn try_get() -> Option<&'static Self> {
        INSTANCE.get()
    }

    /// Gets the `TermAttributes` instance, initializing with auto-determined strategy if necessary.
    ///
    /// This method uses `ColorInitStrategy::determine()` to automatically choose the best
    /// initialization strategy based on available features and environment.
    ///
    /// If not already initialized:
    /// - With `color_detect` feature: performs auto-detection
    /// - Without `color_detect`: uses safe defaults
    ///
    /// # Returns
    /// Reference to the `TermAttributes` instance
    ///
    /// # Panics
    /// Panics if theme initialization fails
    pub fn get_or_init() -> &'static Self {
        if !Self::is_initialized() {
            Self::initialize(&ColorInitStrategy::determine());
        }
        // Safe to unwrap as we just checked/initialized it
        INSTANCE.get().unwrap()
    }

    /// Execute a closure with a temporary `TermAttributes` context
    /// This allows testing and temporary overrides without affecting the global state
    pub fn with_context<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        TERM_ATTRIBUTES_CONTEXT.with(|context| {
            let previous = context.borrow().clone();
            *context.borrow_mut() = Some(self.clone());

            let result = f();

            *context.borrow_mut() = previous;
            result
        })
    }

    /// Gets the current context, falling back to the global instance if no context is set
    #[must_use]
    pub fn current() -> Self {
        TERM_ATTRIBUTES_CONTEXT.with(|context| {
            context
                .borrow()
                .as_ref()
                .map_or_else(|| Self::get_or_init().clone(), Clone::clone)
        })
    }

    /// Initialize with a specific strategy, for testing and special cases
    ///
    /// Returns Ok if successfully initialized or if already initialized with a compatible strategy.
    /// Returns Err if already initialized with an incompatible strategy.
    ///
    /// # Arguments
    /// * `strategy` - The specific `ColorInitStrategy` to use
    ///
    /// # Returns
    /// * `Ok(&'static Self)` - Reference to the `TermAttributes` instance
    /// * `Err(String)` - Error message if incompatible strategy already used
    ///
    /// # Errors
    ///
    /// This function will return an error if the instance is already initialized with an incompatible strategy..
    pub fn try_initialize_with_strategy(
        strategy: &ColorInitStrategy,
    ) -> Result<&'static Self, String> {
        if let Some(existing) = INSTANCE.get() {
            if std::mem::discriminant(&existing.init_strategy) != std::mem::discriminant(strategy) {
                return Err(format!(
                    "TermAttributes already initialized with strategy {:?}, cannot reinitialize with {:?}",
                    existing.init_strategy, strategy
                ));
            }
            return Ok(existing);
        }
        Ok(Self::initialize(strategy))
    }

    /// Gets or initializes `TermAttributes` with the specified strategy
    ///
    /// This method initializes `TermAttributes` with the given strategy if not already initialized,
    /// or returns the existing instance if already initialized (regardless of the original strategy).
    ///
    /// This is the recommended method when you need a specific initialization strategy.
    ///
    /// # Arguments
    /// * `strategy` - The `ColorInitStrategy` to use for initialization
    ///
    /// # Returns
    /// Reference to the `TermAttributes` instance
    ///
    /// # Panics
    /// Panics if theme initialization fails
    pub fn get_or_init_with_strategy(strategy: &ColorInitStrategy) -> &'static Self {
        if Self::is_initialized() {
            INSTANCE.get().unwrap()
        } else {
            Self::initialize(strategy)
        }
    }

    /// Check if `TermAttributes` can be initialized with a given strategy.
    /// This is useful for testing to verify strategy compatibility.
    #[cfg(test)]
    pub fn can_initialize_with_strategy(strategy: &ColorInitStrategy) -> bool {
        if let Some(existing) = INSTANCE.get() {
            std::mem::discriminant(&existing.init_strategy) == std::mem::discriminant(strategy)
        } else {
            true
        }
    }

    /// Creates a new `TermAttributes` instance for testing purposes
    #[must_use]
    pub fn for_testing(
        color_support: ColorSupport,
        term_bg_rgb: Option<[u8; 3]>,
        term_bg_luma: TermBgLuma,
        theme: Theme,
    ) -> Self {
        Self {
            init_strategy: ColorInitStrategy::Default,
            how_initialized: HowInitialized::Defaulted,
            color_support,
            term_bg_hex: term_bg_rgb.map(|rgb| rgb_to_hex(&rgb)),
            term_bg_rgb,
            term_bg_luma,
            theme,
        }
    }

    /// Updates the current theme to the specified built-in theme.
    ///
    /// # Arguments
    /// * `theme_name` - Name of the built-in theme to use
    ///
    /// # Returns
    /// The updated `TermAttributes` instance
    ///
    /// # Errors
    /// Returns a `ThemeError` if:
    /// * The specified theme name is not recognized
    /// * The theme file is corrupted or invalid
    /// * The theme is incompatible with current terminal capabilities
    /// * Theme validation fails
    pub fn with_theme(mut self, theme_name: &str, support: ColorSupport) -> StylingResult<Self> {
        self.theme = Theme::get_theme_runtime_or_builtin_with_color_support(theme_name, support)?;
        Ok(self)
    }

    /// Creates a new instance with the specified color support level
    ///
    /// # Arguments
    /// * `support` - The color support level to set
    ///
    /// # Returns
    /// A new `TermAttributes` instance with the updated color support
    #[must_use]
    pub const fn with_color_support(mut self, support: ColorSupport) -> Self {
        self.color_support = support;
        self
    }
}

#[must_use]
/// Applies styling to text based on the specified role
///
/// This is a convenience function that gets the appropriate style for a role
/// from the currently loaded theme and applies it to the given string.
///
/// # Arguments
/// * `role` - The role that determines the styling to apply
/// * `string` - The text to be styled
///
/// # Returns
/// A new string with ANSI escape codes applied for the role's styling
///
/// # Examples
/// ```
/// use thag_styling::{paint_for_role, Role};
///
/// let styled_error = paint_for_role(Role::Error, "This is an error message");
/// println!("{}", styled_error); // Prints in error styling
/// ```
pub fn paint_for_role(role: Role, string: &str) -> String {
    Style::from(role).paint(string)
}

#[must_use]
/// Returns the appropriate style for the given theme and role
///
/// This is a utility function that extracts the style for a specific role
/// from the provided theme instance.
///
/// # Arguments
/// * `theme` - The theme to get the style from
/// * `role` - The role that determines which style to retrieve
///
/// # Returns
/// A `Style` instance configured for the specified role in the given theme
pub fn style_for_theme_and_role(theme: &Theme, role: Role) -> Style {
    theme.style_for_role(role)
}

// New structures for Themes

/// Defines the role (purpose and relative prominence) of a piece of text
#[derive(Debug, Clone, Copy, EnumIter, Display, PartialEq, Eq, Hash)]
pub enum Role {
    /// Primary heading, highest prominence
    Heading1,
    /// Secondary heading
    Heading2,
    /// Tertiary heading
    Heading3,

    /// Critical errors requiring immediate attention
    Error,
    /// Important cautions or potential issues
    Warning,
    /// Positive completion or status messages
    Success,
    /// General informational messages
    Info,

    /// Text that needs to stand out
    Emphasis,
    /// Code snippets or commands
    Code,
    /// Standard text, default prominence
    Normal,
    /// De-emphasized but clearly visible text
    Subtle,
    /// Completion suggestions or placeholder text (typically italic)
    Hint,

    /// Development/diagnostic information
    Debug,
    /// Links and URLs
    Link,
    /// Quoted text or citations
    Quote,
    /// Commentary or explanatory notes
    Commentary,
}

#[derive(Clone, Debug, Deserialize)]
/// Configuration structure for theme palette colors and styles
///
/// This structure defines the color and style configuration for all message roles
/// used in the theme system. Each field corresponds to a specific role and contains
/// both color information and text styling attributes.
pub struct PaletteConfig {
    heading1: StyleConfig,
    heading2: StyleConfig,
    heading3: StyleConfig,
    error: StyleConfig,
    warning: StyleConfig,
    success: StyleConfig,
    info: StyleConfig,
    emphasis: StyleConfig,
    code: StyleConfig,
    normal: StyleConfig,
    subtle: StyleConfig,
    hint: StyleConfig,
    debug: StyleConfig,
    link: StyleConfig,
    quote: StyleConfig,
    commentary: StyleConfig,
}

/// Color palette containing predefined styles for all message roles
///
/// This structure holds the complete set of styling information for a theme,
/// with each field corresponding to a specific role in the message hierarchy.
/// The styles define both color and text formatting attributes for consistent
/// visual presentation across different message types.
#[derive(Clone, Debug, Default)]
#[allow(missing_docs)]
#[derive(PaletteMethods)]
pub struct Palette {
    /// Style for primary headings (highest prominence)
    pub heading1: Style,
    /// Style for secondary headings
    pub heading2: Style,
    /// Style for tertiary headings
    pub heading3: Style,
    /// Style for critical errors requiring immediate attention
    pub error: Style,
    /// Style for important cautions or potential issues
    pub warning: Style,
    /// Style for positive completion or status messages
    pub success: Style,
    /// Style for general informational messages
    pub info: Style,
    /// Style for text that needs to stand out
    pub emphasis: Style,
    /// Style for code snippets or commands
    pub code: Style,
    /// Style for standard text with default prominence
    pub normal: Style,
    /// Style for de-emphasized but clearly visible text
    pub subtle: Style,
    /// Style for completion suggestions or placeholder text
    pub hint: Style,
    /// Style for development/diagnostic information
    pub debug: Style,
    /// Style for links and URLs
    pub link: Style,
    /// Style for quoted text or citations
    pub quote: Style,
    /// Style for commentary or explanatory notes
    pub commentary: Style,
}

impl Palette {
    /// Returns the appropriate style for the given role
    ///
    /// This method looks up the style configuration for a specific role
    /// in the palette and returns a clone of that style.
    ///
    /// # Arguments
    /// * `role` - The role to get the style for
    ///
    /// # Returns
    /// A `Style` instance configured for the specified role
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
            Role::Link => self.link.clone(),
            Role::Quote => self.quote.clone(),
            Role::Commentary => self.commentary.clone(),
        }
    }
}

// ThemeIndex, THEME_INDEX and BG_LOOKUP
preload_themes! {}

/// Theme definition loaded from TOML files
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ThemeDefinition {
    name: String,
    #[serde(skip)]
    /// Path to the theme file (e.g., "`themes/built_in/dracula.toml`")
    pub filename: PathBuf, // e.g., "themes/built_in/dracula.toml"
    #[serde(skip)]
    /// Whether this is a built-in theme or a custom theme
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

    /// Get the background color if specified
    #[must_use]
    pub fn backgrounds(&self) -> Vec<String> {
        self.backgrounds.clone()
    }
}

#[derive(Clone, Debug)]
/// Represents a complete theme configuration with color palette and styling information
///
/// A `Theme` encapsulates all the styling information needed to consistently format
/// terminal output, including color palettes, text formatting attributes, and metadata
/// about the theme's requirements and characteristics.
///
/// # Examples
///
/// ```
/// use thag_styling::{Role, Theme, ColorSupport};
///
/// // Load a built-in theme
/// let theme = Theme::get_builtin("dracula")?;
///
/// // Get styling for a specific role
/// let error_style = theme.style_for(Role::Error);
/// println!("{}", error_style.paint("This is an error message"));
/// # Ok::<(), thag_styling::StylingError>(())
/// ```
pub struct Theme {
    /// The human-readable name of the theme (e.g., "Dracula", "GitHub Light")
    pub name: String,
    /// Path to the theme definition file (e.g., "`themes/built_in/dracula.toml`")
    pub filename: PathBuf,
    /// Whether this is a built-in theme (true) or a custom user theme (false)
    pub is_builtin: bool,
    /// The background luminance requirement (light or dark) for this theme
    pub term_bg_luma: TermBgLuma,
    /// The minimum color support level required by this theme
    pub min_color_support: ColorSupport,
    /// The complete color palette containing styles for all message roles
    pub palette: Palette,
    /// Background color values as hex strings (e.g., `["#282a36", "#44475a"]`)
    pub backgrounds: Vec<String>,
    /// Background color values as RGB tuples, with the primary/official color first
    pub bg_rgbs: Vec<[u8; 3]>,
    /// A human-readable description of the theme's characteristics and origin
    pub description: String,
    /// Base16/24 color array for ANSI terminal mapping (optional)
    /// Skipped during normal theme loading to save memory.
    /// Load on-demand using `load_base_colors()` when needed (e.g., in `thag_sync_palette`).
    /// Contains 16 colors for base16 themes or 24 colors for base24 themes.
    pub base_colors: Option<Vec<[u8; 3]>>,
}

impl Theme {
    fn from_toml(theme_name: &str, theme_toml: &str) -> Result<Self, StylingError> {
        // vprtln!(V::VV, "About to call toml::from_str(theme_toml)");
        let mut def: ThemeDefinition = toml::from_str(theme_toml)?;
        // vprtln!(V::VV, "Done! def={def:?}");
        def.filename = PathBuf::from(format!("themes/built_in/{theme_name}.toml"));
        def.is_builtin = true;
        // vprtln!(V::VV, "About to call Theme::from_definition({def:?})");
        Self::from_definition(def)
    }

    /// Detects and loads the most appropriate theme for the current terminal.
    /// This function does not involve interrogating the terminal, and may use
    /// configured or defaulted values.
    ///
    /// # Errors
    ///
    /// This function will bubble up any `termbg` error encountered.
    // #[cfg(feature = "color_detect")]
    #[allow(clippy::too_many_lines, clippy::cognitive_complexity, unused_variables)]
    pub fn auto_detect(
        color_support: ColorSupport,
        term_bg_luma: TermBgLuma,
        maybe_term_bg: Option<&[u8; 3]>,
    ) -> StylingResult<Self> {
        // NB: don't call `TermAttributes::get_or_init()` here because it will cause a tight loop
        // since we're called from the TermAttributes::initialize.
        vprtln!(V::VV, "maybe_term_bg={maybe_term_bg:?}");
        let Some(term_bg_rgb) = maybe_term_bg else {
            return fallback_theme(term_bg_luma);
        };

        // let signatures = get_theme_signatures();
        // vprtln!(V::VV, "signatures={signatures:?}");
        let hex = format!(
            "{:02x}{:02x}{:02x}",
            term_bg_rgb[0], term_bg_rgb[1], term_bg_rgb[2]
        );
        let exact_matches = BG_LOOKUP
            .get(&hex)
            .map(|names| Vec::from(*names))
            .unwrap_or_default();
        vprtln!(
            V::VV,
            "term_bg_rgb={term_bg_rgb:?}, hex={hex}, exact_matches for hex={exact_matches:?}"
        );

        // let exact_matches = get_exact_matches(&exact_bg_matches, *term_bg_rgb, color_support);
        // vprtln!(V::VV, "exact_matches={exact_matches:#?}");

        // First filter themes by luma and compatible colour distance
        #[cfg(feature = "config")]
        let eligible_themes: Vec<_> = THEME_INDEX
            .into_iter()
            .filter(|(_, idx)| {
                let mut min_distance = f32::MAX;
                for rgb in idx.bg_rgbs {
                    let distance = color_distance(*term_bg_rgb, *rgb);
                    if distance < min_distance {
                        min_distance = distance;
                    }
                }
                idx.term_bg_luma == term_bg_luma
                    && idx.min_color_support <= color_support
                    && min_distance < THRESHOLD
            })
            .map(|(&name, idx)| (name, idx))
            .collect();
        #[cfg(feature = "config")]
        vprtln!(V::VV, "Found {} eligible themes", eligible_themes.len());
        #[cfg(feature = "config")]
        if let Some(config) = maybe_config() {
            vprtln!(
                V::VV,
                "1. Try exact background RGB match of a preferred theme."
            );
            vprtln!(V::VV, "Looking for match on config styling");
            let preferred_styling = get_preferred_styling(term_bg_luma, &config);

            for preferred_name in preferred_styling {
                vprtln!(V::VV, "preferred_name={preferred_name}");
                if exact_matches.contains(&preferred_name.as_str()) {
                    vprtln!(V::VV, "Found an exact match in {preferred_name}");
                    return Self::get_theme_with_color_support(preferred_name, color_support);
                }
            }

            vprtln!(V::VV, "2. Look for any theme exactly matching colour support and terminal background colour, in hopes of matching existing theme colours.");
            vprtln!(V::VV, "a. Try exact match on fallback names");
            let fallback_styling = get_fallback_styling(term_bg_luma, &config);
            for fallback_name in fallback_styling {
                vprtln!(V::VV, "fallback_name={fallback_name}");
                if exact_matches.contains(&fallback_name.as_str()) {
                    vprtln!(V::VV, "Found an exact match in fallback {fallback_name}");
                    return Self::get_theme_with_color_support(fallback_name, color_support);
                }
            }
            vprtln!(V::VV, "b. Try for any exact match.");
            if let Some(exact_match) = exact_matches.into_iter().next() {
                vprtln!(V::VV, "Found an exact match with {exact_match}");
                return Self::get_theme_with_color_support(exact_match, color_support);
            }
            vprtln!(V::VV, "No exact matches found.");

            vprtln!(V::VV, "3. Try closest match to a preferred theme.");
            let mut best_match = None;
            let mut min_distance = f32::MAX;
            for preferred_name in preferred_styling {
                let preferred_idx = THEME_INDEX.get(preferred_name);
                // vprtln!(V::VV, "theme_name={theme_name}");
                if let Some(idx) = preferred_idx {
                    for rgb in idx.bg_rgbs {
                        let distance = color_distance(*term_bg_rgb, *rgb);
                        if distance < min_distance {
                            min_distance = distance;
                            best_match = Some(preferred_name);
                        }
                    }
                }
            }
            if let Some(theme) = best_match {
                vprtln!(V::VV, "Choosing preferred theme {theme} because it most closely matches terminal bg {term_bg_rgb:?}");
                return Self::get_theme_with_color_support(theme, color_support);
            }

            vprtln!(
                V::V,
                "4. Try closest match to a fallback theme, irrespective of colour support."
            );
            let mut best_match = None;
            let mut min_distance = f32::MAX;
            for fallback_name in fallback_styling {
                let fallback_idx = THEME_INDEX.get(fallback_name);
                // vprtln!(V::VV, "theme_name={theme_name}");
                if let Some(idx) = fallback_idx {
                    for rgb in idx.bg_rgbs {
                        let distance = color_distance(*term_bg_rgb, *rgb);
                        if distance < min_distance {
                            min_distance = distance;
                            best_match = Some(fallback_name);
                        }
                    }
                }
            }
            if let Some(theme) = best_match {
                vprtln!(V::VV, "Choosing preferred theme {theme} because it most closely matches terminal bg {term_bg_rgb:?}");
                return Self::get_theme_with_color_support(theme, color_support);
            }

            vprtln!(V::VV, "5. Try closest match.");
            let mut min_distance = f32::MAX;
            let mut best_match = None;
            for (theme, idx) in &eligible_themes {
                for rgb in idx.bg_rgbs {
                    let distance = color_distance(*term_bg_rgb, *rgb);
                    if distance < min_distance {
                        min_distance = distance;
                        best_match = Some(theme);
                    }
                }
            }
            if let Some(theme) = best_match {
                vprtln!(V::VV, "Found the closest match with {theme}");
                return Self::get_theme_with_color_support(theme, color_support);
            }
        }

        vprtln!(V::VV, "6. Fall back to basic theme.");
        Self::get_theme_with_color_support(
            if term_bg_luma == TermBgLuma::Light {
                "basic_light"
            } else {
                "basic_dark"
            },
            color_support,
        )
    }

    /// Loads a theme from a TOML file.
    ///
    /// # Color support requirements
    /// The TOML file should define a complete theme, including:
    /// - Background luminance requirements
    /// - Style definitions for all message types
    ///
    /// # Arguments
    /// * `path` - Path to the TOML file containing the theme definition
    ///
    /// # Returns
    /// A new `Theme` instance configured according to the TOML definition
    ///
    /// # Errors
    /// Returns `StylingError` if:
    /// - The file cannot be read
    /// - The file contains invalid TOML syntax
    /// - The theme definition is incomplete or invalid
    /// - Color values don't match the declared minimum color support
    /// - Style attributes are invalid
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use thag_styling::StylingError;
    /// use thag_styling::Theme;
    /// let theme = Theme::load_from_file(Path::new("themes/built_in/basic_light.toml"))?;
    /// # Ok::<(), StylingError>(())
    /// ```
    pub fn load_from_file(path: &Path) -> StylingResult<Self> {
        let content = fs::read_to_string(path)?;
        let mut def: ThemeDefinition = toml::from_str(&content)?;
        def.filename = path.to_path_buf();
        def.is_builtin = false;
        Self::from_definition(def)
    }

    /// Load base16/24 colors on-demand for ANSI terminal mapping.
    ///
    /// This method loads the optional `base_colors` array from the theme file.
    /// The array is skipped during normal theme loading to save memory, but can be
    /// loaded when needed by tools like `thag_sync_palette` for ANSI terminal color mapping.
    ///
    /// # Returns
    /// Returns `Ok(())` if colors were loaded or were already present.
    ///
    /// # Errors
    /// Returns `StylingError` if:
    /// - The theme file cannot be read
    /// - The TOML content is invalid
    /// - The `base_colors` array has invalid format
    ///
    /// # Examples
    /// ```no_run
    /// use thag_styling::Theme;
    /// use std::path::Path;
    ///
    /// let mut theme = Theme::load_from_file(Path::new("my-theme.toml"))?;
    /// theme.load_base_colors()?;
    ///
    /// if let Some(colors) = &theme.base_colors {
    ///     println!("Loaded {} base colors", colors.len());
    /// }
    /// # Ok::<(), thag_styling::StylingError>(())
    /// ```
    pub fn load_base_colors(&mut self) -> StylingResult<()> {
        // Already loaded
        if self.base_colors.is_some() {
            return Ok(());
        }

        // Read the theme file
        let content = fs::read_to_string(&self.filename)?;
        let value: toml::Value = toml::from_str(&content)?;

        // Extract base_colors array if present
        if let Some(base_colors_value) = value.get("base_colors") {
            if let Some(array) = base_colors_value.as_array() {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let colors: Vec<[u8; 3]> = array
                    .iter()
                    .filter_map(|v| {
                        let arr = v.as_array()?;
                        if arr.len() == 3 {
                            Some([
                                arr[0].as_integer()? as u8,
                                arr[1].as_integer()? as u8,
                                arr[2].as_integer()? as u8,
                            ])
                        } else {
                            None
                        }
                    })
                    .collect();

                if !colors.is_empty() {
                    self.base_colors = Some(colors);
                }
            }
        }

        Ok(())
    }

    /// Loads a built-in theme by name.
    ///
    /// Built-in themes are compiled into the binary and include:
    /// - "`dracula`" - Dark theme with vibrant colors
    /// - "`basic_light`" - Simple light theme for basic terminals
    /// - "`basic_dark`" - Simple dark theme for basic terminals
    ///
    /// # Arguments
    /// * `name` - The name of the built-in theme to load
    ///
    /// # Returns
    /// A new `Theme` instance configured according to the named theme
    ///
    /// # Errors
    /// Returns `StylingError` if:
    /// - The specified theme name is not recognized
    /// - The theme definition contains invalid color values
    /// - The theme definition contains invalid style attributes
    /// - There's a mismatch between color values and minimum color support
    ///
    /// # Examples
    /// ```
    /// use thag_styling::StylingError;
    /// use thag_styling::Theme;
    /// let theme = Theme::get_builtin("dracula")?;
    /// # Ok::<(), StylingError>(())
    /// ```
    pub fn get_builtin(theme_name: &str) -> StylingResult<Self> {
        let maybe_theme_index = THEME_INDEX.get(theme_name);
        let Some(theme_index) = maybe_theme_index else {
            return Err(StylingError::FromStr(format!(
                "No theme found for name {theme_name}"
            )));
        };
        Self::from_toml(theme_name, theme_index.content)
    }

    /// Loads a built-in theme with the specified color support level.
    ///
    /// This method loads a built-in theme and converts its colors to match
    /// the specified color support level. Colors are automatically downgraded
    /// if necessary (e.g., from `TrueColor` to `Color256` or `Basic`).
    ///
    /// # Arguments
    /// * `theme_name` - The name of the built-in theme to load
    /// * `color_support` - The target color support level
    ///
    /// # Returns
    /// A new `Theme` instance with colors adjusted for the specified support level
    ///
    /// # Errors
    /// Returns `StylingError` if the specified theme name is not recognized
    ///
    pub fn get_theme_with_color_support(
        theme_name: &str,
        color_support: ColorSupport,
    ) -> StylingResult<Self> {
        let mut theme = Self::get_builtin(theme_name)?;
        if color_support != ColorSupport::TrueColor {
            vprtln!(V::VV, "Converting to {color_support:?}");
            theme.convert_to_color_support(color_support);
        }
        // eprintln!("Theme={:#?}", theme);
        Ok(theme)
    }

    /// Loads a theme with runtime loading support and specified color support level.
    ///
    /// This method first attempts to load from user-specified directories, then falls back
    /// to built-in themes. Colors are automatically converted to match the specified
    /// color support level.
    ///
    /// # Arguments
    /// * `theme_name` - The name of the theme to load
    /// * `color_support` - The target color support level
    ///
    /// # Returns
    /// A new `Theme` instance with colors adjusted for the specified support level
    ///
    /// # Errors
    /// Returns `StylingError` if the theme cannot be found or loaded
    pub fn get_theme_runtime_or_builtin_with_color_support(
        theme_name: &str,
        color_support: ColorSupport,
    ) -> StylingResult<Self> {
        let mut theme = Self::get_theme_runtime_or_builtin(theme_name)?;
        if color_support != ColorSupport::TrueColor {
            // Note: vprtln! call disabled to prevent deadlock during `TermAttributes` initialization
            // vprtln!(V::VV, "Converting to {color_support:?}");
            theme.convert_to_color_support(color_support);
        }
        Ok(theme)
    }

    /// Attempts to load a theme from user-specified directories first, then falls back to built-in themes.
    ///
    /// This method first checks for user-defined themes in directories specified by:
    /// 1. `THAG_THEME_DIR` environment variable (highest priority)
    /// 2. `theme_dir` configuration option
    /// 3. Falls back to built-in themes if not found
    ///
    /// # Arguments
    /// * `theme_name` - The name of the theme to load
    ///
    /// # Returns
    /// A new `Theme` instance
    ///
    /// # Errors
    /// Returns `StylingError` if the theme cannot be found or loaded
    pub fn get_theme_runtime_or_builtin(theme_name: &str) -> StylingResult<Self> {
        // First try to load from user-specified directory
        if let Ok(theme) = Self::try_load_from_user_dirs(theme_name) {
            return Ok(theme);
        }

        // Fall back to built-in theme
        Self::get_builtin(theme_name)
    }

    /// Attempts to load a theme from user-specified directories.
    ///
    /// Checks the following locations in order:
    /// 1. `THAG_THEME_DIR` environment variable
    /// 2. `theme_dir` from configuration
    ///
    /// # Arguments
    /// * `theme_name` - The name of the theme to load
    ///
    /// # Returns
    /// A new `Theme` instance if found in user directories
    ///
    /// # Errors
    /// Returns `StylingError` if no theme is found in user directories
    fn try_load_from_user_dirs(theme_name: &str) -> StylingResult<Self> {
        // Check THAG_THEME_DIR environment variable first
        if let Ok(theme_dir) = std::env::var("THAG_THEME_DIR") {
            match Self::load_from_directory(&theme_dir, theme_name) {
                Ok(theme) => {
                    vprtln!(
                        V::V,
                        "Loaded theme '{}' from THAG_THEME_DIR: {}",
                        theme_name,
                        theme_dir
                    );
                    return Ok(theme);
                }
                Err(e) => {
                    vprtln!(
                        V::V,
                        "Error loading theme '{}' from THAG_THEME_DIR {}: {}",
                        theme_name,
                        theme_dir,
                        e
                    );
                }
            }
        }

        // Check config theme_dir
        #[cfg(feature = "config")]
        if let Some(config) = maybe_config() {
            if let Some(ref theme_dir) = config.styling.theme_dir {
                match Self::load_from_directory(theme_dir, theme_name) {
                    Ok(theme) => {
                        vprtln!(
                            V::V,
                            "Loaded theme '{}' from config theme_dir: {}",
                            theme_name,
                            theme_dir
                        );
                        return Ok(theme);
                    }
                    Err(e) => {
                        vprtln!(
                            V::V,
                            "Error loading theme '{}' from config theme_dir {}: {}",
                            theme_name,
                            theme_dir,
                            e
                        );
                    }
                }
            }
        }

        Err(StylingError::FromStr(format!(
            "Theme '{}' not found in user directories",
            theme_name
        )))
    }

    /// Loads a theme from a specific directory.
    ///
    /// Looks for theme files with the following naming patterns:
    /// - `{theme_name}.toml`
    /// - `thag-{theme_name}.toml`
    /// - `thag-{theme_name}-light.toml` (if `theme_name` doesn't end in -light/-dark)
    /// - `thag-{theme_name}-dark.toml` (if `theme_name` doesn't end in -light/-dark)
    ///
    /// # Arguments
    /// * `dir` - Directory path to search in
    /// * `theme_name` - The name of the theme to load
    ///
    /// # Returns
    /// A new `Theme` instance if found
    ///
    /// # Errors
    /// Returns `StylingError` if no matching theme file is found or cannot be loaded
    fn load_from_directory(dir: &str, theme_name: &str) -> StylingResult<Self> {
        use std::path::Path;

        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            return Err(StylingError::FromStr(format!(
                "Theme directory does not exist: {}",
                dir
            )));
        }

        // Try different filename patterns
        let mut patterns = vec![
            format!("{}.toml", theme_name),
            format!("thag-{}.toml", theme_name),
        ];

        // If theme name doesn't already have a variant suffix, try both
        if !theme_name.ends_with("-light") && !theme_name.ends_with("-dark") {
            patterns.push(format!("thag-{}-light.toml", theme_name));
            patterns.push(format!("thag-{}-dark.toml", theme_name));
        }

        for pattern in patterns {
            let theme_path = dir_path.join(&pattern);
            if theme_path.exists() {
                match Self::load_from_file(&theme_path) {
                    Ok(theme) => return Ok(theme),
                    Err(_e) => {
                        // Continue trying other patterns
                    }
                }
            }
        }

        Err(StylingError::FromStr(format!(
            "Theme file for '{}' not found in directory: {}",
            theme_name, dir
        )))
    }

    /// Creates a theme from a theme definition.
    ///
    /// This method converts a `ThemeDefinition` parsed from TOML into a complete
    /// `Theme` instance, including color support validation and background color processing.
    ///
    /// # Arguments
    /// * `def` - The theme definition to convert
    ///
    /// # Returns
    /// A new `Theme` instance based on the definition
    ///
    /// # Errors
    /// Returns `StylingError` if:
    /// - Color support string cannot be parsed
    /// - Background luminance string cannot be parsed
    /// - Palette configuration is invalid
    fn from_definition(def: ThemeDefinition) -> StylingResult<Self> {
        // vprtln!(V::VV, "def.min_color_support={:?}", def.min_color_support);
        let color_support = ColorSupport::from_str(&def.min_color_support);
        // vprtln!(V::VV, "color_support={color_support:?})");

        // Convert hex strings to RGB tuples
        let bg_rgbs: Vec<[u8; 3]> = def
            .backgrounds
            .iter()
            .filter_map(|hex| hex_to_rgb(hex).ok())
            .collect();

        // let bg_rgb = bg_rgbs
        //     .first()
        //     .ok_or(ThemeError::NoValidBackground(def.name.clone()))?;

        Ok(Self {
            name: def.name,
            filename: def.filename,
            is_builtin: def.is_builtin,
            term_bg_luma: TermBgLuma::from_str(&def.term_bg_luma)?,
            min_color_support: color_support?,
            palette: Palette::from_config(&def.palette)?,
            // bg_rgb: *bg_rgb,
            backgrounds: def.backgrounds.clone(),
            bg_rgbs,
            description: def.description,
            base_colors: None, // Loaded on-demand via load_base_colors()
        })
    }

    /// Validates that the theme is compatible with the terminal capabilities.
    ///
    /// Checks both the color support level and background luminance compatibility
    /// to ensure the theme will display correctly in the current terminal.
    ///
    /// # Arguments
    /// * `available_support` - The terminal's color support level
    /// * `term_bg_luma` - The terminal's background luminance (light or dark)
    ///
    /// # Returns
    /// `Ok(())` if the theme is compatible with the terminal
    ///
    /// # Errors
    /// Returns `StylingError` if:
    /// - The terminal's color support is insufficient for the theme
    ///   (e.g., trying to use a 256-color theme in a basic terminal)
    /// - The terminal's background luminance doesn't match the theme's requirements
    ///   (e.g., trying to use a light theme on a dark background)
    /// - Any style in the theme's palette requires unavailable terminal features
    ///
    /// # Examples
    /// ```
    /// use thag_styling::{ColorSupport, StylingError, TermBgLuma, Theme};
    /// let theme = Theme::get_builtin("dracula")?;
    /// theme.validate(&ColorSupport::TrueColor, &TermBgLuma::Dark)?;
    /// # Ok::<(), StylingError>(())
    /// ```
    pub fn validate(
        &self,
        available_support: &ColorSupport,
        term_bg_luma: &TermBgLuma,
    ) -> StylingResult<()> {
        // Check color support
        // vprtln!(V::VV, "self.min_color_support={:?}", self.min_color_support);
        // vprtln!(V::VV, "available_support={available_support:?}");
        if available_support < &self.min_color_support {
            return Err(ThemeError::ColorSupportMismatch {
                required: self.min_color_support,
                available: *available_support,
            }
            .into());
        }

        // Check background compatibility
        if &self.term_bg_luma != term_bg_luma {
            return Err(ThemeError::TermBgLumaMismatch {
                theme: self.term_bg_luma,
                terminal: *term_bg_luma,
            }
            .into());
        }

        // Validate each color in the palette matches the declared min_color_support
        self.validate_palette()?;

        Ok(())
    }

    fn validate_palette(&self) -> StylingResult<()> {
        self.palette.validate_styles(self.min_color_support)?;
        Ok(())
    }

    /// Validates a theme definition before creating a Theme
    #[allow(dead_code)]
    fn validate_definition(def: &ThemeDefinition) -> StylingResult<()> {
        // Validate term_bg_luma value
        if !["light", "dark"].contains(&def.term_bg_luma.as_str()) {
            return Err(ThemeError::InvalidTermBgLuma(def.term_bg_luma.clone()).into());
        }

        // Validate color_support value
        if !["basic", "color256", "true_color"].contains(&def.min_color_support.as_str()) {
            return Err(ThemeError::InvalidColorSupport(def.min_color_support.clone()).into());
        }

        // Validate styles
        for style_name in def
            .palette
            .heading1
            .style
            .iter()
            .chain(&def.palette.heading2.style)
            .chain(&def.palette.heading3.style)
            .chain(&def.palette.error.style)
            .chain(&def.palette.warning.style)
            .chain(&def.palette.success.style)
            .chain(&def.palette.info.style)
            .chain(&def.palette.emphasis.style)
            .chain(&def.palette.code.style)
            .chain(&def.palette.normal.style)
            .chain(&def.palette.subtle.style)
            .chain(&def.palette.hint.style)
            .chain(&def.palette.debug.style)
            .chain(&def.palette.link.style)
            .chain(&def.palette.quote.style)
            .chain(&def.palette.commentary.style)
        {
            if !["bold", "italic", "dim", "underline"].contains(&style_name.as_str()) {
                return Err(ThemeError::InvalidStyle(style_name.clone()).into());
            }
        }

        Ok(())
    }

    /// Loads a theme from a file and validates it against terminal capabilities.
    ///
    /// This is a convenience method that combines `load_from_file` and `validate`
    /// to ensure the loaded theme is compatible with the current terminal.
    ///
    /// # Arguments
    /// * `path` - Path to the TOML file containing the theme definition
    /// * `available_support` - The terminal's color support level
    /// * `term_bg_luma` - The terminal's background luminance (light or dark)
    ///
    /// # Returns
    /// A new validated `Theme` instance configured according to the TOML definition
    ///
    /// # Errors
    /// Returns `StylingError` if:
    /// - The file cannot be read or contains invalid TOML syntax
    /// - The theme definition is incomplete or invalid
    /// - The terminal's color support is insufficient for the theme
    /// - The terminal's background luminance doesn't match the theme's requirements
    /// - Any style in the theme's palette requires unavailable terminal features
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use thag_styling::{ColorSupport, StylingError, TermBgLuma, Theme};
    /// let theme = Theme::load(
    ///     Path::new("themes/built_in/basic_light.toml"),
    ///     ColorSupport::Basic,
    ///     TermBgLuma::Light
    /// )?;
    /// # Ok::<(), StylingError>(())
    /// ```
    pub fn load(
        path: &Path,
        available_support: ColorSupport,
        term_bg_luma: TermBgLuma,
    ) -> StylingResult<Self> {
        let theme = Self::load_from_file(path)?;
        theme.validate(&available_support, &term_bg_luma)?;
        Ok(theme)
    }

    /// Get this theme's `Style` for a `Role`.
    #[must_use]
    pub fn style_for(&self, role: Role) -> Style {
        let palette = &self.palette;

        match role {
            Role::Error => palette.error.clone(),
            Role::Warning => palette.warning.clone(),
            Role::Heading1 => palette.heading1.clone(),
            Role::Heading2 => palette.heading2.clone(),
            Role::Heading3 => palette.heading3.clone(),
            Role::Success => palette.success.clone(),
            Role::Info => palette.info.clone(),
            Role::Emphasis => palette.emphasis.clone(),
            Role::Code => palette.code.clone(),
            Role::Normal => palette.normal.clone(),
            Role::Subtle => palette.subtle.clone(),
            Role::Hint => palette.hint.clone(),
            Role::Debug => palette.debug.clone(),
            Role::Link => palette.link.clone(),
            Role::Quote => palette.quote.clone(),
            Role::Commentary => palette.commentary.clone(),
        }
    }

    /// Returns information about the theme suitable for display
    #[must_use]
    pub fn info(&self) -> String {
        format!(
            "Theme: {}\nType: {}\nFile: {}\nDescription: {}\nBackground: {} = ({}, {}, {})\nMinimum Color Support: {:?}\nBackground Luminance: {:?}",
            self.name,
            if self.is_builtin { "Built-in" } else { "Custom" },
            self.filename.display(),
            self.description,
            rgb_to_hex(&self.bg_rgbs[0]),
            // format!("#{:02x}{:02x}{:02x}", self.bg_rgbs[0].0, self.bg_rgbs[0].1, self.bg_rgbs[0].2),
            self.bg_rgbs[0][0], self.bg_rgbs[0][1], self.bg_rgbs[0][2],
            self.min_color_support,
            self.term_bg_luma,
        )
    }

    /// Returns a list of all available built-in themes
    #[must_use]
    pub fn list_builtin() -> Vec<String> {
        THEME_INDEX.keys().map(ToString::to_string).collect()
    }

    #[must_use]
    fn style_for_role(&self, role: Role) -> Style {
        self.palette.style_for_role(role)
    }

    /// Converts RGB values to an ANSI color index.
    ///
    /// # Panics
    ///
    /// Panics if the color index cannot be converted to a u8.
    #[must_use]
    pub fn convert_rgb_to_ansi(r: u8, g: u8, b: u8) -> u8 {
        // Basic ANSI colors:
        // 0: Black   [0,0,0]
        // 1: Red     [170,0,0]
        // 2: Green   [0,170,0]
        // 3: Yellow  [170,85,0]
        // 4: Blue    [0,0,170]
        // 5: Magenta [170,0,170]
        // 6: Cyan    [0,170,170]
        // 7: White   [170,170,170]
        // 8-15: Bright versions

        let colors = [
            [0, 0, 0],       // Black
            [170, 0, 0],     // Red
            [0, 170, 0],     // Green
            [170, 85, 0],    // Yellow
            [0, 0, 170],     // Blue
            [170, 0, 170],   // Magenta
            [0, 170, 170],   // Cyan
            [170, 170, 170], // White
            // Bright versions
            [85, 85, 85],    // Bright Black
            [255, 85, 85],   // Bright Red
            [85, 255, 85],   // Bright Green
            [255, 255, 85],  // Bright Yellow
            [85, 85, 255],   // Bright Blue
            [255, 85, 255],  // Bright Magenta
            [85, 255, 255],  // Bright Cyan
            [255, 255, 255], // Bright White
        ];

        // Find closest ANSI color using color distance
        let mut closest = 0;
        let mut min_distance = f32::MAX;

        for (i, &[cr, cg, cb]) in colors.iter().enumerate() {
            let distance = color_distance([r, g, b], [cr, cg, cb]);
            if distance < min_distance {
                min_distance = distance;
                closest = i;
            }
        }

        u8::try_from(closest)
            .unwrap_or_else(|_| panic!("Failed to convert color index {closest} to u8"))
    }

    /// Converts all colors in the theme's palette to match the specified color support level.
    ///
    /// This method modifies the theme in place, converting colors from higher color support
    /// levels to lower ones as needed. For example, `TrueColor` RGB values will be converted
    /// to the closest 256-color or basic ANSI color equivalents.
    ///
    /// # Arguments
    /// * `target` - The target color support level to convert to
    pub fn convert_to_color_support(&mut self, target: ColorSupport) {
        match target {
            ColorSupport::TrueColor => (), // No conversion needed
            ColorSupport::Color256 => self.convert_to_256(),
            ColorSupport::Basic | ColorSupport::Undetermined => self.convert_to_basic(),
            ColorSupport::None => self.convert_to_none(),
        }
    }

    fn convert_to_256(&mut self) {
        // Convert each color in the palette
        for style in self.palette.iter_mut() {
            if let Some(ColorInfo {
                value: ColorValue::TrueColor { rgb },
                ..
            }) = &mut style.foreground
            {
                let index = find_closest_color(*rgb);
                // Corrected style conversion
                style.foreground = Some(ColorInfo::color256(index));
            }
        }
        self.min_color_support = ColorSupport::Color256;
    }
    fn convert_to_basic(&mut self) {
        // Convert each color in the palette
        for style in self.palette.iter_mut() {
            if let Some(ColorInfo { value, .. }) = &mut style.foreground {
                match value {
                    ColorValue::TrueColor { rgb } => {
                        let index = Self::convert_rgb_to_ansi(rgb[0], rgb[1], rgb[2]);
                        // Use the index directly to get the AnsiCode
                        let code = if index <= 7 {
                            index + 30
                        } else {
                            index + 90 - 8
                        };
                        let ansi = format!("\x1b[{code}m");
                        // Corrected style conversion
                        style.foreground = Some(ColorInfo::basic(&ansi, index));
                    }
                    ColorValue::Color256 { color256 } => {
                        let index = *color256;
                        // Use the index directly to get the AnsiCode
                        let code = if index <= 7 {
                            index + 30
                        } else {
                            index + 90 - 8
                        };
                        let ansi = format!("\x1b[{code}m");
                        // Corrected style conversion
                        style.foreground = Some(ColorInfo::basic(&ansi, index));
                    }
                    ColorValue::Basic { .. } => (), // Already basic color
                }
            }
        }
        self.min_color_support = ColorSupport::Basic;
    }

    fn convert_to_none(&mut self) {
        // Convert each color in the palette
        for style in self.palette.iter_mut() {
            let index = 0;
            let code = index + 30;
            let ansi = format!("\x1b[{code}m");
            style.foreground = Some(ColorInfo::basic(&ansi, index));
            style.reset();
        }
        self.min_color_support = ColorSupport::None;
    }

    // Convenience methods for styling text with this theme

    /// Style text with the Heading1 role from this theme
    pub fn heading1<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Heading1))
    }

    /// Style text with the Heading2 role from this theme
    pub fn heading2<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Heading2))
    }

    /// Style text with the Heading3 role from this theme
    pub fn heading3<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Heading3))
    }

    /// Style text with the Error role from this theme
    pub fn error<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Error))
    }

    /// Style text with the Warning role from this theme
    pub fn warning<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Warning))
    }

    /// Style text with the Success role from this theme
    pub fn success<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Success))
    }

    /// Style text with the Info role from this theme
    pub fn info_text<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Info))
    }

    /// Style text with the Emphasis role from this theme
    pub fn emphasis<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Emphasis))
    }

    /// Style text with the Code role from this theme
    pub fn code<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Code))
    }

    /// Style text with the Normal role from this theme
    pub fn normal<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Normal))
    }

    /// Style text with the Subtle role from this theme
    pub fn subtle<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Subtle))
    }

    /// Style text with the Hint role from this theme
    pub fn hint<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Hint))
    }

    /// Style text with the Debug role from this theme
    pub fn debug<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Debug))
    }

    /// Style text with the Link role from this theme
    pub fn link<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Link))
    }

    /// Style text with the Quote role from this theme
    pub fn quote<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Quote))
    }

    /// Style text with the Commentary role from this theme
    pub fn commentary<S: AsRef<str>>(&self, text: S) -> StyledString {
        text.as_ref().style_with(self.style_for(Role::Commentary))
    }

    /// Execute a closure with this theme temporarily set as the active theme.
    /// This allows using the normal role-based styling methods (.`emphasis()`,
    /// .`error()`, etc.)
    /// while having them use this theme instead of the globally active one.
    ///
    /// # Example
    /// ```rust
    /// use thag_styling::{Theme, Styleable, StyledPrint};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let guest_theme = Theme::get_builtin("Basic Light")?;
    ///
    /// guest_theme.with_context(|| {
    ///     // These methods now use the guest theme instead of the active one
    ///     "Success!".success().println();
    ///     "Error occurred".error().println();
    ///     "Important info".emphasis().println();
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_context<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Set the thread-local theme context
        THEME_CONTEXT.with(|context| {
            let previous = context.borrow().clone();
            *context.borrow_mut() = Some(self.clone());

            // Execute the closure
            let result = f();

            // Restore previous context
            *context.borrow_mut() = previous;
            result
        })
    }

    /// Gets the current theme context, falling back to the active theme if none is set
    fn current_theme_context() -> Self {
        THEME_CONTEXT.with(|context| {
            context.borrow().as_ref().map_or_else(
                || TermAttributes::get_or_init().theme.clone(),
                std::clone::Clone::clone,
            )
        })
    }
}

/// Converts a 256-color palette index to RGB values.
///
/// This function maps color indices (0-255) to their corresponding RGB values:
/// - 0-15: Standard ANSI colors (black, red, green, etc.)
/// - 16-231: 6×6×6 RGB color cube
/// - 232-255: Grayscale colors from dark to light
///
/// # Arguments
/// * `index` - The color index (0-255) to convert to RGB
///
/// # Returns
/// A tuple of (red, green, blue) values, each in the range 0-255
///
/// # Examples
/// ```
/// use thag_styling::styling::index_to_rgb;
///
/// // Basic colors
/// assert_eq!(index_to_rgb(0), (0, 0, 0));     // Black
/// assert_eq!(index_to_rgb(15), (255, 255, 255)); // White
///
/// // Color cube
/// let rgb = index_to_rgb(196); // Bright red in 256-color palette
/// assert_eq!(rgb, (255, 0, 0));
///
/// // Grayscale
/// let gray = index_to_rgb(244); // Mid-gray
/// assert_eq!(gray, (128, 128, 128));
/// ```
#[must_use]
pub fn index_to_rgb(index: u8) -> [u8; 3] {
    if index < 16 {
        // Standard ANSI colors
        return match index {
            0 => [0, 0, 0],
            1 => [128, 0, 0],
            2 => [0, 128, 0],
            3 => [128, 128, 0],
            4 => [0, 0, 128],
            5 => [128, 0, 128],
            6 => [0, 128, 128],
            7 => [192, 192, 192],
            8 => [128, 128, 128],
            9 => [255, 0, 0],
            10 => [0, 255, 0],
            11 => [255, 255, 0],
            12 => [0, 0, 255],
            13 => [255, 0, 255],
            14 => [0, 255, 255],
            15 => [255, 255, 255],
            _ => unreachable!(),
        };
    }

    if index >= 232 {
        // Grayscale
        let gray = (index - 232) * 10 + 8;
        return [gray, gray, gray];
    }

    // Color cube
    let index = index - 16;
    let r = (index / 36) * 51;
    let g = ((index % 36) / 6) * 51;
    let b = (index % 6) * 51;
    [r, g, b]
}

fn fallback_theme(term_bg_luma: TermBgLuma) -> StylingResult<Theme> {
    let name = if term_bg_luma == TermBgLuma::Light {
        "basic_light"
    } else {
        "basic_dark"
    };

    Theme::get_theme_runtime_or_builtin(name).or_else(|_| {
        // Absolute fallback using THEME_INDEX if runtime loading fails
        Theme::from_toml(
            name,
            THEME_INDEX
                .get(name)
                .expect("Basic theme not found")
                .content,
        )
    })
}

#[allow(clippy::missing_const_for_fn)]
#[cfg(feature = "config")]
fn get_preferred_styling(
    term_bg_luma: TermBgLuma,
    config: &thag_common::config::Config,
) -> &Vec<String> {
    match term_bg_luma {
        TermBgLuma::Light => &config.styling.preferred_light,
        TermBgLuma::Dark => &config.styling.preferred_dark,
        #[cfg(not(feature = "color_detect"))]
        TermBgLuma::Undetermined => &config.styling.preferred_dark,
        #[cfg(feature = "color_detect")]
        TermBgLuma::Undetermined => match *terminal::get_term_bg_luma() {
            TermBgLuma::Light => &config.styling.preferred_light,
            TermBgLuma::Dark | TermBgLuma::Undetermined => &config.styling.preferred_dark,
        },
    }
}

#[allow(clippy::missing_const_for_fn)]
#[cfg(feature = "config")]
fn get_fallback_styling(
    term_bg_luma: TermBgLuma,
    config: &thag_common::config::Config,
) -> &Vec<String> {
    match term_bg_luma {
        TermBgLuma::Light => &config.styling.fallback_light,
        TermBgLuma::Dark => &config.styling.fallback_dark,
        #[cfg(not(feature = "color_detect"))]
        TermBgLuma::Undetermined => &config.styling.fallback_dark,
        #[cfg(feature = "color_detect")]
        TermBgLuma::Undetermined => match *terminal::get_term_bg_luma() {
            TermBgLuma::Light => &config.styling.fallback_light,
            TermBgLuma::Dark | TermBgLuma::Undetermined => &config.styling.fallback_dark,
        },
    }
}

// Helper to calculate color distance
#[allow(clippy::items_after_statements)]
// #[cfg(feature = "color_detect")]
#[must_use]
fn color_distance(c1: [u8; 3], c2: [u8; 3]) -> f32 {
    let dr = (f32::from(c1[0]) - f32::from(c2[0])).powi(2);
    let dg = (f32::from(c1[1]) - f32::from(c2[1])).powi(2);
    let db = (f32::from(c1[2]) - f32::from(c2[2])).powi(2);
    (dr + dg + db).sqrt()
}

/// Convert a hex color string to RGB.
///
/// # Errors
///
/// This function will return an error if the input string is not a valid hex color.
// #[cfg(feature = "color_detect")]
fn hex_to_rgb(hex: &str) -> StylingResult<[u8; 3]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            Ok([r, g, b])
        } else {
            Err(StylingError::Parse)
        }
    } else {
        Err(StylingError::Parse)
    }
}

// Helper to check a single style
fn validate_style(style: &Style, min_support: ColorSupport) -> StylingResult<()> {
    style.foreground.as_ref().map_or_else(
        || Ok(()),
        |color_info| match &color_info.value {
            ColorValue::Basic { .. } => Ok(()), // Basic is always valid
            ColorValue::Color256 { color256: _ } => {
                if min_support < ColorSupport::Color256 {
                    Err(ThemeError::InvalidColorValue(
                        "256-color value used in theme requiring only basic colors".into(),
                    )
                    .into())
                } else {
                    Ok(())
                }
            }
            ColorValue::TrueColor { rgb: _ } => {
                if min_support < ColorSupport::TrueColor {
                    Err(ThemeError::InvalidColorValue(
                        "True color value used in theme requiring only 256 colors".into(),
                    )
                    .into())
                } else {
                    Ok(())
                }
            }
        },
    )
}

// Convenience macros
/// Conditionally logs a message with verbosity control and styling.
///
/// This macro checks if logging is enabled and the verbosity level meets the threshold,
/// then applies appropriate styling based on the role and logs the message.
///
/// The naming is a shorthand reminder of the parameter order: Color (role), Verbosity, println!-style arguments.
///
/// # Arguments
/// * `$role` - The role that determines the styling to apply
/// * `$verbosity` - The verbosity level required for this message
/// * `$($arg:tt)*` - Format string and arguments, same as `println!` macro
///
/// # Examples
/// ```
/// use thag_styling::cvprtln;
/// use thag_styling::Verbosity;
/// use thag_styling::Role;
/// let details = "todos los detalles";
/// cvprtln!(Role::Info, Verbosity::VV, "Detailed info: {details}");
/// ```
#[macro_export]
macro_rules! cvprtln {
    ($style:expr, $verbosity:expr, $($arg:tt)*) => {{
        let verbosity = $crate::get_verbosity();
        if $verbosity <= verbosity {
            $crate::cprtln!($style, $($arg)*)
        }
    }};
}

/// Styled verbosity-gated print line macro (replacement for `cvprtln!`)
/// Conditionally logs a message with verbosity control and styling.
#[macro_export]
macro_rules! svprtln {
    ($style:expr, $verbosity:expr, $($arg:tt)*) => {{
        $crate::cvprtln!($style, $verbosity, $($arg)*)
    }};
}

fn base_distance(c1: [u8; 3], c2: [u8; 3]) -> u32 {
    let dr = f64::from(i32::from(c1[0]) - i32::from(c2[0])) * 0.3;
    let dg = f64::from(i32::from(c1[1]) - i32::from(c2[1])) * 0.59;
    let db = f64::from(i32::from(c1[2]) - i32::from(c2[2])) * 0.11;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let base_distance = db.mul_add(db, dr.mul_add(dr, dg * dg)) as u32;
    base_distance
}

/// Finds the closest color in the 256-color palette to the given RGB value.
///
/// This function maps an RGB color to the closest match in the standard 256-color
/// terminal palette, which consists of:
/// - 16 basic ANSI colors (0-15)
/// - 216 colors in a 6×6×6 RGB cube (16-231)
/// - 24 grayscale colors (232-255)
///
/// The function prioritizes grayscale colors when all RGB components are equal,
/// then falls back to finding the closest match in the 6×6×6 color cube.
///
/// # Arguments
/// * `rgb` - A tuple of (red, green, blue) values, each in the range 0-255
///
/// # Returns
/// The index (0-255) of the closest color in the 256-color palette
///
/// # Examples
/// ```
/// use thag_styling::find_closest_color;
///
/// // Pure red should map to a red in the color cube
/// let red_index = find_closest_color((255, 0, 0));
/// assert!(red_index >= 16); // Not a basic ANSI color
///
/// // Gray should map to the grayscale range
/// let gray_index = find_closest_color((128, 128, 128));
/// assert!(gray_index >= 232);
/// ```
#[must_use]
pub fn find_closest_color(rgb: [u8; 3]) -> u8 {
    const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];

    // Handle grays first (232-255)
    if rgb[0] == rgb[1] && rgb[1] == rgb[2] {
        let r = rgb[0];
        if r < 4 {
            return 16; // black
        }
        if r > 238 {
            return 231; // white
        }
        // Find closest gray (232-255)
        let gray_idx = (f32::from(r) - 8.0) / 10.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let gray_idx = gray_idx.round() as u8;
        if gray_idx < 24 {
            return 232 + gray_idx;
        }
    }

    // Find closest color in the 6x6x6 color cube (16-231)
    let find_closest = |v: u8| {
        u8::try_from(
            STEPS
                .iter()
                .enumerate()
                .min_by_key(|(_i, &s)| (i16::from(s) - i16::from(v)).abs())
                .map_or(0, |(i, _)| i),
        )
        .map_or(0, |v| v)
    };

    let r_idx = find_closest(rgb[0]);
    let g_idx = find_closest(rgb[1]);
    let b_idx = find_closest(rgb[2]);

    // eprintln!(
    //     "Closest color to {rgb:?} is ({}, {}, {}) = {}",
    //     STEPS[r_idx as usize],
    //     STEPS[g_idx as usize],
    //     STEPS[b_idx as usize],
    //     16 + (36 * r_idx) + (6 * g_idx) + b_idx
    // );
    16 + (36 * r_idx) + (6 * g_idx) + b_idx
}

fn find_closest_basic_color(rgb: [u8; 3]) -> u8 {
    // Use weighted Euclidean distance for better perceptual matching

    #[allow(clippy::cast_possible_truncation)]
    BASIC_COLORS
        .iter()
        .enumerate()
        .min_by(|&(_, &c1), &(_, &c2)| {
            base_distance(rgb, c1)
                .partial_cmp(&base_distance(rgb, c2))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map_or(0, |(i, _)| i as u8)
}

#[must_use]
/// Returns the RGB color values for a given color index in the 256-color palette.
///
/// This function maps color indices (0-255) to their corresponding RGB values:
/// - 0-15: Basic ANSI colors (black, red, green, etc.)
/// - 16-231: 6×6×6 RGB color cube
/// - 232-255: Grayscale colors from dark to light
///
/// # Arguments
/// * `color` - The color index (0-255) to convert to RGB
///
/// # Returns
/// A tuple of (red, green, blue) values, each in the range 0-255
///
/// # Examples
/// ```
/// use thag_styling::get_rgb;
///
/// // Basic colors
/// assert_eq!(get_rgb(0), (0, 0, 0));     // Black
/// assert_eq!(get_rgb(15), (255, 255, 255)); // White
///
/// // Color cube
/// let rgb = get_rgb(196); // Bright red in 256-color palette
/// assert_eq!(rgb, (255, 0, 0));
///
/// // Grayscale
/// let gray = get_rgb(244); // Mid-gray
/// assert_eq!(gray, (128, 128, 128));
/// ```
pub const fn get_rgb(color: u8) -> [u8; 3] {
    const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    match color {
        0..=15 => BASIC_COLORS[color as usize],
        16..=231 => {
            let color = color - 16;
            let r = STEPS[((color / 36) % 6) as usize];
            let g = STEPS[((color / 6) % 6) as usize];
            let b = STEPS[(color % 6) as usize];
            [r, g, b]
        }
        232..=255 => {
            let gray = 8 + (color - 232) * 10;
            [gray, gray, gray]
        }
    }
}

/// Main method for testing purposes
///
/// # Errors
///
/// Returns any errors encountered.
#[allow(dead_code)]
pub fn main() -> StylingResult<()> {
    let term_attrs = TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);
    let color_support = term_attrs.color_support;
    let theme = &term_attrs.theme;
    let header_style = Style::for_role(Role::Normal).underline();
    let print_header = |arg| println!("{}", header_style.clone().paint(arg));

    // Section 1: ANSI / Xterm 256 color palette
    if color_support >= ColorSupport::Color256 {
        println!();
        let col_width = 25;
        print_header("ANSI / Xterm 256 color palette:\n");
        let color = Color::fixed(u8::from(&Role::HD1));
        println!(
            "{}{}{}{}",
            color.clone().paint(format!("{:<col_width$}", "Normal")),
            color
                .clone()
                .italic()
                .paint(format!("{:<col_width$}", "Italic")),
            color
                .clone()
                .bold()
                .paint(format!("{:<col_width$}", "Bold")),
            color
                .bold()
                .italic()
                .paint(format!("{:<col_width$}", "Bold Italic")),
            // color.paint(format!("{:<col_width$}", "Normal"))
        );
        println!();
    }

    let theme_name = match theme.term_bg_luma {
        TermBgLuma::Light => "basic_light",
        TermBgLuma::Dark | TermBgLuma::Undetermined => "basic_dark",
    };
    let theme = Theme::get_theme_runtime_or_builtin(theme_name)?;

    // Section 2: ANSI-16 color palette using basic styles
    let header = format!("ANSI-16 color palette in use for {theme_name} theme:\n");
    print_header(&header);
    for role in Role::iter() {
        let style = theme.style_for(role);
        let content = format!("{role} message: role={role}, style={style:?}");
        println!("{}", style.paint(content));
    }

    println!();

    // Section 3: ANSI-16 palette using u8 colors
    let header = format!("ANSI-16 color palette in use for {theme_name} theme (converted via u8 and missing bold/dimmed/italic):\n");
    print_header(&header);
    for role in Role::iter() {
        let style = theme.style_for(role);
        // eprintln!("style={style:?}");
        if let Some(color_info) = style.foreground {
            let index: u8 = color_info.index;
            let color = Color::fixed(index);
            let content = format!("{role} message: role={role:?}, index={index}, color={color:?}");
            println!("{}", color.paint(content));
        }
    }

    println!();

    // Section 4: Current terminal color palette
    let term_attrs = TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);
    let theme = &term_attrs.theme;
    // let user_config = maybe_config();
    // let current = user_config.clone().unwrap_or_default();
    print_header("Color palette in use on this terminal:\n");
    display_theme_roles(theme);
    display_theme_details(theme);
    println!();

    // Section 5: Current terminal attributes
    print_header("This terminal's color attributes:\n");
    vprtln!(
        V::N,
        "Color support={}, theme={}: {}\n",
        color_support.style().bold().underline().dim(),
        theme.name.style().italic(),
        theme.description.style().reversed()
    );

    let name = "Error";
    println!("{name}"); // styled!(name), bold);

    cvprtln!(
        Role::Heading2,
        V::N,
        "Color support={}, theme={}: {}\nMore text to check if styling disrupted",
        // color_support.style().bold().underline().dim(),
        name, // styled!(name, italic, underline),
        theme.name.style().italic(),
        theme.description.style().reversed()
    );

    Ok(())
}

const BASIC_COLORS: [[u8; 3]; 16] = [
    [0, 0, 0],       // black
    [128, 0, 0],     // red
    [0, 128, 0],     // green
    [128, 128, 0],   // yellow
    [0, 0, 128],     // blue
    [128, 0, 128],   // magenta
    [0, 128, 128],   // cyan
    [192, 192, 192], // white
    [128, 128, 128], // bright black (gray)
    [255, 0, 0],     // bright red
    [0, 255, 0],     // bright green
    [255, 255, 0],   // bright yellow
    [0, 0, 255],     // bright blue
    [255, 0, 255],   // bright magenta
    [0, 255, 255],   // bright cyan
    [255, 255, 255], // bright white
];

/// Displays all available roles and their corresponding styles in the given theme.
///
/// This function prints a formatted table showing each role name styled according
/// to its theme configuration, along with a description of what that role represents.
/// This is useful for visualizing how different message types will appear when
/// using the current theme.
///
/// # Arguments
/// * `theme` - The theme to display role styles for
///
/// # Examples
/// ```
/// use thag_styling::{Theme, display_theme_roles};
///
/// let theme = Theme::get_builtin("dracula")?;
/// display_theme_roles(&theme);
/// # Ok::<(), thag_styling::StylingError>(())
/// ```
pub fn display_theme_roles(theme: &Theme) {
    // Role descriptions
    const ROLE_DOCS: &[(&str, &str)] = &[
        ("Heading1", "Primary heading, highest prominence"),
        ("Heading2", "Secondary heading"),
        ("Heading3", "Tertiary heading"),
        ("Error", "Critical errors requiring immediate attention"),
        ("Warning", "Important cautions or potential issues"),
        ("Success", "Positive completion or status messages"),
        ("Info", "General informational messages"),
        ("Emphasis", "Text that needs to stand out"),
        ("Code", "Code snippets or commands"),
        ("Normal", "Standard text, default prominence"),
        ("Subtle", "De-emphasized but clearly visible text"),
        ("Hint", "Completion suggestions or placeholder text"),
        ("Debug", "Development/diagnostic information"),
        ("Link", "Links and URLs"),
        ("Quote", "Quoted text or citations"),
        ("Commentary", "Commentary or explanatory notes"),
    ];

    let col1_width = ROLE_DOCS
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0)
        + 2; // Base width on raw text length

    // println!("\n\tRole Styles:");
    println!(
        "\n\t{} {}",
        theme.style_for(Role::Normal).paint("Role styles:"),
        theme.style_for(Role::Heading1).paint(&theme.name)
    );
    // println!("\t{}", "═".repeat(80));
    println!("\t{}", "─".repeat(80));

    for (role_name, description) in ROLE_DOCS {
        // Convert role name to Role enum variant
        #[allow(clippy::match_same_arms)]
        let role = match *role_name {
            "Heading1" => Role::Heading1,
            "Heading2" => Role::Heading2,
            "Heading3" => Role::Heading3,
            "Error" => Role::Error,
            "Warning" => Role::Warning,
            "Success" => Role::Success,
            "Info" => Role::Info,
            "Emphasis" => Role::Emphasis,
            "Code" => Role::Code,
            "Normal" => Role::Normal,
            "Subtle" => Role::Subtle,
            "Hint" => Role::Hint,
            "Debug" => Role::Debug,
            "Link" => Role::Link,
            "Quote" => Role::Quote,
            "Commentary" => Role::Commentary,
            _ => Role::Normal,
        };

        // Get style for this role
        let style = theme.style_for(role);

        // Print role name in its style, followed by description
        let styled_name = style.paint(role_name);
        let rgb = style
            .foreground
            .as_ref()
            .map(|color_info| match &color_info.value {
                ColorValue::TrueColor { rgb } => *rgb,
                ColorValue::Color256 { color256 } => index_to_rgb(*color256),
                ColorValue::Basic { index, .. } => index_to_rgb(*index),
            });

        // let styled_rgb = style.paint(format!("{:?}", style.foreground.and_then(|v| v.value)));
        let padding = " ".repeat(col1_width.saturating_sub(role_name.len()));
        let hex = rgb.map_or("N/A".to_string(), |rgb| rgb_to_hex(&rgb));
        let styled_hex = style.paint(hex);

        print!("\t{styled_hex} {styled_name} {padding}");
        println!("│ {description}");
    }
    println!("\t{}", "─".repeat(80));
}

#[allow(clippy::too_many_lines)]
/// Displays detailed theme information including theme attributes and terminal attributes.
///
/// This function outputs comprehensive information about the current theme and terminal
/// configuration, including:
/// - Theme metadata (name, type, file path, description)
/// - Background color information
/// - Color palette support level
/// - Terminal attributes (how determined, color support, background properties)
///
/// The output is formatted with appropriate styling and presented in a structured
/// table format with clear visual separators.
///
/// # Examples
/// ```
/// use thag_styling::{display_theme_details, Theme};
/// let theme = Theme::get_builtin("black-metal-bathory_base16")?;
/// display_theme_details(&theme);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn display_theme_details(theme: &Theme) {
    let theme_bgs = &theme.bg_rgbs;
    let theme_bgs = if theme_bgs.is_empty() {
        &theme
            .backgrounds
            .iter()
            .filter_map(|hex| hex_to_rgb(hex).ok())
            .collect()
    } else {
        theme_bgs
    };
    // eprintln!(
    //     "theme_bgs={theme_bgs:?}, backgrounds={:?}",
    //     theme.backgrounds
    // );
    let rgb_disp = if theme_bgs.is_empty() {
        "None".to_string()
    } else {
        // Display all background colors for the theme
        match theme_bgs.len().cmp(&1) {
            std::cmp::Ordering::Less => "None".to_string(),
            std::cmp::Ordering::Equal => dual_format_rgb(theme_bgs[0]),
            std::cmp::Ordering::Greater => theme_bgs
                .iter()
                .map(|rgb| dual_format_rgb(*rgb))
                .collect::<Vec<_>>()
                .join(", "),
        }
    };

    let theme_docs: &[(&str, &str)] = &[
        ("Theme", &theme.name),
        ("Luminance", &theme.term_bg_luma.to_string()),
        (
            "Type",
            if theme.is_builtin {
                "built-in"
            } else {
                "custom"
            },
        ),
        ("File", &theme.filename.display().to_string()),
        ("Description", &theme.description),
        ("Background", &rgb_disp),
        ("Palette", &theme.min_color_support.to_string()),
    ];

    let col1_width = theme_docs
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0)
        + 2; // Base width on raw text length

    let flower_box_len = 80;

    println!(
        "\n\t{}",
        theme.style_for(Role::Normal).paint("Theme attributes:")
    );
    println!("\t{}", "─".repeat(flower_box_len));

    for (attr, description) in theme_docs {
        let styled_name = theme.style_for(Role::Info).paint(attr);
        let padding = " ".repeat(col1_width.saturating_sub(attr.len()));

        print!("\t{styled_name}{padding}");

        // Get style for this role
        let style = theme.style_for(Role::Heading1);

        let description = if *attr == "Theme" {
            style.paint(description)
        } else {
            (*description).to_string()
        };
        println!("│ {description}");
    }

    println!("\t{}\n", "─".repeat(flower_box_len));
}

/// Display terminal attributes information
///
/// Shows information about the current terminal configuration including
/// color support, background detection, and initialization method.
///
/// # Examples
/// ```
/// use thag_styling::{display_terminal_attributes, ColorInitStrategy, TermAttributes};
/// let term_attrs = TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);
/// let theme = &term_attrs.theme;
/// display_terminal_attributes(theme);
/// ```
pub fn display_terminal_attributes(theme: &Theme) {
    let term_attrs = TermAttributes::get_or_init();

    let how_initialized = term_attrs.how_initialized.to_string();
    let terminal_docs: &[(&str, &str)] = &[
        ("How attributes determined", how_initialized.as_str()),
        ("Color support", &term_attrs.color_support.to_string()),
        ("Background luminance", &term_attrs.term_bg_luma.to_string()),
        (
            "Background color",
            &term_attrs
                .term_bg_rgb
                .map_or("None".to_string(), dual_format_rgb),
        ),
    ];

    let col1_width = terminal_docs
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0)
        + 2; // Base width on raw text length

    let flower_box_len = 80;

    println!(
        "\n\t{}",
        theme.style_for(Role::Normal).paint("Terminal attributes:")
    );
    println!("\t{}", "─".repeat(flower_box_len));

    for (attr, description) in terminal_docs {
        let styled_name = theme.style_for(Role::Info).paint(attr);
        let padding = " ".repeat(col1_width.saturating_sub(attr.len()));

        print!("\t{styled_name}{padding}");
        println!("│ {description}");
    }

    println!("\t{}\n", "─".repeat(flower_box_len));
}

fn dual_format_rgb([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02x}{g:02x}{b:02x} = rgb({r}, {g}, {b})")
}

// ANSI text styles
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Effect {
    Bold,
    Dim,
    Italic,
    Reversed,
    Underline,
}

/// A styled text container that applies ANSI terminal effects to text.
///
/// This struct wraps text content with a collection of visual effects that can be
/// applied when the text is displayed in a terminal. Effects include bold, italic,
/// underline, dim, and reversed styling.
///
/// # Type Parameters
/// * `T` - The type of the text content to be styled
///
/// # Examples
/// ```
/// use thag_styling::{AnsiStyleExt, Styled};
///
/// let styled = "Hello".style().bold().italic();
/// println!("{styled}"); // Prints "Hello" in bold italic
/// ```
pub struct Styled<T> {
    text: T,
    effects: Vec<Effect>,
}

/// Extension trait for applying ANSI terminal styling to any displayable type.
///
/// This trait provides a convenient way to add terminal styling effects to any type
/// that implements `Display`. It converts the value to a string and wraps it in
/// a `Styled` container that can then have effects like bold, italic, underline,
/// etc. applied to it.
///
/// # Examples
/// ```
/// use thag_styling::AnsiStyleExt;
///
/// let styled = "Hello".style().bold().underline();
/// println!("{}", styled); // Prints "Hello" with bold and underline effects
///
/// let number = 42;
/// let styled_number = number.style().italic();
/// println!("{}", styled_number); // Prints "42" in italics
/// ```
pub trait AnsiStyleExt {
    /// Creates a `Styled` wrapper around the string representation of this value.
    ///
    /// This method converts the value to a string using its `Display` implementation
    /// and wraps it in a `Styled` container with no initial effects applied.
    /// Effects can then be chained using the methods on `Styled`.
    ///
    /// # Returns
    /// A `Styled<String>` that can have terminal effects applied to it
    ///
    /// # Examples
    /// ```
    /// use thag_styling::AnsiStyleExt;
    ///
    /// let basic = "text".style();
    /// let enhanced = "text".style().bold().italic();
    /// ```
    fn style(&self) -> Styled<String>;
}

impl<T> AnsiStyleExt for T
where
    T: fmt::Display,
{
    fn style(&self) -> Styled<String> {
        Styled {
            text: format!("{self}"),
            effects: Vec::new(),
        }
    }
}

impl<T> Styled<T> {
    /// Applies bold styling to the text.
    ///
    /// # Returns
    /// A new `Styled` instance with bold effect added to the existing effects.
    ///
    /// # Examples
    /// ```
    /// use thag_styling::AnsiStyleExt;
    /// let styled = "Hello".style().bold();
    /// println!("{}", styled); // Prints "Hello" in bold
    /// ```
    #[must_use]
    pub fn bold(mut self) -> Self {
        self.effects.push(Effect::Bold);
        self
    }

    /// Applies dim (faint) styling to the text.
    ///
    /// # Returns
    /// A new `Styled` instance with dim effect added to the existing effects.
    ///
    /// # Examples
    /// ```
    /// use thag_styling::AnsiStyleExt;
    /// let styled = "Hello".style().dim();
    /// println!("{}", styled); // Prints "Hello" in dim/faint text
    /// ```
    #[must_use]
    pub fn dim(mut self) -> Self {
        self.effects.push(Effect::Dim);
        self
    }

    /// Applies underline styling to the text.
    ///
    /// # Returns
    /// A new `Styled` instance with underline effect added to the existing effects.
    ///
    /// # Examples
    /// ```
    /// use thag_styling::AnsiStyleExt;
    /// let styled = "Hello".style().underline();
    /// println!("{}", styled); // Prints "Hello" with underline
    /// ```
    #[must_use]
    pub fn underline(mut self) -> Self {
        self.effects.push(Effect::Underline);
        self
    }

    /// Applies italic styling to the text.
    ///
    /// # Returns
    /// A new `Styled` instance with italic effect added to the existing effects.
    ///
    /// # Examples
    /// ```
    /// use thag_styling::AnsiStyleExt;
    /// let styled = "Hello".style().italic();
    /// println!("{}", styled); // Prints "Hello" in italics
    /// ```
    #[must_use]
    pub fn italic(mut self) -> Self {
        self.effects.push(Effect::Italic);
        self
    }

    /// Applies reversed (inverted foreground/background) styling to the text.
    ///
    /// # Returns
    /// A new `Styled` instance with reversed effect added to the existing effects.
    ///
    /// # Examples
    /// ```
    /// use thag_styling::AnsiStyleExt;
    /// let styled = "Hello".style().reversed();
    /// println!("{}", styled); // Prints "Hello" with inverted colors
    /// ```
    #[must_use]
    pub fn reversed(mut self) -> Self {
        self.effects.push(Effect::Reversed);
        self
    }

    fn to_ansi_code(&self) -> String {
        let mut codes = Vec::new();

        for style in &self.effects {
            codes.push(match style {
                Effect::Bold => "1",
                Effect::Dim => "2",
                Effect::Italic => "3",
                Effect::Reversed => "7",
                Effect::Underline => "4",
            });
        }

        format!("\x1b[{}m", codes.join(";"))
    }

    fn to_ansi_reset_codes(&self) -> String {
        let mut codes = Vec::new();

        for effect in &self.effects {
            codes.push(match effect {
                Effect::Bold | Effect::Dim => "22",
                Effect::Underline => "24",
                Effect::Italic => "23",
                Effect::Reversed => "27",
            });
        }

        // if self.fg.is_some() {
        //     codes.push("39"); // Reset foreground
        // }

        format!("\x1b[{}m", codes.join(";"))
    }
}

impl fmt::Display for Styled<String> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.to_ansi_code(),
            self.text,
            self.to_ansi_reset_codes()
        )
    }
}

/// Trait that allows both Style and Role to be used interchangeably with styling macros
pub trait Styler {
    /// Convert this item to a Style for use with styling macros
    fn to_style(&self) -> Style;

    /// Print a styled line using this style
    ///
    /// # Example
    /// ```ignore
    /// Role::Code.prtln("Hello {}", "world");
    /// Style::from(Role::Error).bold().prtln("Error: {}", message);
    /// (Role::Info).prtln("Using parentheses for disambiguation");
    /// ```
    fn prtln(&self, args: std::fmt::Arguments<'_>) {
        let content = format!("{args}");
        let painted = self.to_style().paint(content);
        println!("{painted}");
    }

    /// Print a styled line with verbosity gating using this style
    ///
    /// # Example
    /// ```ignore
    /// Role::Debug.vprtln(Verbosity::Verbose, "Debug: {}", value);
    /// (Role::Link).vprtln(Verbosity::Debug, "Link info");
    /// ```
    fn vprtln(&self, verbosity: thag_common::Verbosity, args: std::fmt::Arguments<'_>) {
        let current_verbosity = thag_common::get_verbosity();
        if verbosity <= current_verbosity {
            self.prtln(args);
        }
    }

    /// Apply this style to text and return the styled string
    ///
    /// # Example
    /// ```ignore
    /// let styled = Role::Error.paint("Error message");
    /// let styled = Role::Info.paint(format!("Value: {}", 42));
    /// ```
    fn paint<D>(&self, val: D) -> String
    where
        D: std::fmt::Display,
    {
        self.to_style().paint(val)
    }

    /// Return a Style with bold formatting enabled
    ///
    /// # Example
    /// ```ignore
    /// let styled = Role::Error.bold().paint("Bold error");
    /// Role::Info.bold().prtln(format_args!("Bold info: {}", value));
    /// ```
    fn bold(self) -> Style
    where
        Self: Sized,
    {
        self.to_style().bold()
    }

    /// Return a Style with italic formatting enabled
    ///
    /// # Example
    /// ```ignore
    /// let styled = Role::Emphasis.italic().paint("Italic text");
    /// Role::Code.italic().prtln(format_args!("Italic code: {}", code));
    /// ```
    fn italic(self) -> Style
    where
        Self: Sized,
    {
        self.to_style().italic()
    }

    /// Return a Style with dim formatting enabled
    ///
    /// # Example
    /// ```ignore
    /// let styled = Role::Normal.dim().paint("Dimmed text");
    /// Role::Info.dim().prtln(format_args!("Dimmed info: {}", info));
    /// ```
    fn dim(self) -> Style
    where
        Self: Sized,
    {
        self.to_style().dim()
    }

    /// Return a Style with underline formatting enabled
    ///
    /// # Example
    /// ```ignore
    /// let styled = Role::Warning.underline().paint("Underlined warning");
    /// Role::Error.underline().prtln(format_args!("Underlined error: {}", error));
    /// ```
    fn underline(self) -> Style
    where
        Self: Sized,
    {
        self.to_style().underline()
    }
}

/// A styled string that preserves styling context like colored's `ColoredString`
///
/// This type automatically handles reset sequences to ensure that when styled
/// strings are embedded within other styled strings, the outer styling context
/// is properly preserved.
#[derive(Clone, Debug)]
pub struct StyledString {
    content: String,
    style: Style,
}

impl StyledString {
    /// Create a new `StyledString` with the given content and style
    #[must_use]
    pub const fn new(content: String, style: Style) -> Self {
        Self { content, style }
    }

    /// Replace all reset sequences (\x1b[0m) with this style's ANSI codes
    ///
    /// This is the key innovation that allows perfect nesting - each level
    /// replaces the reset sequences of its embedded content with its own
    /// color codes, ensuring the outer context is always restored.
    ///
    /// Enhanced to properly reset text attributes (bold/dim, italic, underline)
    /// that would otherwise leak from inner styled content.
    fn replace_resets_with_style(&self) -> String {
        let reset = "\x1b[0m";
        let style_codes = self.style.to_ansi_codes();

        if !self.content.contains(reset) {
            return self.content.clone();
        }

        // Build the replacement string with attribute resets and style codes
        let replacement = self.build_reset_replacement(&style_codes);

        // Replace all resets with our enhanced replacement
        self.content.replace(reset, &replacement)
    }

    /// Build the complete styling prefix with attribute resets
    ///
    /// This creates a string that:
    /// 1. Always resets all text attributes to prevent bleeding
    /// 2. Applies the current style's ANSI codes
    fn build_styling_prefix(&self) -> String {
        let style_codes = self.style.to_ansi_codes();

        // Always include full attribute reset to prevent bleeding from outer contexts
        format!("\x1b[22;23;24m{}", style_codes)
    }

    /// Build the replacement string for inner resets
    ///
    /// This creates a replacement for \x1b[0m that uses the same prefix
    /// as the main styling to ensure consistent attribute handling
    fn build_reset_replacement(&self, _style_codes: &str) -> String {
        // Use the same prefix as the main styling
        self.build_styling_prefix()
    }

    /// Convert to final styled string with proper ANSI codes
    #[must_use]
    pub fn to_styled(&self) -> String {
        let content_with_replaced_resets = self.replace_resets_with_style();
        let styling_prefix = self.build_styling_prefix();
        format!("{}{}\x1b[0m", styling_prefix, content_with_replaced_resets)
    }

    /// Chain bold styling
    #[must_use]
    pub fn bold(self) -> Self {
        Self {
            content: self.content,
            style: self.style.bold(),
        }
    }

    /// Chain italic styling
    #[must_use]
    pub fn italic(self) -> Self {
        Self {
            content: self.content,
            style: self.style.italic(),
        }
    }

    /// Chain dim styling
    #[must_use]
    pub fn dim(self) -> Self {
        Self {
            content: self.content,
            style: self.style.dim(),
        }
    }

    /// Chain underline styling
    #[must_use]
    pub fn underline(self) -> Self {
        Self {
            content: self.content,
            style: self.style.underline(),
        }
    }
}

impl std::fmt::Display for StyledString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_styled())
    }
}

impl AsRef<str> for StyledString {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

impl From<StyledString> for String {
    fn from(styled: StyledString) -> Self {
        styled.to_styled()
    }
}

/// Extension trait to add convenience methods for `StyledString`
pub trait StyledPrint {
    /// Print the styled string to stdout
    fn print(self);

    /// Print the styled string to stdout with a newline
    fn println(self);

    /// Print the styled string with verbosity gating
    fn vprintln(self, verbosity: thag_common::Verbosity);
}

impl StyledPrint for StyledString {
    fn print(self) {
        print!("{}", self);
    }

    fn println(self) {
        println!("{}", self);
    }

    fn vprintln(self, verbosity: thag_common::Verbosity) {
        let current_verbosity = thag_common::get_verbosity();
        if verbosity <= current_verbosity {
            println!("{}", self);
        }
    }
}

// impl StyledPrint for StyledString {
//     fn print(self) {
//         print!("{}", self);
//     }

//     fn println(self) {
//         println!("{}", self);
//     }

//     fn vprintln(self, verbosity: thag_common::Verbosity) {
//         let current_verbosity = thag_common::get_verbosity();
//         if verbosity <= current_verbosity {
//             println!("{}", self);
//         }
//     }
// }

/// Extension trait for generating ANSI codes from Style
trait StyleAnsiExt {
    fn to_ansi_codes(&self) -> String;
}

impl StyleAnsiExt for Style {
    fn to_ansi_codes(&self) -> String {
        let mut codes = String::new();

        if let Some(color_info) = &self.foreground {
            let ansi = color_info.to_ansi_for_support(TermAttributes::get_or_init().color_support);
            codes.push_str(&ansi);
        }

        if self.bold {
            codes.push_str("\x1b[1m");
        }
        if self.italic {
            codes.push_str("\x1b[3m");
        }
        if self.dim {
            codes.push_str("\x1b[2m");
        }
        if self.underline {
            codes.push_str("\x1b[4m");
        }

        codes
    }
}

/// Trait that allows strings to be styled directly with automatic context preservation
///
/// # Examples
/// ```ignore
/// use thag_styling::{Role, Styleable};
///
/// let styled = "error message".style_with(Role::Error);
/// let nested = format!("Warning: {}", "critical".error()).warning();
/// ```
pub trait Styleable: std::fmt::Display {
    /// Style this text using the given styler, returning a `StyledString`
    fn style_with(&self, styler: impl Styler) -> StyledString;

    // Individual role methods for convenience (like colored's color methods)

    /// Style this text as an error message
    fn error(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.error(self.to_string())
    }

    /// Style this text as a warning message
    fn warning(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.warning(self.to_string())
    }

    /// Style this text as a success message
    fn success(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.success(self.to_string())
    }

    /// Style this text as an informational message
    fn info(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.info_text(self.to_string())
    }

    /// Style this text as emphasized text
    fn emphasis(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.emphasis(self.to_string())
    }

    /// Style this text as code
    fn code(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.code(self.to_string())
    }

    /// Style this text as normal text
    fn normal(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.normal(self.to_string())
    }

    /// Style this text as subtle text
    fn subtle(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.subtle(self.to_string())
    }

    /// Style this text as hint text
    fn hint(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.hint(self.to_string())
    }

    /// Style this text as debug information
    fn debug(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.debug(self.to_string())
    }

    /// Style this text as a link
    fn link(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.link(self.to_string())
    }

    /// Style this text as quoted content
    fn quote(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.quote(self.to_string())
    }

    /// Style this text as commentary or explanatory notes
    fn commentary(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.commentary(self.to_string())
    }

    /// Style this text as a primary heading
    fn heading1(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.heading1(self.to_string())
    }

    /// Style this text as a secondary heading
    fn heading2(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.heading2(self.to_string())
    }

    /// Style this text as a tertiary heading
    fn heading3(&self) -> StyledString {
        let theme = Theme::current_theme_context();
        theme.heading3(self.to_string())
    }
}

impl Styleable for &str {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new((*self).to_string(), styler.to_style())
    }
}

impl Styleable for String {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.clone(), styler.to_style())
    }
}

// Implementations for common numeric types
impl Styleable for i32 {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl Styleable for i64 {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl Styleable for u32 {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl Styleable for u64 {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl Styleable for f32 {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl Styleable for f64 {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl Styleable for usize {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl Styleable for isize {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl Styleable for bool {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl Styleable for char {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl Styleable for std::path::Display<'_> {
    fn style_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl Styler for StyledString {
    fn to_style(&self) -> Style {
        self.style.clone()
    }
}

impl Styler for Style {
    fn to_style(&self) -> Style {
        self.clone()
    }
}

impl Styler for &Style {
    fn to_style(&self) -> Style {
        (*self).clone()
    }
}

impl Styler for Role {
    fn to_style(&self) -> Style {
        Style::from(*self)
    }
}

impl Styler for &Role {
    fn to_style(&self) -> Style {
        Style::from(**self)
    }
}

/// Print line with embedded styled content
/// Format: `cprtln!(style: Style, "Lorem ipsum dolor {} amet", content: &str);`
/// Also accepts Role: `cprtln!(Role::Code, "Hello {}", "world");`
#[macro_export]
macro_rules! cprtln {
    ($style:expr, $($arg:tt)*) => {{
        let content = format!("{}", format_args!($($arg)*));
        let painted = $crate::styling::Styler::to_style(&$style).paint(content);
        println!("{painted}");
    }};
}

/// Styled print line macro (replacement for `cprtln!`)
/// Format: `sprtln!(style: Style, "Lorem ipsum dolor {} amet", content: &str);`
/// Also accepts Role: `sprtln!(Role::Code, "Hello {}", "world");`
#[macro_export]
macro_rules! sprtln {
    ($style:expr, $($arg:tt)*) => {{
        $crate::cprtln!($style, $($arg)*)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    static MOCK_THEME_DETECTION: AtomicBool = AtomicBool::new(false);
    static BLACK_BG: &[u8; 3] = &[0, 0, 0];

    impl TermAttributes {
        fn with_mock_theme(color_support: ColorSupport, term_bg_luma: TermBgLuma) -> Self {
            MOCK_THEME_DETECTION.store(true, Ordering::SeqCst);
            let theme_name = match (color_support, term_bg_luma) {
                (ColorSupport::Basic | ColorSupport::Undetermined, TermBgLuma::Light) => {
                    "basic_light"
                }
                (
                    ColorSupport::Basic | ColorSupport::Undetermined,
                    TermBgLuma::Dark | TermBgLuma::Undetermined,
                ) => "basic_dark",
                (ColorSupport::None, _) => "none",
                (ColorSupport::Color256, TermBgLuma::Light) => "github",
                (
                    ColorSupport::Color256 | ColorSupport::TrueColor,
                    TermBgLuma::Dark | TermBgLuma::Undetermined,
                ) => "dracula",
                (ColorSupport::TrueColor, TermBgLuma::Light) => "one-light",
            };
            let theme =
                Theme::get_theme_runtime_or_builtin_with_color_support(theme_name, color_support)
                    .expect("Failed to load or resolve builtin theme {theme_name}");
            Self::new(color_support, Some(BLACK_BG).copied(), term_bg_luma, theme)
        }
    }

    // use std::io::Write;

    // Use a static Mutex for test output collection
    static TEST_OUTPUT: Mutex<Vec<String>> = Mutex::new(Vec::new());

    // #[profiled]
    fn init_test_output() {
        if let Ok(mut guard) = TEST_OUTPUT.lock() {
            guard.clear();
            guard.push(String::new());
        }
    }

    // #[profiled]
    fn get_test_output() -> Vec<String> {
        match TEST_OUTPUT.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => Vec::new(),
        }
    }

    // #[profiled]
    fn flush_test_output() {
        if let Ok(guard) = TEST_OUTPUT.lock() {
            let mut stdout = std::io::stdout();
            for line in guard.iter() {
                writeln!(stdout, "{}", line).expect("Failed to write to stdout");
            }
        }
    }

    // Tests that need access to internal implementation
    #[test]
    #[serial]
    fn test_styling_default_theme_with_mock() {
        init_test_output();
        let term_attrs = TermAttributes::with_mock_theme(ColorSupport::Color256, TermBgLuma::Dark);
        let defaulted = term_attrs.term_bg_luma;
        // assert!(matches!(defaulted, TermBgLuma::Dark)); // TODO alt: out?
        assert_eq!(defaulted, TermBgLuma::Dark);
        println!();
        let output = get_test_output();
        assert!(!output.is_empty());
        flush_test_output(); // Write captured output to stdout
    }

    #[test]
    #[serial]
    fn test_styling_color_support_levels() {
        init_test_output();
        let none = TermAttributes::with_mock_theme(ColorSupport::None, TermBgLuma::Dark);
        let basic = TermAttributes::with_mock_theme(ColorSupport::Basic, TermBgLuma::Dark);
        let color256 = TermAttributes::with_mock_theme(ColorSupport::Color256, TermBgLuma::Dark);
        let true_color = TermAttributes::with_mock_theme(ColorSupport::TrueColor, TermBgLuma::Dark);

        let test_role = Role::Error;

        let none_style = style_for_theme_and_role(&none.theme, test_role);
        // No color support should return plain text
        assert_eq!(none_style.paint("test"), "test");

        // Basic support should use ANSI 16 colors
        vprtln!(V::VV, "basic={basic:#?}");
        let basic_style = style_for_theme_and_role(&basic.theme, test_role);
        let painted = basic_style.paint("test");
        vprtln!(V::VV, "painted={painted:?}, style={basic_style:?}");
        assert!(painted.contains("\x1b[31m"));
        assert!(painted.ends_with("\u{1b}[0m"));

        // Color_256 support should use a different ANSI string from basic
        let color256_style = style_for_theme_and_role(&color256.theme, test_role);
        let painted = color256_style.paint("test");
        vprtln!(V::VV, "painted={painted:?}");
        assert!(painted.contains("\x1b[38;5;"));
        assert!(painted.ends_with("\u{1b}[0m"));

        // TrueColor support should use RGB formatting
        let true_color_style = style_for_theme_and_role(&true_color.theme, test_role);
        let painted = true_color_style.paint("test");
        vprtln!(V::VV, "painted={painted:?}");
        dbg!(&painted);
        assert!(painted.contains("\x1b[38;2;"));
        assert!(painted.ends_with("\u{1b}[0m"));
        let output = get_test_output();
        assert!(!output.is_empty());
        flush_test_output(); // Write captured output to stdout
    }

    #[test]
    #[serial]
    fn test_styling_theme_variations() {
        init_test_output();
        let attrs_light =
            TermAttributes::with_mock_theme(ColorSupport::Color256, TermBgLuma::Light);
        let attrs_dark = TermAttributes::with_mock_theme(ColorSupport::Color256, TermBgLuma::Dark);

        let test_role = Role::Heading1;

        let light_style = style_for_theme_and_role(&attrs_light.theme, test_role);
        let heading_light = light_style.paint("test");

        let dark_style = style_for_theme_and_role(&attrs_dark.theme, test_role);
        let heading_dark = dark_style.paint("test");

        // Light and dark themes should produce different colors
        assert_ne!(heading_light, heading_dark);
        let output = get_test_output();
        flush_test_output(); // Write captured output to stdout
        assert!(!output.is_empty());
        flush_test_output(); // Write captured output to stdout
    }

    #[test]
    #[serial]
    fn test_styling_style_attributes() {
        init_test_output();
        let attrs = TermAttributes::with_mock_theme(ColorSupport::Color256, TermBgLuma::Dark);

        // Heading1 should be bold
        let error_style = style_for_theme_and_role(&attrs.theme, Role::Heading1);
        let painted = error_style.paint("test");
        eprintln!(
            "theme={}, error_style={error_style:?}, painted={painted:?}",
            attrs.theme.name
        );
        assert!(painted.contains("\x1b[1m"));

        // Hint should be italic
        let hint_style = style_for_theme_and_role(&attrs.theme, Role::Hint);
        let painted = hint_style.paint("test");
        eprintln!(
            "theme={}, hint_style={hint_style:?}, painted={painted:?}",
            attrs.theme.name
        );
        let output = get_test_output();
        assert!(painted.contains("\x1b[3m"));
        flush_test_output(); // Write captured output to stdout
        assert!(!output.is_empty());
        flush_test_output(); // Write captured output to stdout
    }

    #[test]
    #[serial]
    fn test_styling_load_dracula_theme() -> StylingResult<()> {
        init_test_output();
        let theme = Theme::load_from_file(Path::new("themes/built_in/dracula.toml"))?;

        // Check theme metadata
        assert_eq!(theme.term_bg_luma, TermBgLuma::Dark);
        assert_eq!(theme.min_color_support, ColorSupport::TrueColor);
        assert_eq!(theme.bg_rgbs, vec![[40, 42, 54]]);

        // Check a few key styles
        if let ColorValue::TrueColor { rgb } = &theme
            .palette
            .heading1
            .foreground
            .expect("Failed to load heading1 foreground color")
            .value
        {
            assert_eq!(rgb, &[250, 151, 207]);
        } else {
            panic!("Expected TrueColor for heading1");
        }

        // Check style attributes
        assert!(theme.palette.heading1.bold);
        assert!(!theme.palette.normal.bold);
        assert!(theme.palette.hint.italic);

        let output = get_test_output();
        flush_test_output(); // Write captured output to stdout
        assert!(!output.is_empty());
        flush_test_output(); // Write captured output to stdout
        Ok(())
    }

    #[test]
    #[serial]
    fn test_styling_dracula_validation() -> StylingResult<()> {
        init_test_output();
        let theme = Theme::load_from_file(Path::new("themes/built_in/dracula.toml"))?;

        // Should succeed with TrueColor support and dark background
        assert!(theme
            .validate(&ColorSupport::TrueColor, &TermBgLuma::Dark)
            .is_ok());

        // Should fail with insufficient color support
        assert!(theme
            .validate(&ColorSupport::Color256, &TermBgLuma::Dark)
            .is_err());

        // Should fail with wrong background
        assert!(theme
            .validate(&ColorSupport::TrueColor, &TermBgLuma::Light)
            .is_err());

        let output = get_test_output();
        flush_test_output(); // Write captured output to stdout
        assert!(!output.is_empty());
        flush_test_output(); // Write captured output to stdout

        Ok(())
    }

    #[test]
    #[serial]
    fn test_styling_color_support_ordering() {
        init_test_output();
        assert!(ColorSupport::None < ColorSupport::Basic);
        assert!(ColorSupport::Basic < ColorSupport::Color256);
        assert!(ColorSupport::Color256 < ColorSupport::TrueColor);

        // Or more comprehensively:
        let supports = [
            ColorSupport::Undetermined,
            ColorSupport::None,
            ColorSupport::Basic,
            ColorSupport::Color256,
            ColorSupport::TrueColor,
        ];

        for i in 0..supports.len() - 1 {
            assert!(
                supports[i] < supports[i + 1],
                "ColorSupport ordering violated between {:?} and {:?}",
                supports[i],
                supports[i + 1]
            );
        }

        let output = get_test_output();
        flush_test_output(); // Write captured output to stdout
        assert!(!output.is_empty());
        flush_test_output(); // Write captured output to stdout
    }

    #[test]
    #[serial]
    fn test_styler_trait() {
        init_test_output();

        // Test that Role implements Styler
        let role = Role::Code;
        let style_from_role = role.to_style();
        assert_eq!(style_from_role, Style::from(Role::Code));

        // Test that &Role implements Styler
        let role_ref = &Role::Error;
        let style_from_role_ref = role_ref.to_style();
        assert_eq!(style_from_role_ref, Style::from(Role::Error));

        // Test that Style implements Styler
        let original_style = Style::from(Role::Success).bold();
        let style_from_style = original_style.to_style();
        assert_eq!(style_from_style, original_style);

        // Test that &Style implements Styler
        let style_ref = &Style::from(Role::Warning).italic();
        let style_from_style_ref = style_ref.to_style();
        assert_eq!(style_from_style_ref, *style_ref);

        let output = get_test_output();
        flush_test_output(); // Write captured output to stdout
        assert!(!output.is_empty());
    }

    #[test]
    #[serial]
    fn test_styler_direct_methods() {
        init_test_output();

        // Test direct paint method on Role
        let painted_role = Role::Error.paint("Test error");
        assert!(painted_role.contains("Test error"));

        // Test direct paint method on Style
        let style = Style::from(Role::Success).bold();
        let painted_style = style.paint("Test success");
        assert!(painted_style.contains("Test success"));

        // Test chaining methods on Role
        let chained = Role::Warning.bold().italic().paint("Chained warning");
        assert!(chained.contains("Chained warning"));

        // Test individual formatting methods
        let bold_role = Role::Info.bold();
        assert!(bold_role.bold);

        let italic_role = Role::Code.italic();
        assert!(italic_role.italic);

        let dim_role = Role::Normal.dim();
        assert!(dim_role.dim);

        let underline_role = Role::Emphasis.underline();
        assert!(underline_role.underline);

        // Test that chaining works with multiple attributes
        let multi_attr = Role::Debug.bold().italic().dim().underline();
        assert!(multi_attr.bold);
        assert!(multi_attr.italic);
        assert!(multi_attr.dim);
        assert!(multi_attr.underline);

        let _output = get_test_output();
        flush_test_output(); // Write captured output to stdout
                             // Output might be empty for paint tests, but should not crash
    }

    #[test]
    #[serial]
    fn test_styler_trait_methods() {
        init_test_output();

        // Test prtln method with Role
        Role::Code.prtln(format_args!("Testing prtln with Role: {}", "code"));

        // Test prtln with Style
        Style::from(Role::Error)
            .bold()
            .prtln(format_args!("Testing prtln with Style: {}", "error"));

        // Test prtln with Color
        Color::yellow()
            .italic()
            .prtln(format_args!("Testing prtln with Color: {}", "warning"));

        // Test vprtln with Role and different verbosity levels
        Role::Debug.vprtln(
            thag_common::Verbosity::Verbose,
            format_args!("Debug message: {}", "debug"),
        );
        Role::Info.vprtln(
            thag_common::Verbosity::Normal,
            format_args!("Info message: {}", "info"),
        );

        // Test that lower verbosity messages are filtered out
        Role::Link.vprtln(
            thag_common::Verbosity::Debug,
            format_args!("This should be filtered: {}", "link"),
        );

        // Test with parentheses for disambiguation
        (Role::Success).prtln(format_args!("Success with parentheses: {}", "ok"));

        let output = get_test_output();
        flush_test_output(); // Write captured output to stdout
        assert!(!output.is_empty());
        flush_test_output(); // Write captured output to stdout
    }
}
