use crate::color_support::{
    get_color_level, resolve_term_theme, restore_raw_status, ColorSupport, TermTheme,
};
use crate::shared::{Color, Style};
use crate::{
    config, debug_log, lazy_static_var, maybe_config, profile, profile_method, Colors, Config,
    Level,
};
use crossterm::terminal::{self, is_raw_mode_enabled};
use scopeguard::defer;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use supports_color::Stream;
use termbg::Theme as TermbgTheme;

pub struct LogColor {
    pub color_support: ColorSupport,
    pub theme: TermTheme,
    pub detected_theme: AtomicU8, // 0 = undetected, 1 = light, 2 = dark
}

impl LogColor {
    #[must_use]
    pub fn new(color_support: ColorSupport, theme: TermTheme) -> Self {
        profile_method!("LogColor::new");
        Self {
            color_support,
            theme,
            detected_theme: AtomicU8::new(0),
        }
    }

    pub fn get_theme(&self) -> TermTheme {
        profile_method!("LogColor::get_theme");
        if self.theme != TermTheme::AutoDetect {
            return self.theme.clone();
        }

        // Check cache first
        let detected = self.detected_theme.load(Ordering::Relaxed);
        if detected != 0 {
            // Return cached result
            return if detected == 1 {
                TermTheme::Light
            } else {
                TermTheme::Dark
            };
        }

        // Only try detection once
        let theme = Self::detect_terminal_theme();
        self.detected_theme.store(
            match theme {
                TermTheme::Light => 1,
                TermTheme::Dark | TermTheme::AutoDetect => 2,
            },
            Ordering::Relaxed,
        );
        theme
    }

    fn detect_terminal_theme() -> TermTheme {
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
                TermTheme::Dark
            }
        }
    }

    fn detect_theme_internal() -> Result<TermTheme, termbg::Error> {
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
            TermbgTheme::Light => TermTheme::Light,
            TermbgTheme::Dark => TermTheme::Dark,
        })
    }

    pub fn style_for_level(&self, level: Level) -> Style {
        profile_method!("LogColor::style_for_level");
        match (&self.color_support, &self.get_theme()) {
            (&ColorSupport::None, _) => Style::default(),
            (&ColorSupport::Ansi16, &TermTheme::Light) => Self::basic_light_style(level),
            (&ColorSupport::Ansi16, &TermTheme::Dark) => Self::basic_dark_style(level),
            (&ColorSupport::Xterm256, &TermTheme::Light) => Self::full_light_style(level),
            (&ColorSupport::Xterm256, &TermTheme::Dark) => Self::full_dark_style(level),
            (_, &TermTheme::AutoDetect) | (&ColorSupport::AutoDetect, _) => unreachable!(),
        }
    }

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

    fn basic_dark_style(level: Level) -> Style {
        profile!("basic_dark_style");
        match level {
            Level::Error => Color::red().bold(),
            Level::Warning => Color::yellow().bold(),
            Level::Heading => Color::green().bold(),
            Level::Subheading => Color::blue().bold(),
            Level::Emphasis => Color::cyan().bold(),
            Level::Bright => Color::light_yellow(),
            Level::Normal => Color::white().normal(),
            Level::Debug => Color::light_cyan().normal(),
            Level::Ghost => Color::light_gray().italic(),
        }
    }

    fn full_light_style(level: Level) -> Style {
        match level {
            Level::Error => Color::fixed(160).bold(),
            Level::Warning => Color::fixed(164).bold(),
            Level::Heading => Color::fixed(19).bold(),
            Level::Subheading => Color::fixed(26),
            Level::Emphasis => Color::fixed(173).bold(),
            Level::Bright => Color::fixed(46),
            Level::Normal => Color::fixed(16),
            Level::Debug => Color::fixed(32),
            Level::Ghost => Color::fixed(232).italic(),
        }
    }

    fn full_dark_style(level: Level) -> Style {
        profile!("full_dark_style");
        match level {
            Level::Error => Color::fixed(1).bold(),
            Level::Warning => Color::fixed(171).bold(),
            Level::Heading => Color::fixed(42).bold(),
            Level::Subheading => Color::fixed(75).normal(),
            Level::Emphasis => Color::fixed(173).bold(),
            Level::Bright => Color::fixed(3),
            Level::Normal => Color::fixed(231).normal(),
            Level::Debug => Color::fixed(37).normal(),
            Level::Ghost => Color::fixed(251).normal().italic(),
        }
    }

    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let (color_support, theme) = config.colors.clone().into();
        Self::new(color_support, theme)
    }
}

