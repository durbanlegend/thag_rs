//!
//! This module handles detection of terminal capabilities while preserving terminal state.
//! In particular, it manages raw mode status which can be affected by some detection operations.

use crate::{
    lazy_static_var, vprtln, ColorSupport, TermBgLuma, ThagCommonError, ThagCommonResult, V,
};
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled};
use std::io::{stdin, stdout, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use supports_color::Stream;

#[cfg(debug_assertions)]
use crate::debug_log;

fn reset_terminal_state() {
    // eprintln!("Resetting terminal state...");
    print!("\r\n");
    let _ = stdout().flush();
}

#[allow(clippy::module_name_repetitions)]
/// Guards the terminal state during operations that may affect raw mode
///
/// This struct preserves the raw mode status of the terminal and restores it
/// when dropped, ensuring that terminal detection operations don't leave the
/// terminal in an unexpected state.
pub struct TerminalStateGuard {
    /// The raw mode status to restore when the guard is dropped
    pub raw_mode: bool,
}

impl TerminalStateGuard {
    /// Creates a new terminal state guard with the specified raw mode status
    ///
    /// The guard will restore the terminal to this raw mode status when dropped.
    ///
    /// # Arguments
    ///
    /// * `raw_mode` - The raw mode status to restore when the guard is dropped
    #[must_use]
    pub const fn new(raw_mode: bool) -> Self {
        Self { raw_mode }
    }
}

impl Default for TerminalStateGuard {
    fn default() -> Self {
        Self::new(false)
    }
}

#[allow(unused_variables)]
impl Drop for TerminalStateGuard {
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
/// Uses OSC-based `TrueColor` detection as the primary method, with `supports_color`
/// as a fallback. This provides more accurate detection across different terminals
/// and platforms, especially for mintty and terminals that don't set proper
/// environment variables.
///
/// # Raw Mode Handling
///
/// This function preserves the terminal's raw mode status, restoring it if detection
/// operations modify it.
///
/// # Examples
///
/// ```
/// use thag_rs::terminal::detect_term_capabilities;
///
/// let support = detect_term_capabilities();
/// println!("Terminal color support: {:?}", support);
/// ```
#[must_use]
pub fn detect_term_capabilities() -> (&'static ColorSupport, &'static (u8, u8, u8)) {
    if std::env::var("TEST_ENV").is_ok() {
        #[cfg(debug_assertions)]
        debug_log!("Avoiding color detection for testing");
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
        vprtln!(V::V, "Checking colour support");

        // Try our OSC-based detection first
        detect_color_support_osc()
    });

    let term_bg_rgb = lazy_static_var!((u8, u8, u8), {
        let maybe_term_bg = get_term_bg_rgb_unguarded();
        *maybe_term_bg.unwrap_or(&(0, 0, 0))
    });

    (color_support, term_bg_rgb)
}

/// Get fresh color detection without caching - intended for dynamic changes.
/// This should normally be stored in `TermAttributes`
#[must_use]
pub fn get_fresh_color_support() -> ColorSupport {
    detect_color_support_osc()
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
/// Determines if a color is light based on perceived brightness
///
/// Uses the standard perceived brightness formula to calculate luminance
/// from RGB values and determines if the color appears light or dark.
///
/// # Arguments
///
/// * `(r, g, b)` - RGB color values as a tuple of u8 values
///
/// # Returns
///
/// Returns `true` if the color is perceived as light (brightness > 0.5),
/// `false` otherwise.
///
/// # Examples
///
/// ```
/// use thag_rs::terminal::is_light_color;
///
/// assert!(is_light_color((255, 255, 255))); // white is light
/// assert!(!is_light_color((0, 0, 0)));      // black is dark
/// ```
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
pub fn get_term_bg_rgb() -> ThagCommonResult<&'static (u8, u8, u8)> {
    struct RawModeGuard(bool);
    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            if !self.0 {
                let _ = disable_raw_mode();
                reset_terminal_state();
            }
        }
    }

    lazy_static_var!(
        Result < (u8, u8, u8),
        ThagCommonError >, {
            // Save initial state
            let raw_before = is_raw_mode_enabled()
                .map_err(|e| ThagCommonError::Generic(e.to_string()))?;

            // Ensure raw mode for detection
            if !raw_before {
                enable_raw_mode()
                    .map_err(|e| ThagCommonError::Generic(e.to_string()))?;
            }

            let _guard = RawModeGuard(raw_before);

            // Try custom OSC 11 implementation first (more reliable timeout)
            vprtln!(V::V, "Checking terminal background with OSC 11");
            if let Some(rgb) = query_background_osc11_with_timeout(Duration::from_millis(300)) {
                vprtln!(V::V, "OSC 11 query successful: {rgb:?}");
                return Ok(rgb);
            }

            // Fallback to termbg with shorter timeout
            vprtln!(V::V, "OSC 11 failed, trying termbg");
            let timeout = std::time::Duration::from_millis(200);
            match termbg::rgb(timeout) {
                Ok(bg_rgb) => {
                    // Convert 16-bit RGB to 8-bit RGB
                    let rgb = (
                        (bg_rgb.r >> 8) as u8,
                        (bg_rgb.g >> 8) as u8,
                        (bg_rgb.b >> 8) as u8,
                    );
                    vprtln!(V::V, "termbg successful: {rgb:?}");
                    Ok(rgb)
                }
                Err(e) => {
                    vprtln!(V::V, "Both OSC 11 and termbg failed: {e}");
                    Err(ThagCommonError::Generic(format!("Background detection failed: {}", e)))
                }
            }
        }
    )
    .as_ref()
    .map_err(|e| ThagCommonError::Generic(e.to_string()))
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
/// This function has no raw mode handling and should be called from one that does.
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
pub fn get_term_bg_rgb_unguarded() -> ThagCommonResult<&'static (u8, u8, u8)> {
    // Now do theme detection
    lazy_static_var!(
    Result < (u8, u8, u8),
    ThagCommonError >, {
        // Try custom OSC 11 implementation first (more reliable timeout)
        vprtln!(V::V, "Checking terminal background with OSC 11 (unguarded)");
        if let Some(rgb) = query_background_osc11_with_timeout(Duration::from_millis(300)) {
            vprtln!(V::V, "OSC 11 query successful: {rgb:?}");
            return Ok(rgb);
        }

        // Fallback to termbg with shorter timeout
        vprtln!(V::V, "OSC 11 failed, trying termbg (unguarded)");
        let timeout = std::time::Duration::from_millis(200);
        match termbg::rgb(timeout) {
            Ok(bg_rgb) => {
                // Convert 16-bit RGB to 8-bit RGB
                let rgb = (
                    (bg_rgb.r >> 8) as u8,
                    (bg_rgb.g >> 8) as u8,
                    (bg_rgb.b >> 8) as u8,
                );
                vprtln!(V::V, "termbg successful: {rgb:?}");
                Ok(rgb)
            }
            Err(e) => {
                vprtln!(V::V, "Both OSC 11 and termbg failed: {e}");
                Err(ThagCommonError::Generic(format!("Background detection failed: {e}")))
            }
        }
    })
    .as_ref()
    .map_err(|e| ThagCommonError::Generic(e.to_string()))
}

