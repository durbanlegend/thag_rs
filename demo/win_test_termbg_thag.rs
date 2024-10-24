/*[toml]
[dependencies]
thag_rs = { git = "https://github.com/durbanlegend/thag_rs" }
*/

/// This seems to "reliably" swallow the very first character entered in Windows.
//# Purpose: Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.
use std::io::{self, Read};
use thag_rs::termbg;

fn main() {
    let timeout = std::time::Duration::from_millis(100);

    // let term = termbg::terminal();
    let rgb = termbg::rgb(timeout);
    // let theme = termbg::theme(timeout);

    println!("Run with -qq in Windows Terminal to suppress colored lines, type in something and see if first character gets swallowed");
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("buffer={buffer:?}");
}
