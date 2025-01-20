#![allow(clippy::cast_lossless)]
use crate::errors::ThemeError;
use crate::{profile, profile_method, profile_section, ThagError, ThagResult};
use documented::{Documented, DocumentedVariants};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::OnceLock;
use strum::{Display, EnumIter, EnumString, IntoStaticStr};
use thag_proc_macros::{generate_theme_types, AnsiCodeDerive, PaletteMethods};

#[cfg(feature = "color_detect")]
use crate::terminal;

#[cfg(debug_assertions)]
use crate::debug_log;

// Include the generated theme data
include!(concat!(env!("OUT_DIR"), "/theme_data.rs"));

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, AnsiCodeDerive)]
#[serde(rename_all = "snake_case")]
pub enum AnsiCode {
    // Standard colors (30-37)
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,

    // High intensity colors (90-97)
    #[ansi_name = "Dark Gray"]
    BrightBlack = 90,
    BrightRed = 91,
    BrightGreen = 92,
    BrightYellow = 93,
    BrightBlue = 94,
    BrightMagenta = 95,
    BrightCyan = 96,
    BrightWhite = 97,
}

impl AnsiCode {
    // Get the numeric code
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum ColorValue {
    Basic { basic: [String; 2] }, // [ANSI code, index]
    Color256 { color256: u8 },    // 256-color index
    TrueColor { rgb: [u8; 3] },   // RGB values
}

#[derive(Clone, Debug, Deserialize)]
struct StyleConfig {
    #[serde(flatten)]
    color: ColorValue,
    #[serde(default)]
    style: Vec<String>, // ["bold", "italic", etc.]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ColorInfo {
    pub value: ColorValue,
    pub ansi: &'static str,
    pub index: u8,
}

impl ColorInfo {
    #[must_use]
    pub fn new(ansi: &'static str, index: u8) -> Self {
        Self {
            value: ColorValue::Basic {
                basic: [ansi.to_string(), index.to_string()], // This won't work with const fn
            },
            ansi,
            index,
        }
    }

    #[must_use]
    pub fn indexed(index: u8) -> Self {
        Self {
            value: ColorValue::Color256 { color256: index },
            ansi: Box::leak(format!("\x1b[38;5;{index}m").into_boxed_str()),
            index,
        }
    }

