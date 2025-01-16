use crate::errors::ThemeError;
use crate::{cvprtln, profile_method, ThagResult, V};
use documented::{Documented, DocumentedVariants};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::OnceLock;
use strum::{Display, EnumIter, EnumString, IntoStaticStr};
use thag_proc_macros::PaletteMethods;

#[cfg(feature = "color_detect")]
use crate::terminal;

#[cfg(debug_assertions)]
use crate::debug_log;

// Include the generated theme data
include!(concat!(env!("OUT_DIR"), "/theme_data.rs"));

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ColorValue {
    Basic { basic: [String; 2] }, // [ANSI code, index]
    Color256 { color_256: u8 },   // 256-color index
    TrueColor { rgb: [u8; 3] },   // RGB values
}

#[derive(Debug, Deserialize)]
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
            value: ColorValue::Color256 { color_256: index },
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
        match support {
            ColorSupport::TrueColor => Self::rgb(rgb.0, rgb.1, rgb.2),
            ColorSupport::Color256 => Self::indexed(find_closest_color(rgb)),
            _ => Self::indexed(find_closest_basic_color(rgb)),
        }
    }
}

// // Theme background detection
// pub struct ThemeSignature {
//     bg_rgb: (u8, u8, u8),
//     name: &'static str,
//     description: &'static str,
// }

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

    fn from_config(config: &StyleConfig) -> ThagResult<Self> {
        let mut style = match &config.color {
            ColorValue::Basic { basic } => {
                let ansi_str = Box::leak(basic[0].clone().into_boxed_str());
                Style::fg(ColorInfo::new(ansi_str, basic[1].parse()?))
            }
            ColorValue::Color256 { color_256 } => Style::fg(ColorInfo::indexed(*color_256)),
            ColorValue::TrueColor { rgb } => Style::fg(ColorInfo::rgb(rgb[0], rgb[1], rgb[2])),
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
}

impl Default for Style {
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
pub enum TermTheme {
    /// Light background terminal
    Light,
    /// Dark background terminal
    Dark,
    /// Let `thag` autodetect the background luminosity
    Undetermined,
}

impl Default for TermTheme {
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
        let term_attrs = TermAttributes::get_or_default();
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

#[derive(Clone)]
#[non_exhaustive]
pub enum ColorInitStrategy {
    Configure(ColorSupport, TermTheme),
    Default,
    #[cfg(feature = "color_detect")]
    Detect,
}

/// Manages terminal color attributes and styling based on terminal capabilities and theme
pub struct TermAttributes {
    pub color_support: ColorSupport,
    pub theme: TermTheme,
}

/// Global instance of `TermAttributes`
static INSTANCE: OnceLock<TermAttributes> = OnceLock::new();
/// Global flag to enable/disable logging
pub static LOGGING_ENABLED: AtomicBool = AtomicBool::new(true);

impl TermAttributes {
    /// Creates a new `TermAttributes` instance with specified support and theme
    const fn new(color_support: ColorSupport, theme: TermTheme) -> Self {
        Self {
            color_support,
            theme,
        }
    }

    /// Initializes the global `TermAttributes` instance with the specified strategy
    ///
    /// # Examples
    ///
    /// ```
    /// use thag_rs::styling::{TermAttributes, ColorInitStrategy, ColorSupport, TermTheme};
    ///
    /// // Use default settings
    /// let attrs = TermAttributes::initialize(ColorInitStrategy::Default);
    ///
    /// // Configure explicitly
    /// let attrs = TermAttributes::initialize(ColorInitStrategy::Configure(
    ///     ColorSupport::Basic,
    ///     TermTheme::Dark
    /// ));
    /// ```
    pub fn initialize(strategy: ColorInitStrategy) -> &'static Self {
        let term_attrs = INSTANCE.get_or_init(|| match strategy {
            ColorInitStrategy::Configure(support, theme) => Self::new(support, theme),
            ColorInitStrategy::Default => Self::new(ColorSupport::Basic, TermTheme::Dark),
            #[cfg(feature = "color_detect")]
            ColorInitStrategy::Detect => {
                let support = *crate::terminal::detect_color_support();
                let theme = crate::terminal::detect_theme().clone();
                Self::new(support, theme)
            }
        });
        cvprtln!(
            Lvl::Bright,
            V::V,
            "ColorSupport={:?}, TermTheme={:?}",
            term_attrs.color_support,
            term_attrs.theme
        );
        term_attrs
    }

    pub fn is_initialized() -> bool {
        INSTANCE.get().is_some()
    }

    /// Attempts to get the `TermAttributes` instance, returning None if not initialized
    pub fn try_get() -> Option<&'static Self> {
        INSTANCE.get()
    }

    /// Gets the `TermAttributes` instance or returns a default (Basic/Dark) instance
    pub fn get_or_default() -> &'static Self {
        INSTANCE.get_or_init(|| Self::new(ColorSupport::Basic, TermTheme::Dark))
    }

    /// Gets the global `TermAttributes` instance, panicking if it hasn't been initialized
    ///
    /// # Panics
    ///
    /// This function will panic if `initialize` hasn't been called first
    pub fn get() -> &'static Self {
        INSTANCE
            .get()
            .expect("TermAttributes not initialized. Call initialize() first")
    }

    #[must_use]
    pub fn get_theme(&self) -> TermTheme {
        if self.theme != TermTheme::Undetermined {
            return self.theme.clone();
        }

        #[cfg(feature = "color_detect")]
        {
            terminal::detect_theme().clone()
        }

        #[cfg(not(feature = "color_detect"))]
        TermTheme::Dark
    }

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
        match (&self.color_support, &self.theme) {
            (ColorSupport::None, _) => Style::default(),
            (ColorSupport::Basic, TermTheme::Light) => Self::basic_light_style(level),
            (ColorSupport::Basic, TermTheme::Dark) => Self::basic_dark_style(level),
            (ColorSupport::Color256, TermTheme::Light) => Self::full_light_style(level),
            (ColorSupport::Color256, TermTheme::Dark) => Self::full_dark_style(level),
            (support, theme) => {
                #[cfg(debug_assertions)]
                debug_log!(
                    "Using default style due to undetermined settings: support={:?}, theme={:?}",
                    support,
                    theme
                );
                Style::default()
            }
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

    // /// Creates a new TermAttributes instance with specified support and theme for testing
    // #[cfg(test)]
    // pub fn with_mock_theme(
    //     color_support: &'static ColorSupport,
    //     theme: &'static TermTheme,
    // ) -> Self {
    //     Self {
    //         color_support,
    //         theme,
    //     }
    // }
}

