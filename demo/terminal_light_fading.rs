/*[toml]
[dependencies]
coolor = "1.0.0"
terminal-light = "1.5.0"
crossterm = "0.28.1"
*/

/// A fun published example from the `terminal-light` crate. "Demonstrate mixing
/// any ANSI color with the background."
//# Purpose: Mostly recreational.
//# Categories: crates, recreational
use {
    coolor::*,
    crossterm::{
        cursor::{MoveTo, Show},
        style::{self, Stylize},
        terminal::{Clear, ClearType},
        ExecutableCommand,
    },
    std::io::{stdout, Write},
};

fn print_color(ansi: AnsiColor) {
    print!("{}", "█".with(style::Color::AnsiValue(ansi.code)));
}

fn mix(color1: Hsl, weight1: f32, color2: Hsl, weight2: f32) -> Hsl {
    Color::blend(color1, weight1, color2, weight2).hsl()
}

fn main() {
    let maybe_background_color = terminal_light::background_color();
    let bg = match maybe_background_color {
        Ok(bg) => bg,
        _ => {
            println!("Couldn't determine the background color, using default");
            AnsiColor::new(234).into()
        }
    };
    println!("\n Terminal background color: {:?}", bg);
    println!(" Blending all ANSI colors into the background, using only ANSI colors:");
    let bg = bg.hsl();
    for code in 1..=255 {
        let ansi = AnsiColor::new(code);
        print!(" {:>3}  ", code);
        print_color(ansi);
        print!("  ");
        let fg = ansi.to_hsl();
        for i in 0..20 {
            print_color(mix(fg, (20 - i) as f32, bg, i as f32).to_ansi());
        }
        println!();
    }
}
