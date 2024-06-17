/*[toml]
[dependencies]
rustyline = { version = "14.0.0", features=["with-file-history", "default", "derive", "rustyline-derive"] }
*/

use rustyline::error::ReadlineError;
use rustyline::hint::{Hint, Hinter};
use rustyline::history::History;
use rustyline::validate::Validator;
use rustyline::Context;
use rustyline::DefaultEditor;
use rustyline::{
    completion::{Candidate, Completer},
    config,
    highlight::{Highlighter, MatchingBracketHighlighter},
    Editor, Helper, Result,
};

// Define a struct to represent a completion candidate
#[derive(Clone)]
struct MyCandidate {
    display: String,
    replacement: String,
}

impl Candidate for MyCandidate {
    fn display(&self) -> &str {
        &self.display
    }

    fn replacement(&self) -> &str {
        &self.replacement
    }
}

// Implement the Completer trait for our custom completion logic
struct MyCompleter {
    available_commands: Vec<String>,
}

impl Completer for MyCompleter {
    type Candidate = MyCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        let mut completions = Vec::new();
        let mut start = 0;
        let mut chars = line.chars(); // Get character iterator

        // Find the word to be completed by iterating backwards
        let i = 0;
        // Use rev().next() for reverse iteration
        while let Some(c) = chars.rev().next() {
            if !c.is_ascii_alphanumeric() && c != '_' {
                start = i + 1;
                break;
            }
        }

        let word = &line[start..pos];

        // Find matching commands from available_commands
        for command in &self.available_commands {
            if command.starts_with(word) {
                completions.push(MyCandidate {
                    display: command.to_string(),
                    replacement: command.to_string(),
                });
            }
        }

        Ok((start, completions))
    }
}

#[derive(Helper, Completer, Hinter, Validator)]
struct MyHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
    colored_prompt: String,
    completer: MyCompleter,
    highlighter: MatchingBracketHighlighter, // Added Highlighter implementation
}

// Implement the Helper trait for our custom completer
impl Helper for MyHelper {}

impl Hinter for MyHelper {
    /// Specific hint type
    type Hint: Hint + 'static;
}

impl Completer for MyHelper {
    type Candidate = MyCandidate;
}

impl Highlighter for MyHelper {}

impl History for MyHelper {}

impl Validator for MyHelper {}

fn main() {
    let mut editor = DefaultEditor::with_config(
        config::Builder::new()
            .max_history_size(10)
            .expect("REASON")
            .build(),
    )
    .expect("Could not init editor");

    // Define your available commands (replace with your actual commands)
    let available_commands = vec!["help".to_string(), "exit".to_string(), "print".to_string()];

    let my_helper = MyHelper {
        completer: MyCompleter { available_commands },
        highlighter: MatchingBracketHighlighter::new(),
    };
    editor.set_helper(Some(my_helper));

    loop {
        let readline = editor.readline(">> ");
        match readline {
            Ok(line) => {
                // Process user input (line)
                println!("You entered: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }
}
