/*[toml]
[dependencies]
crossterm = "0.29"
log = "0.4.22"
winapi = { version = "0.3.9", features = ["consoleapi", "processenv", "winbase"] }
*/

/// Original is `https://github.com/dalance/termbg/blob/master/src/lib.rs`
/// Copyright (c) 2019 dalance
/// Licence: Apache or MIT
// #![cfg(target_os = "windows")]
use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal;
use log::debug;
use std::io::{self, IsTerminal, Write};
use std::time::{Duration, Instant};
use std::{env, thread};
use winapi::um::wincon;

/// 16bit RGB color
#[derive(Copy, Clone, Debug)]
pub struct Rgb {
pub r: u16,
pub g: u16,
pub b: u16,
}

#[cfg(target_os = "windows")]
fn set_console_color() {
    use winapi::um::{wincon, processenv, winbase};

    unsafe {
        let handle = processenv::GetStdHandle(winbase::STD_OUTPUT_HANDLE);
        // Set the background to blue, for instance
        wincon::SetConsoleTextAttribute(handle, wincon::BACKGROUND_BLUE);
    }
}

#[cfg(target_os = "windows")]
fn from_winapi() -> Result<Rgb, Box<dyn Error>> {
    use winapi::um::wincon;

    debug!("In from_winapi()");
    let info = unsafe {
        let handle = winapi::um::processenv::GetStdHandle(winapi::um::winbase::STD_OUTPUT_HANDLE);
        let mut info: wincon::CONSOLE_SCREEN_BUFFER_INFO = Default::default();
        wincon::GetConsoleScreenBufferInfo(handle, &mut info);
        info
    };

    debug!("info.wAttributes={:x?}", info.wAttributes);

    // Extract the RGB flags and intensity flag from wAttributes
    let r = (wincon::BACKGROUND_RED & info.wAttributes) != 0;
    let g = (wincon::BACKGROUND_GREEN & info.wAttributes) != 0;
    let b = (wincon::BACKGROUND_BLUE & info.wAttributes) != 0;
    let i = (wincon::BACKGROUND_INTENSITY & info.wAttributes) != 0;

    // Cast the booleans to u8 values for further processing
    let r: u8 = r as u8;
    let g: u8 = g as u8;
    let b: u8 = b as u8;
    let i: u8 = i as u8;

    // Use intensity to adjust the final color values
    let (r, g, b) = match (r, g, b, i) {
        (0, 0, 0, 0) => (0, 0, 0),           // Black
        (1, 0, 0, 0) => (128, 0, 0),         // Dark Red
        (0, 1, 0, 0) => (0, 128, 0),         // Dark Green
        (1, 1, 0, 0) => (128, 128, 0),       // Dark Yellow
        (0, 0, 1, 0) => (0, 0, 128),         // Dark Blue
        (1, 0, 1, 0) => (128, 0, 128),       // Dark Magenta
        (0, 1, 1, 0) => (0, 128, 128),       // Dark Cyan
        (1, 1, 1, 0) => (192, 192, 192),     // Light Gray
        (0, 0, 0, 1) => (128, 128, 128),     // Dark Gray
        (1, 0, 0, 1) => (255, 0, 0),         // Bright Red
        (0, 1, 0, 1) => (0, 255, 0),         // Bright Green
        (1, 1, 0, 1) => (255, 255, 0),       // Bright Yellow
        (0, 0, 1, 1) => (0, 0, 255),         // Bright Blue
        (1, 0, 1, 1) => (255, 0, 255),       // Bright Magenta
        (0, 1, 1, 1) => (0, 255, 255),       // Bright Cyan
        (1, 1, 1, 1) => (255, 255, 255),     // White
        _ => unreachable!(),
    };

    debug!("Calculated color: r = {}, g = {}, b = {}", r, g, b);

    // Return the RGB color
    Ok(Rgb { r, g, b })
}

// set_console_color();

let rgb = from_winapi();

println!("rgb={rgb:?}");
