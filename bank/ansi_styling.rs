/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

#![allow(dead_code)]
/// Demonstrates simple RYO styling of `&str` and `String` types for output via a trait.
///
/// E.g. `thag demo/ansi_styling.rs`
//# Purpose: Demonstrate styling text via traits.
//# Categories: ansi, color, demo, dsl, learning, reference, styling, technique, terminal, trait_implementation, xterm
use std::fmt;
use std::fmt::Display;
use thag_styling::{styled, svprtln, Role, V};

// ANSI color codes
#[derive(Clone, Copy)]
enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

// ANSI text effects
#[derive(Clone, Copy)]
enum Effect {
    Bold,
    Underline,
    Italic,
    Reversed,
}

struct Styled<'a> {
    text: &'a str,
    fg: Option<Color>,
    effects: Vec<Effect>,
}

trait AnsiStyleExt<'a> {
    fn style(self) -> Styled<'a>;
}

impl<'a> AnsiStyleExt<'a> for &'a str {
    fn style(self) -> Styled<'a> {
        Styled {
            text: self,
            fg: None,
            effects: Vec::new(),
        }
    }
}

impl<'a> AnsiStyleExt<'a> for &'a String {
    fn style(self) -> Styled<'a> {
        self.as_str().style()
    }
}

impl<'a> Styled<'a> {
    fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    fn bold(mut self) -> Self {
        self.effects.push(Effect::Bold);
        self
    }

    fn underline(mut self) -> Self {
        self.effects.push(Effect::Underline);
        self
    }

    fn italic(mut self) -> Self {
        self.effects.push(Effect::Italic);
        self
    }

    fn reversed(mut self) -> Self {
        self.effects.push(Effect::Reversed);
        self
    }

    fn to_ansi_code(&self) -> String {
        let mut codes = Vec::new();

        for effect in &self.effects {
            codes.push(match effect {
                Effect::Bold => "1",
                Effect::Underline => "4",
                Effect::Italic => "3",
                Effect::Reversed => "7",
            });
        }

        if let Some(color) = self.fg {
            codes.push(match color {
                Color::Black => "30",
                Color::Red => "31",
                Color::Green => "32",
                Color::Yellow => "33",
                Color::Blue => "34",
                Color::Magenta => "35",
                Color::Cyan => "36",
                Color::White => "37",
            });
        }

        format!("\x1b[{}m", codes.join(";"))
    }

    fn to_ansi_reset_codes(&self) -> String {
        let mut codes = Vec::new();

        for effect in &self.effects {
            codes.push(match effect {
                Effect::Bold => "22",
                Effect::Underline => "24",
                Effect::Italic => "23",
                Effect::Reversed => "27",
            });
        }

        if self.fg.is_some() {
            codes.push("39"); // Reset foreground
        }

        format!("\x1b[{}m", codes.join(";"))
    }
}

impl fmt::Display for Styled<'_> {
    // fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //     write!(f, "{}{}{}", self.to_ansi_code(), self.text, "\x1b[0m")
    // }
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.to_ansi_code(),
            self.text,
            self.to_ansi_reset_codes()
        )
    }
}

impl<'a> Styled<'a> {
    pub fn embed(&self, inner: impl Display) -> String {
        format!(
            "{}{}{}",
            self.to_ansi_code(),
            inner,
            // self.to_ansi_reset_codes()
            ""
        )
    }
}

fn main() {
    println!("{}", "Bold Red".style().bold().fg(Color::Red));
    println!(
        "{}",
        // "Underlined Green".style().underline().fg(Color::Green)
        styled!("Underlined Green", fg = Green, underline)
    );
    println!("{}", "Italic Blue".style().italic().fg(Color::Blue));
    println!(
        "{}",
        "Bold, Underlined, Magenta"
            .style()
            .bold()
            .underline()
            .fg(Color::Magenta)
    );
    println!(
        "{}",
        styled!(
            "Italic, Underlined, Yellow, Reversed",
            italic,
            underline,
            fg = Yellow,
            reversed
        )
    );
    println!("{}", "Normal text".style());

    let name = "Error";
    println!("{}", styled!(name, bold, fg = Red));
    println!(
        "{}",
        styled!(format!("User: {}", "alice"), fg = Blue, underline)
    );

    println!(
        "{}{}{}",
        styled!("outer ", fg = Red),
        styled!("inner", fg = Green),
        " still red (not 😕)"
    );

    let outer = styled!("outer ", fg = Red);
    let inner = styled!("inner", fg = Green);
    // Doesn't work either - to revert to the previous colour we must track and reinstate it or split the message printing.
    println!("{}{} world", outer, outer.embed(inner));

    svprtln!(
        Role::WARN,
        V::N,
        "Hello {}, how are you?",
        styled!("world", bold)
    );
}
