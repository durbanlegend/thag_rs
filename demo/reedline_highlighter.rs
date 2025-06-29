/// Published example from `reedline` crate.
///
/// Try typing - among others - the known commands "test", "hello world", "hello world reedline", "this is the reedline crate".
///
/// The latest version of this example is available in the [examples] folder in the `reedline`
/// repository. At time of writing you can run it successfully simply
/// by invoking its URL with the `thag_url` tool, like this:
///
/// ```bash
/// thag_url https://github.com/nushell/reedline/blob/main/examples/highlighter.rs
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
//# Purpose: Explore featured crate.
//# Categories: crates, REPL, technique
// Original `reedline` crate comments:
//
// Create a reedline object with highlighter support.
// cargo run --example highlighter
//
// unmatched input is marked red, matched input is marked green
use reedline::{DefaultPrompt, ExampleHighlighter, Reedline, Signal};
use std::io;

fn main() -> io::Result<()> {
    let commands = vec![
        "test".into(),
        "hello world".into(),
        "hello world reedline".into(),
        "this is the reedline crate".into(),
    ];
    let mut line_editor =
        Reedline::create().with_highlighter(Box::new(ExampleHighlighter::new(commands)));
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
