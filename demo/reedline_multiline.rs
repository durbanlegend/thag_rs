/*[toml]
[dependencies]
#crossterm = "0.28.1"
nu-ansi-term = "0.50.0"
reedline = "0.36.0"
*/

/// Exploratory prototype of REPL support for multi-line expressions. Based on published example
/// `custom_prompt.rs` in `reedline` crate.
//# Purpose: Explore options for handling multi-line expressions in a REPL.
//# Categories: crates, REPL, technique
use nu_ansi_term::{Color, Style};
use reedline::{
    DefaultHinter, DefaultValidator, FileBackedHistory, Prompt, PromptEditMode,
    PromptHistorySearch, PromptHistorySearchStatus, Reedline, Signal,
};
use std::borrow::Cow;
use std::cell::Cell;
use std::io;

pub struct CustomPrompt(Cell<u32>, &'static str);
pub static DEFAULT_MULTILINE_INDICATOR: &str = " :::: ";
impl Prompt for CustomPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        {
            Cow::Owned(self.1.to_string())
        }
    }

    fn render_prompt_right(&self) -> Cow<str> {
        {
            let old = self.0.get();
            self.0.set(old + 1);
            Cow::Owned(String::from("q: quit"))
        }
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<str> {
        Cow::Owned("> ".to_string())
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed(DEFAULT_MULTILINE_INDICATOR)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }
}

fn main() -> io::Result<()> {
    let history = Box::new(
        FileBackedHistory::with_file(20, "history.txt".into())
            .expect("Error configuring history with file"),
    );

    let mut line_editor = Reedline::create()
        .with_validator(Box::new(DefaultValidator))
        .with_hinter(Box::new(
            DefaultHinter::default().with_style(Style::new().italic().fg(Color::Cyan)),
        ))
        // .with_edit_mode(edit_mode)
        .with_history(history);

    println!("Enter a dummy expression to evaluate. Expressions in matching braces, brackets or quotes may span multiple lines.\nAbort with Ctrl-C or Ctrl-D");
    let prompt = CustomPrompt(Cell::new(0), "expr");

    loop {
        let sig = line_editor.read_line(&prompt)?;
        match sig {
            Signal::Success(ref buffer) => {
                println!("{buffer}");
                if buffer == "q" {
                    break Ok(());
                }
            }
            Signal::CtrlD | Signal::CtrlC => {
                println!("\nAborted!");
                break Ok(());
            }
        }
    }
}
