/// Demo the use of a command-line interface to override the colour support to be provided.
/// The owo-colors "supports-colors" feature must be enabled.
//# Purpose: Demo setting colour support via a very simple CLI.
//# Categories: CLI, crates, technique
use clap::{Parser, ValueEnum};
use owo_colors::{OwoColorize, Stream};

#[derive(Debug, Parser)]
struct MyApp {
    #[clap(long, value_enum, global = true, default_value = "auto")]
    color: Color,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Color {
    Always,
    Auto,
    Never,
}

impl Color {
    fn init(self) {
        // Set a supports-color override based on the variable passed in.
        match self {
            Color::Always => owo_colors::set_override(true),
            Color::Auto => {}
            Color::Never => owo_colors::set_override(false),
        }
    }
}

fn main() {
    let app = MyApp::parse();
    app.color.init();

    println!(
        "My number is {}",
        42.if_supports_color(Stream::Stdout, |text| text.cyan())
    );
}
