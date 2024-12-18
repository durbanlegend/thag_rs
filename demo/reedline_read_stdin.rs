/*[toml]
[dependencies]
reedline = "0.36.0"
*/

/// Basic exploration of reading a line from stdin with `reedline`.
//# Purpose: Exploring how to render prompts and read lines of input.
//# Categories: crates, REPL, technique
use reedline::{Prompt, Reedline, Signal};
use std::borrow::Cow;
use std::io;

struct EmptyPrompt;

impl Prompt for EmptyPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        "".to_string().into()
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        "".to_string().into()
    }

    fn render_prompt_indicator(&self, _prompt_mode: reedline::PromptEditMode) -> Cow<str> {
        "".to_string().into()
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        "".to_string().into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: reedline::PromptHistorySearch,
    ) -> Cow<str> {
        "".to_string().into()
    }
}

pub(crate) fn read_stdin() -> Result<String, io::Error> {
    println!("Enter or paste lines of Rust source code at the prompt and press Ctrl-D on a new line when done");

    let mut editor = Reedline::create();
    let prompt = EmptyPrompt;
    let mut buffer = String::new();

    loop {
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                buffer.push_str(&line);
                buffer.push('\n');
            }
            Ok(Signal::CtrlD) => {
                // End on Ctrl-D
                break;
            }
            Ok(Signal::CtrlC) => {
                println!("Operation canceled by user.");
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "Operation canceled by user",
                ));
            }
            Err(err) => {
                println!("Error reading line: {:?}", err);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Error reading line: {:?}", err),
                ));
            }
        }
    }

    Ok(buffer)
}

fn main() {
    match read_stdin() {
        Ok(input) => println!("Received input:\n{}", input),
        Err(e) => eprintln!("Error: {}", e),
    }
}
