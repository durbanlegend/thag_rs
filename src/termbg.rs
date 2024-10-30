/// Original is `https://github.com/dalance/termbg/blob/master/src/lib.rs`
/// Copyright (c) 2019 dalance
/// Licence: Apache or MIT
use crate::{debug_log, CrosstermEventReader, EventReader, ThagError, ThagResult};
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyModifiers},
    terminal::{self, is_raw_mode_enabled},
};
use scopeguard::defer;
use std::{
    env,
    fmt::Debug,
    io::{self, IsTerminal, Write},
    time::{Duration, Instant},
};
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
        debug_log!("term_program={term_program}");
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
        debug_log!(
            r#"This Windows terminal supports virtual terminal processing
(but not OSC 10/11 colour queries if prior to Windows Terminal 1.22 Preview of August 2024)"#
        );
        Terminal::XtermCompatible
    } else {
        debug_log!("Terminal::Windows");
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
    let event_reader = CrosstermEventReader;
    let mut stderr = io::stderr();

    let rgb = match term {
        Terminal::Emacs => Err(ThagError::UnsupportedTerm),
        _ => from_xterm(term, timeout, &event_reader, &mut stderr),
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
pub fn rgb(timeout: Duration) -> Result<Rgb, ThagError> {
    let term = terminal();
    let rgb = match term {
        Terminal::Emacs => Err(ThagError::UnsupportedTerm),
        Terminal::XtermCompatible => {
            let event_reader = CrosstermEventReader;
            let mut stderr = io::stderr();

            from_xterm(term, timeout, &event_reader, &mut stderr)
        } // will time out pre Windows Terminal 1.22
        _ => from_winapi(), // effectively useless unless set via legacy Console
    };
    let fallback = from_env_colorfgbg();
    debug_log!("rgb={rgb:?}, fallback={fallback:?}");
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
    *ENABLE_VT_PROCESSING.get_or_init(|| unsafe {
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

fn from_xterm<R, W>(
    term: Terminal,
    timeout: Duration,
    event_reader: &R,
    buffer: &mut W,
) -> ThagResult<Rgb>
where
    R: EventReader + Debug,
    W: Write + Debug,
{
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
                debug_log!("Failed to check raw mode status: {:?}", e);
                return;
            }
        };

        if is_raw == raw_before {
            debug_log!("Raw mode status unchanged.");
        } else if let Err(e) = restore_raw_status(raw_before) {
            debug_log!("Failed to restore raw mode: {e:?}");
        } else {
            debug_log!("Raw mode restored to previous state.");
        }

        if let Err(e) = clear_stdin() {
            debug_log!("Failed to clear stdin: {e:?}");
        } else {
            debug_log!("Cleared any excess from stdin.");
        }
    }

    if !raw_before {
        terminal::enable_raw_mode()?;
    }

    #[cfg(target_os = "windows")]
    {
        if !enable_virtual_terminal_processing() {
            debug_log!(
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
    write!(buffer, "{query}")?;
    buffer.flush()?;

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
            if &response != "r" {
                dbg!(&response[..50]);
            }
            dbg!(&response);
            if response.contains("rgb:") {
                let rgb_string = response.split_off(response.find("rgb:").unwrap() + 4);
                if rgb_string.chars().filter(|&c| c == '/').count() == 2
                    && rgb_string.split('/').all(|frag| !frag.is_empty())
                {
                    debug_log!("Unrecognized terminator in response code {response:#?}, but found a valid response in pre-timeout check");
                    return parse_response(&mut response, start_time);
                }
            } else {
                debug_log!("Failed to capture response");
                return Err(io::Error::new(io::ErrorKind::TimedOut, "timeout").into());
            }
        }

        // Replaced expensive async_std with blocking loop. Terminal normally responds
        // fast or not at all, and in the latter case we still have the timeout on the
        // main loop.
        if event_reader.poll(Duration::from_millis(100))? {
            // Read the next event.
            // Replaced stdin read that was consuming legit user input in Windows
            // with non-blocking crossterm read event.
            if let Event::Key(key_event) = event_reader.read_event()? {
                // if &key_event.code != &KeyCode::Char('r') {
                //     dbg!(&key_event);
                // }
                debug_log!("key_event={key_event:#?}");
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Char('\\'), KeyModifiers::ALT)   // ST
                    | (KeyCode::Char('g'), KeyModifiers::CONTROL)   // BEL
                    // Insurance in case BEL is not recognosed as ^g
                    | (KeyCode::Char('\u{0007}'), KeyModifiers::NONE)   //BEL
                    => {
                        debug_log!("End of response detected ({key_event:?}).");
                        // response.push('\\');
                        return parse_response(&mut response, start_time);
                    }
                    // Append other characters to buffer
                    (KeyCode::Char(c), KeyModifiers::NONE) => {
                        debug_log!("pushing {c}");
                        response.push(c);
                    }
                    _ => {
                        // Ignore other keys
                        debug_log!("ignoring {key_event:#?}");
                    }
                }
            }
        }
    }
}

fn parse_response(response: &mut String, start_time: Instant) -> Result<Rgb, ThagError> {
    debug_log!("response=: {response}");
    let (r, g, b) = extract_rgb(response)?;
    let elapsed = start_time.elapsed();
    debug_log!("Elapsed time: {:.2?}", elapsed);
    Ok(Rgb { r, g, b })
}

fn extract_rgb(response: &mut String) -> Result<(u16, u16, u16), ThagError> {
    let rgb_string = response.split_off(response.find("rgb:").unwrap() + 4);
    let (r, g, b) = decode_x11_color(&rgb_string)?;
    Ok((r, g, b))
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
    while poll(Duration::from_millis(10))? {
        if let Event::Key(c) = read()? {
            // Discard the input by simply reading it
            debug_log!("discarding char{c:x?}");
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

/// Try to determine the background colour from the legacy Windows Console interface.
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

    debug_log!("In from_winapi()");
    let info = unsafe {
        let handle = winapi::um::processenv::GetStdHandle(winapi::um::winbase::STD_OUTPUT_HANDLE);
        let mut info: wincon::CONSOLE_SCREEN_BUFFER_INFO = Default::default();
        wincon::GetConsoleScreenBufferInfo(handle, &mut info);
        info
    };

    debug_log!("info.wAttributes={:x?}", info.wAttributes);

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

    fn scale_u16_to_u8(rgb: Rgb) -> Rgb {
        Rgb {
            r: if rgb.r % 256 == 0 { rgb.r >> 8 } else { rgb.r },
            g: if rgb.g % 256 == 0 { rgb.g >> 8 } else { rgb.g },
            b: if rgb.b % 256 == 0 { rgb.b >> 8 } else { rgb.b },
        }
    }

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

    use super::*;
    use crate::MockEventReader;
    use crossterm::event::KeyEvent;
    use mockall::mock;
    use std::io::{self, Write};
    use std::time::Duration;

    // Mock the `Write` trait to use in testing
    mock! {
        #[derive(Debug)]
        Writer {}

        impl Write for Writer {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
            fn flush(&mut self) -> io::Result<()>;
        }
    }

    #[test]
    fn test_from_xterm_successful_rgb_parsing() {
        // Initialize the mocks
        let mut mock_writer = MockWriter::new();
        let mut mock_event_reader = MockEventReader::new();

        // Simulate xterm RGB response string, e.g., "rgb:ff/cc/99"
        const RGB_RESPONSE: &[Event] = &[
            Event::Key(KeyEvent::new(KeyCode::Char(']'), KeyModifiers::ALT)),
            Event::Key(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char(';'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('9'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('9'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL)), // BEL terminator
        ];

        // Set up `mock_writer` to expect a specific query write and succeed
        let query_command = b"\x1b]11;?\x1b\\"; // Corrected to match the observed command in MockWriter
        mock_writer
            .expect_write()
            .withf(move |buf| buf == query_command)
            .times(1)
            .returning(|_| Ok(query_command.len()));
        // Expect flush to be called once, returning Ok
        mock_writer.expect_flush().times(1).returning(|| Ok(()));

        // Mock the event reader behavior
        mock_event_reader.expect_poll().returning(|_| Ok(true)); // Always poll successfully

        // Each call to `read_event` should sequentially return the next `KeyEvent` in rgb_response
        let mut response_iter = RGB_RESPONSE.iter().cloned();
        mock_event_reader.expect_read_event().returning(move || {
            response_iter
                .next()
                .ok_or(io::Error::new(io::ErrorKind::TimedOut, "timeout").into())
        });

        // Invoke `from_xterm` with the mocks
        let terminal = Terminal::XtermCompatible; // Initialize `Terminal` as required
        let timeout = Duration::from_secs(1); // Set timeout as needed

        let result = from_xterm(terminal, timeout, &mock_event_reader, &mut mock_writer);
        println!("result={result:?}");
        assert!(result.is_ok(), "from_xterm did not return an Ok result");
        let actual_rgb = scale_u16_to_u8(result.unwrap());
        println!(
            "actual_rgb hex values: {{ {:x}, {:x}, {:x} }}",
            actual_rgb.r, actual_rgb.g, actual_rgb.b
        );

        // Assert the RGB result matches the expected value
        let expected_rgb = Rgb {
            r: 255,
            g: 204,
            b: 153,
        };
        // Scale the `u16` RGB to `u8` and test
        assert_eq!(actual_rgb, expected_rgb, "RGB values do not match expected");
    }
}
