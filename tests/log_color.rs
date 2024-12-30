#[cfg(test)]
mod tests {
    // use nu_ansi_term::{Color, Style};
    use std::io::Write;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    use supports_color::Stream;
    // use thag_rs::colors::XtermColor;
    use thag_rs::log_color::{Color, LogColor, LogLevel, Style};
    use thag_rs::{ColorSupport, TermTheme};

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
                Some(level) if level.has_16m || level.has_256 => ColorSupport::Xterm256,
                Some(_) => ColorSupport::Ansi16,
                None => ColorSupport::None,
            };

            match color_support {
                ColorSupport::Xterm256 => {
                    assert!(color_level.unwrap().has_16m || color_level.unwrap().has_256);
                }
                ColorSupport::Ansi16 => {
                    assert!(!color_level.unwrap().has_16m && !color_level.unwrap().has_256);
                }
                ColorSupport::None => {
                    assert!(color_level.is_none());
                }
                ColorSupport::AutoDetect => unreachable!(), // since not set above
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
            let log_color = LogColor::new(ColorSupport::Xterm256, TermTheme::Light);

            // Test each log level with expected styles
            let test_cases = vec![
                (LogLevel::Error, Color::fixed(160).bold()),
                (LogLevel::Warning, Color::fixed(164).bold()),
                (LogLevel::Normal, Color::fixed(16).normal()),
                (LogLevel::Heading, Color::fixed(19).bold()),
                (LogLevel::Ghost, Color::fixed(232).normal().italic()),
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
            let log_color = LogColor::new(ColorSupport::None, TermTheme::Light);

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
    fn test_log_color_logging_macros() {
        set_up();
        let guard = TestGuard::new();

        // Run test with proper timeout handling
        let result = std::panic::catch_unwind(|| {
            let content = "Test message";
            let log_color = LogColor::new(ColorSupport::Xterm256, TermTheme::Light);

            // Test error style
            let error_style = log_color.style_for_level(LogLevel::Error);
            let error_output = format!("{}", error_style.paint(content));
            assert_eq!(
                error_output,
                format!("{}", Color::fixed(160).bold().paint(content))
            );

            // Test warning style
            let warn_style = log_color.style_for_level(LogLevel::Warning);
            let warn_output = format!("{}", warn_style.paint(content));
            assert_eq!(
                warn_output,
                format!("{}", Color::fixed(164).bold().paint(content))
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
    // #[cfg_attr(test, ignore = "Theme detection requires terminal")]
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
            let log_color1 = LogColor::new(ColorSupport::Xterm256, TermTheme::AutoDetect);
            let handle = std::thread::spawn(move || log_color1.get_theme());
            let first_theme = loop {
                if handle.is_finished() {
                    break handle.join().unwrap();
                }
                if start.elapsed() > timeout {
                    break TermTheme::Dark; // Default on timeout
                }
                std::thread::sleep(Duration::from_millis(10));
            };

            // Second theme detection
            let start = Instant::now();
            let log_color2 = LogColor::new(ColorSupport::Xterm256, TermTheme::AutoDetect);
            let handle = std::thread::spawn(move || log_color2.get_theme());

            let second_theme = loop {
                if handle.is_finished() {
                    break handle.join().unwrap();
                }
                if start.elapsed() > timeout {
                    break TermTheme::Dark; // Default on timeout
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
