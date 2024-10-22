/*[toml]
[dependencies]
# Alternate between the following two termbg dependencies to test the difference:
# termbg = "=0.5.2"
# termbg "0.6.0" incorporates my PR to fix the issue:
termbg = "0.6.0"
#termbg = { git = "https://github.com/durbanlegend/termbg" }
*/

/// This seems to "reliably" swallow the very first character entered in Windows.
//# Purpose: Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.
use std::io::{self, Read};

fn main() {
    let timeout = std::time::Duration::from_millis(500);

    let _rgb = termbg::rgb(timeout);

    println!("Type in something in Windows Terminal and see if first character gets swallowed");
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("buffer={buffer:?}");
}
