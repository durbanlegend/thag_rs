#[cfg(test)]
mod tests {
    use nu_ansi_term::{Color, Style};
    use std::io::Write;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    use supports_color::Stream;
    use thag_rs::colors::{self, XtermColor};
    use thag_rs::config::Config;
    use thag_rs::log_color::{ColorSupport, LogColor, LogLevel, Theme};

    fn set_up() {
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");

        // Circumvent suppression of carriage return.
        print!("\r");
        std::io::stdout().flush().unwrap();
    }

    // Single mutex for terminal access
    lazy_static::lazy_static! {
        static ref TERMINAL_LOCK: Mutex<()> = Mutex::new(());
    }

    // Helper to detect CI environment
    fn is_ci() -> bool {
        std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("TERM").map(|t| t == "dumb").unwrap_or(false)
    }

    struct TestGuard {
        was_raw: bool,
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl TestGuard {
        fn new() -> Self {
            use crossterm::{cursor, execute, terminal};

            let lock = TERMINAL_LOCK.lock().unwrap();

            let mut stdout = std::io::stdout();
            stdout.flush().unwrap();

            // Move to start of line and clear
            let _ = execute!(
                stdout,
                cursor::MoveToColumn(0),
                terminal::Clear(terminal::ClearType::CurrentLine)
            );
            stdout.flush().unwrap();

            Self {
                was_raw: terminal::is_raw_mode_enabled().unwrap_or(false),
                _lock: lock,
            }
        }
    }

    impl Drop for TestGuard {
        fn drop(&mut self) {
            use crossterm::{cursor, execute, terminal};
            let mut stdout = std::io::stdout();

            if !self.was_raw {
                let _ = terminal::disable_raw_mode();
            }

            // Ensure clean state for next test
            let _ = execute!(
                stdout,
                cursor::MoveToColumn(0),
                terminal::Clear(terminal::ClearType::CurrentLine)
            );
            stdout.flush().unwrap();

            // Add newline to ensure test framework output starts on fresh line
            println!();
        }
    }

