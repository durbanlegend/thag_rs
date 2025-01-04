//! Terminal detection functionality for color support and theme
//!
//! This module handles detection of terminal capabilities while preserving terminal state.
//! In particular, it manages raw mode status which can be affected by some detection operations.

use crate::styling::{ColorSupport, TermTheme};
use crate::terminal::{is_raw_mode_enabled, restore_raw_status};
use scopeguard::defer;
use std::io;

#[cfg(feature = "color_detect")]
use supports_color::Stream;

static DETECTED_THEME: AtomicU8 = AtomicU8::new(0);

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
pub fn detect_color_support() -> ColorSupport {
    if std::env::var("TEST_ENV").is_ok() {
        #[cfg(debug_assertions)]
        debug_log!("Avoiding supports_color for testing");
        return ColorSupport::Ansi16;
    }

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

    #[cfg(feature = "color_detect")]
    {
        if let Some(color_level) = supports_color::on(Stream::Stdout) {
            if color_level.has_16m || color_level.has_256 {
                ColorSupport::Xterm256
            } else {
                ColorSupport::Ansi16
            }
        } else {
            ColorSupport::None
        }
    }

    #[cfg(not(feature = "color_detect"))]
    ColorSupport::None
}

/// Detects the terminal's theme (light or dark)
///
/// When the `color_detect` feature is enabled, this function attempts to detect
/// the terminal's background theme using `termbg`. If detection fails or the
/// feature is disabled, it defaults to `TermTheme::Dark`.
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
// pub fn detect_theme() -> TermTheme {
//     if std::env::var("TEST_ENV").is_ok() {
//         return TermTheme::Dark;
//     }

//     let raw_before = is_raw_mode_enabled();
//     if let Ok(raw_then) = raw_before {
//         defer! {
//             let raw_now = match is_raw_mode_enabled() {
//                 Ok(val) => val,
//                 Err(e) => {
//                     #[cfg(debug_assertions)]
//                     debug_log!("Failed to check raw mode status: {:?}", e);
//                     return;
//                 }
//             };

//             if raw_now == raw_then {
//                 #[cfg(debug_assertions)]
//                 debug_log!("Raw mode status unchanged.");
//             } else if let Err(e) = restore_raw_status(raw_then) {
//                 #[cfg(debug_assertions)]
//                 debug_log!("Failed to restore raw mode: {:?}", e);
//             } else {
//                 #[cfg(debug_assertions)]
//                 debug_log!("Raw mode restored to previous state.");
//             }
//         }
//     }

//     #[cfg(feature = "color_detect")]
//     {
//         if let Ok(bg) = termbg::theme() {
//             match bg {
//                 termbg::Theme::Light => TermTheme::Light,
//                 termbg::Theme::Dark => TermTheme::Dark,
//             }
//         } else {
//             TermTheme::Dark
//         }
//     }

//     #[cfg(not(feature = "color_detect"))]
//     TermTheme::Dark
// }

pub fn detect_theme() -> TermTheme {
    // Check cache first
    let detected = DETECTED_THEME.load(Ordering::Relaxed);
    if detected != 0 {
        return if detected == 1 {
            TermTheme::Light
        } else {
            TermTheme::Dark
        };
    }

    let theme = detect_theme_internal();
    DETECTED_THEME.store(
        match theme {
            TermTheme::Light => 1,
            TermTheme::Dark => 2,
        },
        Ordering::Relaxed,
    );
    theme
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
