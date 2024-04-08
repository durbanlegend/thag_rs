//! [dependencies]
//! rustyline = { version = "14.0.0", features = ["derive"] }

// extern crate rustyline;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::Completer;
use rustyline::Highlighter;
use rustyline::Hinter;
use rustyline::{
    Cmd, Editor, EventHandler, Helper, KeyCode, KeyEvent, Modifiers, Result, Validator,
};

#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
struct InputValidator {
    #[rustyline(Validator)]
    brackets: MatchingBracketValidator,
    #[rustyline(Highlighter)]
    highlighter: MatchingBracketHighlighter,
}

fn main() -> Result<()> {
    let h = InputValidator {
        brackets: MatchingBracketValidator::new(),
        highlighter: MatchingBracketHighlighter::new(),
    };
    let mut rl = Editor::new()?;
    rl.set_helper(Some(h));
    rl.bind_sequence(
        KeyEvent(KeyCode::Char('s'), Modifiers::CTRL),
        EventHandler::Simple(Cmd::Newline),
    );

    let input = rl.readline("> ")?;
    println!("Input: {input}");

    Ok(())
}
