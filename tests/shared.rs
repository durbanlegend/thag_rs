use std::sync::Once;
use thag_rs::shared::escape_path_for_windows;

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
