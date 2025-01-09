/*[toml]
[dependencies]
ansi_term = "0.12.1"
strum = { version = "0.26.3", features = ["derive"] }
# thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["color_support", "core", "simplelog"] }
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["color_support", "core", "simplelog"] }
*/
/// Demonstrates the colour and styling options, of `thag_rs`.
///
/// TODO: Demo the full 256-colour palette as per 'demo/colors*.rs`, perhaps using `owo-colors`.
///
/// E.g. `thag demo/styling_demo.rs`
//# Purpose: Demonstrate and test the look of available colour palettes and styling settings.
//# Categories: prototype, reference, testing
use ansi_term::Colour;
use strum::IntoEnumIterator;
use thag_rs::styling::{ColorInitStrategy, Level, TermAttributes, TermTheme};
use thag_rs::{cvprtln, vlog, V};

pub fn main() {
    let term_attrs = TermAttributes::initialize(ColorInitStrategy::Detect);
    let color_support = &term_attrs.color_support;
    let theme = &term_attrs.theme;

    let theme_str = theme.to_string();

    // Section 1: ANSI-16 color palette using basic styles
    println!("ANSI-16 color palette in use for {theme_str} theme:\n");
    for level in Level::iter() {
        let style = match theme {
            TermTheme::Light => TermAttributes::basic_light_style(level),
            TermTheme::Dark | TermTheme::Undetermined => TermAttributes::basic_dark_style(level),
        };
        let content = format!("{level} message: level={level:?}, style={style:?}");
        println!("{}", style.paint(content));
    }

    println!();

    // Section 2: ANSI-16 palette using u8 colors
    println!("ANSI-16 color palette in use for {theme_str} theme (converted via u8 and missing bold/dimmed/italic):\n");
    for level in Level::iter() {
        let style = match theme {
            TermTheme::Light => TermAttributes::basic_light_style(level),
            TermTheme::Dark | TermTheme::Undetermined => TermAttributes::basic_dark_style(level),
        };
        if let Some(color_info) = style.foreground {
            let color_num: u8 = color_info.index;
            let content = format!("{level} message: level={level:?}, color_num={color_num}");
            let style = Colour::Fixed(color_num);
            println!("{}", style.paint(content));
        }
    }

    println!();

    // Section 3: Current terminal color palette
    println!("Color palette in use on this terminal:\n");
    for level in Level::iter() {
        let style = match theme {
            TermTheme::Light => TermAttributes::full_light_style(level),
            TermTheme::Dark | TermTheme::Undetermined => TermAttributes::full_dark_style(level),
        };
        cvprtln!(
            level,
            V::N,
            "My {level} message: level={level:?}, style={style:?}"
        );
    }

    println!();

    // Terminal information and warning message
    vlog!(V::N, "Colour support={color_support:?}, theme={theme:?}");
    cvprtln!(Level::WARN, V::N, "Colored Warning message\n");
}
