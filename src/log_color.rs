// use crate::{debug_log, profile, ThagResult};
use crate::{colors, config, profile, profile_method, Config, TermTheme};
use crossterm::terminal;
// use nu_ansi_term::{Color, Style};
use std::io::{self, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use supports_color::Stream;
use termbg::Theme as TermbgTheme;

#[derive(Debug, Clone)]
pub struct Style {
    foreground: Option<&'static str>,
    bold: bool,
    italic: bool,
}

impl Style {
    #[must_use]
    pub fn new() -> Self {
        Self {
            foreground: None,
            bold: false,
            italic: false,
        }
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn normal(self) -> Self {
        self
    }

    pub fn paint<D>(&self, val: D) -> String
    where
        D: std::fmt::Display,
    {
        let mut result = String::new();

        if let Some(fg) = self.foreground {
            result.push_str(fg);
        }
        if self.bold {
            result.push_str("\x1b[1m");
        }
        if self.italic {
            result.push_str("\x1b[3m");
        }

        result.push_str(&val.to_string());
        result.push_str("\x1b[0m");
        result
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

    pub fn red() -> Style {
        Style {
            foreground: Some(Self::RED),
            bold: false,
            italic: false,
        }
    }

    pub fn green() -> Style {
        Style {
            foreground: Some(Self::GREEN),
            bold: false,
            italic: false,
        }
    }

    pub fn yellow() -> Style {
        Style {
            foreground: Some(Self::YELLOW),
            bold: false,
            italic: false,
        }
    }

    pub fn blue() -> Style {
        Style {
            foreground: Some(Self::BLUE),
            bold: false,
            italic: false,
        }
    }

    pub fn magenta() -> Style {
        Style {
            foreground: Some(Self::MAGENTA),
            bold: false,
            italic: false,
        }
    }

    pub fn cyan() -> Style {
        Style {
            foreground: Some(Self::CYAN),
            bold: false,
            italic: false,
        }
    }

    pub fn white() -> Style {
        Style {
            foreground: Some(Self::WHITE),
            bold: false,
            italic: false,
        }
    }

    pub fn dark_gray() -> Style {
        Style {
            foreground: Some(Self::DARK_GRAY),
            bold: false,
            italic: false,
        }
    }

    pub fn light_yellow() -> Style {
        Style {
            foreground: Some(Self::LIGHT_YELLOW),
            bold: false,
            italic: false,
        }
    }

    pub fn light_cyan() -> Style {
        Style {
            foreground: Some(Self::LIGHT_CYAN),
            bold: false,
            italic: false,
        }
    }

    pub fn light_gray() -> Style {
        Style {
            foreground: Some(Self::LIGHT_GRAY),
            bold: false,
            italic: false,
        }
    }

    pub fn fixed(code: u8) -> Style {
        Style {
            foreground: Some(Box::leak(format!("\x1b[38;5;{}m", code).into_boxed_str())),
            bold: false,
            italic: false,
        }
    }
}

// impl From<Style> for Style {
//     fn from(style: Style) -> Self {
//         style
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Error,
    Warning,
    Heading,
    Subheading,
    Emphasis,
    Bright,
    Normal,
    Debug,
    Ghost,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSupport {
    None,
    Basic, // 16 colors
    Full,  // 256 colors
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    AutoDetect,
}

pub struct LogColor {
    pub color_support: ColorSupport,
    pub theme: Theme,
    pub detected_theme: AtomicU8, // 0 = undetected, 1 = light, 2 = dark
}

impl LogColor {
    #[must_use]
    pub fn new(color_support: ColorSupport, theme: Theme) -> Self {
        profile_method!("LogColor::new");
        Self {
            color_support,
            theme,
            detected_theme: AtomicU8::new(0),
        }
    }

    pub fn get_theme(&self) -> Theme {
        profile_method!("LogColor::get_theme");
        if self.theme != Theme::AutoDetect {
            return self.theme;
        }

        // Check cache first
        let detected = self.detected_theme.load(Ordering::Relaxed);
        if detected != 0 {
            // Return cached result
            return if detected == 1 {
                Theme::Light
            } else {
                Theme::Dark
            };
        }

        // Only try detection once
        let theme = Self::detect_terminal_theme();
        self.detected_theme.store(
            match theme {
                Theme::Light => 1,
                Theme::Dark | Theme::AutoDetect => 2,
            },
            Ordering::Relaxed,
        );
        theme
    }

    fn detect_terminal_theme() -> Theme {
        profile!("detect_terminal_theme");

        // Try to detect theme, handle errors gracefully
        match Self::detect_theme_internal() {
            Ok(theme) => theme,
            Err(e) => {
                // 1. Try to restore terminal to known good state
                let _ = terminal::disable_raw_mode();

                // 2. Warn user about potential issues
                let warning = format!(
                    "\x1b[31mWarning: Terminal theme detection failed: {e}\n\
                     Falling back to dark theme. If terminal appears corrupted,\n\
                     try running 'reset' command or closing and reopening terminal.\x1b[0m\n"
                );
                let _ = io::stderr().write_all(warning.as_bytes());
                let _ = io::stderr().flush();

                // 3. Fall back to safe default
                Theme::Dark
            }
        }
    }

    fn detect_theme_internal() -> Result<Theme, termbg::Error> {
        // Create cleanup guard
        struct RawModeGuard(bool);
        impl Drop for RawModeGuard {
            fn drop(&mut self) {
                if !self.0 {
                    let _ = terminal::disable_raw_mode();
                }
            }
        }

        // Save initial state
        let raw_before = terminal::is_raw_mode_enabled()?;

        // Ensure raw mode for detection
        if !raw_before {
            terminal::enable_raw_mode()?;
        }

        let _guard = RawModeGuard(raw_before);

        // Now do theme detection
        let timeout = std::time::Duration::from_millis(1000);
        let theme = termbg::theme(timeout)?;

        Ok(match theme {
            TermbgTheme::Light => Theme::Light,
            TermbgTheme::Dark => Theme::Dark,
        })
    }

    pub fn style_for_level(&self, level: LogLevel) -> Style {
        profile_method!("LogColor::style_for_level");
        match (self.color_support, self.get_theme()) {
            (ColorSupport::None, _) => Style::new(),
            (ColorSupport::Basic, Theme::Light) => Self::basic_light_style(level),
            (ColorSupport::Basic, Theme::Dark) => Self::basic_dark_style(level),
            (ColorSupport::Full, Theme::Light) => Self::full_light_style(level),
            (ColorSupport::Full, Theme::Dark) => Self::full_dark_style(level),
            (_, Theme::AutoDetect) => unreachable!(), // Handled by get_theme
        }
    }

    fn basic_light_style(level: LogLevel) -> Style {
        // Port existing Ansi16LightStyle logic
        profile!("basic_light_style");
        match level {
            LogLevel::Error => Color::red().bold(),
            LogLevel::Warning => Color::magenta().bold(),
            LogLevel::Heading => Color::blue().bold(),
            LogLevel::Subheading => Color::cyan().bold(),
            LogLevel::Emphasis => Color::green().bold(),
            LogLevel::Bright => Color::green().into(),
            LogLevel::Normal => Color::dark_gray().normal(),
            LogLevel::Debug => Color::cyan().normal(),
            LogLevel::Ghost => Color::cyan().italic(),
        }
    }

    fn basic_dark_style(level: LogLevel) -> Style {
        profile!("basic_dark_style");
        match level {
            LogLevel::Error => Color::red().bold(),
            LogLevel::Warning => Color::yellow().bold(),
            LogLevel::Heading => Color::green().bold(),
            LogLevel::Subheading => Color::blue().bold(),
            LogLevel::Emphasis => Color::cyan().bold(),
            LogLevel::Bright => Color::light_yellow().into(),
            LogLevel::Normal => Color::white().normal(),
            LogLevel::Debug => Color::light_cyan().normal(),
            LogLevel::Ghost => Color::light_gray().italic(),
        }
    }

    fn full_light_style(level: LogLevel) -> Style {
        profile!("full_light_style");
        match level {
            LogLevel::Error => Color::fixed(160).bold(),
            LogLevel::Warning => Color::fixed(164).bold(),
            LogLevel::Heading => Color::fixed(19).bold(),
            LogLevel::Subheading => Color::fixed(26).normal(),
            LogLevel::Emphasis => Color::fixed(173).bold(),
            LogLevel::Bright => Color::fixed(46).into(),
            LogLevel::Normal => Color::fixed(16).normal(),
            LogLevel::Debug => Color::fixed(32).normal(),
            LogLevel::Ghost => Color::fixed(232).normal().italic(),
        }
    }

    fn full_dark_style(level: LogLevel) -> Style {
        profile!("full_dark_style");
        match level {
            LogLevel::Error => Color::fixed(1).bold(),
            LogLevel::Warning => Color::fixed(171).bold(),
            LogLevel::Heading => Color::fixed(42).bold(),
            LogLevel::Subheading => Color::fixed(75).normal(),
            LogLevel::Emphasis => Color::fixed(173).bold(),
            LogLevel::Bright => Color::fixed(3).into(),
            LogLevel::Normal => Color::fixed(231).normal(),
            LogLevel::Debug => Color::fixed(37).normal(),
            LogLevel::Ghost => Color::fixed(251).normal().italic(),
        }
    }
}

impl From<config::Colors> for (ColorSupport, Theme) {
    fn from(colors: config::Colors) -> Self {
        let color_support = match colors.color_support {
            colors::ColorSupport::Xterm256 => ColorSupport::Full,
            colors::ColorSupport::Ansi16 => ColorSupport::Basic,
            colors::ColorSupport::None => ColorSupport::None,
            colors::ColorSupport::AutoDetect => {
                // Port existing auto-detection logic
                if let Some(color_level) = supports_color::on(Stream::Stdout) {
                    if color_level.has_16m || color_level.has_256 {
                        ColorSupport::Full
                    } else {
                        ColorSupport::Basic
                    }
                } else {
                    ColorSupport::None
                }
            }
        };

        let theme = match colors.term_theme {
            TermTheme::Light => Theme::Light,
            TermTheme::Dark => Theme::Dark,
            TermTheme::AutoDetect => Theme::AutoDetect,
        };

        (color_support, theme)
    }
}

impl LogColor {
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let (color_support, theme) = config.colors.clone().into();
        Self::new(color_support, theme)
    }
}

// Global instance
use std::sync::OnceLock;
static INSTANCE: OnceLock<LogColor> = OnceLock::new();

pub fn init(color_support: ColorSupport, theme: Theme) {
    let _ = INSTANCE.set(LogColor::new(color_support, theme));
}

pub fn get() -> &'static LogColor {
    INSTANCE.get_or_init(|| LogColor::new(ColorSupport::None, Theme::Dark))
}

// Convenience macros
#[macro_export]
macro_rules! clog {  // 'c' for colored logging
    ($level:expr, $($arg:tt)*) => {{
        let logger = get_logger();
        let style = logger.style_for_level($level);
        println!("{}", style.paint(format!($($arg)*)));
    }};
}

#[macro_export]
macro_rules! clog_error {
    ($($arg:tt)*) => { log!(LogLevel::Error, $($arg)*) };
}

#[macro_export]
macro_rules! clog_warning {
        ($($arg:tt)*) => { log!(LogLevel::Warning, $($arg)*) };
    }

#[macro_export]
macro_rules! clog_heading {
    ($($arg:tt)*) => { log!(LogLevel::Heading, $($arg)*) };
}

#[macro_export]
macro_rules! clog_subheading {
    ($($arg:tt)*) => { log!(LogLevel::Subheading, $($arg)*) };
}

#[macro_export]
macro_rules! clog_emphasis {
    ($($arg:tt)*) => { log!(LogLevel::Emphasis, $($arg)*) };
}

#[macro_export]
macro_rules! clog_bright {
    ($($arg:tt)*) => { log!(LogLevel::Bright, $($arg)*) };
}

#[macro_export]
macro_rules! clog_normal {
    ($($arg:tt)*) => { log!(LogLevel::Normal, $($arg)*) };
}

#[macro_export]
macro_rules! clog_debug {
    ($($arg:tt)*) => { log!(LogLevel::Debug, $($arg)*) };
}

#[macro_export]
macro_rules! clog_ghost {
    ($($arg:tt)*) => { log!(LogLevel::Ghost, $($arg)*) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    static MOCK_THEME_DETECTION: AtomicBool = AtomicBool::new(false);

    impl LogColor {
        fn with_mock_theme(color_support: ColorSupport, theme: Theme) -> Self {
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
    fn test_log_color_theme_detection_with_mock() {
        init_test();
        let log_color = LogColor::with_mock_theme(ColorSupport::Full, Theme::Dark);
        let detected = log_color.get_theme();
        assert_eq!(detected, Theme::Dark);
        println!();
        flush_test_output();
    }
}
