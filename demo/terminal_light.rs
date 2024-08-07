/*[toml]
[dependencies]
crossterm = "0.28.1"
terminal-light = "1.4.0"
*/

/// Demo of `terminal_light`, a crate that "answers the question "Is the terminal dark
/// or light?". I've added the `clear_screen` method because as is common, `terminal_light`
/// interrogates the terminal with an escape sequence which may mess with its settings
/// and compromise the program's output.
//# Purpose: Demo terminal-light interrogating the background color. Results will vary with OS and terminal type.
use crossterm::{
    cursor::{MoveTo, Show},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::{stdout, Write};

// terminal-light sends an operating system command (OSC) to interrogate the screen
// but with side effects which we undo here.
pub fn clear_screen() {
    let mut out = stdout();
    out.execute(Clear(ClearType::All)).unwrap();
    out.execute(MoveTo(0, 0)).unwrap();
    out.execute(Show).unwrap();
    out.flush().unwrap();
}

let maybe_luma = terminal_light::luma();
clear_screen();
match maybe_luma {
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

let bg_rgb = terminal_light::background_color()
    .map(|c| c.rgb()).unwrap();  // may be an error
clear_screen();
let luma_255 = 0.2126 * (bg_rgb.r as f32) + 0.7152 * (bg_rgb.g as f32) + 0.0722 * (bg_rgb.b as f32);
let luma_0_to_1 = luma_255 / 255.0;
println!("Background color is {bg_rgb:#?}, luma_255={luma_255}, luma_0_to_1={luma_0_to_1}");