    #[must_use]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            value: ColorValue::TrueColor { rgb: [r, g, b] },
            ansi: Box::leak(format!("\x1b[38;2;{r};{g};{b}m").into_boxed_str()),
            index: 0,
        }
    }

    // Helper to create appropriate ColorInfo based on terminal support
    #[must_use]
    pub fn with_support(rgb: (u8, u8, u8), support: ColorSupport) -> Self {
        profile_method!("ColorInfo::with_support");
        match support {
            ColorSupport::TrueColor => Self::rgb(rgb.0, rgb.1, rgb.2),
            ColorSupport::Color256 => Self::indexed(find_closest_color(rgb)),
            _ => Self::indexed(find_closest_basic_color(rgb)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct Style {
    pub foreground: Option<ColorInfo>,
    pub bold: bool,
    pub italic: bool,
    pub dim: bool,
    pub underline: bool,
}

#[derive(Serialize)]
pub struct ThemeOutput {
    name: String,
    description: String,
    term_bg_luma: String,
    min_color_support: String,
    background: Option<String>,
    palette: PaletteOutput,
}

#[derive(Serialize)]
pub struct PaletteOutput {
    heading1: StyleOutput,
    heading2: StyleOutput,
    heading3: StyleOutput,
    error: StyleOutput,
    warning: StyleOutput,
    success: StyleOutput,
    info: StyleOutput,
    emphasis: StyleOutput,
    code: StyleOutput,
    normal: StyleOutput,
    subtle: StyleOutput,
    hint: StyleOutput,
    debug: StyleOutput,
    trace: StyleOutput,
}

#[derive(Serialize)]
pub struct StyleOutput {
    rgb: [u8; 3],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    style: Vec<String>,
}

impl Style {
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

    fn from_config(config: &StyleConfig) -> ThagResult<Self> {
        profile_method!("Style::from_config");
        let mut style = match &config.color {
            ColorValue::Basic {
                basic: [_name, index],
            } => {
                // Use the index directly to get the AnsiCode
                let code = index.parse::<u8>()?;
                let code = if code <= 7 { code + 30 } else { code + 90 };
                let ansi = Box::leak(format!("\x1b[{code}m").into_boxed_str());
                Self::fg(ColorInfo::new(ansi, code))
            }
            ColorValue::Color256 { color256 } => Self::fg(ColorInfo::indexed(*color256)),
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

    #[must_use]
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

    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    #[must_use]
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    #[must_use]
    pub const fn normal(self) -> Self {
        self
    }

    #[must_use]
    pub const fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    #[must_use]
    pub const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn paint<D>(&self, val: D) -> String
    where
        D: std::fmt::Display,
    {
        profile_method!("Style::paint");
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

    #[must_use]
    pub fn with_color_index(index: u8) -> Self {
        Self {
            foreground: Some(ColorInfo::indexed(index)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn for_role(role: Role) -> Self {
        profile_method!("Style::for_role");
        TermAttributes::get().theme.style_for(role)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

pub fn style_to_output(style: &Style) -> StyleOutput {
    let mut style_attrs = Vec::new();
    if style.bold {
        style_attrs.push("bold".to_string());
    }
    if style.italic {
        style_attrs.push("italic".to_string());
    }
    if style.dim {
        style_attrs.push("dim".to_string());
    }
    if style.underline {
        style_attrs.push("underline".to_string());
    }

    let rgb = if let Some(color_info) = &style.foreground {
        match &color_info.value {
            ColorValue::TrueColor { rgb } => *rgb,
            _ => [0, 0, 0], // shouldn't happen for true color themes
        }
    } else {
        [0, 0, 0]
    };

    StyleOutput {
        rgb,
        style: style_attrs,
    }
}

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

    #[must_use]
    pub fn black() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::BLACK, 0)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn red() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::RED, 1)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn green() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::GREEN, 2)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn yellow() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::YELLOW, 3)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn blue() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::BLUE, 4)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn magenta() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::MAGENTA, 5)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn cyan() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::CYAN, 6)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn white() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::WHITE, 7)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn dark_gray() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::DARK_GRAY, 8)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn light_yellow() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::LIGHT_YELLOW, 11)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn light_cyan() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::LIGHT_CYAN, 14)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn light_gray() -> Style {
        Style {
            foreground: Some(ColorInfo::new(Self::LIGHT_GRAY, 15)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn fixed(code: u8) -> Style {
        Style {
            // foreground: Some(Box::leak(format!("\x1b[38;5;{code}m").into_boxed_str())),
            foreground: Some(ColorInfo::indexed(code)),
            ..Default::default()
        }
    }
}

/// An enum to categorise the current terminal's level of colour support as detected, configured
/// or defaulted.
///
/// We fold `TrueColor` into Xterm256 as we're not interested in more than 256
/// colours just for messages.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    Documented,
    DocumentedVariants,
    EnumIter,
    EnumString,
    IntoStaticStr,
    PartialEq,
    PartialOrd,
    Eq,
    Serialize,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ColorSupport {
    /// Still to be determined or defaulted
    Undetermined = 0,
    /// No color support
    None = 1,
    /// Basic 16-color support
    #[default]
    #[serde(alias = "ansi16")] // Accept old "ansi16" value
    Basic = 2,
    /// Full color support, suitable for color palettes of 256 colours (8 bit) or higher.
    #[serde(alias = "xterm256")] // Accept old "256" value
    Color256 = 3,
    /// Full color support, 24 bits -> 16 million colors.
    TrueColor = 4,
}

/// An enum to categorise the current terminal's light or dark theme as detected, configured
/// or defaulted.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Documented,
    DocumentedVariants,
    Display,
    EnumIter,
    EnumString,
    IntoStaticStr,
    PartialEq,
    Eq,
    Serialize,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TermBgLuma {
    /// Light background terminal
    Light,
    /// Dark background terminal
    Dark,
    /// Let `thag` autodetect the background luminosity
    #[cfg(feature = "color_detect")]
    Undetermined,
}

impl Default for TermBgLuma {
    fn default() -> Self {
        #[cfg(feature = "color_detect")]
        {
            Self::Undetermined
        }

        #[cfg(not(feature = "color_detect"))]
        {
            Self::Dark // Safe default when detection isn't available
        }
    }
}

// impl FromStr for TermBgLuma {
//     type Err = ThemeError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "light" => Ok(Self::Light),
//             "dark" => Ok(Self::Dark),
//             "undetermined" => Ok(Self::Undetermined),
//             _ => Err(ThemeError::InvalidTermBgLuma(s.to_string())),
//         }
//     }
// }

/// Represents different message/content levels for styling
#[derive(Debug, Clone, Copy, EnumIter, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Level {
    Error,
    Warning,
    Heading, // HEAD in the original
    Subheading,
    Emphasis,
    Bright,
    Normal,
    Debug,
    Ghost,
}

pub type Lvl = Level;

impl Lvl {
    pub const ERR: Self = Self::Error;
    pub const WARN: Self = Self::Warning;
    pub const EMPH: Self = Self::Emphasis;
    pub const HEAD: Self = Self::Heading;
    pub const SUBH: Self = Self::Subheading;
    pub const BRI: Self = Self::Bright;
    pub const NORM: Self = Self::Normal;
    pub const DBUG: Self = Self::Debug;
    pub const GHST: Self = Self::Ghost;
}

impl Level {
    #[must_use]
    pub fn color_index(&self) -> u8 {
        profile_method!("Level::color_index");
        let term_attrs = TermAttributes::get();
        let style = term_attrs.style_for_level(*self);
        style.foreground.map_or(7, |color_info| color_info.index) // 7 = white as fallback
    }
}

// We can implement conversions to u8 directly here
impl From<&Level> for u8 {
    fn from(level: &Level) -> Self {
        level.color_index()
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ColorInitStrategy {
    Configure(ColorSupport, TermBgLuma),
    Default,
    #[cfg(feature = "color_detect")]
    Detect,
}

impl ColorInitStrategy {
    #[must_use]
    pub fn determine() -> Self {
        profile_method!("ColorInitStrategy::determine");
        {
            // `color_detect` feature overrides configured colour support.
            #[cfg(feature = "color_detect")]
            let strategy = if std::env::var("TEST_ENV").is_ok() {
                #[cfg(debug_assertions)]
                debug_log!("Avoiding colour detection for testing");
                Self::Default
            } else {
                Self::Detect
            };

            #[cfg(all(not(feature = "color_detect"), feature = "config"))]
            let strategy = if std::env::var("TEST_ENV").is_ok() {
                #[cfg(debug_assertions)]
                debug_log!("Avoiding colour detection for testing");
                Self::Default
            } else if let Some(config) = maybe_config() {
                Self::Configure(config.colors.color_support, config.colors.term_theme)
            } else {
                Self::Default
            };

            #[cfg(all(not(feature = "color_detect"), not(feature = "config")))]
            let strategy = Self::Default;

            strategy
        }
    }
}

/// Manages terminal color attributes and styling based on terminal capabilities and theme
#[derive(Debug)]
pub struct TermAttributes {
    pub color_support: ColorSupport,
    pub term_bg_rgb: Option<&'static (u8, u8, u8)>,
    pub term_bg_luma: TermBgLuma,
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
        term_bg: Option<&'static (u8, u8, u8)>,
        term_bg_luma: TermBgLuma,
        theme: Theme,
    ) -> Self {
        Self {
            color_support,
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
    pub fn initialize(strategy: &ColorInitStrategy) -> &'static Self {
        profile_method!("TermAttributes::initialize");
        let get_or_init = INSTANCE.get_or_init(|| -> Self {
            profile_section!("TermArrtibutes::get_or_init");
            match *strategy {
                ColorInitStrategy::Configure(support, bg_luma) => {
                    let theme_name = match bg_luma {
                        TermBgLuma::Light => "basic_light",
                        TermBgLuma::Dark => "basic_dark",
                        #[cfg(feature = "color_detect")]
                        TermBgLuma::Undetermined => "basic_dark", // Safe fallback
                    };
                    let theme =
                        Theme::load_builtin(theme_name).expect("Failed to load builtin theme");
                    Self {
                        color_support: support,
                        theme,
                        term_bg_rgb: None::<&'static (u8, u8, u8)>,
                        term_bg_luma: match bg_luma {
                            TermBgLuma::Light => TermBgLuma::Light,
                            TermBgLuma::Dark => TermBgLuma::Dark,
                            #[cfg(feature = "color_detect")]
                            TermBgLuma::Undetermined => TermBgLuma::Dark,
                        },
                    }
                }
                ColorInitStrategy::Default => {
                    let theme =
                        Theme::load_builtin("basic_dark").expect("Failed to load basic dark theme");
                    Self {
                        color_support: ColorSupport::Basic,
                        theme,
                        term_bg_rgb: None::<&'static (u8, u8, u8)>,
                        term_bg_luma: TermBgLuma::Dark,
                    }
                }
                #[cfg(feature = "color_detect")]
                ColorInitStrategy::Detect => {
                    let support = *crate::terminal::detect_color_support();
                    let term_bg_rgb = terminal::get_term_bg().ok();
                    let term_bg_luma = terminal::get_term_bg_luma();
                    // eprintln!("support={support:?}, term_bg={term_bg_rgb:?}");
                    // TODO: dethagomize error message
                    let theme = Theme::auto_detect(support, *term_bg_luma, term_bg_rgb)
                        .expect("Failed to auto-detect theme");
                    Self {
                        color_support: support,
                        theme: theme.clone(),
                        term_bg_rgb,
                        term_bg_luma: *term_bg_luma,
                    }
                }
            }
        });
        // eprintln!("Returning {get_or_init:#?}");
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
        let strategy = ColorInitStrategy::determine();
        // eprintln!(
        //     "strategy={strategy:?}. initialized={}",
        //     Self::is_initialized()
        // );
        if !Self::is_initialized() {
            Self::initialize(&strategy);
        }
        // Safe to unwrap as we just checked/initialized it
        // eprintln!("INSTANCE.get()={:?}", INSTANCE.get());
        INSTANCE.get().unwrap()
    }

    /// Gets the global `TermAttributes` instance, panicking if it hasn't been initialized
    ///
    /// # Panics
    ///
    /// This function will panic if `initialize` hasn't been called first
    pub fn get() -> &'static Self {
        INSTANCE
            .get()
            .expect("TermAttributes not initialized. Call get_or_init()")
    }

    // #[must_use]
    // pub fn get_theme(&self) -> TermTheme {
    //     if self.theme != TermTheme::Undetermined {
    //         return self.theme.clone();
    //     }

    //     #[cfg(feature = "color_detect")]
    //     {
    //         terminal::detect_theme().clone()
    //     }

    //     #[cfg(not(feature = "color_detect"))]
    //     TermTheme::Dark
    // }

    /// Returns the appropriate style for the given message level
    ///
    /// The style is determined by the current color support level and theme.
    ///
    /// # Examples
    ///
    /// ```
    /// use thag_rs::styling::{TermAttributes, Level};
    ///
    /// let attrs = TermAttributes::get_or_default();
    /// let error_style = attrs.style_for_level(Level::Error);
    /// println!("{}", error_style.paint("This is an error message"));
    /// ```
    #[must_use]
    #[allow(unused_variables)]
    pub fn style_for_level(&self, level: Level) -> Style {
        profile_method!("TermAttrs::style_for_level");

        // Convert Level to Role
        let role = Role::from(level);

        // Validate theme against terminal capabilities
        match self
            .theme
            .validate(&self.color_support, &self.theme.term_bg_luma)
        {
            Ok(()) => {
                let style = self.theme.style_for(role);
                if style == Style::default() {
                    #[cfg(debug_assertions)]
                    debug_log!("No style defined for role {:?}", role);
                }
                style
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                debug_log!("Theme validation failed: {:?}", e);
                Style::default()
            }
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
    pub fn with_theme(mut self, theme_name: &str) -> ThagResult<Self> {
        self.theme = Theme::load_builtin(theme_name)?;
        Ok(self)
    }

    // If you need to override color support
    #[must_use]
    pub const fn with_color_support(mut self, support: ColorSupport) -> Self {
        self.color_support = support;
        self
    }
}

/// Returns the style for basic (16-color) light theme
#[must_use]
pub fn basic_light_style(level: Level) -> Style {
    match level {
        Level::Error => Color::red().bold(),
        Level::Warning => Color::magenta().bold(),
        Level::Heading => Color::blue().bold(),
        Level::Subheading => Color::cyan().bold(),
        Level::Emphasis => Color::green().bold(),
        Level::Bright => Color::green(),
        Level::Normal => Color::dark_gray(),
        Level::Debug => Color::cyan(),
        Level::Ghost => Color::cyan().italic(),
    }
}

/// Returns the style for basic (16-color) dark theme
#[must_use]
pub fn basic_dark_style(level: Level) -> Style {
    match level {
        Level::Error => Color::red().bold(),
        Level::Warning => Color::yellow().bold(),
        Level::Heading => Color::green().bold(),
        Level::Subheading => Color::blue().bold(),
        Level::Emphasis => Color::cyan().bold(),
        Level::Bright => Color::light_yellow(),
        Level::Normal => Color::white(),
        Level::Debug => Color::light_cyan(),
        Level::Ghost => Color::light_gray().italic(),
    }
}

/// Returns the style for full (256-color) light theme
#[must_use]
pub fn full_light_style(level: Level) -> Style {
    match level {
        Level::Error => Color::fixed(160).bold(),   // GuardsmanRed
        Level::Warning => Color::fixed(164).bold(), // DarkPurplePizzazz
        Level::Heading => Color::fixed(19).bold(),  // MidnightBlue
        Level::Subheading => Color::fixed(26).bold(), // ScienceBlue
        Level::Emphasis => Color::fixed(167).bold(), // RomanOrange
        Level::Bright => Color::fixed(42).bold(),   // CaribbeanGreen
        Level::Normal => Color::fixed(16),          // Black
        Level::Debug => Color::fixed(32),           // LochmaraBlue
        Level::Ghost => Color::fixed(232).italic(), // DarkCodGray
    }
}

/// Returns the style for full (256-color) dark theme
#[must_use]
pub fn full_dark_style(level: Level) -> Style {
    match level {
        Level::Error => Color::fixed(1).bold(),      // UserRed
        Level::Warning => Color::fixed(171).bold(),  // LighterHeliotrope
        Level::Heading => Color::fixed(33).bold(),   // AzureRadiance
        Level::Subheading => Color::fixed(44),       // RobinEggBlue
        Level::Emphasis => Color::fixed(173).bold(), // Copperfield
        Level::Bright => Color::fixed(118).italic(), // ChartreuseGreen
        Level::Normal => Color::fixed(231),          // White
        Level::Debug => Color::fixed(37),            // BondiBlue
        Level::Ghost => Color::fixed(251).italic(),  // Silver
    }
}

#[must_use]
pub fn style_string(lvl: Level, string: &str) -> String {
    TermAttributes::get().style_for_level(lvl).paint(string)
}

#[must_use]
pub fn style_for_role(role: Role, string: &str) -> String {
    Style::for_role(role).paint(string)
}

// New structures for Themes

/// Defines the role (purpose and relative prominence) of a piece of text
#[derive(Debug, Clone, Copy)]
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

impl From<Level> for Role {
    fn from(level: Level) -> Self {
        profile_method!("Role::from");
        match level {
            Level::Error => Self::Error,
            Level::Warning => Self::Warning,
            Level::Heading => Self::Heading1,
            Level::Subheading => Self::Heading2,
            Level::Emphasis => Self::Emphasis,
            Level::Bright => Self::Info,   // Highlighting important info
            Level::Normal => Self::Normal, // Default display style
            Level::Debug => Self::Debug,
            Level::Ghost => Self::Hint,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, PaletteMethods, Serialize)]
pub struct Palette {
    pub heading1: Style,
    pub heading2: Style,
    pub heading3: Style,
    pub error: Style,
    pub warning: Style,
    pub success: Style,
    pub info: Style,
    pub emphasis: Style,
    pub code: Style,
    pub normal: Style,
    pub subtle: Style,
    pub hint: Style,
    pub debug: Style,
    pub trace: Style,
}

// ThemeDefinition & ThemeSignature and their impls
generate_theme_types! {}

#[derive(Clone, Debug, Serialize)]
#[allow(dead_code)]
pub struct Theme {
    pub name: String,      // e.g., "Dracula"
    pub filename: PathBuf, // e.g., "themes/built_in/dracula.toml"
    pub is_builtin: bool,  // true for built-in themes, false for custom
    pub term_bg_luma: TermBgLuma,
    pub min_color_support: ColorSupport,
    pub palette: Palette,
    pub background: Option<String>,
    pub description: String,
}

impl Theme {
    /// Detects and loads the most appropriate theme for the current terminal
    ///
    /// # Errors
    ///
    /// This function will bubble up any `termbg` error encountered.
    #[cfg(feature = "color_detect")]
    pub fn auto_detect(
        color_support: ColorSupport,
        term_bg_luma: TermBgLuma,
        maybe_term_bg: Option<&(u8, u8, u8)>,
    ) -> ThagResult<Self> {
        // Helper to calculate color distance
        #[allow(clippy::items_after_statements)]
        fn color_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f32 {
            profile!("color_distance");
            let dr = (c1.0 as f32 - c2.0 as f32).powi(2);
            let dg = (c1.1 as f32 - c2.1 as f32).powi(2);
            let db = (c1.2 as f32 - c2.2 as f32).powi(2);
            (dr + dg + db).sqrt()
        }

        profile_method!("Theme::auto_detect");
        // Causes a tight loop because we're called from the TermAttributes::initialize
        // eprintln!("About to call TermAttributes::get_or_init()");
        // let term_attrs = TermAttributes::get_or_init();
        // let term_attrs = TermAttributes::get();
        // let term_bg_rgb = term_attrs
        //     .term_bg_rgb
        //     .ok_or(ThemeError::BackgroundDetectionFailed)?;
        // let color_support = term_attrs.color_support;
        // let term_bg_luma = term_attrs.term_bg_luma;

        if let Some(term_bg_rgb) = maybe_term_bg {
            let signatures = get_theme_signatures();
            // eprintln!("signatures={signatures:?}");
            // Filter themes by luma first
            let matching_luma_themes: Vec<_> = signatures
                .iter()
                .filter(|(_, sig)| sig.term_bg_luma == term_bg_luma)
                .collect();
            // eprintln!("matching_luma_themes={matching_luma_themes:?}");

            // Try exact RGB match within luma-matching themes
            for (theme_name, sig) in &matching_luma_themes {
                if *term_bg_rgb == sig.bg_rgb && color_support >= sig.min_color_support {
                    // eprintln!("Found an exact match!");
                    return Self::load_builtin(theme_name);
                }
            }

            // Try closest match with progressive color support reduction
            for required_support in [
                ColorSupport::TrueColor,
                ColorSupport::Color256,
                ColorSupport::Basic,
            ] {
                if color_support >= required_support {
                    let mut best_match = None;
                    let mut min_distance = f32::MAX;

                    for (theme_name, sig) in &matching_luma_themes {
                        if sig.min_color_support == required_support {
                            let distance = color_distance(*term_bg_rgb, sig.bg_rgb);
                            if distance < min_distance {
                                min_distance = distance;
                                best_match = Some(theme_name);
                            }
                        }
                    }

                    if let Some(theme_name) = best_match {
                        return Self::load_builtin(theme_name);
                    }
                }
            }

            // Fall back to basic theme
            Ok(Self::load_builtin(if term_bg_luma == TermBgLuma::Light {
                "basic_light"
            } else {
                "basic_dark"
            })?)
        } else {
            // Fall back to basic theme
            Ok(Self::load_builtin(if term_bg_luma == TermBgLuma::Light {
                "basic_light"
            } else {
                "basic_dark"
            })?)
        }
    }

    /// Loads a theme from a TOML file.
    ///
    /// The TOML file should define a complete theme, including:
    /// - Color support requirements
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
        profile_method!("Theme::load_from_file");
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
    /// - "`light_256`" - Rich light theme for 256-color terminals
    /// - "`dark_256`" - Rich dark theme for 256-color terminals
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
    /// let theme = Theme::load_builtin("dracula")?;
    /// # Ok::<(), ThagError>(())
    /// ```
    pub fn load_builtin(name: &str) -> ThagResult<Self> {
        profile_method!("Theme::load_builtin");
        let theme_toml = BUILT_IN_THEMES
            .get(name)
            .ok_or_else(|| ThemeError::UnknownTheme(name.to_string()))?;

        let mut def: ThemeDefinition = toml::from_str(theme_toml)?;
        def.filename = PathBuf::from(format!("themes/built_in/{name}.toml"));
        def.is_builtin = true;
        // eprintln!("About to call Theme::from_definition({def:?})");
        Self::from_definition(def)
    }

    fn from_definition(def: ThemeDefinition) -> ThagResult<Self> {
        profile_method!("Theme::from_definition");
        // eprintln!("def.min_color_support={:?}", def.min_color_support);
        let color_support = ColorSupport::from_str(&def.min_color_support);
        // eprintln!("color_support={color_support:?})");
        Ok(Self {
            name: def.name,
            filename: def.filename,
            is_builtin: def.is_builtin,
            term_bg_luma: TermBgLuma::from_str(&def.term_bg_luma)?,
            min_color_support: color_support?,
            palette: Palette::from_config(&def.palette)?,
            background: def.background,
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
    /// use thag_rs::ThagError;
    /// use thag_rs::styling::{ColorSupport, TermBgLuma, Theme};
    /// let theme = Theme::load_builtin("dracula")?;
    /// theme.validate(ColorSupport::TrueColor, TermBgLuma::Dark)?;
    /// # Ok::<(), ThagError>(())
    /// ```
    pub fn validate(
        &self,
        available_support: &ColorSupport,
        term_bg_luma: &TermBgLuma,
    ) -> ThagResult<()> {
        profile_method!("Theme::validate");
        // Check color support
        // eprintln!("self.min_color_support={:?}", self.min_color_support);
        // eprintln!("available_support={available_support:?}");
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
        profile_method!("Theme::validate_palette");
        self.palette.validate_styles(self.min_color_support)?;
        Ok(())
    }

    /// Validates a theme definition before creating a Theme
    #[allow(dead_code)]
    fn validate_definition(def: &ThemeDefinition) -> ThagResult<()> {
        profile_method!("Theme::validate_definition");
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
        // ... chain all style arrays
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
    /// use thag_rs::ThagError;
    /// use thag_rs::styling::{ColorSupport, TermBgLuma, Theme};
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
        profile_method!("Theme::load");
        let theme = Self::load_from_file(path)?;
        theme.validate(&available_support, &term_bg_luma)?;
        Ok(theme)
    }

    #[must_use]
    pub fn style_for(&self, role: Role) -> Style {
        profile_method!("Theme::style_for");
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
            "Theme: {}\nType: {}\nFile: {}\nDescription: {}\nBackground: {}\nMinimum Color Support: {:?}\nBackground Luminance: {:?}",
            self.name,
            if self.is_builtin { "Built-in" } else { "Custom" },
            self.filename.display(),
            self.description,
            self.background.as_deref().unwrap_or("None"),
            self.min_color_support,
            self.term_bg_luma,
        )
    }

    /// Returns a list of all available built-in themes
    #[must_use]
    pub fn list_builtin() -> Vec<String> {
        BUILT_IN_THEMES.keys().map(ToString::to_string).collect()
    }

    pub fn to_256_color(&self) -> Result<Theme, Box<dyn std::error::Error>> {
        // Helper to convert a Style from true color to 256-color
        fn convert_style(style: &Style) -> Result<Style, Box<dyn std::error::Error>> {
            if let Some(color_info) = &style.foreground {
                match &color_info.value {
                    ColorValue::TrueColor { rgb } => {
                        let index = find_closest_color((rgb[0], rgb[1], rgb[2]));
                        let mut new_style = Style::fg(ColorInfo::indexed(index));
                        // Preserve other style attributes
                        new_style.bold = style.bold;
                        new_style.italic = style.italic;
                        new_style.dim = style.dim;
                        new_style.underline = style.underline;
                        Ok(new_style)
                    }
                    _ => Ok(style.clone()), // Already 256 or basic color
                }
            } else {
                Ok(style.clone())
            }
        }

        // Create new theme with converted palette
        Ok(Theme {
            name: format!("{} 256", self.name),
            description: format!("{} (256 colors)", self.description),
            term_bg_luma: self.term_bg_luma,
            min_color_support: ColorSupport::Color256,
            palette: Palette {
                heading1: convert_style(&self.palette.heading1)?,
                heading2: convert_style(&self.palette.heading2)?,
                heading3: convert_style(&self.palette.heading3)?,
                error: convert_style(&self.palette.error)?,
                warning: convert_style(&self.palette.warning)?,
                success: convert_style(&self.palette.success)?,
                info: convert_style(&self.palette.info)?,
                emphasis: convert_style(&self.palette.emphasis)?,
                code: convert_style(&self.palette.code)?,
                normal: convert_style(&self.palette.normal)?,
                subtle: convert_style(&self.palette.subtle)?,
                hint: convert_style(&self.palette.hint)?,
                debug: convert_style(&self.palette.debug)?,
                trace: convert_style(&self.palette.trace)?,
            },
            background: self.background.clone(),
            is_builtin: self.is_builtin,
            filename: self.filename.clone(),
        })
    }

    pub fn to_output(&self) -> ThemeOutput {
        ThemeOutput {
            name: self.name.clone(),
            description: self.description.clone(),
            term_bg_luma: self.term_bg_luma.to_string().to_lowercase(),
            min_color_support: "true_color".to_string(),
            background: self.background.clone(),
            palette: PaletteOutput {
                heading1: style_to_output(&self.palette.heading1),
                heading2: style_to_output(&self.palette.heading2),
                heading3: style_to_output(&self.palette.heading3),
                error: style_to_output(&self.palette.error),
                warning: style_to_output(&self.palette.warning),
                success: style_to_output(&self.palette.success),
                info: style_to_output(&self.palette.info),
                emphasis: style_to_output(&self.palette.emphasis),
                code: style_to_output(&self.palette.code),
                normal: style_to_output(&self.palette.normal),
                subtle: style_to_output(&self.palette.subtle),
                hint: style_to_output(&self.palette.hint),
                debug: style_to_output(&self.palette.debug),
                trace: style_to_output(&self.palette.trace),
            },
        }
    }
}

// Helper to check a single style
fn validate_style(style: &Style, min_support: ColorSupport) -> ThagResult<()> {
    profile_method!("Theme::style_for");
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

// Convenience macros
/// A line print macro that conditionally prints a message using `cprtln` if the current global verbosity
/// is at least as verbose as the `Verbosity` (alias `V`) level passed in.
///
/// The message will be styled and coloured according to the `MessageLevel` (alias `Lvl`) passed in.
///
/// Format: `cvprtln!(&Level: Lvl, verbosity: V, "Lorem ipsum dolor {} amet", content: &str);`
#[macro_export]
macro_rules! cvprtln {
    ($level:expr, $verbosity:expr, $($arg:tt)*) => {{
        if $verbosity <= $crate::logging::get_verbosity() {
            let term_attrs = $crate::styling::TermAttributes::get_or_init();
            let style = term_attrs.style_for_level($level);
            let content = format!($($arg)*);
            let verbosity = $crate::logging::get_verbosity();
            $crate::vlog!(verbosity, "{}", style.paint(content));
        }
    }};
}

// #[allow(dead_code)]
// fn color_distance(c1: (u8, u8, u8), c2: (u8, u8, u8), is_system: bool) -> u32 {
//     let base_distance = base_distance(c1, c2);

//     // Give a slight preference to system colors when they're close matches
//     #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
//     if is_system {
//         (f64::from(base_distance) * 0.9) as u32
//     } else {
//         base_distance
//     }
// }

fn base_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> u32 {
    profile!("base_distance");
    let dr = f64::from(i32::from(c1.0) - i32::from(c2.0)) * 0.3;
    let dg = f64::from(i32::from(c1.1) - i32::from(c2.1)) * 0.59;
    let db = f64::from(i32::from(c1.2) - i32::from(c2.2)) * 0.11;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let base_distance = db.mul_add(db, dr.mul_add(dr, dg * dg)) as u32;
    base_distance
}

pub fn find_closest_color(rgb: (u8, u8, u8)) -> u8 {
    const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];

    profile!("find_closest_color");

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
        profile!("find_closest");
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

    16 + (36 * r_idx) + (6 * g_idx) + b_idx
}

fn find_closest_basic_color(rgb: (u8, u8, u8)) -> u8 {
    profile!("find_closest_basic_color");
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

// Helper function to get RGB values for a color number (for verification)
#[must_use]
pub fn get_rgb(color: u8) -> (u8, u8, u8) {
    const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];

    profile!("get_rgb");
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
    let _dracula = Theme::load_builtin("dracula")?;

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

#[macro_export]
macro_rules! clog {
    ($level:expr, $($arg:tt)*) => {{
        if $crate::styling::LOGGING_ENABLED.load(std::sync::atomic::Ordering::SeqCst) {
            let attrs = $crate::styling::TermAttributes::get_or_init();
            let style = attrs.style_for_level($level);
            println!("{}", style.paint(format!($($arg)*)));
        }
    }};
}

#[macro_export]
macro_rules! clog_error {
    ($($arg:tt)*) => { $crate::clog!($crate::Level::Error, $($arg)*) };
}

#[macro_export]
macro_rules! clog_warning {
        ($($arg:tt)*) => { $crate::clog!($crate::Level::Warning, $($arg)*) };
    }

#[macro_export]
macro_rules! clog_heading {
    ($($arg:tt)*) => { $crate::clog!($crate::Level::Heading, $($arg)*) };
}

#[macro_export]
macro_rules! clog_subheading {
    ($($arg:tt)*) => { $crate::clog!($crate::Level::Subheading, $($arg)*) };
}

#[macro_export]
macro_rules! clog_emphasis {
    ($($arg:tt)*) => { $crate::clog!($crate::Level::Emphasis, $($arg)*) };
}

#[macro_export]
macro_rules! clog_bright {
    ($($arg:tt)*) => { $crate::clog!($crate::Level::Bright, $($arg)*) };
}

#[macro_export]
macro_rules! clog_normal {
    ($($arg:tt)*) => { $crate::clog!($crate::Level::Normal, $($arg)*) };
}

#[macro_export]
macro_rules! clog_debug {
    ($($arg:tt)*) => { $crate::clog!($crate::Level::Debug, $($arg)*) };
}

#[macro_export]
macro_rules! clog_ghost {
    ($($arg:tt)*) => { $crate::clog!($crate::Level::Ghost, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog {
    ($verbosity:expr, $level:expr, $($arg:tt)*) => {{
        if $crate::styling::LOGGING_ENABLED.load(std::sync::atomic::Ordering::SeqCst) {
            let logger = $crate::logging::LOGGER.lock().unwrap();
            let message = format!($($arg)*);

            #[cfg(feature = "color_support")]
            {
                let color_logger = $crate::styling::TermAttributes::get();
                let style = color_logger.style_for_level($level);
                logger.log($verbosity, &style.paint(message));
            }

            #[cfg(not(feature = "color_support"))]
            {
                if verbosity as u8 <= self.verbosity as u8 {
                    println!("{}", message);
                }

                logger.log($verbosity, &message);
            }
        }
    }};
}

#[macro_export]
macro_rules! cvlog_error {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::Level::Error, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_warning {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::Level::Warning, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_heading {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::Level::Heading, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_subheading {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::Level::Subheading, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_emphasis {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::Level::Emphasis, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_bright {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::Level::Bright, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_normal {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::Level::Normal, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_debug {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::Level::Debug, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_ghost {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::Level::Ghost, $verbosity, $($arg)*) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};

    static MOCK_THEME_DETECTION: AtomicBool = AtomicBool::new(false);
    static BLACK_BG: &'static (u8, u8, u8) = (0, 0, 0);

    impl TermAttributes {
        fn with_mock_theme(color_support: ColorSupport, term_bg_luma: TermBgLuma) -> Self {
            MOCK_THEME_DETECTION.store(true, Ordering::SeqCst);
            let theme_name = match term_bg_luma {
                TermBgLuma::Light => "basic_light",
                TermBgLuma::Dark | TermBgLuma::Undetermined => "basic_dark",
            };
            let theme = Theme::load_builtin(theme_name).expect("Failed to load builtin theme");
            Self::new(color_support, BLACK_BG, term_bg_luma, theme)
        }
    }

    use std::io::Write;

    thread_local! {
        static TEST_OUTPUT: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
    }

    fn init_test() {
        TEST_OUTPUT.with(|output| {
            output.borrow_mut().push(String::new());
        });
    }

    // At end of each test or in test teardown
    fn flush_test_output() {
        TEST_OUTPUT.with(|output| {
            let mut stdout = std::io::stdout();
            for line in output.borrow().iter() {
                writeln!(stdout, "{}", line).unwrap();
            }
            output.borrow_mut().clear();
        });
    }

    // // Tests that need access to internal implementation
    // #[test]
    // fn test_styling_default_theme_with_mock() {
    //     init_test();
    //     let term_attrs = TermAttributes::with_mock_theme(ColorSupport::Color256, TermTheme::Dark);
    //     let defaulted = term_attrs.get_theme();
    //     assert_eq!(defaulted, TermTheme::Dark);
    //     println!();
    //     flush_test_output();
    // }

    #[test]
    fn test_styling_no_color_support() {
        let term_attrs = TermAttributes::with_mock_theme(ColorSupport::None, TermBgLuma::Dark);
        let style = term_attrs.style_for_level(Level::Error);
        assert_eq!(style.paint("test"), "test"); // Should have no ANSI codes
    }

    #[test]
    fn test_styling_color_support_levels() {
        let none = TermAttributes::with_mock_theme(ColorSupport::None, TermBgLuma::Dark);
        let basic = TermAttributes::with_mock_theme(ColorSupport::Basic, TermBgLuma::Dark);
        let full = TermAttributes::with_mock_theme(ColorSupport::Color256, TermBgLuma::Dark);

        let test_level = Level::Error;

        // No color support should return plain text
        assert_eq!(none.style_for_level(test_level).paint("test"), "test");

        // Basic support should use ANSI 16 colors
        assert!(basic
            .style_for_level(test_level)
            .paint("test")
            .contains("\x1b[31m"));

        // Full support should use 256 colors
        assert!(full
            .style_for_level(test_level)
            .paint("test")
            .contains("\x1b[38;5;1m"));
    }

    #[test]
    fn test_styling_theme_variations() {
        let attrs_light =
            TermAttributes::with_mock_theme(ColorSupport::Color256, TermBgLuma::Light);
        let attrs_dark = TermAttributes::with_mock_theme(ColorSupport::Color256, TermBgLuma::Dark);

        let heading_light = attrs_light.style_for_level(Level::Heading).paint("test");
        let heading_dark = attrs_dark.style_for_level(Level::Heading).paint("test");

        // Light and dark themes should produce different colors
        assert_ne!(heading_light, heading_dark);
    }

    #[test]
    fn test_styling_level_styling() {
        let attrs = TermAttributes::with_mock_theme(ColorSupport::Color256, TermBgLuma::Dark);

        // Test each level has distinct styling
        let styles: Vec<String> = vec![
            Level::Error,
            Level::Warning,
            Level::Heading,
            Level::Subheading,
            Level::Emphasis,
            Level::Bright,
            Level::Normal,
            Level::Debug,
            Level::Ghost,
        ]
        .iter()
        .map(|level| attrs.style_for_level(*level).paint("test"))
        .collect();

        // Check that all styles are unique
        for (i, style1) in styles.iter().enumerate() {
            for (j, style2) in styles.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        style1, style2,
                        "Styles for different levels should be distinct"
                    );
                }
            }
        }
    }

    #[test]
    fn test_styling_style_attributes() {
        let attrs = TermAttributes::with_mock_theme(ColorSupport::Color256, TermBgLuma::Dark);

        // Error should be bold
        let error_style = attrs.style_for_level(Level::Error).paint("test");
        assert!(error_style.contains("\x1b[1m"));

        // Ghost should be italic
        let ghost_style = attrs.style_for_level(Level::Ghost).paint("test");
        assert!(ghost_style.contains("\x1b[3m"));
    }

    #[test]
    fn test_styling_load_dracula_theme() -> ThagResult<()> {
        let theme = Theme::load_from_file(Path::new("themes/built_in/dracula.toml"))?;

        // Check theme metadata
        assert_eq!(theme.term_bg_luma, TermBgLuma::Dark);
        assert_eq!(theme.min_color_support, ColorSupport::TrueColor);
        assert_eq!(theme.background.as_deref(), Some("#282a36"));

        // Check a few key styles
        if let ColorValue::TrueColor { rgb } = &theme.palette.heading1.foreground.unwrap().value {
            assert_eq!(rgb, &[255, 121, 198]);
        } else {
            panic!("Expected TrueColor for heading1");
        }

        // Check style attributes
        assert!(theme.palette.heading1.bold);
        assert!(!theme.palette.normal.bold);
        assert!(theme.palette.hint.italic);

        Ok(())
    }

    #[test]
    fn test_styling_dracula_validation() -> ThagResult<()> {
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

        Ok(())
    }

    #[test]
    fn test_styling_color_support_ordering() {
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
    }
}
