/*[toml]
[dependencies]
terminal-light = "1.4.0"
*/

/// This seems to "reliably" swallow the very first character entered in Window, prior to `termbg` 0.6.0..
//# Purpose: Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.
use std::io::{self, Read};

fn main() {
    let _ = terminal_light::luma();

    println!("Run with -qq in Windows Terminal to suppress colored lines, type in something and see if first character gets swallowed");
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("buffer={buffer:?}");
}
