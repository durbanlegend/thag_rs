//! Terminal detection functionality for color support and theme
//!
//! This module handles detection of terminal capabilities while preserving terminal state.
//! In particular, it manages raw mode status which can be affected by some detection operations.

use crate::styling::{ColorSupport, TermBgLuma};
use crate::{lazy_static_var, vlog, ThagError, ThagResult, V};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled};
use std::io::{stdout, Write};
use supports_color::Stream;
use thag_profiler::profiled;

#[cfg(debug_assertions)]
use crate::debug_log;

#[profiled]
fn reset_terminal_state() {
    // eprintln!("Resetting terminal state...");
    print!("\r\n");
    let _ = stdout().flush();
}

#[allow(clippy::module_name_repetitions)]
pub struct TerminalStateGuard {
    pub raw_mode: bool,
}

impl TerminalStateGuard {
    #[must_use]
    pub const fn new(raw_mode: bool) -> Self {
        Self { raw_mode }
    }
}

impl Default for TerminalStateGuard {
    #[profiled]
    fn default() -> Self {
        Self::new(false)
    }
}

#[allow(unused_variables)]
impl Drop for TerminalStateGuard {
    #[profiled]
    fn drop(&mut self) {
        let raw_now = match is_raw_mode_enabled() {
            Ok(val) => val,
            Err(e) => {
                #[cfg(debug_assertions)]
                debug_log!("Failed to check raw mode status: {:?}", e);
                return;
            }
        };

        if raw_now == self.raw_mode {
            #[cfg(debug_assertions)]
            debug_log!("Raw mode status unchanged.");
        } else if let Err(e) = restore_raw_status(self.raw_mode) {
            #[cfg(debug_assertions)]
            debug_log!("Failed to restore raw mode: {:?}", e);
        } else {
            #[cfg(debug_assertions)]
            debug_log!("Raw mode restored to previous state.");
        }
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
#[profiled]
pub fn detect_term_capabilities() -> (&'static ColorSupport, &'static (u8, u8, u8)) {
    if std::env::var("TEST_ENV").is_ok() {
        #[cfg(debug_assertions)]
        debug_log!("Avoiding supports_color for testing");
        return (&ColorSupport::Basic, &(0, 0, 0));
    }

    let maybe_raw_mode = is_raw_mode_enabled();
    let raw_now = match maybe_raw_mode {
        Ok(val) => val,
        #[allow(unused_variables)]
        Err(e) => {
            #[cfg(debug_assertions)]
            debug_log!("Failed to check raw mode status: {:?}", e);
            return (&ColorSupport::Basic, &(0, 0, 0));
        }
    };

    let _guard = TerminalStateGuard::new(raw_now);

    let color_support = lazy_static_var!(ColorSupport, {
        vlog!(V::V, "Checking colour support");

        supports_color::on(Stream::Stdout).map_or(ColorSupport::None, |color_level| {
            if color_level.has_16m {
                ColorSupport::TrueColor
            } else if color_level.has_256 {
                ColorSupport::Color256
            } else {
                ColorSupport::Basic
            }
        })
    });

    let term_bg_rgb = lazy_static_var!((u8, u8, u8), {
        let maybe_term_bg = get_term_bg_rgb_unguarded();
        *maybe_term_bg.unwrap_or(&(0, 0, 0))
    });

    (color_support, term_bg_rgb)
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
#[profiled]
pub fn get_term_bg_luma() -> &'static TermBgLuma {
    lazy_static_var!(TermBgLuma, {
        let _guard = TerminalStateGuard::new(false);

        let maybe_term_bg = get_term_bg_rgb();
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
#[profiled]
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
/// use thag_rs::terminal::get_term_bg_rgb;
/// use thag_rs::ThagError;
///
/// let maybe_term_bg_rgb = get_term_bg_rgb();
/// println!("Terminal background: {maybe_term_bg_rgb:?}");
/// ```
#[profiled]
pub fn get_term_bg_rgb() -> ThagResult<&'static (u8, u8, u8)> {
    struct RawModeGuard(bool);
    impl Drop for RawModeGuard {
        #[profiled]
        fn drop(&mut self) {
            if !self.0 {
                let _ = disable_raw_mode();
                reset_terminal_state();
            }
        }
    }

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
            vlog!(V::V, "Checking terminal background");
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
    .as_ref()
    .map_err(|e| ThagError::from(e.to_string()))
}

/// Detects the terminal's background color.
///
/// This version is meant to be called from
/// a guarded function that handles raw mode and does a single terminal reset for all
/// terminal interrogations, to avoid multiple newlines generated by multiple resets.
///
/// This function attempts to detect the terminal's background color using `termbg`.
/// If detection fails it defaults to `TermTheme::Dark`.
///
/// # Raw Mode Handling
///
/// This function has no raw mode handling and shouldbe called from one that does.
/// Alternatively, use `get_term_bg_rgb` to implement raw mode handling.
///
/// # Errors
///
/// This function will wrap and return any error returned by `termbg::rgb`.
///
/// # Examples
///
/// ```
/// use thag_rs::terminal::get_term_bg_rgb;
/// use thag_rs::ThagError;
///
/// let maybe_term_bg_rgb = get_term_bg_rgb();
/// println!("Terminal background: {maybe_term_bg_rgb:?}");
/// ```
#[profiled]
pub fn get_term_bg_rgb_unguarded() -> ThagResult<&'static (u8, u8, u8)> {
    // Now do theme detection
    lazy_static_var!(
    Result < (u8, u8, u8),
    termbg::Error >, {
        vlog!(V::V, "Checking terminal background");
        let timeout = std::time::Duration::from_millis(500);
        let bg_rgb = termbg::rgb(timeout)?;

        // Convert 16-bit RGB to 8-bit RGB
        let (r, g, b): (u8, u8, u8) = (
            (bg_rgb.r >> 8) as u8,
            (bg_rgb.g >> 8) as u8,
            (bg_rgb.b >> 8) as u8,
        );

        Ok((r, g, b))
    })
    .as_ref()
    .map_err(|e| ThagError::from(e.to_string()))
}

/// Restore the raw or cooked terminal status as saved in the boolean argument.
///
/// # Errors
///
/// This function will bubble up any errors returned by `crossterm`.
#[profiled]
pub fn restore_raw_status(raw_before: bool) -> ThagResult<()> {
    if raw_before {
        enable_raw_mode()?;
    } else {
        disable_raw_mode()?;
    }
    Ok(())
}
