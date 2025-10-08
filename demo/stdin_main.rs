/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["tui", "simplelog"] }
*/

#![allow(clippy::uninlined_format_args)]
/**
A demo version of src/stdin.rs.

E.g. `thag demo/stdin_main.rs < demo/hello.rs`
*/
//# Purpose: Debugging and demonstration.
//# Categories: demo, testing, tui
use edit::edit_file;
use ratatui::style::{Color, Modifier, Style};
use std::{
    fmt::Debug,
    fs::OpenOptions,
    io::{self, BufRead, IsTerminal},
    path::PathBuf,
};
use thag_rs::{
    // debug_log,
    tui_editor::{script_key_handler, tui_edit, EditData, History, KeyAction, KeyDisplay},
    vprtln,
    CrosstermEventReader,
    EventReader,
    KeyDisplayLine,
    ThagError,
    ThagResult,
    V,
};

#[allow(dead_code)]
fn main() -> ThagResult<()> {
    let event_reader = CrosstermEventReader;
    for line in &edit(&event_reader)? {
        vprtln!(V::N, "{line}");
    }
    Ok(())
}

// Edit the stdin stream.
//
// # Errors
//
// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
// # Panics
//
// If the terminal cannot be reset.
pub fn edit<R: EventReader + Debug>(event_reader: &R) -> ThagResult<Vec<String>> {
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let history_path = PathBuf::from(cargo_home).join("rs_stdin_history.json");
    let mut history = History::load_from_file(&history_path);

    let input = std::io::stdin();

    // debug_log!("input.is_terminal()? {}", input.is_terminal());
    let initial_content = if input.is_terminal() {
        String::new()
    } else {
        read()?
    };

    if !initial_content.trim().is_empty() {
        history.add_entry(&initial_content);
        history.save_to_file(&history_path)?;
    }

    let mut edit_data = EditData {
        return_text: true,
        initial_content: &initial_content,
        save_path: None,
        history_path: Some(&history_path),
        history: Some(history),
    };
    let add_keys = [
        KeyDisplayLine::new(371, "Ctrl+Alt+s", "Save a copy"),
        KeyDisplayLine::new(372, "F3", "Discard saved and unsaved changes, and exit"),
        // KeyDisplayLine::new(373, "F4", "Clear text buffer (Ctrl+y or Ctrl+u to restore)"),
    ];
    let display = KeyDisplay {
        title: "Enter / paste / edit Rust script.  ^D: submit  ^Q: quit  ^L: keys  ^T: toggle highlighting",
        title_style: Style::from((Color::Yellow, Modifier::BOLD)),
        remove_keys: &[""; 0],
        add_keys: &add_keys,
    };
    let (key_action, maybe_text) = tui_edit(
        event_reader,
        &mut edit_data,
        &display,
        |key_event, maybe_term, textarea, edit_data, popup, saved, status_message| {
            script_key_handler(
                key_event,
                maybe_term, // maybe_save_file,
                textarea,
                edit_data,
                popup,
                saved,
                status_message,
            )
        },
    )?;
    match key_action {
        KeyAction::Quit(_saved) => Ok(vec![]),
        // KeyAction::SaveAndExit => false,
        KeyAction::Submit => maybe_text.ok_or(ThagError::Cancelled),
        _ => Err(ThagError::FromStr(
            format!("Logic error: {key_action:?} should not return from tui_edit").into(),
        )),
    }
}

// Prompt for and read Rust source code from stdin.
//
// # Errors
//
// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
pub fn read() -> Result<String, std::io::Error> {
    if std::io::stdin().is_terminal() {
        vprtln!(V::N, "Enter or paste lines of Rust source code at the prompt and press Ctrl-D on a new line when done");
    }
    let buffer = read_to_string(&mut std::io::stdin().lock())?;
    Ok(buffer)
}

// Read Rust source code into a String from the provided reader (e.g., stdin or a mock reader).
//
// # Errors
//
// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
pub fn read_to_string<R: BufRead>(input: &mut R) -> Result<String, io::Error> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Open the history file in an editor.
/// # Errors
/// Will return `Err` if there is an error editing the file.
#[allow(clippy::unnecessary_wraps)]
pub fn edit_history() -> ThagResult<Option<String>> {
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let history_path = PathBuf::from(cargo_home).join("rs_stdin_history.json");
    println!("history_path={}", history_path.display());
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&history_path)?;
    edit_file(&history_path)?;
    Ok(Some(String::from("End of history file edit")))
}
