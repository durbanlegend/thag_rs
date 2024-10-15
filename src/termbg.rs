// #![cfg(not(target_os = "windows"))]
use crossterm::event::{poll, read, Event, KeyCode};
/// Original is `https://github.com/dalance/termbg/blob/master/src/lib.rs`
/// Copyright (c) 2019 dalance
/// Licence: Apache or MIT
use crossterm::terminal;
use log::debug;
use std::io::{self, IsTerminal, Write};
use std::time::{Duration, Instant};
use std::{env, thread};
#[cfg(target_os = "windows")]
// use win32console::console::{ConsoleMode, WinConsole};

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

    // if let Ok(term_program) = env::var("TERM_PROGRAM") {
    //     debug!("term_program={term_program}");
    //     if term_program == "vscode" {
    //         return Terminal::XtermCompatible;
    //     }
    // }

    if env::var("INSIDE_EMACS").is_ok() {
        return Terminal::Emacs;
    }

    // Windows Terminal is Xterm-compatible
    // https://github.com/microsoft/terminal/issues/3718
    // if env::var("WT_SESSION").is_ok() {
    //     debug!("Found $WT_SESSION");
    //     Terminal::XtermCompatible
    // } else {
        debug!("Terminal::Windows");
        Terminal::Windows
    // }
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
        // Terminal::XtermCompatible => from_xterm(term, timeout),
        _ => from_winapi(),
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

// fn from_xterm(term: Terminal, timeout: Duration) -> ThagResult<Rgb> {
//     if !std::io::stdin().is_terminal()
//         || !std::io::stdout().is_terminal()
//         || !std::io::stderr().is_terminal()
//     {
//         // Not a terminal, so don't try to read the current background color.
//         return Err(ThagError::UnsupportedTerm);
//     }

//     let mut stderr = io::stderr();

//     #[cfg(target_os = "windows")]
//     {
//         let old_mode = WinConsole::input().get_mode()?;
//         log::debug!("old_mode={old_mode:x?}");
//         let new_mode = old_mode | ConsoleMode::ENABLE_VIRTUAL_TERMINAL_INPUT;
//         // We change the input mode so the characters are not displayed
//         let _ = WinConsole::input().set_mode(new_mode)?;
//         let check_new_mode = WinConsole::input().get_mode()?;
//         log::debug!("check_new_mode={check_new_mode:x?}");

//         // Try some Set Graphics Rendition (SGR) terminal escape sequences
//         writeln!(stderr, "\x1b[31mThis text has a red foreground using SGR.31.")?;
//         writeln!(stderr, "\x1b[1mThis text has a bright (bold) red foreground using SGR.1 to affect the previous color setting.")?;
//         writeln!(stderr, "\x1b[mThis text has returned to default colors using SGR.0 implicitly.")?;
//         writeln!(stderr,
//             "\x1b[34;46mThis text shows the foreground and background change at the same time."
//         )?;
//         writeln!(stderr, "\x1b[0mThis text has returned to default colors using SGR.0 explicitly.")?;
//         writeln!(stderr, "\x1b[31;32;33;34;35;36;101;102;103;104;105;106;107mThis text attempts to apply many colors in the same command. Note the colors are applied from left to right so only the right-most option of foreground cyan (SGR.36) and background bright white (SGR.107) is effective.")?;
//         writeln!(stderr, "\x1b[39mThis text has restored the foreground color only.")?;
//         writeln!(stderr, "\x1b[49mThis text has restored the background color only.")?;
//     }

//     debug!("term={term:?}");

//     // Query by XTerm control sequence
//     let query = match term {
//         Terminal::Tmux => "\x1bPtmux;\x1b\x1b]11;?\x07\x1b\\\x03",
//         Terminal::Screen => "\x1bP\x1b]11;?\x07\x1b\\\x03",
//         _ => "\x1b]11;?\x1b\\",
//     };

//     // Conditionally avoid raw mode on Windows
//     let raw_before = terminal::is_raw_mode_enabled()?;
//     if !raw_before && !cfg!(target_os = "windows") {
//         terminal::enable_raw_mode()?;
//     }

//     // Send query to terminal
//     writeln!(stderr, "{query}")?;
//     stderr.flush()?;

//     let start_time = Instant::now();
//     let mut response = String::new(); // Store the response as a String
//     let mut rgb_start = false; // Flag to check when rgb: is encountered
//     let mut parsing_rgb = false; // Flag to track if we're reading the color value

//     // Adjust timeout for Windows (if needed)
//     let timeout = if cfg!(target_os = "windows") {
//         Duration::from_secs(1) // Longer timeout for Windows terminals
//     } else {
//         timeout
//     };

//     // Use blocking I/O with a timeout loop
//     loop {
//         // Check for timeout
//         if start_time.elapsed() > timeout {
//             clear_stdin()?;
//             debug!("timed out!");
//             return Err(io::Error::new(io::ErrorKind::TimedOut, "timeout").into());
//         }

//         // Use crossterm's poll to wait for input events with a timeout
//         if poll(Duration::from_millis(10))? {
//             // Read the next event (non-blocking)
//             if let Event::Key(key_event) = read()? {
//                 match key_event.code {
//                     KeyCode::Char(c) => {
//                         // Accumulate characters
//                         response.push(c);
//                         debug!("char={c}; response={response}, rgb_start={rgb_start}");

//                         // Start reading once we see "rgb:"
//                         if response.ends_with("rgb:") {
//                             rgb_start = true;
//                             parsing_rgb = true;
//                         }