/// Detects color support using OSC sequences with fallback to `supports_color`
fn detect_color_support_osc() -> ColorSupport {
    // Check for mintty first - always supports TrueColor
    if is_mintty() {
        vprtln!(V::V, "Mintty detected - TrueColor supported");
        return ColorSupport::TrueColor;
    }

    // Check environment variables for override/hints - these take precedence over terminal detection
    if let Some(support) = check_env_color_support() {
        vprtln!(V::V, "Environment variable override: {:?}", support);
        return support;
    }

    // Check for problematic terminals that claim truecolor but don't handle it correctly
    if is_apple_terminal() {
        vprtln!(
            V::V,
            "Apple Terminal detected - forcing 256-color mode due to RGB rendering issues (salmon pink bug)"
        );
        return ColorSupport::Color256;
    }

    // Try OSC-based `TrueColor` detection
    if test_truecolor_support() {
        vprtln!(V::V, "OSC TrueColor test passed");
        return ColorSupport::TrueColor;
    }

    // Check for 256-color support
    if supports_256_color() {
        vprtln!(V::V, "256-color support detected");
        return ColorSupport::Color256;
    }

    // Fallback to `supports_color` crate
    vprtln!(V::N, "Using supports_color crate fallback");
    supports_color::on(Stream::Stdout).map_or(ColorSupport::Basic, |color_level| {
        if color_level.has_16m {
            ColorSupport::TrueColor
        } else if color_level.has_256 {
            ColorSupport::Color256
        } else {
            ColorSupport::Basic
        }
    })
}

