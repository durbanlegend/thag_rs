/// Published example `basic.rs` from `reedline` crate.
///
/// The latest version of this example is available in the [examples] folder in the `reedline`
/// repository. At time of writing you can run it successfully simply
/// by invoking its URL with the `thag_url` tool, like this:
///
/// ```bash
/// thag_url https://github.com/nushell/reedline/blob/main/examples/basic.rs
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
//# Purpose: demo featured crates.
//# Categories: crates, REPL, technique
// Original `reedline` crate comments:
//
// Create a default reedline object to handle user input
// cargo run --example basic
//
// You can browse the local (non persistent) history using Up/Down or Ctrl n/p.
use reedline::{DefaultPrompt, Reedline, Signal};
use std::io;

fn main() -> io::Result<()> {
    // Create a new Reedline engine with a local History that is not synchronized to a file.
    // let mut line_editor = Reedline::create();

    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::default();

    loop {
        let sig = line_editor.read_line(&prompt)?;
        match sig {
            Signal::Success(buffer) => {
                println!("We processed: {buffer}");
            }
            Signal::CtrlD | Signal::CtrlC => {
                println!("\nAborted!");
                break Ok(());
            }
        }
    }
}
