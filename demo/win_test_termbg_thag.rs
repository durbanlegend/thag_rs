/*[toml]
[dependencies]
thag_rs = "0.1.5"
*/

/// This seems to "reliably" swallow the very first character entered in Windows, prior to `termbg` 0.6.0.
//# Purpose: Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.
use std::io::{self, Read};
use thag_rs::termbg;

fn main() {
    let timeout = std::time::Duration::from_millis(100);

    let _rgb = termbg::rgb(timeout);

    println!("Run with -qq in Windows Terminal to suppress colored lines, type in something and see if first character gets swallowed");
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("buffer={buffer:?}");
}