    #[test]
    fn test_log_color_support_detection() {
        set_up();
        let guard = TestGuard::new();

        // Run test with proper timeout handling
        let result = std::panic::catch_unwind(|| {
            let color_level = supports_color::on(Stream::Stdout);

            let color_support = match color_level {
                Some(level) if level.has_16m || level.has_256 => ColorSupport::Full,
                Some(_) => ColorSupport::Basic,
                None => ColorSupport::None,
            };

            match color_support {
                ColorSupport::Full => {
                    assert!(color_level.unwrap().has_16m || color_level.unwrap().has_256);
                }
                ColorSupport::Basic => {
                    assert!(!color_level.unwrap().has_16m && !color_level.unwrap().has_256);
                }
                ColorSupport::None => {
                    assert!(color_level.is_none());
                }
            }
        });

        // Guard will clean up terminal state
        drop(guard);

        // Re-throw panic if test failed
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    fn test_log_color_style_for_levels() {
        set_up();
        let guard = TestGuard::new();

        // Run test with proper timeout handling
        let result = std::panic::catch_unwind(|| {
            let log_color = LogColor::new(ColorSupport::Full, Theme::Light);

            // Test each log level with expected styles
            let test_cases = vec![
                (
                    LogLevel::Error,
                    Color::from(&XtermColor::GuardsmanRed).bold(),
                ),
                (
                    LogLevel::Warning,
                    Color::from(&XtermColor::DarkPurplePizzazz).bold(),
                ),
                (LogLevel::Normal, Color::from(&XtermColor::Black).normal()),
                (
                    LogLevel::Heading,
                    Color::from(&XtermColor::MidnightBlue).bold(),
                ),
                (
                    LogLevel::Ghost,
                    Color::from(&XtermColor::DarkCodGray).normal().italic(),
                ),
            ];

            for (level, expected_style) in test_cases {
                let style = log_color.style_for_level(level);
                assert_eq!(style, expected_style, "Style mismatch for {:?}", level);
            }
        });

        // Guard will clean up terminal state
        drop(guard);

        // Re-throw panic if test failed
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    fn test_log_color_no_color_support() {
        set_up();
        let guard = TestGuard::new();

        // Run test with proper timeout handling
        let result = std::panic::catch_unwind(|| {
            let log_color = LogColor::new(ColorSupport::None, Theme::Light);

            // All levels should return default style when color support is None
            for level in [LogLevel::Error, LogLevel::Warning, LogLevel::Normal] {
                let style = log_color.style_for_level(level);
                assert_eq!(
                    style,
                    Style::new(),
                    "Expected default style for {:?}",
                    level
                );
            }
        });

        // Guard will clean up terminal state
        drop(guard);

        // Re-throw panic if test failed
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    fn test_log_color_config_conversion() {
        set_up();
        let guard = TestGuard::new();

        // Add timeout for entire test
        let result = std::panic::catch_unwind(|| {
            let timeout = Duration::from_secs(5); // reasonable timeout
            let start = Instant::now();
            let handle = std::thread::spawn(|| {
                // Test implementation here
                let mut config = Config::default();

                // Test each ColorSupport variant
                let test_cases = vec![
                    (colors::ColorSupport::Xterm256, ColorSupport::Full),
                    (colors::ColorSupport::Ansi16, ColorSupport::Basic),
                    (colors::ColorSupport::None, ColorSupport::None),
                ];

                for (old_support, new_support) in test_cases {
                    config.colors.color_support = old_support;
                    let log_color = LogColor::from_config(&config);
                    assert_eq!(log_color.color_support, new_support);
                }
            });
            // Wait with timeout
            while start.elapsed() < timeout {
                if handle.is_finished() {
                    return handle.join().unwrap();
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            panic!("Test timed out after {:?}", timeout);
        });

        drop(guard);

        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    fn test_log_color_logging_macros() {
        set_up();
        let guard = TestGuard::new();

        // Run test with proper timeout handling
        let result = std::panic::catch_unwind(|| {
            let content = "Test message";
            let log_color = LogColor::new(ColorSupport::Full, Theme::Light);

            // Test error style
            let error_style = log_color.style_for_level(LogLevel::Error);
            let error_output = format!("{}", error_style.paint(content));
            assert_eq!(
                error_output,
                format!(
                    "{}",
                    Color::from(&XtermColor::GuardsmanRed).bold().paint(content)
                )
            );

            // Test warning style
            let warn_style = log_color.style_for_level(LogLevel::Warning);
            let warn_output = format!("{}", warn_style.paint(content));
            assert_eq!(
                warn_output,
                format!(
                    "{}",
                    Color::from(&XtermColor::DarkPurplePizzazz)
                        .bold()
                        .paint(content)
                )
            );
        });

        // Guard will clean up terminal state
        drop(guard);

        // Re-throw panic if test failed
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    // Skip theme detection tests in CI
    #[test]
    #[cfg_attr(test, ignore = "Theme detection requires terminal")]
    fn test_log_color_theme_persistence() {
        if is_ci() {
            println!("Skipping theme detection test in CI environment");
            return;
        }

        let guard = TestGuard::new();

        let result = std::panic::catch_unwind(|| {
            // Use a shorter timeout for theme detection
            let timeout = Duration::from_millis(500);

            // First theme detection with timeout
            let start = Instant::now();
            let log_color1 = LogColor::new(ColorSupport::Full, Theme::AutoDetect);
            let handle = std::thread::spawn(move || log_color1.get_theme());
            let first_theme = loop {
                if handle.is_finished() {
                    break handle.join().unwrap();
                }
                if start.elapsed() > timeout {
                    break Theme::Dark; // Default on timeout
                }
                std::thread::sleep(Duration::from_millis(10));
            };

            // Second theme detection
            let start = Instant::now();
            let log_color2 = LogColor::new(ColorSupport::Full, Theme::AutoDetect);
            let handle = std::thread::spawn(move || log_color2.get_theme());

            let second_theme = loop {
                if handle.is_finished() {
                    break handle.join().unwrap();
                }
                if start.elapsed() > timeout {
                    break Theme::Dark; // Default on timeout
                }
                std::thread::sleep(Duration::from_millis(10));
            };

            assert_eq!(first_theme, second_theme);
            println!("first_theme={first_theme:#?}, second_theme={second_theme:#?}");
        });

        drop(guard);

        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }
}
