//! Terminal detection functionality for color support and theme
//!
//! This module handles detection of terminal capabilities while preserving terminal state.
//! In particular, it manages raw mode status which can be affected by some detection operations.

use crate::styling::{ColorSupport, TermBgLuma};
use crate::{lazy_static_var, profile, ThagError, ThagResult};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled};
use scopeguard::defer;
use std::io::{stdout, Write};
use supports_color::Stream;

#[cfg(debug_assertions)]
use crate::debug_log;

fn reset_terminal_state() {
    print!("\r");
    let _ = stdout().flush();
}

#[allow(clippy::module_name_repetitions)]
pub struct TerminalStateGuard;

impl TerminalStateGuard {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for TerminalStateGuard {
    fn default() -> Self {
        Self::new()
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
    profile!("detect_color_support");
    if std::env::var("TEST_ENV").is_ok() {
        #[cfg(debug_assertions)]
        debug_log!("Avoiding supports_color for testing");
        return &ColorSupport::Basic;
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
            if color_level.has_16m {
                ColorSupport::TrueColor
            } else if color_level.has_256 {
                ColorSupport::Color256
            } else {
                ColorSupport::Basic
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
/// use thag_rs::terminal::get_term_bg_luma;
///
/// let luma = get_term_bg_luma();
/// println!("Terminal background intensity: {:?}", luma);
/// ```
pub fn get_term_bg_luma() -> &'static TermBgLuma {
    profile!("get_term_bg_luma");

    lazy_static_var!(TermBgLuma, {
        let _guard = TerminalStateGuard::new();

        let maybe_term_bg = get_term_bg();
        maybe_term_bg.map_or(TermBgLuma::Dark, |rgb| {
            if is_light_color(*rgb) {
                TermBgLuma::Light
            } else {
                TermBgLuma::Dark
            }
        })
    })
}

#[must_use]
pub fn is_light_color((r, g, b): (u8, u8, u8)) -> bool {
    // Using perceived brightness formula
    let brightness =
        f32::from(b).mul_add(0.114, f32::from(r).mul_add(0.299, f32::from(g) * 0.587)) / 255.0;
    brightness > 0.5
}

/// Detects the terminal's background color.
///
/// This function attempts to detect the terminal's background color using `termbg`.
/// If detection fails it defaults to `TermTheme::Dark`.
///
/// # Raw Mode Handling
///
/// This function preserves the terminal's raw mode status, restoring it if detection
/// operations modify it.
///
/// # Errors
///
/// This function will wrap and return any error returned by `termbg::rgb`.
///
/// # Examples
///
/// ```
/// use thag_rs::terminal::get_term_bg;
/// use thag_rs::ThagError;
///
/// let maybe_term_bg = get_term_bg();
/// println!("Terminal bckground: {maybe_term_bg:?}");
/// ```
pub fn get_term_bg() -> ThagResult<&'static (u8, u8, u8)> {
    struct RawModeGuard(bool);
    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            if !self.0 {
                let _ = disable_raw_mode();
                reset_terminal_state();
            }
        }
    }

    profile!("get_term_bg");

    lazy_static_var!(
        Result < (u8, u8, u8),
        termbg::Error >, {
            // Save initial state
            let raw_before = is_raw_mode_enabled()?;

            // Ensure raw mode for detection
            if !raw_before {
                enable_raw_mode()?;
            }

            let _guard = RawModeGuard(raw_before);

            // Now do theme detection
            eprintln!("Checking terminal background");
            let timeout = std::time::Duration::from_millis(500);
            let bg_rgb = termbg::rgb(timeout)?;

            // Convert 16-bit RGB to 8-bit RGB
            let (r, g, b): (u8, u8, u8) = (
                (bg_rgb.r >> 8) as u8,
                (bg_rgb.g >> 8) as u8,
                (bg_rgb.b >> 8) as u8,
            );

            Ok((r, g, b))
        }
    )
    // .map(|x| &x)
    .as_ref()
    .map_err(|e| ThagError::from(e.to_string()))
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
