use crate::errors::ThemeError;
use crate::{lazy_static_var, vprtln, ColorSupport, TermBgLuma, ThagError, ThagResult, V};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::OnceLock;
use strum::{Display, EnumIter};
use thag_proc_macros::{preload_themes, PaletteMethods};

#[cfg(feature = "color_detect")]
use crate::terminal::{self, get_term_bg_rgb, is_light_color};

#[cfg(feature = "config")]
use crate::config::maybe_config;

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

    fn term_bg_rgb(&self) -> Option<(u8, u8, u8)> {
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
use crate::debug_log;

/// Create an inquire `RenderConfig` that respects the current `thag_rs` theme
#[cfg(all(feature = "color_detect", feature = "tools"))]
#[must_use]
pub fn themed_inquire_config() -> inquire::ui::RenderConfig<'static> {
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

    let render_config = RenderConfig::<'_> {
        // Map inquire UI elements to thag_rs Roles - respects Black Metal, etc.
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
    };

    render_config
}

/// Helper functions for inquire UI theming integration
#[cfg(all(feature = "color_detect", feature = "tools"))]
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
        /// Array containing ANSI code and index as strings
        basic: [String; 2],
    }, // [ANSI code, index]
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
    style: Vec<String>, // ["bold", "italic", etc.]
}

/// Contains color information including the color value, ANSI escape sequence, and palette index
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorInfo {
    /// The color value in one of the supported formats (`Basic`, `Color256`, or `TrueColor`)
    pub value: ColorValue,
    /// The ANSI escape sequence string for this color
    pub ansi: &'static str,
    /// The color palette index (0-255 for indexed colors, or closest match for RGB)
    pub index: u8,
}

impl ColorInfo {
    /// Creates a new `ColorInfo` with basic ANSI color format
    ///
    /// # Arguments
    /// * `ansi` - The ANSI escape sequence for this color
    /// * `index` - The color palette index (0-15 for basic colors)
    #[must_use]
    pub fn basic(ansi: &'static str, index: u8) -> Self {
        Self {
            value: ColorValue::Basic {
                basic: [ansi.to_string(), index.to_string()], // This won't work with const fn
            },
            ansi,
            index,
        }
    }

    /// Creates a new `ColorInfo` with 256-color palette format
    ///
    /// # Arguments
    /// * `index` - The color index in the 256-color palette (0-255)
    #[must_use]
    pub fn color256(index: u8) -> Self {
        Self {
            value: ColorValue::Color256 { color256: index },
            ansi: Box::leak(format!("\x1b[38;5;{index}m").into_boxed_str()),
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
            ansi: Box::leak(format!("\x1b[38;2;{r};{g};{b}m").into_boxed_str()),
            index: 0,
        }
    }