impl From<config::Colors> for (ColorSupport, TermTheme) {
    fn from(colors: Colors) -> Self {
        let color_support = match colors.color_support {
            ColorSupport::Xterm256 => ColorSupport::Xterm256,
            ColorSupport::Ansi16 => ColorSupport::Ansi16,
            ColorSupport::None => ColorSupport::None,
            ColorSupport::AutoDetect => {
                supports_color::on(Stream::Stdout).map_or(ColorSupport::None, |color_level| {
                    if color_level.has_16m || color_level.has_256 {
                        ColorSupport::Xterm256
                    } else {
                        ColorSupport::Ansi16
                    }
                })
            }
        };

        (color_support, colors.term_theme)
    }
}

// Global instance
use std::sync::OnceLock;
static INSTANCE: OnceLock<LogColor> = OnceLock::new();

pub static LOGGING_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn disable_logging() {
    LOGGING_ENABLED.store(false, Ordering::SeqCst);
}

pub fn enable_logging() {
    LOGGING_ENABLED.store(true, Ordering::SeqCst);
}

pub fn init(color_support: ColorSupport, theme: TermTheme) {
    let _ = INSTANCE.set(LogColor::new(color_support, theme));
}

pub fn get() -> &'static LogColor {
    INSTANCE.get_or_init(|| LogColor::new(ColorSupport::None, TermTheme::Dark))
}

pub fn initialize() -> &'static LogColor {
    profile!("initialize");

    if std::env::var("TEST_ENV").is_ok() {
        #[cfg(debug_assertions)]
        debug_log!("Avoiding supports_color for testing");
        return lazy_static_var!(
            LogColor,
            LogColor::new(ColorSupport::Ansi16, TermTheme::Dark)
        );
    }

    let raw_before = terminal::is_raw_mode_enabled();
    if let Ok(raw_then) = raw_before {
        defer! {
            let raw_now = match is_raw_mode_enabled() {
                Ok(val) => val,
                Err(e) => {
                    #[cfg(debug_assertions)]
                    debug_log!("Failed to check raw mode status: {:?}", e);
                    return;
                }
            };

            if raw_now == raw_then {
                #[cfg(debug_assertions)]
                debug_log!("Raw mode status unchanged.");
            } else if let Err(e) = restore_raw_status(raw_then) {
                    #[cfg(debug_assertions)]
                    debug_log!("Failed to restore raw mode: {:?}", e);
            } else {
                #[cfg(debug_assertions)]
                debug_log!("Raw mode restored to previous state.");
            }
        }
    }

    lazy_static_var!(LogColor, {
        let color_support = maybe_config()
            .as_ref()
            .map_or_else(get_color_level, |config| {
                match config.colors.color_support {
                    ColorSupport::Xterm256 | ColorSupport::Ansi16 | ColorSupport::None => {
                        Some(config.colors.color_support.clone())
                    }
                    ColorSupport::AutoDetect => {
                        Some(get_color_level().unwrap_or(ColorSupport::None))
                    }
                }
            })
            .unwrap_or(ColorSupport::None);

        let term_theme = maybe_config().map_or_else(
            || resolve_term_theme().unwrap_or_default(),
            |config| {
                if matches!(&config.colors.term_theme, &TermTheme::AutoDetect) {
                    resolve_term_theme().unwrap_or_default()
                } else {
                    config.colors.term_theme
                }
            },
        );

        LogColor::new(color_support, term_theme)
    })
}

// Convenience macros

#[macro_export]
macro_rules! clog {  // 'c' for colored logging
    ($level:expr, $($arg:tt)*) => {{
        if $crate::log_color::LOGGING_ENABLED.load(std::sync::atomic::Ordering::SeqCst) {
            let logger = $crate::log_color::get();
            let style = logger.style_for_level($level);
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
        if $crate::log_color::LOGGING_ENABLED.load(std::sync::atomic::Ordering::SeqCst) {
            let logger = $crate::logging::LOGGER.lock().unwrap();
            let message = format!($($arg)*);

            #[cfg(feature = "color_support")]
            {
                let color_logger = $crate::log_color::get();
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

    impl LogColor {
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
    fn test_log_color_theme_detection_with_mock() {
        init_test();
        let log_color = LogColor::with_mock_theme(ColorSupport::Xterm256, TermTheme::Dark);
        let detected = log_color.get_theme();
        assert_eq!(detected, TermTheme::Dark);
        println!();
        flush_test_output();
    }
}
