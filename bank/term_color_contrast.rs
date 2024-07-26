/*[toml]
[dependencies]
termcolor = "1.4.1"
*/

use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn luminance(r: u8, g: u8, b: u8) -> f64 {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

// Calculate contrast ratio between two luminance values
fn contrast_ratio(l1: f64, l2: f64) -> f64 {
    if l1 > l2 {
        (l1 + 0.05) / (l2 + 0.05)
    } else {
        (l2 + 0.05) / (l1 + 0.05)
    }
}

// Find contrasting color for a given RGB color
fn find_contrast_color(rgb: (u8, u8, u8)) -> (u8, u8, u8) {
    let (r, g, b) = rgb;
    let l = luminance(r, g, b);

    // Choose white or black as contrasting color depending on luminance
    if contrast_ratio(l, 1.0) > contrast_ratio(l, 0.0) {
        (0, 0, 0) // black
    } else {
        (255, 255, 255) // white
    }
}

let rgb_color = Color::Green; // Example RGB color
let contrast_color = find_contrast_color((0, 255, 0));

let mut stdout = StandardStream::stdout(ColorChoice::Always);
stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(0, 255, 0))).set_bg(Some(Color::Rgb(contrast_color.0, contrast_color.1, contrast_color.2))))?;
writeln!(&mut stdout, "green text!")?;
stdout.reset()?; // Added as instructed
