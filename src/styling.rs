use documented::{Documented, DocumentedVariants};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
// use std::sync::atomic::Ordering;
use crate::profile_method;
use std::sync::OnceLock;
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

#[cfg(feature = "color_detect")]
use crate::terminal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorInfo {
    pub ansi: &'static str,
    pub index: Option<u8>, // Store the original color index if it exists
}

impl ColorInfo {
    #[must_use]
    pub const fn new(ansi: &'static str, index: Option<u8>) -> Self {
        Self { ansi, index }
    }

    #[must_use]
    pub const fn basic(ansi: &'static str) -> Self {
        Self::new(ansi, None)
    }

    #[must_use]
    pub fn indexed(index: u8) -> Self {
        Self::new(
            Box::leak(format!("\x1b[38;5;{index}m").into_boxed_str()),
            Some(index),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    pub foreground: Option<ColorInfo>,
    pub bold: bool,
    pub italic: bool,
    pub dim: bool,
}

impl Style {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            foreground: None,
            bold: false,
            italic: false,
            dim: false,
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

    pub fn paint<D>(&self, val: D) -> String
    where
        D: std::fmt::Display,
    {
        let mut result = String::new();

        if let Some(ref fg) = self.foreground {
            result.push_str(fg.ansi);
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

        result.push_str(&val.to_string());
        result.push_str("\x1b[0m");
        result
    }

    #[must_use]
    pub fn with_color_index(index: u8) -> Self {
        Self {
            foreground: Some(ColorInfo {
                ansi: Box::leak(format!("\x1b[38;5;{index}m").into_boxed_str()),
                index: Some(index),
            }),
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
    // Basic colors (ANSI 16)
    const BLACK: &'static str = "\x1b[30m";
    const RED: &'static str = "\x1b[31m";
    const GREEN: &'static str = "\x1b[32m";
    const YELLOW: &'static str = "\x1b[33m";
    const BLUE: &'static str = "\x1b[34m";
    const MAGENTA: &'static str = "\x1b[35m";
    const CYAN: &'static str = "\x1b[36m";
    const WHITE: &'static str = "\x1b[37m";

    // Bright colors
    const DARK_GRAY: &'static str = "\x1b[90m";
    const LIGHT_RED: &'static str = "\x1b[91m";
    const LIGHT_GREEN: &'static str = "\x1b[92m";
    const LIGHT_YELLOW: &'static str = "\x1b[93m";
    const LIGHT_BLUE: &'static str = "\x1b[94m";
    const LIGHT_MAGENTA: &'static str = "\x1b[95m";
    const LIGHT_CYAN: &'static str = "\x1b[96m";
    const LIGHT_GRAY: &'static str = "\x1b[97m";

    #[must_use]
    pub fn red() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::RED)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn green() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::GREEN)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn yellow() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::YELLOW)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn blue() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::BLUE)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn magenta() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::MAGENTA)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn cyan() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::CYAN)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn white() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::WHITE)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn dark_gray() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::DARK_GRAY)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn light_yellow() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::LIGHT_YELLOW)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn light_cyan() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::LIGHT_CYAN)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn light_gray() -> Style {
        Style {
            foreground: Some(ColorInfo::basic(Self::LIGHT_GRAY)),
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
    Eq,
    Serialize,
)]
#[strum(serialize_all = "snake_case")]
pub enum ColorSupport {
    /// Full color support, suitable for color palettes of 256 colours (16 bit) or higher.
    Xterm256,
    /// Basic 16-color support
    Ansi16,
    /// No color support
    None,
    /// Auto-detect from terminal
    #[default]
    AutoDetect,
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

// We can implement conversions to u8 directly here
impl From<&Level> for u8 {
    fn from(level: &Level) -> Self {
        match level {
            Level::Error => 160,     // GuardsmanRed
            Level::Warning => 164,   // DarkPurplePizzazz
            Level::Heading => 10,    // UserBrightGreen
            Level::Subheading => 26, // ScienceBlue
            Level::Emphasis => 173,  // Copperfield
            Level::Bright => 46,     // Green
            Level::Normal => 16,     // Black
            Level::Debug => 32,      // LochmaraBlue
            Level::Ghost => 232,     // DarkCodGray
        }
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub enum ColorInitStrategy {
    Configure(&'static ColorSupport, &'static TermTheme),
    Default,
    #[cfg(feature = "color_detect")]
    Detect,
}

/// Manages terminal color attributes and styling based on terminal capabilities and theme
pub struct TermAttributes {
    color_support: &'static ColorSupport,
    theme: &'static TermTheme,
}

/// Global instance of `TermAttributes`
static INSTANCE: OnceLock<TermAttributes> = OnceLock::new();
/// Global flag to enable/disable logging
pub static LOGGING_ENABLED: AtomicBool = AtomicBool::new(true);

impl TermAttributes {
    /// Creates a new `TermAttributes` instance with specified support and theme
    const fn new(color_support: &'static ColorSupport, theme: &'static TermTheme) -> Self {
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
    ///     ColorSupport::Ansi16,
    ///     TermTheme::Dark
    /// ));
    /// ```
    pub fn initialize(strategy: &ColorInitStrategy) -> &'static Self {
        INSTANCE.get_or_init(|| match strategy {
            &ColorInitStrategy::Configure(support, theme) => Self::new(support, theme),
            &ColorInitStrategy::Default => Self::new(&ColorSupport::Ansi16, &TermTheme::Dark),
            #[cfg(feature = "color_detect")]
            &ColorInitStrategy::Detect => {
                let support = crate::terminal::detect_color_support();
                let theme = crate::terminal::detect_theme();
                Self::new(support, theme)
            }
        })
    }

    /// Gets or initializes the global `TermAttributes` instance with default settings
    ///
    /// This is a convenience method that initializes with `ColorInitStrategy::Default`
    /// if the instance hasn't been initialized yet.
    pub fn get() -> &'static Self {
        INSTANCE.get_or_init(|| Self::new(&ColorSupport::Ansi16, &TermTheme::Dark))
    }

    #[must_use]
    pub fn get_theme(&self) -> &'static TermTheme {
        if self.theme != &TermTheme::Undetermined {
            return self.theme;
        }

        #[cfg(feature = "color_detect")]
        {
            terminal::detect_theme()
        }

        #[cfg(not(feature = "color_detect"))]
        &TermTheme::Dark
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
    /// let attrs = TermAttributes::get();
    /// let error_style = attrs.style_for_level(Level::Error);
    /// println!("{}", error_style.paint("This is an error message"));
    /// ```
    #[must_use]
    pub fn style_for_level(&self, level: Level) -> Style {
        profile_method!("TermAttrs::style_for_level");
        match (&self.color_support, &self.theme) {
            (&ColorSupport::None, _) => Style::default(),
            (&ColorSupport::Ansi16, &TermTheme::Light) => Self::basic_light_style(level),
            (&ColorSupport::Ansi16, &TermTheme::Dark) => Self::basic_dark_style(level),
            (&ColorSupport::Xterm256, &TermTheme::Light) => Self::full_light_style(level),
            (&ColorSupport::Xterm256, &TermTheme::Dark) => Self::full_dark_style(level),
            (_, &TermTheme::Undetermined) | (&ColorSupport::AutoDetect, _) => unreachable!(),
        }
    }

    /// Returns the style for basic (16-color) light theme
    fn basic_light_style(level: Level) -> Style {
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
    fn basic_dark_style(level: Level) -> Style {
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
    fn full_light_style(level: Level) -> Style {
        match level {
            Level::Error => Color::fixed(160).bold(),    // GuardsmanRed
            Level::Warning => Color::fixed(164).bold(),  // DarkPurplePizzazz
            Level::Heading => Color::fixed(19).bold(),   // MidnightBlue
            Level::Subheading => Color::fixed(26),       // ScienceBlue
            Level::Emphasis => Color::fixed(173).bold(), // Copperfield
            Level::Bright => Color::fixed(46),           // Green
            Level::Normal => Color::fixed(16),           // Black
            Level::Debug => Color::fixed(32),            // LochmaraBlue
            Level::Ghost => Color::fixed(232).italic(),  // DarkCodGray
        }
    }

    /// Returns the style for full (256-color) dark theme
    fn full_dark_style(level: Level) -> Style {
        match level {
            Level::Error => Color::fixed(1).bold(),      // UserRed
            Level::Warning => Color::fixed(171).bold(),  // LighterHeliotrope
            Level::Heading => Color::fixed(42).bold(),   // CaribbeanGreen
            Level::Subheading => Color::fixed(75),       // DarkMalibuBlue
            Level::Emphasis => Color::fixed(173).bold(), // Copperfield
            Level::Bright => Color::fixed(3),            // UserYellow
            Level::Normal => Color::fixed(231),          // White
            Level::Debug => Color::fixed(37),            // BondiBlue
            Level::Ghost => Color::fixed(251).italic(),  // Silver
        }
    }
}

#[must_use]
pub fn style_string(lvl: Level, string: &str) -> String {
    TermAttributes::get().style_for_level(lvl).paint(string)
}

// Convenience macros
#[macro_export]
macro_rules! clog {
    ($level:expr, $($arg:tt)*) => {{
        if $crate::styling::LOGGING_ENABLED.load(std::sync::atomic::Ordering::SeqCst) {
            let attrs = $crate::styling::TermAttributes::get();
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
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvlog!($verbosity, $crate::Level::Error, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_warning {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvlog!($verbosity, $crate::Level::Warning, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_heading {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvlog!($verbosity, $crate::Level::Heading, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_subheading {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvlog!($verbosity, $crate::Level::Subheading, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_emphasis {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvlog!($verbosity, $crate::Level::Emphasis, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_bright {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvlog!($verbosity, $crate::Level::Bright, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_normal {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvlog!($verbosity, $crate::Level::Normal, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_debug {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvlog!($verbosity, $crate::Level::Debug, $($arg)*) };
}

#[macro_export]
macro_rules! cvlog_ghost {
    ($verbosity:expr, $($arg:tt)*) => { $crate::cvlog!($verbosity, $crate::Level::Ghost, $($arg)*) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    static MOCK_THEME_DETECTION: AtomicBool = AtomicBool::new(false);

    impl TermAttributes {
        fn with_mock_theme(
            color_support: &'static ColorSupport,
            theme: &'static TermTheme,
        ) -> Self {
            MOCK_THEME_DETECTION.store(true, Ordering::SeqCst);
            Self::new(&color_support, &theme)
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
        let term_attrs = TermAttributes::with_mock_theme(&ColorSupport::Xterm256, &TermTheme::Dark);
        let defaulted = term_attrs.get_theme();
        assert_eq!(defaulted, &TermTheme::Dark);
        println!();
        flush_test_output();
    }
}
