#![cfg(not(target_os = "windows"))]
use crossterm::event::{poll, read, Event, KeyCode};
/// Original is `https://github.com/dalance/termbg/blob/master/src/lib.rs`
/// Copyright (c) 2019 dalance
/// Licence: Apache or MIT
use crossterm::terminal;
use std::io::{self, IsTerminal, Read, Write};
use std::time::{Duration, Instant};
use std::{env, thread};

use crate::errors::{ThagError, ThagResult};

/// Terminal
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Terminal {
    Screen,
    Tmux,
    XtermCompatible,
    Windows,
    Emacs,
}

/// 16bit RGB color
#[derive(Copy, Clone, Debug)]
pub struct Rgb {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

/// Background theme
#[derive(Copy, Clone, Debug)]
pub enum Theme {
    Light,
    Dark,
}

/// get detected terminal
#[cfg(not(target_os = "windows"))]
#[must_use]
pub fn terminal() -> Terminal {
    if env::var("INSIDE_EMACS").is_ok() {
        return Terminal::Emacs;
    }

    if env::var("TMUX").is_ok() {
        Terminal::Tmux
    } else {
        let is_screen = env::var("TERM").map_or(false, |term| term.starts_with("screen"));
        if is_screen {
            Terminal::Screen
        } else {
            Terminal::XtermCompatible
        }
    }
}

/// get background color by `RGB`
///
/// # Errors
///
/// This function will return an error if the terminal is of type Emacs.
#[cfg(not(target_os = "windows"))]
pub fn rgb(timeout: Duration) -> ThagResult<Rgb> {
    let term = terminal();
    let rgb = match term {
        Terminal::Emacs => Err(ThagError::UnsupportedTerm),
        _ => from_xterm(term, timeout),
    };
    let fallback = from_env_colorfgbg();
    if rgb.is_ok() {
        rgb
    } else if fallback.is_ok() {
        fallback
    } else {
        rgb
    }
}

/// get background color by `RGB`
///
/// # Errors
///
/// This function will bubble up any errors returned by `xterm_latency`.
#[cfg(not(target_os = "windows"))]
pub fn latency(timeout: Duration) -> ThagResult<Duration> {
    let term = terminal();
    match term {
        Terminal::Emacs => Ok(Duration::from_millis(0)),
        _ => xterm_latency(timeout),
    }
}

/// get background color by `Theme`
///
/// # Errors
///
/// This function will bubble up any errors returned by `rgb`.
pub fn theme(timeout: Duration) -> ThagResult<Theme> {
    let rgb = rgb(timeout)?;

    // ITU-R BT.601
    let y = f64::from(rgb.b).mul_add(
        0.114,
        f64::from(rgb.r).mul_add(0.299, f64::from(rgb.g) * 0.587),
    );

    if y > 32768.0 {
        Ok(Theme::Light)
    } else {
        Ok(Theme::Dark)
    }
}

/// Interrogates an xterm terminal, which may interfere with the user's terminal interaction,
/// especially on Windows which is why we avoid it there.
///
/// # Errors
///
/// This function will return an error if Rust has decided that the "terminal" is not a terminal.
fn from_xterm(term: Terminal, timeout: Duration) -> ThagResult<Rgb> {
    if !std::io::stdin().is_terminal()
        || !std::io::stdout().is_terminal()
        || !std::io::stderr().is_terminal()
    {
        // Not a terminal, so don't try to read the current background color.
        return Err(ThagError::UnsupportedTerm);
    }

    // Query by XTerm control sequence
    let query = match term {
        Terminal::Tmux => "\x1bPtmux;\x1b\x1b]11;?\x07\x1b\\\x03",
        Terminal::Screen => "\x1bP\x1b]11;?\x07\x1b\\\x03",
        _ => "\x1b]11;?\x1b\\",
    };

    let mut stderr = io::stderr();

    // Conditionally avoid raw mode on Windows
    let raw_before = terminal::is_raw_mode_enabled()?;
    if !raw_before && !cfg!(target_os = "windows") {
        terminal::enable_raw_mode()?;
    }

    // Send query to terminal
    write!(stderr, "{query}")?;
    stderr.flush()?;

    let start_time = Instant::now();
    let mut response = String::new(); // Store the response as a String
    let mut rgb_start = false; // Flag to check when rgb: is encountered
    let mut parsing_rgb = false; // Flag to track if we're reading the color value

    // Adjust timeout for Windows (if needed)
    let timeout = if cfg!(target_os = "windows") {
        Duration::from_secs(1) // Longer timeout for Windows terminals
    } else {
        timeout
    };

    // Use blocking I/O with a timeout loop
    loop {
        // Check for timeout
        if start_time.elapsed() > timeout {
            log::debug!("timed out!");
            return Err(io::Error::new(io::ErrorKind::TimedOut, "timeout").into());
        }

        // Use crossterm's poll to wait for input events with a timeout
        if poll(Duration::from_millis(10))? {
            // Read the next event (non-blocking)
            if let Event::Key(key_event) = read()? {
                match key_event.code {
                    KeyCode::Char(c) => {
                        // Accumulate characters
                        response.push(c);
                        log::debug!("char={c}; response={response}, rgb_start={rgb_start}");

                        // Start reading once we see "rgb:"
                        if response.ends_with("rgb:") {
                            rgb_start = true;
                            parsing_rgb = true;
                        }

                        // Keep parsing until the '\'
                        if parsing_rgb && c == '\\' {
                            log::debug!("break, hooray!");
                            break;
                        }
                    }
                    KeyCode::Esc => {
                        // Manually push the ESC (0x1b) into the response
                        response.push(0x1b as char);
                        log::debug!("Esc!: response={response}, rgb_start={rgb_start}");
                    }
                    _ => {}
                }
            }
        } else {
            // Small sleep to avoid busy-waiting
            thread::sleep(Duration::from_millis(10));
        }
    }

    // log::debug!(
    //     "buffer={}",
    //     std::str::from_utf8(&buffer).map_err(|e| ThagError::FromStr(e.to_string().into()))?
    // );

    // // Clear any remaining characters from stdin to avoid consuming unintended input
    // while poll(Duration::from_millis(10))? {
    //     if let Event::Key(c) = read()? {
    //         // Discard the characters by reading them but doing nothing with them
    //         log::debug!("discarding char{c:x?}");
    //     }
    // }

    // Discard unwanted characters left in stdin
    clear_stdin()?;

    // Restore raw mode state if necessary
    if !raw_before && !cfg!(target_os = "windows") {
        terminal::disable_raw_mode()?;
    }

    // Extract the RGB value after 'rgb:'
    if let Some(start_idx) = response.find("rgb:") {
        let color_value = &response[start_idx + 4..]; // Get the color string
                                                      // Ok(color_value.to_string())
                                                      // Convert the collected buffer into a string and parse it
                                                      // let color_value = String::from_utf8_lossy(&buffer);
        let (r, g, b) = decode_x11_color(&color_value)?;
        Ok(Rgb { r, g, b })
    } else {
        Err("RGB color value not found".into())
    }

    // // Convert the collected buffer into a string and parse it
    // let s = String::from_utf8_lossy(&buffer);
    // let (r, g, b) = decode_x11_color(&s)?;
    // Ok(Rgb { r, g, b })
}

// Helper function to discard extra characters
fn clear_stdin() -> Result<(), Box<dyn std::error::Error>> {
    // let mut buf = [0; 1];
    while poll(Duration::from_millis(10))? {
        if let Event::Key(c) = read()? {
            // Discard the input by simply reading it
            log::debug!("discarding char{c:x?}");
        }
    }
    Ok(())
}

/// .
///
/// # Errors
///
/// This function will return an `UnsupportedTerm` error if there is no environment variable `COLORFGBG`,
/// or a `FromStr` error if the value of that variable can not be parsed into integers.
fn from_env_colorfgbg() -> ThagResult<Rgb> {
    let var = env::var("COLORFGBG").map_err(|_| ThagError::UnsupportedTerm)?;
    let fgbg: Vec<_> = var.split(';').collect();
    let bg = fgbg.get(1).ok_or(ThagError::UnsupportedTerm)?;
    let bg = bg
        .parse::<u8>()
        .map_err(|_| ThagError::FromStr(var.into()))?;

    // rxvt default color table
    #[allow(clippy::match_same_arms)]
    let (r, g, b) = match bg {
        // black
        0 => (0, 0, 0),
        // red
        1 => (205, 0, 0),
        // green
        2 => (0, 205, 0),
        // yellow
        3 => (205, 205, 0),
        // blue
        4 => (0, 0, 238),
        // magenta
        5 => (205, 0, 205),

        // cyan
        6 => (0, 205, 205),
        // white
        7 => (229, 229, 229),
        // bright black
        8 => (127, 127, 127),
        // bright red
        9 => (255, 0, 0),
        // bright green
        10 => (0, 255, 0),
        // bright yellow
        11 => (255, 255, 0),
        // bright blue
        12 => (92, 92, 255),
        // bright magenta
        13 => (255, 0, 255),
        // bright cyan
        14 => (0, 255, 255),

        // bright white
        15 => (255, 255, 255),
        _ => (0, 0, 0),
    };

    Ok(Rgb {
        r: r * 256,
        g: g * 256,
        b: b * 256,
    })
}

/// Measures the latency of the xterm terminal.
///
/// # Errors
///
/// This function will return an error if it encounters a `crossterm` error or a timeout.
fn xterm_latency(timeout: Duration) -> ThagResult<Duration> {
    // Query by XTerm control sequence
    let query = "\x1b[5n";

    let mut stderr = io::stderr();

    // Don F: Ensure we don't interfere with the raw or cooked mode of the terminal
    let raw_before = terminal::is_raw_mode_enabled()?;
    if !raw_before {
        terminal::enable_raw_mode()?;
    }

    write!(stderr, "{query}")?;
    stderr.flush()?;

    let start = Instant::now();
    let mut stdin = io::stdin();
    let mut buf = [0; 1];

    // Manual timeout handling
    loop {
        // Check for timeout
        if start.elapsed() > timeout {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "timeout").into());
        }