/// Check if running in mintty (always supports `TrueColor`)
fn is_mintty() -> bool {
    std::env::var("TERM_PROGRAM").is_ok_and(|term| term == "mintty")
}

/// Check if running in Apple Terminal (which has RGB rendering issues)
fn is_apple_terminal() -> bool {
    std::env::var("TERM_PROGRAM").is_ok_and(|term| term == "Apple_Terminal")
}

/// Query background color using OSC 11 with robust timeout handling
fn query_background_osc11_with_timeout(timeout: Duration) -> Option<(u8, u8, u8)> {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let result = (|| -> Option<(u8, u8, u8)> {
            if enable_raw_mode().is_err() {
                return None;
            }

            let mut stdout = std::io::stdout();
            let mut stdin = std::io::stdin();

            // Send OSC 11 query (background color)
            let query = "\x1b]11;?\x07";
            stdout.write_all(query.as_bytes()).ok()?;
            stdout.flush().ok()?;

            // Read response with timeout
            let response = read_terminal_response_with_timeout(&mut stdin, timeout)?;

            // Parse OSC 11 response
            parse_osc11_background_response(&response)
        })();

        let _ = disable_raw_mode();
        let _ = tx.send(result);
    });

    // Wait for result with timeout
    rx.recv_timeout(timeout + Duration::from_millis(50))
        .map_or_else(
            |_| {
                vprtln!(V::V, "OSC 11 query timed out");
                None
            },
            |result| {
                let _ = handle.join();
                result
            },
        )
}

/// Read terminal response with proper timeout handling
fn read_terminal_response_with_timeout(
    stdin: &mut std::io::Stdin,
    timeout: Duration,
) -> Option<String> {
    use std::io::Read;

    let mut buffer = Vec::new();
    let mut temp_buffer = [0u8; 1];
    let start = Instant::now();

    while start.elapsed() < timeout {
        match stdin.read(&mut temp_buffer) {
            Ok(1) => {
                buffer.push(temp_buffer[0]);

                // Check for response terminators
                if temp_buffer[0] == 0x07 || // BEL
                   (buffer.len() >= 2 && buffer[buffer.len()-2] == 0x1b && buffer[buffer.len()-1] == b'\\')
                {
                    break;
                }

                // Safety limit to prevent infinite buffering
                if buffer.len() > 1024 {
                    vprtln!(V::V, "OSC response buffer limit exceeded");
                    break;
                }
            }
            Ok(0) => {
                thread::sleep(Duration::from_millis(1));
            }
            Ok(_) => {
                // Handle case where more than 1 byte is read
                thread::sleep(Duration::from_millis(1));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
            }
            Err(_) => {
                vprtln!(V::V, "Error reading terminal response");
                break;
            }
        }
    }

    if buffer.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&buffer).to_string())
    }
}

/// Parse OSC 11 background color response
fn parse_osc11_background_response(response: &str) -> Option<(u8, u8, u8)> {
    // OSC 11 response format: ESC]11;rgb:RRRR/GGGG/BBBB BEL
    if let Some(start_pos) = response.find("\x1b]11;") {
        let response_part = &response[start_pos..];

        if let Some(rgb_pos) = response_part.find("rgb:") {
            let rgb_data = &response_part[rgb_pos + 4..];

            let end_pos = rgb_data
                .find('\x07')
                .or_else(|| rgb_data.find('\x1b'))
                .unwrap_or(rgb_data.len());

            if end_pos > 0 {
                let rgb_sequence = &rgb_data[..end_pos];
                let parts: Vec<&str> = rgb_sequence.split('/').collect();

                if parts.len() == 3 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        parse_hex_component_bg(parts[0]),
                        parse_hex_component_bg(parts[1]),
                        parse_hex_component_bg(parts[2]),
                    ) {
                        return Some((r, g, b));
                    }
                }
            }
        }

        // Also try #RRGGBB format
        if let Some(hash_pos) = response_part.find('#') {
            let hex_data = &response_part[hash_pos + 1..];
            if hex_data.len() >= 6 {
                let hex_str = &hex_data[..6];
                if hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex_str[0..2], 16),
                        u8::from_str_radix(&hex_str[2..4], 16),
                        u8::from_str_radix(&hex_str[4..6], 16),
                    ) {
                        return Some((r, g, b));
                    }
                }
            }
        }
    }

    None
}

