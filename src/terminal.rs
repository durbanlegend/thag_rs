//! Terminal detection functionality for color support and theme
//!
//! This module handles detection of terminal capabilities while preserving terminal state.
//! In particular, it manages raw mode status which can be affected by some detection operations.

use crate::styling::{ColorSupport, TermTheme};
use crate::{lazy_static_var, profile, ThagResult};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled};
use scopeguard::defer;
use std::io::{stdout, Write};
use supports_color::Stream;
use termbg::Theme as TermbgTheme;

#[cfg(debug_assertions)]
use crate::debug_log;

// fn reset_terminal_state() {
//     // Create a sequence that should work across terminals
//     let reset_sequence = format!(
//         "{}{}{}{}{}{}",
//         "\x1B[s",    // Save cursor position
//         "\x1B[?25h", // Show cursor
//         "\x1B[?7h",  // Enable line wrap
//         "\x1B[0m",   // Reset attributes
//         "\x1B[r",    // Reset scroll region
//         "\x1B[u"     // Restore cursor position
//     );

//     // Apply to main screen
//     print!("\x1B[?1049l{}", reset_sequence);

//     // Apply to alternate screen
//     print!("\x1B[?1049h{}", reset_sequence);

//     // Return to main screen
//     print!("\x1B[?1049l");

//     let _ = stdout().flush();
// }

fn reset_terminal_state() {
    print!("\r");
    let _ = stdout().flush();
}

struct TerminalStateGuard;

impl TerminalStateGuard {
    const fn new() -> Self {
        Self
    }
}

impl Drop for TerminalStateGuard {
    fn drop(&mut self) {
        reset_terminal_state();
    }
}

/// Detects the terminal's color support level
///
/// When the `color_detect` feature is enabled, this function uses `supports_color`
/// to determine the actual terminal capabilities. Otherwise, it returns `ColorSupport::None`.
///
/// # Raw Mode Handling
///
/// This function preserves the terminal's raw mode status, restoring it if detection
/// operations modify it.
///
/// # Examples
///
/// ```
/// use thag_rs::terminal::detect_color_support;
///
/// let support = detect_color_support();
/// println!("Terminal color support: {:?}", support);
/// ```
#[must_use]
#[allow(unused_variables)]
pub fn detect_color_support() -> &'static ColorSupport {
    if std::env::var("TEST_ENV").is_ok() {
        #[cfg(debug_assertions)]
        debug_log!("Avoiding supports_color for testing");
        return &ColorSupport::Ansi16;
    }

    reset_terminal_state();

    lazy_static_var!(ColorSupport, {
        let _guard = TerminalStateGuard::new();
        println!("Checking colour support");
        let raw_before = is_raw_mode_enabled();
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

        supports_color::on(Stream::Stdout).map_or(ColorSupport::None, |color_level| {
            if color_level.has_16m || color_level.has_256 {
                ColorSupport::Xterm256
            } else {
                ColorSupport::Ansi16
            }
        })
    })
}

/// Detects the terminal's theme (light or dark)
///
/// This function attempts to detect the terminal's background theme using `termbg`.
/// If detection fails it defaults to `TermTheme::Dark`.
///
/// # Raw Mode Handling
///
/// This function preserves the terminal's raw mode status, restoring it if detection
/// operations modify it.
///
/// # Examples
///
/// ```
/// use thag_rs::terminal::detect_theme;
///
/// let theme = detect_theme();
/// println!("Terminal theme: {:?}", theme);
/// ```
pub fn detect_theme() -> &'static TermTheme {
    lazy_static_var!(TermTheme, {
        let _guard = TerminalStateGuard::new();
        detect_theme_internal().unwrap_or(TermTheme::Dark)
    })
}

fn detect_theme_internal() -> Result<TermTheme, termbg::Error> {
    struct RawModeGuard(bool);
    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            if !self.0 {
                let _ = disable_raw_mode();
                reset_terminal_state();
            }
        }
    }

    // Save initial state
    let raw_before = is_raw_mode_enabled()?;

    // Ensure raw mode for detection
    if !raw_before {
        enable_raw_mode()?;
    }

    let _guard = RawModeGuard(raw_before);

    // Now do theme detection
    println!("Checking terminal theme");
    let timeout = std::time::Duration::from_millis(500);
    let theme = termbg::theme(timeout)?;

    Ok(match theme {
        TermbgTheme::Light => TermTheme::Light,
        TermbgTheme::Dark => TermTheme::Dark,
    })
}

/// Restore the raw or cooked terminal status as saved in the boolean argument.
///
/// # Errors
///
/// This function will bubble up any errors returned by `crossterm`.
pub fn restore_raw_status(raw_before: bool) -> ThagResult<()> {
    profile!("restore_raw_status");
    if raw_before {
        enable_raw_mode()?;
    } else {
        disable_raw_mode()?;
    }
    Ok(())
}
