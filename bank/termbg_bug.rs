/*[toml]
[dependencies]
termbg = "0.5.2"
#termbg = { path = "/Users/donforbes/Documents/GitHub/termbg" }
*/

use std::io::{self, Read};

fn main() {
    let timeout = std::time::Duration::from_millis(100);

    // let term = termbg::terminal();
    let _rgb = termbg::rgb(timeout);
    // let theme = termbg::theme(timeout);

    println!("Type in something and see if first character gets swallowed in Windows Terminal");
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("buffer={buffer:?}");
}
