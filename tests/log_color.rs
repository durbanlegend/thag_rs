#[cfg(test)]
pub mod test_utils {
    use std::cell::RefCell;
    use std::io::Write;

    thread_local! {
        static TEST_OUTPUT: RefCell<Vec<String>> = RefCell::new(Vec::new());
    }

    pub fn init_test() {
        TEST_OUTPUT.with(|output| {
            output.borrow_mut().push(String::new());
        });
    }

    pub fn write_test_output(text: &str) {
        TEST_OUTPUT.with(|output| {
            output.borrow_mut().last_mut().unwrap().push_str(text);
        });
    }

    pub fn flush_test_output() {
        TEST_OUTPUT.with(|output| {
            let mut stdout = std::io::stdout();
            for line in output.borrow().iter() {
                writeln!(stdout, "{}", line).unwrap();
            }
            output.borrow_mut().clear();
        });
    }

    // // Optional: implement Drop for automatic flushing
    // pub struct TestGuard;

    // impl Drop for TestGuard {
    //     fn drop(&mut self) {
    //         flush_test_output();
    //     }
    // }
}

#[cfg(test)]
mod tests {
    // use crate::test_utils::{init_test, write_test_output, TestGuard};
    use nu_ansi_term::{Color, Style};
    use std::io::Write;
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

    // struct TestGuard;

    // impl Drop for TestGuard {
    //     fn drop(&mut self) {
    //         // Reset terminal state
    //         let _ = crossterm::terminal::disable_raw_mode();
    //         let _ = crossterm::terminal::LeaveAlternateScreen;
    //         let _ = std::io::stdout().flush();
    //     }
    // }

    struct TestGuard {
        was_alternate: bool,
        was_raw: bool,
    }

    impl TestGuard {
        fn new() -> Self {
            use crossterm::{cursor, execute, terminal};
            let mut stdout = std::io::stdout();

            // Force main screen and column 0 at start
            let _ = execute!(
                stdout,
                terminal::LeaveAlternateScreen,
                cursor::MoveToColumn(0),
                terminal::Clear(terminal::ClearType::CurrentLine)
            );
            let _ = stdout.flush();

            let was_raw = terminal::is_raw_mode_enabled().unwrap_or(false);
            Self {
                was_alternate: false,
                was_raw,
            }
        }
    }

    impl Drop for TestGuard {
        fn drop(&mut self) {
            use crossterm::{cursor, execute, terminal};
            let mut stdout = std::io::stdout();

            // Sequence cleanup operations with flushing
            let _ = execute!(stdout, terminal::LeaveAlternateScreen);
            let _ = stdout.flush();

            if !self.was_raw {
                let _ = terminal::disable_raw_mode();
            }
            let _ = stdout.flush();

            let _ = execute!(
                stdout,
                cursor::MoveToColumn(0),
                terminal::Clear(terminal::ClearType::CurrentLine)
            );
            let _ = stdout.flush();
        }
    }

    // use std::sync::Mutex;

    // lazy_static::lazy_static! {
    //     static ref TEST_OUTPUT_LOCK: Mutex<()> = Mutex::new(());
    // }

    // fn init_test() {
    //     let _lock = TEST_OUTPUT_LOCK.lock().unwrap();
    //     // Ensure we're writing to a clean line
    //     print!("\r\x1B[2K"); // CR + clear line
    //     std::io::stdout().flush().unwrap();
    // }

    // fn init_test() {
    //     // print!("\x1B[1G\x1B[K");
    //     // std::io::stdout().flush().unwrap();
    //     use crossterm::cursor;
    //     use crossterm::terminal;
    //     use crossterm::ExecutableCommand;
    //     use std::io::stdout;
    //     // let is_raw = terminal::is_raw_mode_enabled();
    //     // println!("is_raw={is_raw:#?}");
    //     // let mut stdout = stdout();
    //     // let _ = stdout.execute(cursor::MoveToColumn(0));
    //     // let _ = stdout.execute(terminal::Clear(terminal::ClearType::CurrentLine));
    //     let raw_mode = terminal::is_raw_mode_enabled().unwrap_or(false);
    //     eprintln!("Test starting. Raw mode: {}", raw_mode);

    //     // Try both approaches
    //     print!("\r");
    //     std::io::stdout().flush().unwrap();

    //     let mut stdout = stdout();
    //     let _ = stdout.execute(cursor::MoveToColumn(0));
    //     let _ = stdout.execute(terminal::Clear(terminal::ClearType::CurrentLine));

    //     eprintln!("After cursor moves");
    // }

    // Add this attribute to see if tests are being discovered
    // #[test]
    // #[ignore]
    // fn test_log_color_debug_test_discovery() {
    //     println!("Test module is being discovered");
    //     assert!(true);
    // }

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
            let timeout = std::time::Duration::from_millis(1000);
            let _theme = termbg::theme(timeout);
            // ...
        });

        // Guard will clean up terminal state
        drop(guard);

        // Re-throw panic if test failed
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    // #[test]
    // fn test_log_color_theme_detection() {
    //     set_up();
    //     let log_color = LogColor::new(ColorSupport::Full, Theme::AutoDetect);
    //     let detected = log_color.get_theme();
    //     assert!(matches!(detected, Theme::Light | Theme::Dark));
    // }

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
            let timeout = std::time::Duration::from_millis(1000);
            let _theme = termbg::theme(timeout);
            // ...
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
            let timeout = std::time::Duration::from_millis(1000);
            let _theme = termbg::theme(timeout);
            // ...
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

        // Run test with proper timeout handling
        let result = std::panic::catch_unwind(|| {
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
            let timeout = std::time::Duration::from_millis(1000);
            let _theme = termbg::theme(timeout);
            // ...
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
            let timeout = std::time::Duration::from_millis(1000);
            let _theme = termbg::theme(timeout);
            // ...
        });

        // Guard will clean up terminal state
        drop(guard);

        // Re-throw panic if test failed
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    fn test_log_color_theme_persistence() {
        set_up();
        let guard = TestGuard::new();

        // Run test with proper timeout handling
        let result = std::panic::catch_unwind(|| {
            let log_color = LogColor::new(ColorSupport::Full, Theme::AutoDetect);

            // First detection should persist
            let first_theme = log_color.get_theme();
            let second_theme = log_color.get_theme();
            assert_eq!(
                first_theme, second_theme,
                "Theme should persist after first detection"
            );
            let timeout = std::time::Duration::from_millis(1000);
            let _theme = termbg::theme(timeout);
            // ...
        });

        // Guard will clean up terminal state
        drop(guard);

        // Re-throw panic if test failed
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }
}
