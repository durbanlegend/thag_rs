/*[toml]
[dependencies]
#crossterm = "*"
reedline = "0.32.0"
#reedline-repl-rs = "1.1.1"
*/

// Create a default reedline object to handle user input
// cargo run --example basic
//
// You can browse the local (non persistent) history using Up/Down or Ctrl n/p.

use std::io;
use {
    crossterm::event::{KeyCode, KeyModifiers},
    reedline::{
        default_emacs_keybindings, DefaultPrompt, EditCommand, Emacs, Reedline, ReedlineEvent,
        Signal,
    },
};

fn main() -> io::Result<()> {
    // Create a new Reedline engine with a local History that is not synchronized to a file.
    // let mut line_editor = Reedline::create();

    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Char('m'),
        ReedlineEvent::Edit(vec![EditCommand::BackspaceWord]),
    );
    let edit_mode = Box::new(Emacs::new(keybindings));

    let mut line_editor = Reedline::create().with_edit_mode(edit_mode);
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