/// Parse hex component from OSC response (for background detection)
fn parse_hex_component_bg(hex_str: &str) -> Result<u8, std::num::ParseIntError> {
    let clean_hex: String = hex_str
        .chars()
        .take_while(char::is_ascii_hexdigit)
        .collect();

    match clean_hex.len() {
        4 => {
            // Take first 2 characters for 8-bit value (high byte)
            u8::from_str_radix(&clean_hex[0..2], 16)
        }
        2 => u8::from_str_radix(&clean_hex, 16),
        _ => {
            let val = u16::from_str_radix(&clean_hex, 16).unwrap_or(0);
            Ok(val.min(255) as u8)
        }
    }
}

/// Check environment variables for color support hints
fn check_env_color_support() -> Option<ColorSupport> {
    // Check for NO_COLOR (takes precedence)
    if std::env::var("NO_COLOR").is_ok() {
        return Some(ColorSupport::None);
    }

    // Check for THAG_COLOR_MODE (thag-specific override)
    if let Ok(thag_color_mode) = std::env::var("THAG_COLOR_MODE") {
        match thag_color_mode.to_lowercase().as_str() {
            "none" | "off" | "0" => return Some(ColorSupport::None),
            "basic" | "16" | "1" => return Some(ColorSupport::Basic),
            "256" | "2" => return Some(ColorSupport::Color256),
            "truecolor" | "24bit" | "rgb" | "3" => return Some(ColorSupport::TrueColor),
            _ => {}
        }
    }

    // Check for FORCE_COLOR
    if let Ok(force_color) = std::env::var("FORCE_COLOR") {
        match force_color.as_str() {
            "0" => return Some(ColorSupport::None),
            "1" => return Some(ColorSupport::Basic),
            "2" => return Some(ColorSupport::Color256),
            "3" => return Some(ColorSupport::TrueColor),
            _ => {}
        }
    }

    // Check CLICOLOR_FORCE
    if std::env::var("CLICOLOR_FORCE").is_ok() {
        return Some(ColorSupport::Basic);
    }

    None
}

/// Test `TrueColor` support using OSC 10 sequences
#[allow(clippy::cast_sign_loss, missing_docs)]
fn test_truecolor_support() -> bool {
    // Skip OSC testing for terminals that don't respond to queries but may still support truecolor
    if is_apple_terminal() {
        vprtln!(V::V, "Skipping OSC truecolor test for Apple Terminal");
        return false; // Force fallback to 256-color mode for Apple Terminal
    }

    let timeout = Duration::from_millis(150);
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let result = (|| -> bool {
            if enable_raw_mode().is_err() {
                return false;
            }

            let mut stdout = stdout();
            let mut stdin = stdin();

            // Query original foreground color
            let query = "\x1b]10;?\x07";
            if stdout.write_all(query.as_bytes()).is_err() {
                return false;
            }
            if stdout.flush().is_err() {
                return false;
            }

            let Some(original_color) = read_osc10_response(&mut stdin, timeout) else {
                return false;
            };

            // Set test color
            let test_color = (123u8, 234u8, 45u8);
            let set_cmd = format!(
                "\x1b]10;rgb:{:02x}{:02x}/{:02x}{:02x}/{:02x}{:02x}\x07",
                test_color.0, test_color.0, test_color.1, test_color.1, test_color.2, test_color.2
            );
            if stdout.write_all(set_cmd.as_bytes()).is_err() {
                return false;
            }
            if stdout.flush().is_err() {
                return false;
            }

            thread::sleep(Duration::from_millis(20));

            // Query it back
            if stdout.write_all(query.as_bytes()).is_err() {
                return false;
            }
            if stdout.flush().is_err() {
                return false;
            }

            let Some(queried_color) = read_osc10_response(&mut stdin, timeout) else {
                return false;
            };

            // Restore original color
            let restore_cmd = format!(
                "\x1b]10;rgb:{:02x}{:02x}/{:02x}{:02x}/{:02x}{:02x}\x07",
                original_color.0,
                original_color.0,
                original_color.1,
                original_color.1,
                original_color.2,
                original_color.2
            );
            if stdout.write_all(restore_cmd.as_bytes()).is_err() {
                return false;
            }
            if stdout.flush().is_err() {
                return false;
            }

            // Check if colors match (within tolerance)
            let distance = ((i16::from(test_color.0) - i16::from(queried_color.0)).abs()
                + (i16::from(test_color.1) - i16::from(queried_color.1)).abs()
                + (i16::from(test_color.2) - i16::from(queried_color.2)).abs())
                as u16;

            distance <= 50
        })();

        let _ = disable_raw_mode();
        let _ = tx.send(result);
    });

    rx.recv_timeout(timeout + Duration::from_millis(100))
        .is_ok_and(|result| {
            let _ = handle.join();
            result
        })
}

