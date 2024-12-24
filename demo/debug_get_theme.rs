/*[toml]
[dependencies]
env_logger = "0.11.3"
log = "0.4.21"
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["color_support", "core", "simplelog"] }
*/

/// Debug an integration test case.
//# Purpose: Demo debugging a test case without the Cargo harness.
//# Categories: technique, testing
use env_logger::Builder;
// use std::ffi::OsStr;
use std::io::Write;
use std::time::{Duration, Instant};
use thag_rs::log_color::{ColorSupport, LogColor, Theme};

#[allow(dead_code)]
struct TestGuard {
    was_alternate: bool,
    was_raw: bool,
    clear_screen: bool, // New flag
}

impl TestGuard {
    fn new(clear_screen: bool) -> Self {
        use crossterm::{cursor, execute, terminal};
        // let stdout = std::io::stdout();

        // // Only clear if flag is set
        // if clear_screen {
        //     let _ = execute!(
        //         stdout,
        //         terminal::LeaveAlternateScreen,
        //         cursor::MoveToColumn(0),
        //         terminal::Clear(terminal::ClearType::CurrentLine)
        //     );
        // }

        let result = std::panic::catch_unwind(|| {
            let mut stdout = std::io::stdout();
            // Only clear if flag is set
            if clear_screen {
                let _ = execute!(
                    stdout,
                    terminal::LeaveAlternateScreen,
                    cursor::MoveToColumn(0),
                    terminal::Clear(terminal::ClearType::CurrentLine)
                );
            }
            let _ = stdout.flush();
            terminal::is_raw_mode_enabled().unwrap_or(false)
        });

        Self {
            was_alternate: false,
            was_raw: result.unwrap_or(false),
            clear_screen,
        }
    }
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        use crossterm::{cursor, execute, terminal};
        let mut stdout = std::io::stdout();

        // Sequence cleanup operations with flushing
        // let _ = execute!(stdout, terminal::LeaveAlternateScreen);
        let _ = stdout.flush();

        if !self.was_raw {
            let _ = terminal::disable_raw_mode();
        }
        let _ = stdout.flush();

        // let _ = execute!(
        //     stdout,
        //     cursor::MoveToColumn(0),
        //     terminal::Clear(terminal::ClearType::CurrentLine)
        // );
        // Only clear if flag is set
        if self.clear_screen {
            let _ = execute!(
                stdout,
                terminal::LeaveAlternateScreen,
                cursor::MoveToColumn(0),
                terminal::Clear(terminal::ClearType::CurrentLine)
            );
        }
        let _ = stdout.flush();
    }
}

fn main() {
    Builder::new().filter_level(log::LevelFilter::Debug).init();

    let guard = TestGuard::new(
        // Only clear in WezTerm
        std::env::var("TERM_PROGRAM")
            .map(|v| v == "WezTerm")
            .unwrap_or(false),
    );

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
