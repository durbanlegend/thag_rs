/// Original is `https://github.com/dalance/termbg/blob/master/src/lib.rs`
/// Copyright (c) 2019 dalance
/// Licence: Apache or MIT
use crate::{ThagError, ThagResult};
use crossterm::event::{self, poll, read, Event, KeyCode};
use crossterm::terminal::{self, is_raw_mode_enabled};
use log::debug;
use scopeguard::defer;
use std::env;
use std::io::{self, IsTerminal, Write};
use std::time::{Duration, Instant};
#[cfg(target_os = "windows")]
use {
    std::sync::OnceLock, winapi::um::consoleapi::SetConsoleMode,
    winapi::um::handleapi::INVALID_HANDLE_VALUE, winapi::um::processenv::GetStdHandle,
    winapi::um::winbase::STD_OUTPUT_HANDLE, winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING,
};

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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

/// Background theme
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

    if env::var("TMUX").is_ok() || env::var("TERM").is_ok_and(|x| x.starts_with("tmux-")) {
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

/// get detected terminal
#[cfg(target_os = "windows")]
pub fn terminal() -> Terminal {
    use log::debug;

    // As of 2024-10-16, only Windows Terminal 1.22 (preview) supports *querying*
    // rgb values. Since xterm OSC is MS's roadmap, I'm leaving this in for when
    // VS Code hopefully follows suit. But right now it will time out
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        debug!("term_program={term_program}");
        if term_program == "vscode" {
            return Terminal::XtermCompatible;
        }
    }

    if env::var("INSIDE_EMACS").is_ok() {
        return Terminal::Emacs;
    }

    // Windows Terminal is Xterm-compatible
    // https://github.com/microsoft/terminal/issues/3718.
    // But this excludes OSC 10/11 colour queries until Windows Terminal 1.22
    // https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release/:
    // "Applications can now query ... the default foreground (OSC 10 ?) [and] background (OSC 11 ?)"
    // Don't use WT_SESSION for this purpose:
    // https://github.com/Textualize/rich/issues/140
    // if env::var("WT_SESSION").is_ok() {
    if enable_virtual_terminal_processing() {
        debug!(
            r#"This Windows terminal supports virtual terminal processing
(but not OSC 10/11 colour queries if prior to Windows Terminal 1.22 Preview of August 2024)"#
        );
        Terminal::XtermCompatible
    } else {
        debug!("Terminal::Windows");
        Terminal::Windows
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
#[cfg(target_os = "windows")]
pub fn rgb(_timeout: Duration) -> Result<Rgb, ThagError> {
    let term = terminal();
    let rgb = match term {
        Terminal::Emacs => Err(ThagError::UnsupportedTerm),
        Terminal::XtermCompatible => from_xterm(term, timeout), // will time out pre Windows Terminal 1.22
        _ => from_winapi(), // effectively useless unless set via legacy Console
    };
    let fallback = from_env_colorfgbg();
    debug!("rgb={rgb:?}, fallback={fallback:?}");
    if rgb.is_ok() {
        rgb
    } else if fallback.is_ok() {
        fallback
    } else {
        rgb
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

// Function to enable virtual terminal processing for Windows
#[cfg(target_os = "windows")]
fn enable_virtual_terminal_processing() -> bool {
    static ENABLE_VT_PROCESSING: OnceLock<bool> = OnceLock::new();
    ENABLE_VT_PROCESSING.get_or_init(|| unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle != INVALID_HANDLE_VALUE {
            let mut mode: u32 = 0;
            if winapi::um::consoleapi::GetConsoleMode(handle, &mut mode) != 0 {
                // Try to set virtual terminal processing mode
                if SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0 {
                    // Success in enabling VT
                    return true;
                } else {
                    // Failed to enable VT, optionally log error
                    eprintln!("Failed to enable Virtual Terminal Processing.");
                }
            }
        }
        // Return false if enabling VT failed
        false
    })
}

fn from_xterm(term: Terminal, timeout: Duration) -> ThagResult<Rgb> {
    if !std::io::stdin().is_terminal()
        || !std::io::stdout().is_terminal()
        || !std::io::stderr().is_terminal()
    {
        return Err(ThagError::UnsupportedTerm);
    }

    let raw_before = is_raw_mode_enabled()?;

    defer! {
        let is_raw = match is_raw_mode_enabled() {
            Ok(val) => val,
            Err(e) => {
                debug!("Failed to check raw mode status: {:?}", e);
                return;
            }
        };

        if is_raw == raw_before {
            debug!("Raw mode status unchanged.");
        } else if let Err(e) = restore_raw_status(raw_before) {
            debug!("Failed to restore raw mode: {e:?}");
        } else {
            debug!("Raw mode restored to previous state.");
        }

        if let Err(e) = clear_stdin() {
            debug!("Failed to clear stdin: {e:?}");
        } else {
            debug!("Cleared any excess from stdin.");
        }
    }

    if !raw_before {
        terminal::enable_raw_mode()?;
    }

    let mut stderr = io::stderr();

    #[cfg(target_os = "windows")]
    {
        if !enable_virtual_terminal_processing() {
            debug!(
                "Virtual Terminal Processing could not be enabled. Falling back to default behavior."
            );
            return from_winapi();
        }
    }

    // Query by XTerm control sequence
    let query = match term {
        Terminal::Tmux => "\x1bPtmux;\x1b\x1b]11;?\x07\x1b\\",
        Terminal::Screen => "\x1bP\x1b]11;?\x07\x1b\\",
        _ => "\x1b]11;?\x1b\\",
    };

    // Send query
    writeln!(stderr, "{query}")?;
    stderr.flush()?;

    let start_time = Instant::now();
    let mut response = String::new();

    let timeout = if cfg!(target_os = "windows") {
        Duration::from_secs(2) // Adjust as needed
    } else {
        timeout
    };

    // Main loop for capturing terminal response
    loop {
        if start_time.elapsed() > timeout {
            debug!("Failed to capture response");
            return Err(io::Error::new(io::ErrorKind::TimedOut, "timeout").into());
        }

		// Replaced expensive async_std with blocking loop. Terminal normally responds
		// fast or not at all, and in the latter case we still have the timeout on the
		// main loop.
        if poll(Duration::from_millis(100))? {
            // Read the next event.
            // Replaced stdin read that was consuming legit user input in Windows
            // with non-blocking crossterm read event.
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    // End on backslash character
                    KeyCode::Char('\\') => {
                        debug!("End of response detected (backslash character).");
                        // response.push('\\');
                        let rgb_string = response.split_off(response.find("rgb:").unwrap() + 4);
                        let (r, g, b) = decode_x11_color(&rgb_string)?;

                        // Err("RGB color value not found".into())
                        // Print the duration it took to capture the response
                        let elapsed = start_time.elapsed();
                        debug!("Elapsed time: {:.2?}", elapsed);

                        return Ok(Rgb { r, g, b });
                    }
                    // Append other characters to buffer
                    KeyCode::Char(c) => {
                        // debug!("pushing {c}");
                        response.push(c);
                    }
                    _ => {
                        // Ignore other keys
                    }
                }
            }
        }
    }
}

fn restore_raw_status(raw_before: bool) -> Result<(), ThagError> {
    let raw_now = is_raw_mode_enabled()?;
    if raw_now == raw_before {
        return Ok(());
    }
    if raw_before {
        terminal::enable_raw_mode()?;
    } else {
        terminal::disable_raw_mode()?;
    }
    Ok(())
}

/// Discard any unread input returned by the OSC 11 query.
///
/// # Errors
///
/// This function will return an error if Rust has decided that the "terminal" is not a terminal.
// Helper function to discard extra characters
fn clear_stdin() -> Result<(), Box<dyn std::error::Error>> {
    // let mut buf = [0; 1];
    while poll(Duration::from_millis(10))? {
        if let Event::Key(c) = read()? {
            // Discard the input by simply reading it
            debug!("discarding char{c:x?}");
        }
    }
    Ok(())
}

/// Seems to be for Rxvt terminal emulator only.
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

/// Try to determine the background colour from the lagacy Windows Console interface.
/// Unfortunately, unless the colour was explicitly set by that interface, it will
/// just return the default of rgb(0,0,0). This renders it effectively useless for
/// modern Windows.
///
/// # Errors
///
/// This function will bubble up any errors returned by the Windows API.
#[cfg(target_os = "windows")]
fn from_winapi() -> Result<Rgb, ThagError> {
    use winapi::um::wincon;

    debug!("In from_winapi()");
    let info = unsafe {
        let handle = winapi::um::processenv::GetStdHandle(winapi::um::winbase::STD_OUTPUT_HANDLE);
        let mut info: wincon::CONSOLE_SCREEN_BUFFER_INFO = Default::default();
        wincon::GetConsoleScreenBufferInfo(handle, &mut info);
        info
    };

    debug!("info.wAttributes={:x?}", info.wAttributes);

    let r = (wincon::BACKGROUND_RED & info.wAttributes) != 0;
    let g = (wincon::BACKGROUND_GREEN & info.wAttributes) != 0;
    let b = (wincon::BACKGROUND_BLUE & info.wAttributes) != 0;
    let i = (wincon::BACKGROUND_INTENSITY & info.wAttributes) != 0;

    let r: u8 = r as u8;
    let g: u8 = g as u8;
    let b: u8 = b as u8;
    let i: u8 = i as u8;

    let (r, g, b) = match (r, g, b, i) {
        (0, 0, 0, 0) => (0, 0, 0),
        (1, 0, 0, 0) => (128, 0, 0),
        (0, 1, 0, 0) => (0, 128, 0),
        (1, 1, 0, 0) => (128, 128, 0),
        (0, 0, 1, 0) => (0, 0, 128),
        (1, 0, 1, 0) => (128, 0, 128),
        (0, 1, 1, 0) => (0, 128, 128),
        (1, 1, 1, 0) => (192, 192, 192),
        (0, 0, 0, 1) => (128, 128, 128),
        (1, 0, 0, 1) => (255, 0, 0),
        (0, 1, 0, 1) => (0, 255, 0),
        (1, 1, 0, 1) => (255, 255, 0),
        (0, 0, 1, 1) => (0, 0, 255),
        (1, 0, 1, 1) => (255, 0, 255),
        (0, 1, 1, 1) => (0, 255, 255),
        (1, 1, 1, 1) => (255, 255, 255),
        _ => unreachable!(),
    };

    Ok(Rgb {
        r: r * 256,
        g: g * 256,
        b: b * 256,
    })
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
