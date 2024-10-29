/*[toml]
[dependencies]
# crossterm = "0.28.1"
# firestorm = "0.5.1"
# log = "0.4.22"
nu-ansi-term = { version = "0.50.0", features = ["derive_serde_style"] }
# ratatui = "0.28"
# scopeguard = "1.2.0"
# serde = "1.0.213"
strum = { version = "0.26.2", features = ["derive", "strum_macros", "phf"] }
# supports-color= "3.0.0"
termbg = "0.6.0"
# thag_rs = "0.1.5"
# thag_rs = { git = "https://github.com/durbanlegend/thag_rs" }
thag_rs = { path = "/Users/donf/projects/thag_rs/" }
*/

#![allow(clippy::implicit_return)]
use nu_ansi_term::{Color, Style};
use strum::IntoEnumIterator;
use termbg::terminal;
use thag_rs::colors::{coloring, ColorSupport, MessageStyle, XtermColor};
use thag_rs::logging::V;
use thag_rs::{cprtln, cvprtln, vlog, Lvl};
/// Runner for current version of `src/colors.rs`, as it's become too enmeshed with other modules to split out nicely.
/// We just borrow the main method here and add all the necessary dependencies and imports.
///
/// E.g. `thag demo/colors.rs`
//# Purpose: Test the look of the various colours.
/// Main function for use by testing or the script runner.
#[allow(dead_code)]
pub fn main() {
    #[allow(unused_variables)]
    let term = terminal();
    // shared::clear_screen();

    let (maybe_color_support, term_theme) = coloring();

    match maybe_color_support {
        None => {
            vlog!(V::N, "No colour support found for terminal");
        }
        Some(support) => {
            if matches!(support, ColorSupport::Xterm256) {
                vlog!(V::N, "");
                XtermColor::iter().for_each(|variant| {
                    let color = Color::from(&variant);
                    vlog!(V::N, "{}", color.paint(variant.to_string()));
                });
            }

            for variant in Lvl::iter() {
                let variant_string: &str = &variant.to_string();
                let message_style = MessageStyle::from(&variant);
                let style = Style::from(&variant);
                cvprtln!(
                    variant,
                    V::N,
                    "My {variant_string} message: message_style={message_style:?}, style={style:?}"
                );
            }

            println!("\nTerm : {term:?}");
            vlog!(
                V::N,
                "Colour support={support:?}, term_theme={term_theme:?}"
            );
            vlog!(
                V::N,
                "{}",
                Style::from(&Lvl::WARN).paint("Colored Warning message\n")
            );
        }
    }
}
