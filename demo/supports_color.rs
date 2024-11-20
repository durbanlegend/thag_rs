/*[toml]
[dependencies]
supports-color= "3.0.0"
*/

/// Demo of crate `supports-color` that `thag_rs` uses to detect the level of
/// colour support of the terminal in use.
/// Caution: from testing I suspect that `supports-color` may mess with the terminal
/// settings. Obviously that doesn't matter in a demo that exists before doing
/// serious work, but it can wreak havoc with your program's output.
//# Purpose: Demo featured crate doing its job.
//# Categories: crates
use crossterm::{
    cursor::{MoveTo, Show},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::{stdout, Write};
use supports_color::Stream;

let color_support_option = supports_color::on(Stream::Stdout);

if let Some(support) = color_support_option {
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