#[must_use]
pub fn style_string(lvl: Level, string: &str) -> String {
    TermAttributes::get_or_default()
        .style_for_level(lvl)
        .paint(string)
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
        match level {
            Level::Error => Role::Error,
            Level::Warning => Role::Warning,
            Level::Heading => Role::Heading1,
            Level::Subheading => Role::Heading2,
            Level::Emphasis => Role::Emphasis,
            Level::Bright => Role::Info,   // Highlighting important info
            Level::Normal => Role::Normal, // Default display style
            Level::Debug => Role::Debug,
            Level::Ghost => Role::Hint,
        }
    }
}

#[derive(Debug, Deserialize)]
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

#[derive(Clone, Debug, PaletteMethods)]
pub struct Palette {
    heading1: Style,
    heading2: Style,
    heading3: Style,
    error: Style,
    warning: Style,
    success: Style,
    info: Style,
    emphasis: Style,
    code: Style,
    normal: Style,
    subtle: Style,
    hint: Style,
    debug: Style,
    trace: Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermBgLuma {
    Light,
    Dark,
}

impl FromStr for TermBgLuma {
    type Err = ThemeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Err(ThemeError::InvalidTermBgLuma(s.to_string())),
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ThemeDefinition {
    name: String,
    description: String,
    term_bg_luma: String,
    min_color_support: String,
    background: Option<String>,
    palette: PaletteConfig, // Rename from Palette to make the role clear
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Theme {
    pub term_bg_luma: TermBgLuma,
    pub min_color_support: ColorSupport,
    pub palette: Palette,
    pub background: Option<String>,
    pub description: String,
}

impl Theme {
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
    /// let theme = Theme::load_from_file(Path::new("themes/custom.toml"))?;
    /// ```
    pub fn load_from_file(path: &Path) -> ThagResult<Self> {
        let content = fs::read_to_string(path)?;
        let def: ThemeDefinition = toml::from_str(&content)?;
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
    /// let theme = Theme::load_builtin("dracula")?;
    /// ```
    pub fn load_builtin(name: &str) -> ThagResult<Self> {
        let theme_toml = BUILT_IN_THEMES
            .get(name)
            .ok_or_else(|| ThemeError::UnknownTheme(name.to_string()))?;

        let def: ThemeDefinition = toml::from_str(theme_toml)?;
        Self::from_definition(def)
    }

    fn from_definition(def: ThemeDefinition) -> ThagResult<Self> {
        Ok(Theme {
            term_bg_luma: TermBgLuma::from_str(&def.term_bg_luma)?,
            min_color_support: ColorSupport::from_str(&def.min_color_support)?,
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
    /// let theme = Theme::load_builtin("dracula")?;
    /// theme.validate(ColorSupport::Color256, TermBgLuma::Dark)?;
    /// ```
    pub fn validate(
        &self,
        available_support: ColorSupport,
        term_bg_luma: TermBgLuma,
    ) -> ThagResult<()> {
        // Check color support
        eprintln!("self.min_color_support={:?}", self.min_color_support);
        eprintln!("available_support={available_support:?}");
        if available_support < self.min_color_support {
            return Err(ThemeError::ColorSupportMismatch {
                required: self.min_color_support,
                available: available_support,
            }
            .into());
        }

        // Check background compatibility
        if self.term_bg_luma != term_bg_luma {
            return Err(ThemeError::TermBgLumaMismatch {
                theme: self.term_bg_luma,
                terminal: term_bg_luma,
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
        if !["basic", "color_256", "true_color"].contains(&def.min_color_support.as_str()) {
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
    /// let theme = Theme::load(
    ///     Path::new("themes/custom.toml"),
    ///     ColorSupport::Color256,
    ///     TermBgLuma::Dark
    /// )?;
    /// ```
    pub fn load(
        path: &Path,
        available_support: ColorSupport,
        term_bg_luma: TermBgLuma,
    ) -> ThagResult<Self> {
        let theme = Self::load_from_file(path)?;
        theme.validate(available_support, term_bg_luma)?;
        Ok(theme)
    }

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
}

// Helper to check a single style
fn validate_style(style: &Style, min_support: ColorSupport) -> ThagResult<()> {
    if let Some(color_info) = &style.foreground {
        match &color_info.value {
            ColorValue::Basic { basic: _ } => Ok(()), // Basic is always valid
            ColorValue::Color256 { color_256: _ } => {
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
        }
    } else {
        Ok(())
    }
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
            let term_attrs = $crate::styling::TermAttributes::get_or_default();
            let style = term_attrs.style_for_level($level);
            let content = format!($($arg)*);
            let verbosity = $crate::logging::get_verbosity();
            $crate::vlog!(verbosity, "{}", style.paint(content));
        }
    }};
}

#[allow(dead_code)]
fn color_distance(c1: (u8, u8, u8), c2: (u8, u8, u8), is_system: bool) -> u32 {
    let base_distance = base_distance(c1, c2);

    // Give a slight preference to system colors when they're close matches
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    if is_system {
        (f64::from(base_distance) * 0.9) as u32
    } else {
        base_distance
    }
}

fn base_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> u32 {
    let dr = f64::from(i32::from(c1.0) - i32::from(c2.0)) * 0.3;
    let dg = f64::from(i32::from(c1.1) - i32::from(c2.1)) * 0.59;
    let db = f64::from(i32::from(c1.2) - i32::from(c2.2)) * 0.11;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let base_distance = (dr * dr + dg * dg + db * db) as u32;
    base_distance
}

fn find_closest_color(rgb: (u8, u8, u8)) -> u8 {
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

// Helper function to get RGB values for a color number (for verification)
#[must_use]
pub fn get_rgb(color: u8) -> (u8, u8, u8) {
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
    let _dracula = Theme::load_builtin("dracula")?;

    // Load custom theme
    let _custom = Theme::load_from_file(Path::new("custom_theme.toml"))?;

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
            let attrs = $crate::styling::TermAttributes::get_or_default();
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

    impl TermAttributes {
        fn with_mock_theme(color_support: ColorSupport, theme: TermTheme) -> Self {
            MOCK_THEME_DETECTION.store(true, Ordering::SeqCst);
            Self::new(color_support, theme)
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

    // Tests that need access to internal implementation
    #[test]
    fn test_styling_default_theme_with_mock() {
        init_test();
        let term_attrs = TermAttributes::with_mock_theme(ColorSupport::Color256, TermTheme::Dark);
        let defaulted = term_attrs.get_theme();
        assert_eq!(defaulted, TermTheme::Dark);
        println!();
        flush_test_output();
    }

    #[test]
    fn test_styling_no_color_support() {
        let term_attrs = TermAttributes::with_mock_theme(ColorSupport::None, TermTheme::Dark);
        let style = term_attrs.style_for_level(Level::Error);
        assert_eq!(style.paint("test"), "test"); // Should have no ANSI codes
    }

    #[test]
    fn test_styling_color_support_levels() {
        let none = TermAttributes::with_mock_theme(ColorSupport::None, TermTheme::Dark);
        let basic = TermAttributes::with_mock_theme(ColorSupport::Basic, TermTheme::Dark);
        let full = TermAttributes::with_mock_theme(ColorSupport::Color256, TermTheme::Dark);

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
        let attrs_light = TermAttributes::with_mock_theme(ColorSupport::Color256, TermTheme::Light);
        let attrs_dark = TermAttributes::with_mock_theme(ColorSupport::Color256, TermTheme::Dark);

        let heading_light = attrs_light.style_for_level(Level::Heading).paint("test");
        let heading_dark = attrs_dark.style_for_level(Level::Heading).paint("test");

        // Light and dark themes should produce different colors
        assert_ne!(heading_light, heading_dark);
    }

    #[test]
    fn test_styling_level_styling() {
        let attrs = TermAttributes::with_mock_theme(ColorSupport::Color256, TermTheme::Dark);

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
        let attrs = TermAttributes::with_mock_theme(ColorSupport::Color256, TermTheme::Dark);

        // Error should be bold
        let error_style = attrs.style_for_level(Level::Error).paint("test");
        assert!(error_style.contains("\x1b[1m"));

        // Ghost should be italic
        let ghost_style = attrs.style_for_level(Level::Ghost).paint("test");
        assert!(ghost_style.contains("\x1b[3m"));
    }

    #[test]
    fn test_load_dracula_theme() -> ThagResult<()> {
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
    fn test_dracula_validation() -> ThagResult<()> {
        let theme = Theme::load_from_file(Path::new("themes/built_in/dracula.toml"))?;

        // Should succeed with TrueColor support and dark background
        assert!(theme
            .validate(ColorSupport::TrueColor, TermBgLuma::Dark)
            .is_ok());

        // Should fail with insufficient color support
        assert!(theme
            .validate(ColorSupport::Color256, TermBgLuma::Dark)
            .is_err());

        // Should fail with wrong background
        assert!(theme
            .validate(ColorSupport::TrueColor, TermBgLuma::Light)
            .is_err());

        Ok(())
    }

    #[test]
    fn test_color_support_ordering() {
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