//                         // Keep parsing until the '\'
//                         if parsing_rgb && c == '\\' {
//                             debug!("break, hooray!");
//                             break;
//                         }
//                     }
//                     KeyCode::Esc => {
//                         // Manually push the ESC (0x1b) into the response
//                         // response.push(0x1b as char);
//                         debug!("Esc!: response={response}, rgb_start={rgb_start}");
//                     }
//                     _ => {}
//                 }
//             }
//         } else {
//             // Small sleep to avoid busy-waiting
//             thread::sleep(Duration::from_millis(10));
//         }
//     }

//     // Discard unwanted characters left in stdin
//     clear_stdin()?;

//     // Restore raw mode state if necessary
//     if !raw_before && !cfg!(target_os = "windows") {
//         terminal::disable_raw_mode()?;
//     }

//     // Extract the RGB value after 'rgb:'
//     if let Some(start_idx) = response.find("rgb:") {
//         let color_value = &response[start_idx + 4..]; // Get the color string
//                                                       // Ok(color_value.to_string())
//                                                       // Convert the collected buffer into a string and parse it
//                                                       // let color_value = String::from_utf8_lossy(&buffer);
//         let (r, g, b) = decode_x11_color(&color_value)?;
//         Ok(Rgb { r, g, b })
//     } else {
//         Err("RGB color value not found".into())
//     }
// }

#[allow(dead_code)]
fn from_xterm(term: Terminal, timeout: Duration) -> ThagResult<Rgb> {
    if !std::io::stdin().is_terminal()
        || !std::io::stdout().is_terminal()
        || !std::io::stderr().is_terminal()
    {
        return Err(ThagError::UnsupportedTerm);
    }

    let mut stderr = io::stderr();

    // #[cfg(target_os = "windows")]
    // {
    //     let old_mode = WinConsole::input().get_mode()?;
    //     let new_mode = old_mode | ConsoleMode::ENABLE_VIRTUAL_TERMINAL_INPUT;
    //     WinConsole::input().set_mode(new_mode)?;

    //     writeln!(stderr, "\x1b[31mTesting SGR sequences...\x1b[0m")?;
    //     // ... (your other sequences for Windows testing)
    //     // Try some Set Graphics Rendition (SGR) terminal escape sequences
    //     writeln!(
    //         stderr,
    //         "\x1b[31mThis text has a red foreground using SGR.31."
    //     )?;
    //     writeln!(stderr, "\x1b[1mThis text has a bright (bold) red foreground using SGR.1 to affect the previous color setting.")?;
    //     writeln!(
    //         stderr,
    //         "\x1b[mThis text has returned to default colors using SGR.0 implicitly."
    //     )?;
    //     writeln!(
    //         stderr,
    //         "\x1b[34;46mThis text shows the foreground and background change at the same time."
    //     )?;
    //     writeln!(
    //         stderr,
    //         "\x1b[0mThis text has returned to default colors using SGR.0 explicitly."
    //     )?;
    //     writeln!(stderr, "\x1b[31;32;33;34;35;36;101;102;103;104;105;106;107mThis text attempts to apply many colors in the same command. Note the colors are applied from left to right so only the right-most option of foreground cyan (SGR.36) and background bright white (SGR.107) is effective.")?;
    //     writeln!(
    //         stderr,
    //         "\x1b[39mThis text has restored the foreground color only."
    //     )?;
    //     writeln!(
    //         stderr,
    //         "\x1b[49mThis text has restored the background color only."
    //     )?;
    // }

    // Query by XTerm control sequence
    let query = match term {
        Terminal::Tmux => "\x1bPtmux;\x1b\x1b]11;?\x07\x1b\\",
        Terminal::Screen => "\x1bP\x1b]11;?\x07\x1b\\",
        _ => "\x1b]11;?\x1b\\",
    };

    let raw_before = terminal::is_raw_mode_enabled()?;
    if !raw_before && !cfg!(target_os = "windows") {
        terminal::enable_raw_mode()?;
    }

    // Send query
    writeln!(stderr, "{query}")?;
    stderr.flush()?;

    let start_time = Instant::now();
    let mut response = String::new();

    let timeout = if cfg!(target_os = "windows") {
        Duration::from_secs(1) // Adjust as needed
    } else {
        timeout
    };

    // Main loop for capturing terminal response
    loop {
        if start_time.elapsed() > timeout {
            clear_stdin()?; // Ensure stdin is cleared on timeout
            return Err(io::Error::new(io::ErrorKind::TimedOut, "timeout").into());
        }

        if poll(Duration::from_millis(10))? {
            if let Event::Key(key_event) = read()? {
                if let KeyCode::Char(c) = key_event.code {
                    response.push(c);

                    if response.ends_with("rgb:") {
                        // Start parsing the RGB color value
                        let color_value = response.split_off(response.find("rgb:").unwrap() + 4);
                        if let Some(rgb_string) = color_value.split('\\').next() {
                            let (r, g, b) = decode_x11_color(rgb_string)?;
                            if !raw_before && !cfg!(target_os = "windows") {
                                terminal::disable_raw_mode()?;
                            }

                            // Err("RGB color value not found".into())
                            return Ok(Rgb { r, g, b });
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}

/// Interrogates an xterm terminal, which may interfere with the user's terminal interaction,
/// especially on Windows which is why I've updated the logic to abandon stdin after.
///
/// # Errors
///
/// This function will return an error if Rust has decided that the "terminal" is not a terminal.
// Helper function to discard extra characters
#[allow(dead_code)]
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

/// Decodes an X11 colour.
///
/// # Errors
///
/// This function will return a `FromStr` error if it fails to parse a hex colour code.
#[allow(dead_code)]
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
