/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/
use thag_styling::{display_color_comparison, TermAttributes};

fn main() {
    let term_attrs = TermAttributes::get_or_init();
    let theme = &term_attrs.theme;

    display_color_comparison(theme);
}