    /// Creates appropriate `ColorInfo` based on terminal color support level
    ///
    /// # Arguments
    /// * `rgb` - RGB color values as a tuple (r, g, b)
    /// * `support` - The color support level of the terminal
    #[must_use]
    pub fn with_support(rgb: (u8, u8, u8), support: ColorSupport) -> Self {
        match support {
            ColorSupport::TrueColor => Self::rgb(rgb.0, rgb.1, rgb.2),
            ColorSupport::Color256 => Self::color256(find_closest_color(rgb)),
            _ => Self::color256(find_closest_basic_color(rgb)),
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
    fn from_config(config: &StyleConfig) -> ThagResult<Self> {
        let mut style = match &config.color {
            ColorValue::Basic {
                basic: [_name, index],
            } => {
                // Use the index directly to get the AnsiCode
                let index = index.parse::<u8>()?;
                let code = if index <= 7 {
                    index + 30
                } else {
                    index + 90 - 8
                };
                let ansi = Box::leak(format!("\x1b[{code}m").into_boxed_str());
                Self::fg(ColorInfo::basic(ansi, index))
            }
            ColorValue::Color256 { color256 } => Self::fg(ColorInfo::color256(*color256)),
            ColorValue::TrueColor { rgb } => {
                let rgb_tuple = (rgb[0], rgb[1], rgb[2]);
                let mut color_info = ColorInfo::rgb(rgb[0], rgb[1], rgb[2]);
                color_info.index = find_closest_color(rgb_tuple);
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
    pub fn from_fg_hex(hex: &str) -> ThagResult<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                let mut color_info = ColorInfo::rgb(r, g, b);
                color_info.index = find_closest_color((r, g, b));
                Ok(Self::fg(color_info))
            } else {
                Err(ThagError::Parse)
            }
        } else {
            Err(ThagError::Parse)
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

    /// Returns a new Style with bold formatting enabled
    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Returns a new Style with italic formatting enabled
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

    /// Returns a new Style with dim/faint formatting enabled
    #[must_use]
    pub const fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Returns a new Style with underline formatting enabled
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

        if let Some(color_info) = &self.foreground {
            result.push_str(color_info.ansi);
            needs_reset = true;
        }
        if self.bold {
            result.push_str("\x1b[1m");
            needs_reset = true;
        }
        if self.italic {
            result.push_str("\x1b[3m");
            needs_reset = true;
        }
        if self.dim {
            result.push_str("\x1b[2m");
            needs_reset = true;
        }
        if self.underline {
            result.push_str("\x1b[4m");
            needs_reset = true;
        }

        result.push_str(&val.to_string());

        if needs_reset {
            result.push_str("\x1b[0m");
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
        TermAttributes::get_or_init().theme.style_for(role)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
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
            let ansi = Box::leak(format!("\x1b[{code}m").into_boxed_str());
            Style::fg(ColorInfo::basic(ansi, index))
        } else {
            Style::fg(ColorInfo::color256(index))
        }
    }
}

/// An enum to categorise the current terminal's light or dark theme as detected, configured
/// or defaulted.
/// Type alias for `Role` - provides shorter naming for role constants
pub type Lvl = Role;

impl Lvl {
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
    /// Short alias for `Role::Trace`
    pub const TRCE: Self = Self::Trace;
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
    /// - `Option<(u8, u8, u8)>`: Optional RGB values for the background color
    Configure(ColorSupport, TermBgLuma, Option<(u8, u8, u8)>),
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
    pub fn determine() -> &'static Self {
        lazy_static_var!(ColorInitStrategy, {
            // `color_detect` feature overrides configured colour support.
            #[cfg(feature = "color_detect")]
            let strategy = if std::env::var("TEST_ENV").is_ok() {
                #[cfg(debug_assertions)]
                debug_log!("Avoiding colour detection for testing");
                Self::Default
            } else if cfg!(target_os = "windows") {
                if let Some(config) = maybe_config() {
                    let term_bg_luma = config.styling.term_bg_luma;
                    let term_bg_luma = match term_bg_luma {
                        TermBgLuma::Undetermined => *terminal::get_term_bg_luma(),
                        _ => term_bg_luma,
                    };
                    Self::Configure(
                        config.styling.color_support,
                        term_bg_luma,
                        resolve_config_term_bg_rgb(&config),
                    )
                } else {
                    Self::Default
                }
            } else {
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
        })
    }
}

#[cfg(feature = "color_detect")]
fn resolve_config_term_bg_rgb(config: &crate::Config) -> Option<(u8, u8, u8)> {
    let term_bg_rgb = config.styling.term_bg_rgb;
    match term_bg_rgb {
        None => get_term_bg_rgb().map_or(None, |rgb| Some(*rgb)),
        _ => term_bg_rgb,
    }
}

#[derive(Debug, Display)]
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
#[derive(Debug)]
pub struct TermAttributes {
    /// Indicates how the terminal attributes were initialized (configured, defaulted, or detected)
    pub how_initialized: HowInitialized,
    /// The level of color support available in the terminal
    pub color_support: ColorSupport,
    /// The terminal background color as a hex string (e.g., "#1e1e1e")
    pub term_bg_hex: Option<String>,
    /// The terminal background color as RGB values (red, green, blue)
    pub term_bg_rgb: Option<(u8, u8, u8)>,
    /// The luminance (light/dark) of the terminal background
    pub term_bg_luma: TermBgLuma,
    /// The currently loaded theme containing color palette and styling information
    pub theme: Theme,
}

/// Global instance of `TermAttributes`
static INSTANCE: OnceLock<TermAttributes> = OnceLock::new();
/// Global flag to enable/disable logging
pub static LOGGING_ENABLED: AtomicBool = AtomicBool::new(true);

impl TermAttributes {
    /// Creates a new `TermAttributes` instance with specified support and theme
    #[allow(dead_code)]
    const fn new(
        color_support: ColorSupport,
        term_bg: Option<(u8, u8, u8)>,
        term_bg_luma: TermBgLuma,
        theme: Theme,
    ) -> Self {
        Self {
            how_initialized: HowInitialized::Defaulted,
            color_support,
            term_bg_hex: None,
            term_bg_rgb: term_bg,
            term_bg_luma,
            theme,
        }
    }

    /// Initialize terminal attributes based on the provided strategy.
    ///
    /// This function initializes the terminal attributes singleton with color support
    /// and theme settings according to the specified strategy.
    ///
    /// # Arguments
    /// * `strategy` - The initialization strategy to use (Configure, Default, or Detect)
    ///
    /// # Returns
    /// A reference to the initialized `TermAttributes` instance
    ///
    /// # Panics
    /// Panics if:
    /// * Built-in theme loading fails (which should never happen with correct installation)
    /// * Theme conversion fails during initialization
    #[allow(clippy::too_many_lines)]
    pub fn initialize(strategy: &ColorInitStrategy) -> &'static Self {
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
                    let theme = Theme::get_theme_with_color_support(theme_name, support)
                        .expect("Failed to load builtin theme");
                    Self {
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
                        Theme::get_theme_with_color_support("basic_dark", ColorSupport::Basic)
                            .expect("Failed to load basic dark theme");
                    Self {
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
                        let (color_support, term_bg_rgb_ref) =
                            crate::terminal::detect_term_capabilities();
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
                                Theme::get_builtin("none").expect("Failed to load `none` theme")
                            } else {
                                Theme::auto_detect(color_support, term_bg_luma, Some(&term_bg_rgb))
                                    .expect("Failed to auto-detect theme")
                            };
                            Self {
                                how_initialized: HowInitialized::Configured,
                                color_support,
                                term_bg_hex: Some(rgb_to_hex(&term_bg_rgb)),
                                term_bg_rgb: Some(term_bg_rgb),
                                term_bg_luma,
                                theme,
                            }
                        } else {
                            let theme = Theme::get_theme_with_color_support(
                                "basic_dark",
                                ColorSupport::Basic,
                            )
                            .expect("Failed to load basic dark theme");
                            Self {
                                how_initialized: HowInitialized::Defaulted,
                                color_support: ColorSupport::Basic,
                                term_bg_hex: None,
                                term_bg_rgb: None,
                                term_bg_luma: TermBgLuma::Dark,
                                theme,
                            }
                        }
                    }
                    #[cfg(not(feature = "config"))]
                    {
                        let theme =
                            Theme::get_theme_with_color_support("basic_dark", ColorSupport::Basic)
                                .expect("Failed to load basic dark theme");
                        Self {
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

    /// Gets the `TermAttributes` instance, initializing if necessary.
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
        // eprintln!(
        //     "strategy={strategy:?}. initialized={}",
        //     Self::is_initialized()
        // );
        if !Self::is_initialized() {
            Self::initialize(ColorInitStrategy::determine());
        }
        // Safe to unwrap as we just checked/initialized it
        // vprtln!(V::VV, "INSTANCE.get()={:?}", INSTANCE.get());
        INSTANCE.get().unwrap()
    }

    // /// Returns the appropriate style for the given message role
    // ///
    // /// The style is determined by the current color support level and theme.
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// #![allow(deprecated)]
    // /// use thag_rs::styling::{AnsiCode, Role, TermAttributes};
    // ///
    // /// let attrs = TermAttributes::get_or_init();
    // /// let error_style = attrs.style_for_level(Role::Error);
    // /// println!("{}", error_style.paint("This is an error message"));
    // /// ```
    // #[must_use]
    // #[deprecated = "Use `Style::for_role`"]
    // #[allow(unused_variables)]
    // #[profiled]
    // pub fn style_for_role(&self, role: Role) -> Style {
    //     Style::for_role(role)
    // }

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
    pub fn with_theme(mut self, theme_name: &str, support: ColorSupport) -> ThagResult<Self> {
        self.theme = Theme::get_theme_with_color_support(theme_name, support)?;
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
/// use thag_rs::styling::{paint_for_role, Role};
///
/// let styled_error = paint_for_role(Role::Error, "This is an error message");
/// println!("{}", styled_error); // Prints in error styling
/// ```
pub fn paint_for_role(role: Role, string: &str) -> String {
    Style::for_role(role).paint(string)
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
#[derive(Debug, Clone, Copy, EnumIter, Display, PartialEq, Eq)]
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
    /// Detailed execution tracking
    Trace,
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
    trace: StyleConfig,
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
    /// Style for detailed execution tracking
    pub trace: Style,
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
            Role::Trace => self.trace.clone(),
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
/// use thag_rs::styling::{Role, Theme, ColorSupport};
///
/// // Load a built-in theme
/// let theme = Theme::get_builtin("dracula")?;
///
/// // Get styling for a specific role
/// let error_style = theme.style_for(Role::Error);
/// println!("{}", error_style.paint("This is an error message"));
/// # Ok::<(), thag_rs::ThagError>(())
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
    pub bg_rgbs: Vec<(u8, u8, u8)>,
    /// A human-readable description of the theme's characteristics and origin
    pub description: String,
}

impl Theme {
    fn from_toml(theme_name: &str, theme_toml: &str) -> Result<Self, ThagError> {
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
        maybe_term_bg: Option<&(u8, u8, u8)>,
    ) -> ThagResult<Self> {
        // NB: don't call `TermAttributes::get_or_init()` here because it will cause a tight loop
        // since we're called from the TermAttributes::initialize.
        vprtln!(V::VV, "maybe_term_bg={maybe_term_bg:?}");
        let Some(term_bg_rgb) = maybe_term_bg else {
            return fallback_theme(term_bg_luma);
        };

        // let signatures = get_theme_signatures();
        // vprtln!(V::VV, "signatures={signatures:?}");
        let hex = rgb_to_bare_hex(term_bg_rgb);
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
                    let color_distance = color_distance(*term_bg_rgb, *rgb);
                    if color_distance < min_distance {
                        min_distance = color_distance;
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

            // vprtln!(
            //     V::V,
            //     "2. If TrueColor, look for exact RGB match of a preferred Color256 theme."
            // );
            // if color_support == ColorSupport::TrueColor {
            //     // Look for matching 256-color theme
            //     let next_best_matches = eligible_themes
            //         .iter()
            //         .filter(|(_, idx)| {
            //             idx.matches_background(*term_bg_rgb)
            //                 && idx.min_color_support == ColorSupport::Color256
            //         })
            //         .map(|(name, _)| (*name).to_string())
            //         .collect::<Vec<String>>();
            //     vprtln!(V::VV, "next_best_matches={next_best_matches:#?}");

            //     for preferred_name in preferred_styling {
            //         vprtln!(V::VV, "preferred_name={preferred_name}");
            //         if next_best_matches.contains(preferred_name) {
            //             vprtln!(
            //                 V::V,
            //                 "Found an exact match at reduced color in {preferred_name}"
            //             );
            //             return Self::1910(preferred_name);
            //         }
            //     }
            // }

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
                    let color_distance = color_distance(*term_bg_rgb, *rgb);
                    if color_distance < min_distance {
                        min_distance = color_distance;
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
    /// Returns `ThagError` if:
    /// - The file cannot be read
    /// - The file contains invalid TOML syntax
    /// - The theme definition is incomplete or invalid
    /// - Color values don't match the declared minimum color support
    /// - Style attributes are invalid
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use thag_rs::ThagError;
    /// use thag_rs::styling::Theme;
    /// let theme = Theme::load_from_file(Path::new("themes/built_in/basic_light.toml"))?;
    /// # Ok::<(), ThagError>(())
    /// ```
    pub fn load_from_file(path: &Path) -> ThagResult<Self> {
        let content = fs::read_to_string(path)?;
        let mut def: ThemeDefinition = toml::from_str(&content)?;
        def.filename = path.to_path_buf();
        def.is_builtin = false;
        Self::from_definition(def)
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
    /// Returns `ThagError` if:
    /// - The specified theme name is not recognized
    /// - The theme definition contains invalid color values
    /// - The theme definition contains invalid style attributes
    /// - There's a mismatch between color values and minimum color support
    ///
    /// # Examples
    /// ```
    /// use thag_rs::ThagError;
    /// use thag_rs::styling::Theme;
    /// let theme = Theme::get_builtin("dracula")?;
    /// # Ok::<(), ThagError>(())
    /// ```
    pub fn get_builtin(theme_name: &str) -> ThagResult<Self> {
        let maybe_theme_index = THEME_INDEX.get(theme_name);
        let Some(theme_index) = maybe_theme_index else {
            return Err(ThagError::FromStr(
                format!("No theme found for name {theme_name}").into(),
            ));
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
    /// Returns `ThagError` if the specified theme name is not recognized
    ///
    fn get_theme_with_color_support(
        theme_name: &str,
        color_support: ColorSupport,
    ) -> ThagResult<Self> {
        let mut theme = Self::get_builtin(theme_name)?;
        if color_support != ColorSupport::TrueColor {
            vprtln!(V::VV, "Converting to {color_support:?}");
            theme.convert_to_color_support(color_support);
        }
        // eprintln!("Theme={:#?}", theme);
        Ok(theme)
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
    /// Returns `ThagError` if:
    /// - Color support string cannot be parsed
    /// - Background luminance string cannot be parsed
    /// - Palette configuration is invalid
    fn from_definition(def: ThemeDefinition) -> ThagResult<Self> {
        // vprtln!(V::VV, "def.min_color_support={:?}", def.min_color_support);
        let color_support = ColorSupport::from_str(&def.min_color_support);
        // vprtln!(V::VV, "color_support={color_support:?})");

        // Convert hex strings to RGB tuples
        let bg_rgbs: Vec<(u8, u8, u8)> = def
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
    /// Returns `ThagError` if:
    /// - The terminal's color support is insufficient for the theme
    ///   (e.g., trying to use a 256-color theme in a basic terminal)
    /// - The terminal's background luminance doesn't match the theme's requirements
    ///   (e.g., trying to use a light theme on a dark background)
    /// - Any style in the theme's palette requires unavailable terminal features
    ///
    /// # Examples
    /// ```
    /// use thag_rs::{TermBgLuma, ThagError};
    /// use thag_rs::styling::{ColorSupport, Theme};
    /// let theme = Theme::get_builtin("dracula")?;
    /// theme.validate(&ColorSupport::TrueColor, &TermBgLuma::Dark)?;
    /// # Ok::<(), ThagError>(())
    /// ```
    pub fn validate(
        &self,
        available_support: &ColorSupport,
        term_bg_luma: &TermBgLuma,
    ) -> ThagResult<()> {
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

    fn validate_palette(&self) -> ThagResult<()> {
        self.palette.validate_styles(self.min_color_support)?;
        Ok(())
    }

    /// Validates a theme definition before creating a Theme
    #[allow(dead_code)]
    fn validate_definition(def: &ThemeDefinition) -> ThagResult<()> {
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
            .chain(&def.palette.trace.style)
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
    /// Returns `ThagError` if:
    /// - The file cannot be read or contains invalid TOML syntax
    /// - The theme definition is incomplete or invalid
    /// - The terminal's color support is insufficient for the theme
    /// - The terminal's background luminance doesn't match the theme's requirements
    /// - Any style in the theme's palette requires unavailable terminal features
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use thag_rs::{TermBgLuma, ThagError};
    /// use thag_rs::styling::{ColorSupport, Theme};
    /// let theme = Theme::load(
    ///     Path::new("themes/built_in/basic_light.toml"),
    ///     ColorSupport::Basic,
    ///     TermBgLuma::Light
    /// )?;
    /// # Ok::<(), ThagError>(())
    /// ```
    pub fn load(
        path: &Path,
        available_support: ColorSupport,
        term_bg_luma: TermBgLuma,
    ) -> ThagResult<Self> {
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
            Role::Trace => palette.trace.clone(),
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
            self.bg_rgbs[0].0, self.bg_rgbs[0].1, self.bg_rgbs[0].2,
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
        // 0: Black   (0,0,0)
        // 1: Red     (170,0,0)
        // 2: Green   (0,170,0)
        // 3: Yellow  (170,85,0)
        // 4: Blue    (0,0,170)
        // 5: Magenta (170,0,170)
        // 6: Cyan    (0,170,170)
        // 7: White   (170,170,170)
        // 8-15: Bright versions

        let colors = [
            (0, 0, 0),       // Black
            (170, 0, 0),     // Red
            (0, 170, 0),     // Green
            (170, 85, 0),    // Yellow
            (0, 0, 170),     // Blue
            (170, 0, 170),   // Magenta
            (0, 170, 170),   // Cyan
            (170, 170, 170), // White
            // Bright versions
            (85, 85, 85),    // Bright Black
            (255, 85, 85),   // Bright Red
            (85, 255, 85),   // Bright Green
            (255, 255, 85),  // Bright Yellow
            (85, 85, 255),   // Bright Blue
            (255, 85, 255),  // Bright Magenta
            (85, 255, 255),  // Bright Cyan
            (255, 255, 255), // Bright White
        ];

        // Find closest ANSI color using color distance
        let mut closest = 0;
        let mut min_distance = f32::MAX;

        for (i, &(cr, cg, cb)) in colors.iter().enumerate() {
            let distance = color_distance((r, g, b), (cr, cg, cb));
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
                let index = find_closest_color((rgb[0], rgb[1], rgb[2]));
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
                        let ansi = Box::leak(format!("\x1b[{code}m").into_boxed_str());
                        // Corrected style conversion
                        style.foreground = Some(ColorInfo::basic(ansi, index));
                    }
                    ColorValue::Color256 { color256 } => {
                        let rgb = index_to_rgb(*color256);
                        let index = Self::convert_rgb_to_ansi(rgb.0, rgb.1, rgb.2);
                        // Use the index directly to get the AnsiCode
                        let code = if index <= 7 {
                            index + 30
                        } else {
                            index + 90 - 8
                        };
                        let ansi = Box::leak(format!("\x1b[{code}m").into_boxed_str());
                        // Corrected style conversion
                        style.foreground = Some(ColorInfo::basic(ansi, index));
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
            let ansi = Box::leak(format!("\x1b[{code}m").into_boxed_str());
            style.foreground = Some(ColorInfo::basic(ansi, index));
            style.reset();
        }
        self.min_color_support = ColorSupport::None;
    }
}

fn index_to_rgb(index: u8) -> (u8, u8, u8) {
    if index < 16 {
        // Standard ANSI colors
        return match index {
            0 => (0, 0, 0),
            1 => (128, 0, 0),
            2 => (0, 128, 0),
            3 => (128, 128, 0),
            4 => (0, 0, 128),
            5 => (128, 0, 128),
            6 => (0, 128, 128),
            7 => (192, 192, 192),
            8 => (128, 128, 128),
            9 => (255, 0, 0),
            10 => (0, 255, 0),
            11 => (255, 255, 0),
            12 => (0, 0, 255),
            13 => (255, 0, 255),
            14 => (0, 255, 255),
            15 => (255, 255, 255),
            _ => unreachable!(),
        };
    }

    if index >= 232 {
        // Grayscale
        let gray = (index - 232) * 10 + 8;
        return (gray, gray, gray);
    }

    // Color cube
    let index = index - 16;
    let r = (index / 36) * 51;
    let g = ((index % 36) / 6) * 51;
    let b = (index % 6) * 51;
    (r, g, b)
}

fn fallback_theme(term_bg_luma: TermBgLuma) -> ThagResult<Theme> {
    let name = if term_bg_luma == TermBgLuma::Light {
        "basic_light"
    } else {
        "basic_dark"
    };

    Theme::from_toml(
        name,
        THEME_INDEX
            .get(name)
            .expect("Basic theme not found")
            .content,
    )
}

#[allow(clippy::missing_const_for_fn)]
#[cfg(feature = "config")]
fn get_preferred_styling(term_bg_luma: TermBgLuma, config: &crate::Config) -> &Vec<String> {
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
fn get_fallback_styling(term_bg_luma: TermBgLuma, config: &crate::Config) -> &Vec<String> {
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

// fn get_exact_matches(
//     eligible_themes: &Vec<&str>,
//     term_bg_rgb: (u8, u8, u8),
//     color_support: ColorSupport,
// ) -> Vec<String> {
//     eligible_themes
//         .iter()
//         .map(|theme_name| (theme_name, THEME_INDEX.get(theme_name)))
//         .filter_map(|(theme_name, maybe_idx)| {
//             if let Some(idx) = maybe_idx {
//                 if idx.matches_background(term_bg_rgb) && idx.min_color_support == color_support {
//                     Some((theme_name, idx))
//                 } else {
//                     None
//                 }
//             } else {
//                 None
//             }
//         })
//         .map(|(name, _)| (*name).to_string())
//         .collect::<Vec<String>>()
// }

// #[cfg(feature = "config")]
// fn get_reduced_palette_matches(
//     eligible_themes: &Vec<(&str, &ThemeIndex)>,
//     term_bg_rgb: (u8, u8, u8),
// ) -> Vec<String> {
//     eligible_themes
//         .iter()
//         .filter(|(_, idx)| idx.matches_background(term_bg_rgb))
//         .map(|(name, _)| (*name).to_string())
//         .collect()
// }

// Helper to calculate color distance
#[allow(clippy::items_after_statements)]
// #[cfg(feature = "color_detect")]
#[must_use]
fn color_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f32 {
    let dr = (f32::from(c1.0) - f32::from(c2.0)).powi(2);
    let dg = (f32::from(c1.1) - f32::from(c2.1)).powi(2);
    let db = (f32::from(c1.2) - f32::from(c2.2)).powi(2);
    (dr + dg + db).sqrt()
}

/// Convert a hex color string to RGB.
///
/// # Errors
///
/// This function will return an error if the input string is not a valid hex color.
// #[cfg(feature = "color_detect")]
fn hex_to_rgb(hex: &str) -> ThagResult<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            Ok((r, g, b))
        } else {
            Err(ThagError::Parse)
        }
    } else {
        Err(ThagError::Parse)
    }
}

// Helper to check a single style
fn validate_style(style: &Style, min_support: ColorSupport) -> ThagResult<()> {
    style.foreground.as_ref().map_or_else(
        || Ok(()),
        |color_info| match &color_info.value {
            ColorValue::Basic { basic: _ } => Ok(()), // Basic is always valid
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

// // #[cfg(feature = "color_detect")]
// fn matches_background(bg: (u8, u8, u8)) -> ThagResult<bool> {
//     if let Some(config) = maybe_config() {
//         let mut found = false;
//         for hex in &config.styling.backgrounds {
//             // vprtln!(V::VV, "name=");
//             let theme_bg = hex_to_rgb(hex)?;
//             // if color_distance(bg, theme_bg) < THRESHOLD {
//             if bg == theme_bg {
//                 found = true;
//                 break;
//             }
//         }
//         Ok(found)
//     } else {
//         Ok(false)
//     }
// }

// #[must_use]
// pub fn rgb_to_hex((r, g, b): &(u8, u8, u8)) -> String {
//     format!("#{r:02x}{g:02x}{b:02x}")
// }

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
/// use thag_rs::cvprtln;
/// use thag_rs::logging::Verbosity;
/// use thag_rs::styling::Role;
/// let details = "todos los detalles";
/// cvprtln!(Role::Info, Verbosity::VV, "Detailed info: {}", details);
/// ```
#[macro_export]
macro_rules! cvprtln {
    ($role:expr, $verbosity:expr, $($arg:tt)*) => {{
        if $verbosity <= $crate::shared::get_verbosity() {
            let style = $crate::styling::Style::for_role($role);
            let content = format!($($arg)*);
            let verbosity = $crate::shared::get_verbosity();
            $crate::vprtln!(verbosity, "{}", style.paint(content));
        }
    }};
}

fn base_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> u32 {
    let dr = f64::from(i32::from(c1.0) - i32::from(c2.0)) * 0.3;
    let dg = f64::from(i32::from(c1.1) - i32::from(c2.1)) * 0.59;
    let db = f64::from(i32::from(c1.2) - i32::from(c2.2)) * 0.11;
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
/// use thag_rs::styling::find_closest_color;
///
/// // Pure red should map to a red in the color cube
/// let red_index = find_closest_color((255, 0, 0));
/// assert!(red_index >= 16); // Not a basic ANSI color
///
/// // Gray should map to the grayscale range
/// let gray_index = find_closest_color((128, 128, 128));
/// assert!(gray_index >= 232 && gray_index <= 255);
/// ```
#[must_use]
pub fn find_closest_color(rgb: (u8, u8, u8)) -> u8 {
    const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];

    // Handle grays first (232-255)
    let (r, g, b) = rgb;
    if r == g && g == b {
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

    let r_idx = find_closest(r);
    let g_idx = find_closest(g);
    let b_idx = find_closest(b);

    // eprintln!(
    //     "Closest color to {rgb:?} is ({}, {}, {}) = {}",
    //     STEPS[r_idx as usize],
    //     STEPS[g_idx as usize],
    //     STEPS[b_idx as usize],
    //     16 + (36 * r_idx) + (6 * g_idx) + b_idx
    // );
    16 + (36 * r_idx) + (6 * g_idx) + b_idx
}

fn find_closest_basic_color(rgb: (u8, u8, u8)) -> u8 {
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
/// use thag_rs::styling::get_rgb;
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
pub const fn get_rgb(color: u8) -> (u8, u8, u8) {
    const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    match color {
        0..=15 => BASIC_COLORS[color as usize],
        16..=231 => {
            let color = color - 16;
            let r = STEPS[((color / 36) % 6) as usize];
            let g = STEPS[((color / 6) % 6) as usize];
            let b = STEPS[(color % 6) as usize];
            (r, g, b)
        }
        232..=255 => {
            let gray = 8 + (color - 232) * 10;
            (gray, gray, gray)
        }
    }
}

// Usage:
#[allow(dead_code)]
fn main() -> ThagResult<()> {
    // Load built-in theme
    let _dracula = Theme::get_builtin("dracula")?;

    // Load custom theme
    let _custom = Theme::load_from_file(Path::new("themes/examples/custom_theme.toml"))?;

    Ok(())
}

const BASIC_COLORS: [(u8, u8, u8); 16] = [
    (0, 0, 0),       // black
    (128, 0, 0),     // red
    (0, 128, 0),     // green
    (128, 128, 0),   // yellow
    (0, 0, 128),     // blue
    (128, 0, 128),   // magenta
    (0, 128, 128),   // cyan
    (192, 192, 192), // light gray
    (128, 128, 128), // dark gray
    (255, 0, 0),     // bright red
    (0, 255, 0),     // bright green
    (255, 255, 0),   // bright yellow
    (0, 0, 255),     // bright blue
    (255, 0, 255),   // bright magenta
    (0, 255, 255),   // bright cyan
    (255, 255, 255), // white
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
/// use thag_rs::styling::{Theme, display_theme_roles};
///
/// let theme = Theme::get_builtin("dracula")?;
/// display_theme_roles(&theme);
/// # Ok::<(), thag_rs::ThagError>(())
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
        ("Trace", "Detailed execution tracking"),
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
        paint_for_role(Role::Normal, "Role styles:"),
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
            "Trace" => Role::Trace,
            _ => Role::Normal,
        };

        // Get style for this role
        let style = theme.style_for(role);

        // Print role name in its style, followed by description
        let styled_name = style.paint(role_name);
        let padding = " ".repeat(col1_width.saturating_sub(role_name.len()));

        print!("\t{styled_name}{padding}");
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
/// use thag_rs::styling::{display_theme_details, Theme};
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
        (
            "Type",
            if theme.is_builtin {
                "Built-in"
            } else {
                "Custom"
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

    println!("\n\t{}", paint_for_role(Role::Normal, "Theme attributes:"));
    println!("\t{}", "─".repeat(flower_box_len));

    for (attr, description) in theme_docs {
        let styled_name = paint_for_role(Role::Info, attr);
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
/// use thag_rs::styling::display_terminal_attributes;
/// display_terminal_attributes();
/// ```
pub fn display_terminal_attributes() {
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
        paint_for_role(Role::Normal, "Terminal attributes:")
    );
    println!("\t{}", "─".repeat(flower_box_len));

    for (attr, description) in terminal_docs {
        let styled_name = paint_for_role(Role::Info, attr);
        let padding = " ".repeat(col1_width.saturating_sub(attr.len()));

        print!("\t{styled_name}{padding}");
        println!("│ {description}");
    }

    println!("\t{}\n", "─".repeat(flower_box_len));
}

fn dual_format_rgb((r, g, b): (u8, u8, u8)) -> String {
    format!("#{r:02x}{g:02x}{b:02x} = rgb({r}, {g}, {b})")
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
    static BLACK_BG: &(u8, u8, u8) = &(0, 0, 0);

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
            let theme = Theme::get_theme_with_color_support(theme_name, color_support)
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
    fn test_styling_load_dracula_theme() -> ThagResult<()> {
        init_test_output();
        let theme = Theme::load_from_file(Path::new("themes/built_in/dracula.toml"))?;

        // Check theme metadata
        assert_eq!(theme.term_bg_luma, TermBgLuma::Dark);
        assert_eq!(theme.min_color_support, ColorSupport::TrueColor);
        assert_eq!(theme.bg_rgbs, vec![(40, 42, 54)]);

        // Check a few key styles
        if let ColorValue::TrueColor { rgb } = &theme
            .palette
            .heading1
            .foreground
            .expect("Failed to load heading1 foreground color")
            .value
        {
            assert_eq!(rgb, &[255, 85, 85]);
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
    fn test_styling_dracula_validation() -> ThagResult<()> {
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
}
