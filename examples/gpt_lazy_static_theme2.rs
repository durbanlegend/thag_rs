/*[toml]
[dependencies]
lazy_static = "1.4.0"
termbg = "0.5.0"
wsl = "0.1.0"
*/

use lazy_static::lazy_static;
use termbg::Theme;

#[derive(Debug, PartialEq)]
enum TermTheme {
    Light,
    Dark,
}

lazy_static! {
    static ref TERM_THEME: TermTheme = {
        let timeout = std::time::Duration::from_millis(100);
        // debug!("Check terminal background color");
        match termbg::theme(timeout) {
            Ok(Theme::Light) => TermTheme::Light,
            Ok(Theme::Dark) | Err(_) => TermTheme::Dark,
        }
    };
}

fn main() {
    // Directly match the static variable without a mutex
    match *TERM_THEME {
        TermTheme::Light => println!("The theme is Light"),
        TermTheme::Dark => println!("The theme is Dark"),
    }
}
