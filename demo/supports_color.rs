/*[toml]
[dependencies]
supports-color= "3.0.0"
*/

/// Demo of crate `supports-color` that `rs-script` uses to detect the level
/// of colour support of the terminal in use.
//# Purpose: Demo featured crate doing its job.
use supports_color::Stream;

if let Some(support) = supports_color::on(Stream::Stdout) {
    if support.has_16m {
        println!("This terminal supports 16 million (RGB) colors");
    } else if support.has_256 {
        println!("This terminal supports 256 colors.");
    } else if support.has_basic {
        println!("This terminal only supports 16 basic ANSI colors.");
    }
} else {
    println!("No color support.");
}
