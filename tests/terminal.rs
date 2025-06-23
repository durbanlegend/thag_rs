#[cfg(test)]
mod tests {
    use ratatui::crossterm::terminal::disable_raw_mode;
    use ratatui::crossterm::terminal::is_raw_mode_enabled;
    use std::env;
    use std::env::set_var;
    use std::sync::Once;
    use thag_rs::terminal::TerminalStateGuard;
    use thag_rs::{
        terminal::{
            detect_term_capabilities, get_term_bg_luma, is_light_color, restore_raw_status,
        },
        ColorSupport, TermBgLuma,
    };

    #[cfg(feature = "simplelog")]
    use log::LevelFilter;

    #[cfg(feature = "simplelog")]
    use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

    #[cfg(feature = "simplelog")]
    use std::sync::OnceLock;

    #[cfg(feature = "simplelog")]
    use thag_rs::debug_log;

    #[cfg(feature = "simplelog")]
    static LOGGER: OnceLock<()> = OnceLock::new();

    fn init_logger() {
        // Choose between simplelog and env_logger based on compile feature
        #[cfg(feature = "simplelog")]
        LOGGER.get_or_init(|| {
            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Debug,
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    LevelFilter::Debug,
                    Config::default(),
                    std::fs::File::create("app.log").unwrap(),
                ),
            ])
            .unwrap();
            debug_log!("Initialized simplelog");
        });

        #[cfg(not(feature = "simplelog"))] // This will use env_logger if simplelog is not active
        {
            let _ = env_logger::builder().is_test(true).try_init();
        }
    }

    // Set environment variables before running tests
    fn set_up() {
        static INIT: Once = Once::new();
        #[cfg(windows)]
        {
            INIT.call_once(|| unsafe {
                set_var("TEST_ENV", "1");
                set_var("VISUAL", "powershell.exe /C Get-Content");
                set_var("EDITOR", "powershell.exe /C Get-Content");
                init_logger();
            });
        }
        #[cfg(not(windows))]
        {
            INIT.call_once(|| unsafe {
                set_var("TEST_ENV", "1");
                set_var("VISUAL", "cat");
                set_var("EDITOR", "cat");
                init_logger();
            });
        }
    }

    #[test]
    fn test_is_light_color() {
        // Test known light colors
        assert!(is_light_color((255, 255, 255))); // White
        assert!(is_light_color((200, 200, 200))); // Light gray
        assert!(is_light_color((255, 255, 200))); // Light yellow

        // Test known dark colors
        assert!(!is_light_color((0, 0, 0))); // Black
        assert!(!is_light_color((50, 50, 50))); // Dark gray
        assert!(!is_light_color((128, 0, 0))); // Dark red
    }

    #[test]
    fn test_detect_color_support_in_test_env() {
        unsafe {
            env::set_var("TEST_ENV", "1");
        }
        let (support, _) = detect_term_capabilities();
        assert_eq!(*support, ColorSupport::Basic);
        env::remove_var("TEST_ENV");
    }

    #[test]
    fn test_terminal_state_guard() {
        let _guard = TerminalStateGuard::new(false);
        // The guard should reset terminal state when dropped
        // We can't easily test the actual terminal state,
        // but we can verify the guard can be created and dropped
    }

    // #[ignore = "Trivial tests"]
    // #[test]
    // fn test_get_term_bg_error_handling() {
    //     // Test that we get a result (either Ok or Err)
    //     let result = get_term_bg();
    //     match result {
    //         Ok(rgb) => {
    //             // Verify RGB values are valid (0-255)
    //             assert!(rgb.0 <= 255);
    //             assert!(rgb.1 <= 255);
    //             assert!(rgb.2 <= 255);
    //         }
    //         Err(_) => {
    //             // Error case is also acceptable as terminal detection might fail
    //         }
    //     }
    // }

    #[test]
    fn test_get_term_bg_luma() {
        let luma = get_term_bg_luma();
        // Verify we get either Light or Dark
        assert!(matches!(*luma, TermBgLuma::Light | TermBgLuma::Dark));
    }

    #[ignore = "Can't be run headless"]
    #[test]
    fn test_restore_raw_status() {
        // Test restoring to non-raw mode
        let result = restore_raw_status(false);
        assert!(result.is_ok());

        // Test restoring to raw mode
        let result = restore_raw_status(true);
        assert!(result.is_ok());

        // Clean up: ensure we end in non-raw mode
        let _ = disable_raw_mode();
    }

    // Mock tests for terminal-dependent functions
    #[test]
    fn test_color_support_detection() {
        let (support, _) = detect_term_capabilities();
        assert!(matches!(
            *support,
            ColorSupport::None
                | ColorSupport::Basic
                | ColorSupport::Color256
                | ColorSupport::TrueColor
        ));
    }

    #[test]
    fn test_raw_mode_preservation() {
        set_up();
        let initial_raw_mode = is_raw_mode_enabled().unwrap_or(false);

        // Run detection
        let _support = detect_term_capabilities();

        // Verify raw mode status is preserved
        let final_raw_mode = is_raw_mode_enabled().unwrap_or(false);
        assert_eq!(initial_raw_mode, final_raw_mode);
    }
}
