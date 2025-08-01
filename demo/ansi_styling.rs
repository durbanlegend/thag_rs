#![allow(dead_code)]
/// Demonstrates simple RYO styling of `&str` and `String` types for output via a trait.
///
/// E.g. `thag demo/ansi_styling.rs`
//# Purpose: Demonstrate styling text via traits.
//# Categories: ansi, color, demo, dsl, learning, reference, styling, technique, terminal, trait_implementation, xterm
use std::fmt;
use std::fmt::Display;
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::styled;

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

// ANSI text styles
#[derive(Clone, Copy)]
enum Style {
    Bold,
    Underline,
    Italic,
    Reversed,
}

struct Styled<'a> {
    text: &'a str,
    fg: Option<Color>,
    styles: Vec<Style>,
}

trait AnsiStyleExt<'a> {
    fn style(self) -> Styled<'a>;
}

impl<'a> AnsiStyleExt<'a> for &'a str {
    fn style(self) -> Styled<'a> {
        Styled {
            text: self,
            fg: None,
            styles: Vec::new(),
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
        self.styles.push(Style::Bold);
        self
    }

    fn underline(mut self) -> Self {
        self.styles.push(Style::Underline);
        self
    }

    fn italic(mut self) -> Self {
        self.styles.push(Style::Italic);
        self
    }

    fn reversed(mut self) -> Self {
        self.styles.push(Style::Reversed);
        self
    }

    fn to_ansi_code(&self) -> String {
        let mut codes = Vec::new();

        for style in &self.styles {
            codes.push(match style {
                Style::Bold => "1",
                Style::Underline => "4",
                Style::Italic => "3",
                Style::Reversed => "7",
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

        for style in &self.styles {
            codes.push(match style {
                Style::Bold => "22",
                Style::Underline => "24",
                Style::Italic => "23",
                Style::Reversed => "27",
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

// impl<T: Display> Styled<'_> {
//     pub fn embed(&self, inner: impl Display) -> String {
//         format!(
//             "{}{}{}",
//             inner,
//             self.to_ansi_reset_codes(),
//             self.to_ansi_code()
//         )
//     }
// }
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
        styled!(fg=Green, underline, => "Underlined Green")
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
            italic,
            underline,
            fg = Yellow,
            reversed,
                => "Italic, Underlined, Yellow, Reversed"
        )
    );
    println!("{}", "Normal text".style());

    let name = "Error";
    println!("{}", styled!(bold, fg=Red, => name));
    println!(
        "{}",
        styled!(fg=Blue, underline, => format!("User: {}", "alice"))
    );

    println!(
        "{}{}{}",
        styled!(fg=Red, => "outer "),
        styled!(fg=Green, => "inner"),
        " still red (not 😕)"
    );

    let outer = styled!(fg=Red, => "outer ");
    let inner = styled!(fg=Green, => "inner");
    // Doesn't work either - to revert to the previous colour we must track and reinstate it or split the message printing.
    println!("{}{} world", outer, outer.embed(inner));
}
