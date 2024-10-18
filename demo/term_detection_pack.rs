/*[toml]
[dependencies]
crossterm = "0.28.1"
supports-color= "3.0.0"
termbg = "0.5.2"
terminal-light = "1.4.0"
*/

/// A basic tool I cobbled together that uses different crates to a) test terminal
/// types on different platforms, b) determine and cross-check if a light or dark
/// theme is in use and c) determine the level of colour supported reported by
/// the terminal.
//# Purpose: Allow checking of terminals on platforms to be supported, also test reliability of different crates.
use crossterm::{
    cursor::{MoveTo, Show},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::stdout;
use supports_color::Stream;
use termbg;

// termbg sends an operating system command (OSC) to interrogate the screen
// but with side effects which we undo here.
pub fn clear_screen() {
    // let mut out = stdout();
    // out.execute(Clear(ClearType::All)).unwrap();
    // out.execute(MoveTo(0, 0)).unwrap();
    // out.execute(Show).unwrap();
    // out.flush().unwrap();
}

let timeout = std::time::Duration::from_millis(100);

let term = termbg::terminal();
let rgb = termbg::rgb(timeout);
let theme = termbg::theme(timeout);
clear_screen();

println!("  Term : {:?}", term);

match rgb {
    Ok(rgb) => {
        // Note: to go from 16-bit color range (0-65535) returned by xterm to 8-bit RGB range (0-255),
        // we need to divide by 65535 / 255 = 257.
        // While it's clear that 256 x 256 = 65536, it may not be so obvious that 255 * 257 = 65535!
        // Search for 257 in https://retrocomputing.stackexchange.com/questions/27436/classic-mac-os-colors-to-modern-rgb.
        // Also note that the 16-bit colours are generally doubled up, like D7D7. I.e. 256xD7 + D7, which
        // may make dividing by 257 seem more intuitive.
        println!("  Color: R={}, G={}, B={}", rgb.r / 257, rgb.g / 257, rgb.b / 257);
        println!("  Color={rgb:#?}");
    }
    Err(e) => {
        println!("  Color: detection failed {:?}", e);
    }
}

match theme {
    Ok(theme) => {
        println!("  Theme: {:?}", theme);
    }
    Err(e) => {
        println!("  Theme: detection failed {:?}", e);
    }
}

println!("\nCrate terminal_light:");

let luma = terminal_light::luma();
println!("luma={luma:#?}");
match luma {
    Ok(luma) if luma > 0.5 => {
        // Use a "light mode" skin.
        println!("Light mode");
    }
    Ok(luma) if luma < 0.5 => {
        // Use a "dark mode" skin.
        println!("Dark mode");
    }
    _ => {
        // Either we couldn't determine the mode or it's kind of medium.
        // We should use an intermediate skin, or one defining the background.
        println!("Intermediate mode");
    }
}

match terminal_light::background_color()
    .map(|c| c.rgb()) {
        Ok(bg_rgb) =>
 {
let luma_255 = 0.2126 * (bg_rgb.r as f32) + 0.7152 * (bg_rgb.g as f32) + 0.0722 * (bg_rgb.b as f32);
let luma_0_to_1 = luma_255 / 255.0;
println!("\nBackground color is {bg_rgb:#?}, luma_255={luma_255}, luma_0_to_1={luma_0_to_1}");
}
Err(_) => println!("terminal_light::background_color() not supported"),    }

println!("\nCrate supports-color:");

if let Some(support) = supports_color::on(Stream::Stdout) {
    if support.has_16m {
        println!("16 million (RGB) colors are supported");
    } else if support.has_256 {
        println!("256 colors are supported.");
    } else if support.has_basic {
        println!("Only basic ANSI colors are supported.");
    }
} else {
    println!("No color support.");
}
