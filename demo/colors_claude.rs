/*[toml]
[dependencies]
nu-ansi-term = { version = "0.50.0", features = ["derive_serde_style"] }
strum = { version = "0.26.3", features = ["derive", "strum_macros"] }
# NB: Use git tag `config_quoted_or_unquoted_booleans` if using misc.unquote without quotes (ironically).
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", tag = "config_quoted_or_unquoted_booleans" }
*/

#![allow(clippy::implicit_return)]
use nu_ansi_term::{Color, Style};
use strum::IntoEnumIterator;
use termbg::terminal;
use thag_rs::logging::V;
use thag_rs::{cvprtln, vlog, Lvl};
/// Runner for legacy module `src/colors.rs`, removed after "0.1.9".
/// This demo now uses basic ANSI colors via nu-ansi-term instead of the removed colors module.
/// NB: If you have the [misc] option `unquote` specified in your `config.toml` and
/// don't use the git tag `config_quoted_or_unquoted_booleans` in the `toml` block
/// you will need to ensure that its boolean value is a quoted string ("true" or "false")
/// in order to run this demo, otherwise it will fail trying to process the config
/// file correctly.
///
/// E.g. `thag demo/colors.rs`
//# Purpose: Originally to test the look of the various colours. Replaced by `demo/styling_demo.rs`.
//# Categories: ANSI, color, demo, styling, terminal, testing
// Main function for use by testing or the script runner.
#[allow(dead_code)]
pub fn main() {
    #[allow(unused_variables)]
    let term = terminal();

    // Use basic ANSI colors instead of the removed colors module
    println!("Terminal color demonstration using nu-ansi-term:\n");

    // Demonstrate different message levels using cvprtln
    println!("Message level demonstrations:\n");
    for variant in Lvl::iter() {
        let variant_string: &str = &variant.to_string();
        cvprtln!(
            variant,
            V::N,
            "My {variant_string} message using cvprtln macro"
        );
    }

    println!("\nBasic ANSI color demonstrations:\n");

    // Demonstrate basic ANSI colors
    println!("{}", Color::Red.paint("This is Red text"));
    println!("{}", Color::Green.paint("This is Green text"));
    println!("{}", Color::Blue.paint("This is Blue text"));
    println!("{}", Color::Yellow.paint("This is Yellow text"));
    println!("{}", Color::Purple.paint("This is Purple text"));
    println!("{}", Color::Cyan.paint("This is Cyan text"));
    println!("{}", Color::White.paint("This is White text"));
    println!("{}", Color::Black.paint("This is Black text"));

    println!("\nStyle demonstrations:\n");

    // Demonstrate text styles
    println!("{}", Style::new().bold().paint("Bold text"));
    println!("{}", Style::new().italic().paint("Italic text"));
    println!("{}", Style::new().underline().paint("Underlined text"));
    println!("{}", Style::new().dimmed().paint("Dimmed text"));

    // Combine color and style
    println!("{}", Color::Red.bold().paint("Bold red text"));
    println!("{}", Color::Blue.italic().paint("Italic blue text"));
    println!(
        "{}",
        Color::Green.underline().paint("Underlined green text")
    );

    println!("\nTerm : {term:?}");
    vlog!(V::N, "Colors demo completed using nu-ansi-term");
}
