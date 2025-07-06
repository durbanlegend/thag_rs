/// Published example from `reedline` crate.
///
/// The latest version of this example is available in the [examples] folder in the `reedline`
/// repository. At time of writing you can run it successfully just
/// by invoking its URL with the `thag_url` tool, like this:
///
/// ```bash
/// thag_url https://github.com/nushell/reedline/blob/main/examples/hinter.rs
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
//# Purpose: Explore featured crate.
//# Categories: crates, REPL, technique
// Original `reedline` crate comments:
//
// Create a reedline object with in-line hint support.
// cargo run --example hinter
//
// Fish-style history based hinting.
// assuming history ["abc", "ade"]
// pressing "a" hints to abc.
// Up/Down or Ctrl p/n, to select next/previous match
use nu_ansi_term::{Color, Style};
use reedline::{DefaultHinter, DefaultPrompt, Reedline, Signal};
use std::io;

fn main() -> io::Result<()> {
    let mut line_editor = Reedline::create().with_hinter(Box::new(
        DefaultHinter::default().with_style(Style::new().italic().fg(Color::LightCyan)),
    ));
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
