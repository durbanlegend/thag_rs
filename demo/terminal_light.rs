/// Demo of `terminal_light`, a crate that "answers the question "Is the terminal dark
/// or light?".
//# Purpose: Demo terminal-light interrogating the background color. Results will vary with OS and terminal type.
//# Categories: crates
use std::io::{stdout, Write};

let maybe_luma = terminal_light::luma();
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
let luma_255 = 0.2126 * (bg_rgb.r as f32) + 0.7152 * (bg_rgb.g as f32) + 0.0722 * (bg_rgb.b as f32);
let luma_0_to_1 = luma_255 / 255.0;
println!("Background color is {bg_rgb:#?}, luma_255={luma_255}, luma_0_to_1={luma_0_to_1}");
