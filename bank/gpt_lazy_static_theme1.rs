/*[toml]
[dependencies]
lazy_static = "1.4.0"
*/

#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;

#[derive(Debug, PartialEq)]
enum TermBgLuma {
    Light,
    Dark,
}

lazy_static! {
    static ref TERM_THEME: Mutex<TermBgLuma> = Mutex::new(TermBgLuma::Light);
}

fn main() {
    // To access the TERM_THEME, we need to lock it first
    let theme = TERM_THEME.lock().unwrap();

    // Match the dereferenced value of the Mutex guard
    match *theme {
        TermBgLuma::Light => println!("The theme is Light"),
        TermBgLuma::Dark => println!("The theme is Dark"),
    }
}
