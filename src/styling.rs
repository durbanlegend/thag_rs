use crate::errors::ThemeError;
use crate::styling::Role::{Heading1, Info, Normal};
use crate::{lazy_static_var, vlog, ThagError, ThagResult, V};
use documented::{Documented, DocumentedVariants};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::OnceLock;
use strum::{Display, EnumIter, EnumString, IntoStaticStr};
use thag_proc_macros::{preload_themes, AnsiCodeDerive, PaletteMethods};
use thag_profiler::{enable_profiling, end, profile, profiled};

#[cfg(feature = "color_detect")]
use crate::terminal::{self, get_term_bg_rgb, is_light_color};

#[cfg(feature = "config")]
use crate::config::maybe_config;

#[allow(unused_imports)]
#[cfg(debug_assertions)]
use crate::debug_log;

// Include the generated theme data
// include!(concat!(env!("OUT_DIR"), "/theme_data.rs"));

// #[cfg(feature = "color_detect")]
#[cfg(feature = "config")]
const THRESHOLD: f32 = 30.0; // Adjust this value as needed

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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorInfo {
    pub value: ColorValue,
    pub ansi: &'static str,
    pub index: u8,
}

impl ColorInfo {
    #[must_use]
    #[profiled]
    pub fn basic(ansi: &'static str, index: u8) -> Self {
        Self {
            value: ColorValue::Basic {
                basic: [ansi.to_string(), index.to_string()], // This won't work with const fn
            },
            ansi,
            index,
        }
    }

    #[must_use]
    #[profiled]
    pub fn color256(index: u8) -> Self {
        Self {
            value: ColorValue::Color256 { color256: index },
            ansi: Box::leak(format!("\x1b[38;5;{index}m").into_boxed_str()),
            index,
        }
    }

    #[must_use]
    #[profiled]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            value: ColorValue::TrueColor { rgb: [r, g, b] },
            ansi: Box::leak(format!("\x1b[38;2;{r};{g};{b}m").into_boxed_str()),
            index: 0,
        }
    }

    // Helper to create appropriate ColorInfo based on terminal support
    #[must_use]
    #[profiled]
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
    pub foreground: Option<ColorInfo>,
    pub bold: bool,
    pub italic: bool,
    pub dim: bool,
    pub underline: bool,
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

    // Used by proc macro palette_methods.
    #[profiled]
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
    #[profiled]
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
    #[profiled]
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

    #[profiled]
    pub fn reset(&mut self) {
        self.bold = false;
        self.italic = false;
        self.dim = false;
        self.underline = false;
    }

    #[profiled]
    pub fn paint<D>(&self, val: D) -> String
    where
        D: std::fmt::Display,
    {
        // vlog!(V::V, "self.foreground={:#?}", self.foreground);
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

    #[must_use]
    #[profiled]
    pub fn with_color_index(index: u8) -> Self {
        Self {
            foreground: Some(ColorInfo::color256(index)),
            ..Default::default()
        }
    }

    #[must_use]
    /// Get the `Style` for a `Role` from the currently loaded theme.
    #[profiled]
    pub fn for_role(role: Role) -> Self {
        TermAttributes::get_or_init().theme.style_for(role)
    }
}

impl Default for Style {
    #[profiled]
    fn default() -> Self {
        Self::new()
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
    #[profiled]
    pub fn black() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::BLACK, 0)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
    pub fn red() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::RED, 1)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
    pub fn green() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::GREEN, 2)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
    pub fn yellow() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::YELLOW, 3)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
    pub fn blue() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::BLUE, 4)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
    pub fn magenta() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::MAGENTA, 5)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
    pub fn cyan() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::CYAN, 6)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
    pub fn white() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::WHITE, 7)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
    pub fn dark_gray() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::DARK_GRAY, 8)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
    pub fn light_yellow() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::LIGHT_YELLOW, 11)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
    pub fn light_cyan() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::LIGHT_CYAN, 14)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
    pub fn light_gray() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::LIGHT_GRAY, 15)),
            ..Default::default()
        }
    }

    #[must_use]
    #[profiled]
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

