#[cfg(test)]
mod tests {
    use serial_test::parallel;
    use std::sync::Once;
    use thag_common::{
        escape_path_for_windows, set_global_verbosity, OutputManager, Verbosity, OUTPUT_MANAGER,
    };

    // Set environment variables before running tests
    fn set_up() {
        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe {
            std::env::set_var("TEST_ENV", "1");
            std::env::set_var("VISUAL", "cat");
            std::env::set_var("EDITOR", "cat");
        });
    }

    #[test]
    fn test_shared_escape_path_for_windows() {
        set_up();
        #[cfg(windows)]
        {
            let path = r"C:\path\to\file";
            let escaped_path = escape_path_for_windows(path);
            assert_eq!(escaped_path, r"C:/path/to/file");
        }

        #[cfg(not(windows))]
        {
            let path = "/path/to/file";
            let escaped_path = escape_path_for_windows(path);
            assert_eq!(escaped_path, path);
        }
    }

    // A utility function to reset the global output manager for testing.
    fn reset_global_output_manager() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            drop(OUTPUT_MANAGER.lock().unwrap());
        });
    }

    #[test]
    #[parallel]
    fn test_shared_output_manager_new() {
        set_up();
        let output_manager = OutputManager::new(Verbosity::Quiet);
        assert_eq!(output_manager.verbosity, Verbosity::Quiet);

        let output_manager = OutputManager::new(Verbosity::Normal);
        assert_eq!(output_manager.verbosity, Verbosity::Normal);

        let output_manager = OutputManager::new(Verbosity::Verbose);
        assert_eq!(output_manager.verbosity, Verbosity::Verbose);
    }

    #[test]
    #[parallel]
    fn test_shared_global_output_manager() {
        set_up();
        reset_global_output_manager();

        set_global_verbosity(Verbosity::Verbose).expect("Error setting global verbosity");
        {
            let output_manager = OUTPUT_MANAGER.lock().unwrap();
            assert_eq!(output_manager.verbosity, Verbosity::Verbose);
        }

        set_global_verbosity(Verbosity::Quiet).expect("Error setting global verbosity");
        {
            let output_manager = OUTPUT_MANAGER.lock().unwrap();
            assert_eq!(output_manager.verbosity, Verbosity::Quiet);
        }
    }
}
