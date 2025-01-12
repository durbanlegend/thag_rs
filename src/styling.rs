use crate::{cvprtln, profile_method, V};
use documented::{Documented, DocumentedVariants};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::OnceLock;
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

#[cfg(feature = "color_detect")]
use crate::terminal;

#[cfg(debug_assertions)]
use crate::debug_log;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorInfo {
    pub ansi: &'static str,
    pub index: u8, // Store the original color index
}

impl ColorInfo {
    #[must_use]
    pub const fn new(ansi: &'static str, index: u8) -> Self {
        Self { ansi, index }
    }

    #[must_use]
    pub fn indexed(index: u8) -> Self {
        Self::new(
            Box::leak(format!("\x1b[38;5;{index}m").into_boxed_str()),
            index,
        )
    }
}

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
            foreground: Some(ColorInfo {
                ansi: Box::leak(format!("\x1b[38;5;{index}m").into_boxed_str()),
                index,
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
    #[default]
    Ansi16,
    /// No color support
    None,
    /// Still to be determined or defaulted
    Undetermined,
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
    ///     ColorSupport::Ansi16,
    ///     TermTheme::Dark
    /// ));
    /// ```
    pub fn initialize(strategy: ColorInitStrategy) -> &'static Self {
        let term_attrs = INSTANCE.get_or_init(|| match strategy {
            ColorInitStrategy::Configure(support, theme) => Self::new(support, theme),
            ColorInitStrategy::Default => Self::new(ColorSupport::Ansi16, TermTheme::Dark),
            #[cfg(feature = "color_detect")]
            ColorInitStrategy::Detect => {
                let support = crate::terminal::detect_color_support().clone();
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

    /// Gets the `TermAttributes` instance or returns a default (Ansi16/Dark) instance
    pub fn get_or_default() -> &'static Self {
        INSTANCE.get_or_init(|| Self::new(ColorSupport::Ansi16, TermTheme::Dark))
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
            (ColorSupport::Ansi16, TermTheme::Light) => Self::basic_light_style(level),
            (ColorSupport::Ansi16, TermTheme::Dark) => Self::basic_dark_style(level),
            (ColorSupport::Xterm256, TermTheme::Light) => Self::full_light_style(level),
            (ColorSupport::Xterm256, TermTheme::Dark) => Self::full_dark_style(level),
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
        let term_attrs = TermAttributes::with_mock_theme(ColorSupport::Xterm256, TermTheme::Dark);
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
        let basic = TermAttributes::with_mock_theme(ColorSupport::Ansi16, TermTheme::Dark);
        let full = TermAttributes::with_mock_theme(ColorSupport::Xterm256, TermTheme::Dark);

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
        let attrs_light = TermAttributes::with_mock_theme(ColorSupport::Xterm256, TermTheme::Light);
        let attrs_dark = TermAttributes::with_mock_theme(ColorSupport::Xterm256, TermTheme::Dark);

        let heading_light = attrs_light.style_for_level(Level::Heading).paint("test");
        let heading_dark = attrs_dark.style_for_level(Level::Heading).paint("test");

        // Light and dark themes should produce different colors
        assert_ne!(heading_light, heading_dark);
    }

    #[test]
    fn test_styling_level_styling() {
        let attrs = TermAttributes::with_mock_theme(ColorSupport::Xterm256, TermTheme::Dark);

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
        let attrs = TermAttributes::with_mock_theme(ColorSupport::Xterm256, TermTheme::Dark);

        // Error should be bold
        let error_style = attrs.style_for_level(Level::Error).paint("test");
        assert!(error_style.contains("\x1b[1m"));

        // Ghost should be italic
        let ghost_style = attrs.style_for_level(Level::Ghost).paint("test");
        assert!(ghost_style.contains("\x1b[3m"));
    }
}
