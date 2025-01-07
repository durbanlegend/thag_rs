/*[toml]
[dependencies]
nu-ansi-term = { version = "0.50.0", features = ["derive_serde_style"] }
strum = { version = "0.26.3", features = ["derive", "strum_macros", "phf"] }
termbg = "0.6.0"
# thag_rs = "0.1.9"
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["color_support", "core", "simplelog"] }
# thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["color_support", "core", "simplelog"] }
*/

#![allow(clippy::implicit_return)]
use nu_ansi_term::{Color, Style};
use strum::IntoEnumIterator;
use termbg::terminal;
use thag_rs::colors::{coloring, MessageStyle, XtermColor};
use thag_rs::logging::V;
use thag_rs::{cvprtln, vlog, ColorSupport, Lvl};
/// Runner for current version of `src/colors.rs`, as it's become too enmeshed with other modules to split out nicely.
/// We just borrow the main method here and add all the necessary dependencies and imports.
///
/// E.g. `thag demo/colors.rs`
//# Purpose: Test the look of the various colours.
//# Categories: testing
// Main function for use by testing or the script runner.
#[allow(dead_code)]
pub fn main() {
    #[allow(unused_variables)]
    let term = terminal();

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

            println!();

            // Convert to title case
            let term_theme_str = term_theme.to_string();
            println!("ANSI-16 color palette in use for {term_theme_str} theme:\n");
            for variant in MessageStyle::iter() {
                let variant_str: &str = &variant.to_string();
                let variant_prefix = format!("ansi16_{term_theme_str}_");
                if !variant_str.starts_with(&variant_prefix) {
                    continue;
                }
                let xterm_color = XtermColor::from(&variant);
                let color_num = u8::from(&xterm_color);
                let style = Style::from(&variant);
                let content = format!(
                    "{variant_str} message: message_style={variant_str:?}, style={style:?}, color_num={color_num}"
                );
                println!("{}", style.paint(content));
            }

            println!();

            println!("ANSI-16 color palette in use for {term_theme_str} theme (converted via XtermColor and missing bold/dimmed/italic):\n");
            for variant in MessageStyle::iter() {
                let variant_str: &str = &variant.to_string();
                let variant_prefix = format!("ansi16_{term_theme_str}_");
                if !variant_str.starts_with(&variant_prefix) {
                    continue;
                }
                let xterm_color = XtermColor::from(&variant);
                let color = Color::from(&xterm_color);
                let style = Style::from(color);
                let content = format!(
                    "{variant_str} message: message_style={variant_str:?}, style={style:?}"
                );
                println!("{}", style.paint(content));
            }

            println!();

            println!("XtermColor::user* colours for comparison:\n");
            for variant in XtermColor::iter().take(16) {
                let variant_str: &str = &variant.to_string();
                let color = Color::from(&variant);
                let style = Style::from(color);
                let content = format!(
                    "{variant_str} message: message_style={variant_str:?}, style={style:?}"
                );
                println!("{}", style.paint(content));
            }

            println!();
            println!("Color palette in use on this terminal:\n");
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
            cvprtln!(Lvl::WARN, V::N, "Colored Warning message\n");
        }
    }
}
