// use crate::{debug_log, profile, ThagResult};
use crate::{colors, config, profile, profile_method, Config, TermTheme};
use crossterm::terminal;
use nu_ansi_term::{Color, Style};
use std::io::{self, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use supports_color::Stream;
use termbg::Theme as TermbgTheme;

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
            LogLevel::Error => Color::Red.bold(),
            LogLevel::Warning => Color::Magenta.bold(),
            LogLevel::Heading => Color::Blue.bold(),
            LogLevel::Subheading => Color::Cyan.bold(),
            LogLevel::Emphasis => Color::Green.bold(),
            LogLevel::Bright => Color::Green.into(), // .bold(),
            LogLevel::Normal => Color::DarkGray.normal(),
            LogLevel::Debug => Color::Cyan.normal(),
            LogLevel::Ghost => Color::Cyan.italic(),
        }
    }

    fn basic_dark_style(level: LogLevel) -> Style {
        // Port existing Ansi16DarkStyle logic
        profile!("basic_dark_style");
        match level {
            LogLevel::Error => Color::Red.bold(),
            LogLevel::Warning => Color::Yellow.bold(),
            LogLevel::Heading => Color::Green.bold(),
            LogLevel::Subheading => Color::Blue.bold(),
            LogLevel::Emphasis => Color::Cyan.bold(),
            LogLevel::Bright => Color::LightYellow.into(), // .bold(),
            LogLevel::Normal => Color::White.normal(),
            LogLevel::Debug => Color::LightCyan.normal(),
            LogLevel::Ghost => Color::LightGray.italic(),
        }
    }

    fn full_light_style(level: LogLevel) -> Style {
        // Port existing Xterm256LightStyle logic
        profile!("full_light_style");
        match level {
            // GuardsmanRed
            LogLevel::Error => Color::Fixed(160).bold(),
            // DarkPurplePizzazz
            LogLevel::Warning => Color::Fixed(164).bold(),
            // MidnightBlue
            LogLevel::Heading => Color::Fixed(19).bold(),
            // ScienceBlue
            LogLevel::Subheading => Color::Fixed(26).normal(),
            // Copperfield
            LogLevel::Emphasis => Color::Fixed(173).bold(),
            // Green
            LogLevel::Bright => Color::Fixed(46).into(),
            // Black
            LogLevel::Normal => Color::Fixed(16).normal(),
            // LochmaraBlue
            LogLevel::Debug => Color::Fixed(32).normal(),
            // DarkCodGray
            LogLevel::Ghost => Color::Fixed(232).normal().italic(),
        }
    }

    fn full_dark_style(level: LogLevel) -> Style {
        // Port existing Xterm256DarkStyle logic
        profile!("full_dark_style");
        match level {
            // UserRed
            LogLevel::Error => Color::Fixed(1).bold(),
            // LighterHeliotrope
            LogLevel::Warning => Color::Fixed(171).bold(),
            // CaribbeanGreen
            LogLevel::Heading => Color::Fixed(42).bold(),
            // DarkMalibuBlue
            LogLevel::Subheading => Color::Fixed(75).normal(),
            // Copperfield
            LogLevel::Emphasis => Color::Fixed(173).bold(),
            // UserYellow
            LogLevel::Bright => Color::Fixed(3).into(),
            // White
            LogLevel::Normal => Color::Fixed(231).normal(),
            // BondiBlue
            LogLevel::Debug => Color::Fixed(37).normal(),
            // Silver
            LogLevel::Ghost => Color::Fixed(251).normal().italic(),
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
