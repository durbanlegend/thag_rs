/*[toml]
[dependencies]
crossterm = "0.27.0"
lazy_static = "1.4.0"
termbg = "0.5.0"
wsl = "0.1.0"
*/

use crossterm::{
    cursor::{MoveTo, Show},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use lazy_static::lazy_static;
use std::io::{stdout, Write};
use termbg::Theme;

pub fn clear_screen() {
    let mut out = stdout();
    // out.execute(Hide).unwrap();
    out.execute(Clear(ClearType::All)).unwrap();
    out.execute(MoveTo(0, 0)).unwrap();
    out.execute(Show).unwrap();
    out.flush().unwrap();
}

#[derive(Debug, PartialEq)]
enum TermTheme {
    Light,
    Dark,
}

lazy_static! {
    static ref TERM_THEME: TermTheme = {
        let timeout = std::time::Duration::from_millis(100);
        // debug!("Check terminal background color");
        let theme = termbg::theme(timeout);
        clear_screen();
        match theme {
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