        // Try reading from stdin (non-blocking with a small delay)
        if stdin.read_exact(&mut buf).is_ok() {
            // Response terminated by 'n'
            if buf[0] == b'n' {
                break;
            }
        } else {
            // Sleep for a short time to avoid busy waiting
            thread::sleep(Duration::from_millis(10));
        }
    }

    let end = start.elapsed();

    // Don F: Ensure we don't interfere with the raw or cooked mode of the terminal
    if !raw_before {
        terminal::disable_raw_mode()?;
    }

    Ok(end)
}

/// Decodes an X11 colour.
///
/// # Errors
///
/// This function will return a `FromStr` error if it fails to parse a hex colour code.
fn decode_x11_color(s: &str) -> ThagResult<(u16, u16, u16)> {
    fn decode_hex(s: &str) -> ThagResult<u16> {
        let len = s.len();
        let mut ret =
            u16::from_str_radix(s, 16).map_err(|_| ThagError::FromStr(String::from(s).into()))?;
        ret <<= (4 - len) * 4;
        Ok(ret)
    }

    let rgb: Vec<_> = s.split('/').collect();

    let r = rgb
        .first()
        .ok_or_else(|| ThagError::FromStr(String::from(s).into()))?;
    let g = rgb
        .get(1)
        .ok_or_else(|| ThagError::FromStr(String::from(s).into()))?;
    let b = rgb
        .get(2)
        .ok_or_else(|| ThagError::FromStr(String::from(s).into()))?;
    let r = decode_hex(r)?;
    let g = decode_hex(g)?;
    let b = decode_hex(b)?;

    Ok((r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_x11_color() {
        let s = "0000/0000/0000";
        assert_eq!((0, 0, 0), decode_x11_color(s).unwrap());

        let s = "1111/2222/3333";
        assert_eq!((0x1111, 0x2222, 0x3333), decode_x11_color(s).unwrap());

        let s = "111/222/333";
        assert_eq!((0x1110, 0x2220, 0x3330), decode_x11_color(s).unwrap());

        let s = "11/22/33";
        assert_eq!((0x1100, 0x2200, 0x3300), decode_x11_color(s).unwrap());

        let s = "1/2/3";
        assert_eq!((0x1000, 0x2000, 0x3000), decode_x11_color(s).unwrap());
    }
}
