/*[toml]
[dependencies]
crossterm = "0.28.1"
env_logger = "0.11.3"
log = "0.4.21"
# thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["color_detect", "core", "simplelog"] }
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["color_detect", "core", "env_logger" ] }
*/

/// Debug an integration test case. This switches on debug logging
//# Purpose: Demo debugging a test case without the Cargo harness.
//# Categories: technique, testing
use env_logger::Builder;
// use std::ffi::OsStr;
use std::io::Write;
use std::time::{Duration, Instant};
use thag_rs::styling::{ColorInitStrategy, TermAttributes, TermTheme};

struct TestGuard {
    was_raw: bool,
}

impl TestGuard {
    fn new() -> Self {
        use crossterm::{cursor, execute, terminal};

        // Just handle cursor and raw mode
        let mut stdout = std::io::stdout();
        let _ = execute!(
            stdout,
            cursor::MoveToColumn(0),
            terminal::Clear(terminal::ClearType::CurrentLine)
        );
        let _ = stdout.flush();

        Self {
            was_raw: terminal::is_raw_mode_enabled().unwrap_or(false),
        }
    }
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        use crossterm::{cursor, execute, terminal};
        let mut stdout = std::io::stdout();

        // Restore raw mode if needed
        if !self.was_raw {
            let _ = terminal::disable_raw_mode();
        }

        // Just ensure cursor position and line clarity
        let _ = execute!(
            stdout,
            cursor::MoveToColumn(0),
            terminal::Clear(terminal::ClearType::CurrentLine)
        );
        let _ = stdout.flush();
    }
}

fn main() {
    Builder::new().filter_level(log::LevelFilter::Debug).init();

    let guard = TestGuard::new();

    let result = std::panic::catch_unwind(|| {
        let strategy = ColorInitStrategy::Detect;

        // Use a shorter timeout for theme detection
        let timeout = Duration::from_millis(500);

        // First theme detection with timeout
        let start = Instant::now();
        let log_color1 = TermAttributes::initialize(strategy.clone());
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
        let log_color2 = TermAttributes::initialize(strategy);
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
