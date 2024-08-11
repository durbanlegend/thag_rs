/*[toml]
[dependencies]
supports-color = "3.0.0"
*/

use std::io::{self, Read};
use supports_color::Stream;

fn main() {
    let _ = supports_color::on(Stream::Stdout);

    println!("Run with -qq in Windows Terminal to suppress colored lines, type in something and see if first character gets swallowed");
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("buffer={buffer:?}");
}
