/*[toml]
[dependencies]
nu-ansi-term = "0.50.0"
reedline = "0.32.0"
*/

// Pressing [Enter] will in other cases give you a multi-line prompt.

use nu_ansi_term::{Color, Style};
use reedline::{DefaultHinter, DefaultPrompt, DefaultValidator, Reedline, Signal};
use std::io;

fn main() -> io::Result<()> {
    println!("Enter a dummy expression to evaluate. Expressions in matching braces, brackets or quotes may span multiple lines.\nAbort with Ctrl-C or Ctrl-D");
    let mut line_editor = Reedline::create()
        .with_validator(Box::new(DefaultValidator))
        .with_hinter(Box::new(
            DefaultHinter::default().with_style(Style::new().italic().fg(Color::LightGray)),
        ));

    let prompt = DefaultPrompt::default();

    loop {
        let sig = line_editor.read_line(&prompt)?;
        match sig {
            Signal::Success(ref buffer) => {
                println!("We processed: {buffer} - signal {sig:#?}");
            }
            Signal::CtrlD | Signal::CtrlC => {
                println!("\nAborted! - signal {sig:#?}");
                break Ok(());
            }
        }
    }
}