/// An enum to categorise the current terminal's level of colour support as detected, configured
/// or defaulted.
///
/// We fold `TrueColor` into Xterm256 as we're not interested in more than 256
/// colours just for messages.
#[derive(
    Clone,
    Copy,
    Debug,
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
    #[serde(alias = "ansi16")] // Accept old "ansi16" value
    Basic = 2,
    /// Full color support, suitable for color palettes of 256 colours (8 bit) or higher.
    #[serde(alias = "xterm256")] // Accept old "256" value
    Color256 = 3,
    /// Full color support, 24 bits -> 16 million colors.
    TrueColor = 4,
}

impl Default for ColorSupport {
    #[profiled]
    fn default() -> Self {
        #[cfg(feature = "color_detect")]
        {
            Self::Undetermined
        }

        #[cfg(not(feature = "color_detect"))]
        {
            Self::Basic // Safe default when detection isn't available
        }
    }
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
    Undetermined,
}

impl Default for TermBgLuma {
    #[profiled]
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

// For backward compatibility
pub type Level = Role;

impl Level {
    pub const HEAD: Self = Self::Heading1;
    pub const SUBH: Self = Self::Heading2;
    // pub const ERR: Self = Self::Error;
    // pub const WARN: Self = Self::Warning;
    pub const BRI: Self = Self::Info;
    // pub const EMPH: Self = Self::Emphasis;
    // pub const NORM: Self = Self::Normal;
    pub const GHOS: Self = Self::Hint;
    // pub const DBUG: Self = Self::Debug;
}

pub type Lvl = Role;

impl Lvl {
    pub const HD1: Self = Self::Heading1;
    pub const HD2: Self = Self::Heading2;
    pub const HD3: Self = Self::Heading3;
    pub const ERR: Self = Self::Error;
    pub const WARN: Self = Self::Warning;
    pub const SUCC: Self = Self::Success;
    pub const INFO: Self = Self::Info;
    pub const EMPH: Self = Self::Emphasis;
    pub const CODE: Self = Self::Code;
    pub const NORM: Self = Self::Normal;
    pub const SUBT: Self = Self::Subtle;
    pub const HINT: Self = Self::Hint;
    pub const DBUG: Self = Self::Debug;
    pub const TRCE: Self = Self::Trace;
}

impl Role {
    #[must_use]
    #[profiled]
    pub fn color_index(&self) -> u8 {
        let style = Style::for_role(*self);
        style.foreground.map_or(7, |color_info| color_info.index) // 7 = white as fallback
    }
}

// We can implement conversions to u8 directly here
impl From<&Role> for u8 {
    #[profiled]
    fn from(role: &Role) -> Self {
        role.color_index()
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ColorInitStrategy {
    Configure(ColorSupport, TermBgLuma, Option<(u8, u8, u8)>),
    Default,
    // #[cfg(feature = "color_detect")]
    Match,
}

impl ColorInitStrategy {
    #[must_use]
    #[profiled]
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
#[profiled]
fn resolve_config_term_bg_rgb(config: &crate::Config) -> Option<(u8, u8, u8)> {
    let term_bg_rgb = config.styling.term_bg_rgb;
    match term_bg_rgb {
        None => get_term_bg_rgb().map_or(None, |rgb| Some(*rgb)),
        _ => term_bg_rgb,
    }
}

#[derive(Debug, Display)]
pub enum HowInitialized {
    Configured,
    Defaulted,
    Detected,
}

/// Manages terminal color attributes and styling based on terminal capabilities and theme
#[derive(Debug)]
pub struct TermAttributes {
    pub how_initialized: HowInitialized,
    pub color_support: ColorSupport,
    pub term_bg_hex: Option<String>,
    pub term_bg_rgb: Option<(u8, u8, u8)>,
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
    #[profiled]
    pub fn initialize(strategy: &ColorInitStrategy) -> &'static Self {
        let get_or_init = INSTANCE.get_or_init(|| -> Self {
            profile!("instance_get_or_init", time);
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
    #[profiled]
    pub fn is_initialized() -> bool {
        INSTANCE.get().is_some()
    }

    /// Attempts to get the `TermAttributes` instance, returning None if not initialized
    #[profiled]
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
    #[profiled]
    pub fn get_or_init() -> &'static Self {
        // eprintln!(
        //     "strategy={strategy:?}. initialized={}",
        //     Self::is_initialized()
        // );
        if !Self::is_initialized() {
            Self::initialize(ColorInitStrategy::determine());
        }
        // Safe to unwrap as we just checked/initialized it
        // vlog!(V::V, "INSTANCE.get()={:?}", INSTANCE.get());
        INSTANCE.get().unwrap()
    }

    // /// Gets the global `TermAttributes` instance, panicking if it hasn't been initialized
    // ///
    // /// # Panics
    // ///
    // /// This function will panic if `initialize` hasn't been called first
    // pub fn get() -> &'static Self {
    //     INSTANCE
    //         .get()
    //         .expect("TermAttributes not initialized. Call get_or_init()")
    // }

    // #[must_use]
    // pub fn get_theme(&self) -> TermBgLuma {
    //     if self.theme != TermBgLuma::Undetermined {
    //         return self.theme.clone();
    //     }

    //     #[cfg(feature = "color_detect")]
    //     {
    //         terminal::detect_theme().clone()
    //     }

    //     #[cfg(not(feature = "color_detect"))]
    //     TermBgLuma::Dark
    // }

    /// Returns the appropriate style for the given message level
    ///
    /// The style is determined by the current color support level and theme.
    ///
    /// # Examples
    ///
    /// ```
    /// #![allow(deprecated)]
    /// use thag_rs::styling::{AnsiCode, TermAttributes, Level};
    ///
    /// let attrs = TermAttributes::get_or_init();
    /// let error_style = attrs.style_for_level(Level::Error);
    /// println!("{}", error_style.paint("This is an error message"));
    /// ```
    #[must_use]
    #[deprecated = "Use `Style::for_role`"]
    #[allow(unused_variables)]
    #[profiled]
    pub fn style_for_level(&self, level: Level) -> Style {
        Style::for_role(level)

        // // Convert Level to Role
        // let role = Role::from(level);

        // // Validate theme against terminal capabilities
        // match self
        //     .theme
        //     .validate(&self.color_support, &self.theme.term_bg_luma)
        // {
        //     Ok(()) => {
        //         let style = if self.color_support == ColorSupport::None {
        //             Style::default()
        //         } else {
        //             self.theme.style_for(role)
        //         };
        //         if style == Style::default() {
        //             #[cfg(debug_assertions)]
        //             debug_log!("No style defined for role {:?}", role);
        //         }
        //         style
        //     }
        //     Err(e) => {
        //         #[cfg(debug_assertions)]
        //         debug_log!("Theme validation failed: {:?}", e);
        //         Style::default()
        //     }
        // }
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
    #[profiled]
    pub fn with_theme(mut self, theme_name: &str, support: ColorSupport) -> ThagResult<Self> {
        self.theme = Theme::get_theme_with_color_support(theme_name, support)?;
        Ok(self)
    }

    // If you need to override color support
    #[must_use]
    pub const fn with_color_support(mut self, support: ColorSupport) -> Self {
        self.color_support = support;
        self
    }
}

// /// Returns the style for basic (16-color) light theme
// #[must_use]
// pub fn basic_light_style(level: Level) -> Style {
//     match level {
//         Level::Error => Color::red().bold(),
//         Level::Warning => Color::magenta().bold(),
//         Level::Heading => Color::blue().bold(),
//         Level::Subheading => Color::cyan().bold(),
//         Level::Emphasis => Color::green().bold(),
//         Level::Bright => Color::green(),
//         Level::Normal => Color::dark_gray(),
//         Level::Debug => Color::cyan(),
//         Level::Ghost => Color::cyan().italic(),
//     }
// }

// /// Returns the style for basic (16-color) dark theme
// #[must_use]
// pub fn basic_dark_style(level: Level) -> Style {
//     match level {
//         Level::Error => Color::red().bold(),
//         Level::Warning => Color::yellow().bold(),
//         Level::Heading => Color::green().bold(),
//         Level::Subheading => Color::blue().bold(),
//         Level::Emphasis => Color::cyan().bold(),
//         Level::Bright => Color::light_yellow(),
//         Level::Normal => Color::white(),
//         Level::Debug => Color::light_cyan(),
//         Level::Ghost => Color::light_gray().italic(),
//     }
// }

// /// Returns the style for full (256-color) light theme
// #[must_use]
// pub fn full_light_style(level: Level) -> Style {
//     match level {
//         Level::Error => Color::fixed(160).bold(),   // GuardsmanRed
//         Level::Warning => Color::fixed(164).bold(), // DarkPurplePizzazz
//         Level::Heading => Color::fixed(19).bold(),  // MidnightBlue
//         Level::Subheading => Color::fixed(26).bold(), // ScienceBlue
//         Level::Emphasis => Color::fixed(167).bold(), // RomanOrange
//         Level::Bright => Color::fixed(42).bold(),   // CaribbeanGreen
//         Level::Normal => Color::fixed(16),          // Black
//         Level::Debug => Color::fixed(32),           // LochmaraBlue
//         Level::Ghost => Color::fixed(232).italic(), // DarkCodGray
//     }
// }

// /// Returns the style for full (256-color) dark theme
// #[must_use]
// pub fn full_dark_style(level: Level) -> Style {
//     match level {
//         Level::Error => Color::fixed(1).bold(),      // UserRed
//         Level::Warning => Color::fixed(171).bold(),  // LighterHeliotrope
//         Level::Heading => Color::fixed(33).bold(),   // AzureRadiance
//         Level::Subheading => Color::fixed(44),       // RobinEggBlue
//         Level::Emphasis => Color::fixed(173).bold(), // Copperfield
//         Level::Bright => Color::fixed(118).italic(), // ChartreuseGreen
//         Level::Normal => Color::fixed(231),          // White
//         Level::Debug => Color::fixed(37),            // BondiBlue
//         Level::Ghost => Color::fixed(251).italic(),  // Silver
//     }
// }

#[must_use]
#[profiled]
pub fn paint_for_role(role: Role, string: &str) -> String {
    Style::for_role(role).paint(string)
}

#[must_use]
#[profiled]
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

// impl From<Level> for Role {
//     fn from(level: Level) -> Self {
//         profile_method!("Role::from");
//         match level {
//             Level::Error => Self::Error,
//             Level::Warning => Self::Warning,
//             Level::Heading => Self::Heading1,
//             Level::Subheading => Self::Heading2,
//             Level::Emphasis => Self::Emphasis,
//             Level::Bright => Self::Info,   // Highlighting important info
//             Level::Normal => Self::Normal, // Default display style
//             Level::Debug => Self::Debug,
//             Level::Ghost => Self::Hint,
//         }
//     }
// }

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

#[derive(Clone, Debug, Default, PaletteMethods)]
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

impl Palette {
    #[must_use]
    #[profiled]
    pub fn style_for_role(&self, role: Role) -> Style {
        match role {
            Heading1 => self.heading1.clone(),
            Role::Heading2 => self.heading2.clone(),
            Role::Heading3 => self.heading3.clone(),
            Role::Error => self.error.clone(),
            Role::Warning => self.warning.clone(),
            Role::Success => self.success.clone(),
            Info => self.info.clone(),
            Role::Emphasis => self.emphasis.clone(),
            Role::Code => self.code.clone(),
            Normal => self.normal.clone(),
            Role::Subtle => self.subtle.clone(),
            Role::Hint => self.hint.clone(),
            Role::Debug => self.debug.clone(),
            Role::Trace => self.trace.clone(),
        }
    }
}

// Make sure each test gets a fresh Palette
// impl Default for Palette {
//     fn default() -> Self {
//         Self {
//             heading1: Style::default(),
//             heading2: Style::default(),
//             heading3: Style::default(),
//             error: Style::default(),
//             warning: Style::default(),
//             success: Style::default(),
//             info: Style::default(),
//             emphasis: Style::default(),
//             code: Style::default(),
//             normal: Style::default(),
//             subtle: Style::default(),
//             hint: Style::default(),
//             debug: Style::default(),
//             trace: Style::default(),
//         }
//     }
// }

// ThemeDefinition & ThemeSignature and their impls
// generate_theme_types! {}

// ThemeIndex, THEME_INDEX and BG_LOOKUP
preload_themes! {}

/// Theme definition loaded from TOML files
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ThemeDefinition {
    name: String,
    #[serde(skip)]
    pub filename: PathBuf, // e.g., "themes/built_in/dracula.toml"
    #[serde(skip)]
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
    #[profiled]
    pub fn term_bg_luma(&self) -> &str {
        &self.term_bg_luma
    }

    /// Get the minimum color support requirement
    #[must_use]
    #[profiled]
    pub fn min_color_support(&self) -> &str {
        &self.min_color_support
    }

    /// Get the background color if specified
    #[must_use]
    #[profiled]
    pub fn backgrounds(&self) -> Vec<String> {
        self.backgrounds.clone()
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Theme {
    pub name: String,      // e.g., "Dracula"
    pub filename: PathBuf, // e.g., "themes/built_in/dracula.toml"
    pub is_builtin: bool,  // true for built-in themes, false for custom
    pub term_bg_luma: TermBgLuma,
    pub min_color_support: ColorSupport,
    pub palette: Palette,
    pub backgrounds: Vec<String>,
    pub bg_rgbs: Vec<(u8, u8, u8)>, // Official first
    pub description: String,
}

impl Theme {
    #[profiled]
    fn from_toml(theme_name: &str, theme_toml: &str) -> Result<Self, ThagError> {
        // vlog!(V::V, "About to call toml::from_str(theme_toml)");
        let mut def: ThemeDefinition = toml::from_str(theme_toml)?;
        // vlog!(V::V, "Done! def={def:?}");
        def.filename = PathBuf::from(format!("themes/built_in/{theme_name}.toml"));
        def.is_builtin = true;
        // vlog!(V::V, "About to call Theme::from_definition({def:?})");
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
    #[profiled]
    pub fn auto_detect(
        color_support: ColorSupport,
        term_bg_luma: TermBgLuma,
        maybe_term_bg: Option<&(u8, u8, u8)>,
    ) -> ThagResult<Self> {
        // NB: don't call `TermAttributes::get_or_init()` here because it will cause a tight loop
        // since we're called from the TermAttributes::initialize.
        vlog!(V::V, "maybe_term_bg={maybe_term_bg:?}");
        let Some(term_bg_rgb) = maybe_term_bg else {
            return fallback_theme(term_bg_luma);
        };

        // let signatures = get_theme_signatures();
        // vlog!(V::V, "signatures={signatures:?}");
        let hex = rgb_to_bare_hex(term_bg_rgb);
        let exact_matches = BG_LOOKUP
            .get(&hex)
            .map(|names| Vec::from(*names))
            .unwrap_or_default();
        vlog!(
            V::V,
            "term_bg_rgb={term_bg_rgb:?}, hex={hex}, exact_matches for hex={exact_matches:?}"
        );

        // let exact_matches = get_exact_matches(&exact_bg_matches, *term_bg_rgb, color_support);
        // vlog!(V::V, "exact_matches={exact_matches:#?}");

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
        vlog!(V::V, "Found {} eligible themes", eligible_themes.len());
        #[cfg(feature = "config")]
        if let Some(config) = maybe_config() {
            vlog!(
                V::V,
                "1. Try exact background RGB match of a preferred theme."
            );
            vlog!(V::V, "Looking for match on config styling");
            let preferred_styling = get_preferred_styling(term_bg_luma, &config);

            for preferred_name in preferred_styling {
                vlog!(V::V, "preferred_name={preferred_name}");
                if exact_matches.contains(&preferred_name.as_str()) {
                    vlog!(V::V, "Found an exact match in {preferred_name}");
                    return Self::get_theme_with_color_support(preferred_name, color_support);
                }
            }

            // vlog!(
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
            //     vlog!(V::V, "next_best_matches={next_best_matches:#?}");

            //     for preferred_name in preferred_styling {
            //         vlog!(V::V, "preferred_name={preferred_name}");
            //         if next_best_matches.contains(preferred_name) {
            //             vlog!(
            //                 V::V,
            //                 "Found an exact match at reduced color in {preferred_name}"
            //             );
            //             return Self::1910(preferred_name);
            //         }
            //     }
            // }

            vlog!(V::V, "2. Look for any theme exactly matching colour support and terminal background colour, in hopes of matching existing theme colours.");
            vlog!(V::V, "a. Try exact match on fallback names");
            let fallback_styling = get_fallback_styling(term_bg_luma, &config);
            for fallback_name in fallback_styling {
                vlog!(V::V, "fallback_name={fallback_name}");
                if exact_matches.contains(&fallback_name.as_str()) {
                    vlog!(V::V, "Found an exact match in fallback {fallback_name}");
                    return Self::get_theme_with_color_support(fallback_name, color_support);
                }
            }
            vlog!(V::V, "b. Try for any exact match.");
            if let Some(exact_match) = exact_matches.into_iter().next() {
                vlog!(V::V, "Found an exact match with {exact_match}");
                return Self::get_theme_with_color_support(exact_match, color_support);
            }
            vlog!(V::V, "No exact matches found.");

            vlog!(V::V, "3. Try closest match to a preferred theme.");
            let mut best_match = None;
            let mut min_distance = f32::MAX;
            for preferred_name in preferred_styling {
                let preferred_idx = THEME_INDEX.get(preferred_name);
                // vlog!(V::V, "theme_name={theme_name}");
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
                vlog!(V::V, "Choosing preferred theme {theme} because it most closely matches terminal bg {term_bg_rgb:?}");
                return Self::get_theme_with_color_support(theme, color_support);
            }

            vlog!(
                V::V,
                "4. Try closest match to a fallback theme, irrespective of colour support."
            );
            let mut best_match = None;
            let mut min_distance = f32::MAX;
            for fallback_name in fallback_styling {
                let fallback_idx = THEME_INDEX.get(fallback_name);
                // vlog!(V::V, "theme_name={theme_name}");
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
                vlog!(V::V, "Choosing preferred theme {theme} because it most closely matches terminal bg {term_bg_rgb:?}");
                return Self::get_theme_with_color_support(theme, color_support);
            }

            vlog!(V::V, "5. Try closest match.");
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
                vlog!(V::V, "Found the closest match with {theme}");
                return Self::get_theme_with_color_support(theme, color_support);
            }
        }

        vlog!(V::V, "6. Fall back to basic theme.");
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
    #[profiled]
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
    #[profiled]
    pub fn get_builtin(theme_name: &str) -> ThagResult<Self> {
        let maybe_theme_index = THEME_INDEX.get(theme_name);
        let Some(theme_index) = maybe_theme_index else {
            return Err(ThagError::FromStr(
                format!("No theme found for name {theme_name}").into(),
            ));
        };
        Self::from_toml(theme_name, theme_index.content)
    }

    // New method to get theme with specific color support
    #[profiled]
    fn get_theme_with_color_support(
        theme_name: &str,
        color_support: ColorSupport,
    ) -> ThagResult<Self> {
        let mut theme = Self::get_builtin(theme_name)?;
        if color_support != ColorSupport::TrueColor {
            vlog!(V::V, "Converting to {color_support:?}");
            theme.convert_to_color_support(color_support);
        }
        // eprintln!("Theme={:#?}", theme);
        Ok(theme)
    }

    #[profiled]
    fn from_definition(def: ThemeDefinition) -> ThagResult<Self> {
        // vlog!(V::V, "def.min_color_support={:?}", def.min_color_support);
        let color_support = ColorSupport::from_str(&def.min_color_support);
        // vlog!(V::V, "color_support={color_support:?})");

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
    /// use thag_rs::ThagError;
    /// use thag_rs::styling::{ColorSupport, TermBgLuma, Theme};
    /// let theme = Theme::get_builtin("dracula")?;
    /// theme.validate(&ColorSupport::TrueColor, &TermBgLuma::Dark)?;
    /// # Ok::<(), ThagError>(())
    /// ```
    #[profiled]
    pub fn validate(
        &self,
        available_support: &ColorSupport,
        term_bg_luma: &TermBgLuma,
    ) -> ThagResult<()> {
        // Check color support
        // vlog!(V::V, "self.min_color_support={:?}", self.min_color_support);
        // vlog!(V::V, "available_support={available_support:?}");
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

    #[profiled]
    fn validate_palette(&self) -> ThagResult<()> {
        self.palette.validate_styles(self.min_color_support)?;
        Ok(())
    }

    /// Validates a theme definition before creating a Theme
    #[allow(dead_code)]
    #[profiled]
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
    /// use thag_rs::ThagError;
    /// use thag_rs::styling::{ColorSupport, TermBgLuma, Theme};
    /// let theme = Theme::load(
    ///     Path::new("themes/built_in/basic_light.toml"),
    ///     ColorSupport::Basic,
    ///     TermBgLuma::Light
    /// )?;
    /// # Ok::<(), ThagError>(())
    /// ```
    #[profiled]
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
    #[profiled]
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
    #[profiled]
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
    #[profiled]
    pub fn list_builtin() -> Vec<String> {
        THEME_INDEX.keys().map(ToString::to_string).collect()
    }

    #[must_use]
    #[profiled]
    fn style_for_role(&self, role: Role) -> Style {
        self.palette.style_for_role(role)
    }

    /// Converts RGB values to an ANSI color index.
    ///
    /// # Panics
    ///
    /// Panics if the color index cannot be converted to a u8.
    #[must_use]
    #[profiled]
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

    #[profiled]
    pub fn convert_to_color_support(&mut self, target: ColorSupport) {
        match target {
            ColorSupport::TrueColor => (), // No conversion needed
            ColorSupport::Color256 => self.convert_to_256(),
            ColorSupport::Basic | ColorSupport::Undetermined => self.convert_to_basic(),
            ColorSupport::None => self.convert_to_none(),
        }
    }

    #[profiled]
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
    #[profiled]
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

    #[profiled]
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

#[profiled]
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

#[profiled]
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

#[cfg(feature = "config")]
#[profiled]
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

#[cfg(feature = "config")]
#[profiled]
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
//     profile!("get_reduced_palette_matches", time);
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
#[profiled]
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
#[profiled]
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
#[profiled]
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
//             // vlog!(V::V, "name=");
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
/// A line print macro that conditionally prints a message using `cprtln` if the current global verbosity
/// is at least as verbose as the `Verbosity` (alias `V`) level passed in.
///
/// The message will be styled and coloured according to the `MessageLevel` (alias `Lvl`) passed in.
///
/// Format: `cvprtln!(role: &Role, verbosity: Verbosity, "Lorem ipsum dolor {} amet", content: &str);`
#[macro_export]
macro_rules! cvprtln {
    ($role:expr, $verbosity:expr, $($arg:tt)*) => {{
        if $verbosity <= $crate::logging::get_verbosity() {
            let style = $crate::styling::Style::for_role($role);
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

#[profiled]
fn base_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> u32 {
    let dr = f64::from(i32::from(c1.0) - i32::from(c2.0)) * 0.3;
    let dg = f64::from(i32::from(c1.1) - i32::from(c2.1)) * 0.59;
    let db = f64::from(i32::from(c1.2) - i32::from(c2.2)) * 0.11;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let base_distance = db.mul_add(db, dr.mul_add(dr, dg * dg)) as u32;
    base_distance
}

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
    profile!("find_closest_section", time);
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
    end!("find_closest_section");

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

#[profiled]
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

// Helper function to get RGB values for a color number (for verification)
#[must_use]
#[profiled]
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
#[enable_profiling]
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

#[profiled]
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
    println!("\n\t{}", paint_for_role(Normal, "Role styles:"));
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
#[profiled]
pub fn display_theme_details() {
    let term_attrs = TermAttributes::get_or_init();
    let theme = &term_attrs.theme;
    let theme_bgs = &term_attrs.theme.bg_rgbs;
    let theme_bgs = if theme_bgs.is_empty() {
        &term_attrs
            .theme
            .backgrounds
            .iter()
            .filter_map(|hex| hex_to_rgb(hex).ok())
            .collect()
    } else {
        theme_bgs
    };
    // eprintln!(
    //     "theme_bgs={theme_bgs:?}, backgrounds={:?}",
    //     term_attrs.theme.backgrounds
    // );
    let rgb_disp = if theme_bgs.is_empty() {
        "None".to_string()
    } else if let Some(term_bg_rgb) = term_attrs.term_bg_rgb {
        let mut min_distance = f32::MAX;
        let mut closest_rgb: (u8, u8, u8) = (0, 0, 0);
        for rgb in theme_bgs {
            let color_distance = color_distance(term_bg_rgb, *rgb);
            if color_distance < min_distance {
                min_distance = color_distance;
                closest_rgb = *rgb;
            }
        }
        dual_format_rgb(closest_rgb)
    } else {
        "None".to_string()
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

    println!("\n\t{}", paint_for_role(Normal, "Theme attributes:"));
    println!("\t{}", "─".repeat(flower_box_len));

    for (attr, description) in theme_docs {
        let styled_name = paint_for_role(Info, attr);
        let padding = " ".repeat(col1_width.saturating_sub(attr.len()));

        print!("\t{styled_name}{padding}");
        let description = if *attr == "Theme" {
            paint_for_role(Heading1, description)
        } else {
            (*description).to_string()
        };
        println!("│ {description}");
    }

    println!("\t{}\n", "─".repeat(flower_box_len));

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

    println!("\n\t{}", paint_for_role(Normal, "Terminal attributes:"));
    println!("\t{}", "─".repeat(flower_box_len));

    for (attr, description) in terminal_docs {
        let styled_name = paint_for_role(Info, attr);
        let padding = " ".repeat(col1_width.saturating_sub(attr.len()));

        print!("\t{styled_name}{padding}");
        println!("│ {description}");
    }

    println!("\t{}\n", "─".repeat(flower_box_len));
}

#[profiled]
fn dual_format_rgb((r, g, b): (u8, u8, u8)) -> String {
    format!("#{r:02x}{g:02x}{b:02x} = rgb({r}, {g}, {b})")
}

#[macro_export]
macro_rules! clog {
    ($role:expr, $($arg:tt)*) => {{
        if $crate::styling::LOGGING_ENABLED.load(std::sync::atomic::Ordering::SeqCst) {
            let style = $crate::styling::Style::for_role($role);
            println!("{}", style.paint(format!($($arg)*)));
        }
    }};
}

#[macro_export]
macro_rules! clog_error {
    ($($arg:tt)*) => { $crate::clog!($crate::styling::Role::Error, $($arg)*) };
}

#[macro_export]
macro_rules! clog_warning {
        ($($arg:tt)*) => { $crate::clog!($crate::styling::Role::Warning, $($arg)*) };
    }

#[macro_export]
macro_rules! clog_heading1 {
    ($($arg:tt)*) => { $crate::clog!($crate::styling::Role::Heading1, $($arg)*) };
}

#[macro_export]
macro_rules! clog_heading2 {
    ($($arg:tt)*) => { $crate::clog!($crate::styling::Role::Heading2, $($arg)*) };
}

#[macro_export]
macro_rules! heading3 {
    ($($arg:tt)*) => { $crate::clog!($crate::styling::Role::Heading3, $($arg)*) };
}

#[macro_export]
macro_rules! clog_emphasis {
    ($($arg:tt)*) => { $crate::clog!($crate::styling::Role::Emphasis, $($arg)*) };
}

#[macro_export]
macro_rules! clog_success {
    ($($arg:tt)*) => { $crate::clog!($crate::styling::Role::Success, $($arg)*) };
}

#[macro_export]
macro_rules! clog_info {
    ($($arg:tt)*) => { $crate::clog!($crate::styling::Role::Info, $($arg)*) };
}

#[macro_export]
macro_rules! clog_normal {
    ($($arg:tt)*) => { $crate::clog!($crate::styling::Role::Normal, $($arg)*) };
}

#[macro_export]
macro_rules! clog_debug {
    ($($arg:tt)*) => { $crate::clog!($crate::styling::Role::Debug, $($arg)*) };
}

#[macro_export]
macro_rules! clog_subtle {
    ($($arg:tt)*) => { $crate::clog!($crate::styling::Role::Subtle, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog {
    ($verbosity:expr, $level:expr, $($arg:tt)*) => {{
        if $crate::styling::LOGGING_ENABLED.load(std::sync::atomic::Ordering::SeqCst) {
            let logger = $crate::logging::LOGGER.lock().expect("Failed to lock logger");
            let message = format!($($arg)*);

            #[cfg(feature = "color_support")]
            {
                let color_logger = $crate::styling::TermAttributes::get();
                let style = $crate::styling::Style::for_role($role);
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
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::styling::Role::Error, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_warning {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::styling::Role::Warning, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_heading {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::styling::Role::Heading1, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_subheading {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::styling::Role::Heading2, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_emphasis {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::styling::Role::Emphasis, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_bright {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::styling::Role::Info, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_normal {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::styling::Role::Normal, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_debug {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::styling::Role::Debug, $verbosity, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_ghost {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvprtln!($crate::styling::Role::Hint, $verbosity, $($arg)*) };
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
        #[profiled]
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
        vlog!(V::V, "basic={basic:#?}");
        let basic_style = style_for_theme_and_role(&basic.theme, test_role);
        let painted = basic_style.paint("test");
        vlog!(V::V, "painted={painted:?}, style={basic_style:?}");
        assert!(painted.contains("\x1b[31m"));
        assert!(painted.ends_with("\u{1b}[0m"));

        // Color_256 support should use a different ANSI string from basic
        let color256_style = style_for_theme_and_role(&color256.theme, test_role);
        let painted = color256_style.paint("test");
        vlog!(V::V, "painted={painted:?}");
        assert!(painted.contains("\x1b[38;5;"));
        assert!(painted.ends_with("\u{1b}[0m"));

        // TrueColor support should use RGB formatting
        let true_color_style = style_for_theme_and_role(&true_color.theme, test_role);
        let painted = true_color_style.paint("test");
        vlog!(V::V, "painted={painted:?}");
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
        let error_style = style_for_theme_and_role(&attrs.theme, Heading1);
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
