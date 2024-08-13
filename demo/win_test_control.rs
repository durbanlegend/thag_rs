/// This is the "control" test for the `demo/win_test_*.rs` scripts. It seems to reliably NOT swallow the first character.
//# Purpose: Show how crates *not* sending an OSC to the terminal in Windows will *not* the first character you enter to be swallowed.
use std::io::{self, Read};

fn main() {
    println!("Run with -qq in Windows Terminal to suppress background execution of suspect crates, then type in something and see if first character gets swallowed");
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("buffer={buffer:?}");
}
