/*[toml]
[dependencies]
reedline = "0.36.0"
*/

/// Exploring `reedline` crate.
//# Purpose: explore featured crate.
//# Categories: crates, REPL, technique
use reedline::{
    DefaultHinter, DefaultValidator, FileBackedHistory, Prompt, PromptEditMode,
    PromptHistorySearch, PromptHistorySearchStatus, Reedline, Signal,
};
use std::borrow::Cow;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    struct EvalPrompt(&'static str);
    impl Prompt for EvalPrompt {
        fn render_prompt_left(&self) -> Cow<str> {
            Cow::Owned(self.0.to_string())
        }

        fn render_prompt_right(&self) -> Cow<str> {
            Cow::Owned(String::new())
        }

        fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<str> {
            Cow::Owned("> ".to_string())
        }

        fn render_prompt_multiline_indicator(&self) -> Cow<str> {
            Cow::Borrowed("")
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

    let history_file = "history.txt";
    let history = Box::new(
        FileBackedHistory::with_file(25, history_file.into())
            .expect("Error configuring history with file"),
    );

    let mut line_editor = Reedline::create()
        .with_validator(Box::new(DefaultValidator))
        .with_hinter(Box::new(DefaultHinter::default()))
        .with_history(history);

    let prompt = EvalPrompt("");
    let mut input = String::new();
    loop {
        let sig = line_editor.read_line(&prompt)?;
        let line: &str = match sig {
            Signal::Success(ref buffer) => buffer,
            Signal::CtrlD | Signal::CtrlC => {
                // std::process::exit(0);
                break;
            }
        };
        input.push_str(line);
    }
    // Process user input (line)

    let rs_source = input.trim();
    println!("{rs_source}");

    Ok(())
}
