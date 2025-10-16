/*[toml]
[dependencies]
crossterm = "0.29"
reedline = "0.36.0"
*/

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use reedline::{DefaultPrompt, Reedline, Signal};
use std::io::{stdout, Write};

fn main() {
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::default();

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => {
                print_colored_text(&buffer, Color::Green);
            }
            Ok(Signal::CtrlC) => {
                break;
            }
            Ok(Signal::CtrlD) => {
                line_editor.clear_screen().unwrap();
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
            }
        }
    }
}

fn print_colored_text(text: &str, color: Color) {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(color), Print(text), ResetColor).unwrap();
}
