/*[toml]
[dependencies]
lazy_static = "1.4.0"
*/

#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;

#[derive(Debug, PartialEq)]
enum TermTheme {
    Light,
    Dark,
}

lazy_static! {
    static ref TERM_THEME: Mutex<TermTheme> = Mutex::new(TermTheme::Light);
}

fn main() {
    // To access the TERM_THEME, we need to lock it first
    let theme = TERM_THEME.lock().unwrap();

    // Match the dereferenced value of the Mutex guard
    match *theme {
        TermTheme::Light => println!("The theme is Light"),
        TermTheme::Dark => println!("The theme is Dark"),
    }
}
