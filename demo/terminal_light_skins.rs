/*[toml]
[dependencies]
coolor = "1.0.0"
terminal-light = "1.4.0"
crossterm = "0.28.1"
*/

/// A published example from the `terminal-light` crate. A simple example of
/// choosing an appropriate skin based on the terminal theme.
//# Purpose: Demo of the `terminal-light` crate.
//# Categories: crates
// This example selects a different skin for light or dark terminals
use {
    coolor::*,
    crossterm::{
        cursor::{MoveTo, Show},
        style::{self, Color, ContentStyle, Stylize},
        terminal::{Clear, ClearType},
        ExecutableCommand,
    },
    std::io::{stdout, Write},
};

struct Skin {
    high_contrast: ContentStyle,
    low_contrast: ContentStyle,
    code: ContentStyle,
}

// terminal-light sends an operating system command (OSC) to interrogate the screen
// but with side effects which we undo here.
pub fn clear_screen() {
    // let mut out = stdout();
    // out.execute(Clear(ClearType::All)).unwrap();
    // out.execute(MoveTo(0, 0)).unwrap();
    // out.execute(Show).unwrap();
    // out.flush().unwrap();
}

fn main() {
    let maybe_luma = terminal_light::luma();
    // clear_screen();
    let skin = match maybe_luma {
        Ok(luma) if luma > 0.6 => {
            // light theme
            Skin {
                high_contrast: ContentStyle {
                    foreground_color: Some(Color::Rgb { r: 40, g: 5, b: 0 }),
                    ..Default::default()
                },
                low_contrast: ContentStyle {
                    foreground_color: Some(Color::Rgb {
                        r: 120,
                        g: 120,
                        b: 80,
                    }),
                    ..Default::default()
                },
                code: ContentStyle {
                    foreground_color: Some(Color::Rgb {
                        r: 50,
                        g: 50,
                        b: 50,
                    }),
                    background_color: Some(Color::Rgb {
                        r: 210,
                        g: 210,
                        b: 210,
                    }),
                    ..Default::default()
                },
            }
        }
        _ => {
            // dark theme
            Skin {
                high_contrast: ContentStyle {
                    foreground_color: Some(Color::Rgb {
                        r: 250,
                        g: 180,
                        b: 0,
                    }),
                    ..Default::default()
                },
                low_contrast: ContentStyle {
                    foreground_color: Some(Color::Rgb {
                        r: 180,
                        g: 150,
                        b: 0,
                    }),
                    ..Default::default()
                },
                code: ContentStyle {
                    foreground_color: Some(Color::Rgb {
                        r: 220,
                        g: 220,
                        b: 220,
                    }),
                    background_color: Some(Color::Rgb {
                        r: 80,
                        g: 80,
                        b: 80,
                    }),
                    ..Default::default()
                },
            }
        }
    };
    println!(
        "\n {}",
        skin.low_contrast
            .apply("This line is easy to read but low intensity")
    );
    println!(
        "\n {}",
        skin.high_contrast
            .apply("This line has a much greater contrast")
    );
    println!("\n {}", skin.code.apply("this.is_meant_to_be(some_code);"));
    println!();
}