/// Read OSC 10 response from stdin
fn read_osc10_response(stdin: &mut std::io::Stdin, timeout: Duration) -> Option<(u8, u8, u8)> {
    let mut buffer = Vec::new();
    let mut temp_buffer = [0u8; 1];
    let start = Instant::now();

    while start.elapsed() < timeout {
        match stdin.read(&mut temp_buffer) {
            Ok(1..) => {
                buffer.push(temp_buffer[0]);

                if buffer.len() >= 20 {
                    let response = String::from_utf8_lossy(&buffer);
                    if response.contains('\x07') || response.contains("\x1b\\") {
                        if let Some(rgb) = parse_osc10_response(&response) {
                            return Some(rgb);
                        }
                    }
                }

                if buffer.len() > 512 {
                    break;
                }
            }
            Ok(0) => thread::sleep(Duration::from_millis(1)),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
            }
            Err(_) => break,
        }
    }

    None
}

/// Parse OSC 10 response to extract RGB values
fn parse_osc10_response(response: &str) -> Option<(u8, u8, u8)> {
    if let Some(start_pos) = response.find("\x1b]10;") {
        let response_part = &response[start_pos..];

        if let Some(rgb_pos) = response_part.find("rgb:") {
            let rgb_data = &response_part[rgb_pos..];

            let end_pos = rgb_data
                .find('\x07')
                .or_else(|| rgb_data.find('\x1b'))
                .unwrap_or(rgb_data.len());

            if end_pos >= 18 {
                let rgb_sequence = &rgb_data[4..end_pos];
                let parts: Vec<&str> = rgb_sequence.split('/').collect();

                if parts.len() == 3
                    && parts[0].len() == 4
                    && parts[1].len() == 4
                    && parts[2].len() == 4
                    && parts
                        .iter()
                        .all(|part| part.chars().all(|c| c.is_ascii_hexdigit()))
                {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u16::from_str_radix(parts[0], 16).map(|v| (v >> 8) as u8),
                        u16::from_str_radix(parts[1], 16).map(|v| (v >> 8) as u8),
                        u16::from_str_radix(parts[2], 16).map(|v| (v >> 8) as u8),
                    ) {
                        return Some((r, g, b));
                    }
                }
            }
        }
    }

    None
}

/// Check if terminal supports 256 colors based on TERM variable
fn supports_256_color() -> bool {
    std::env::var("TERM").is_ok_and(|term| term.ends_with("256color") || term.ends_with("256"))
}

/// Restore the raw or cooked terminal status as saved in the boolean argument.
///
/// # Errors
///
/// This function will bubble up any errors returned by `crossterm`.
pub fn restore_raw_status(raw_before: bool) -> ThagCommonResult<()> {
    if raw_before {
        enable_raw_mode()?;
    } else {
        disable_raw_mode()?;
    }
    Ok(())
}
