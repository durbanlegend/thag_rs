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
use thag_rs::colors::{coloring, ColorSupport, MessageStyle, XtermColor};
use thag_rs::logging::V;
use thag_rs::{cvprtln, vlog, Lvl};
/// Runner for legacy module `src/colors.rs`, removed after "0.1.9".
/// NB: If you have the [misc] option `unquote` specified in your `config.toml` and
/// don't use the git tag `config_quoted_or_unquoted_booleans` in the `toml` block
/// you will need to ensure that its boolean value is a quoted string ("true" or "false")
/// in order to run this demo, otherwise it will fail trying to process the config
/// file due to a breaking change after "0.1.9".
///
/// E.g. `thag demo/colors.rs`
//# Purpose: Originally to test the look of the various colours. Replaced by `demo/styling_demo.rs`.
//# Categories: demo, testing
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
            cvprtln!(&Lvl::WARN, V::N, "Colored Warning message\n");
        }
    }
}
