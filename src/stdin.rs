#![allow(clippy::uninlined_format_args)]
use crate::colors::TuiSelectionBg;
use crate::errors::ThagResult;
use crate::logging::Verbosity;
use crate::regex;
use crate::shared::KeyDisplayLine;
use crate::tui_editor::{
    script_key_handler, tui_edit, CrosstermEventReader, EditData, EventReader, History, KeyAction,
    KeyDisplay,
};
use crate::{debug_log, log, ThagError};
use clap::Parser;
use crossterm::event::DisableMouseCapture;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use edit::edit_file;
use mockall::predicate::str;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::Terminal;
use regex::Regex;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{self, BufRead, IsTerminal};
use std::path::PathBuf;
use strum::{EnumIter, EnumString, IntoStaticStr};
use tui_textarea::TextArea;

#[derive(Debug, Parser, EnumIter, EnumString, IntoStaticStr)]
#[command(
    name = "",
    disable_help_flag = true,
    disable_help_subcommand = true,
    verbatim_doc_comment
)] // Disable automatic help subcommand and flag
#[strum(serialize_all = "snake_case")]
/// REPL mode lets you type or paste a Rust expression to be evaluated.
/// Start by choosing the eval option and entering your expression. Expressions between matching braces,
/// brackets, parens or quotes may span multiple lines.
/// If valid, the expression will be converted into a Rust program, and built and run using Cargo.
/// Dependencies will be inferred from imports if possible using a Cargo search, but the overhead
/// of doing so can be avoided by placing them in Cargo.toml format at the top of the expression in a
/// comment block of the form
/// /*[toml]
/// [dependencies]
/// ...
/// */
/// From here they will be extracted to a dedicated Cargo.toml file.
/// In this case the whole expression must be enclosed in curly braces to include the TOML in the expression.
/// At any stage before exiting the REPL, or at least as long as your TMPDIR is not cleared, you can
/// go back and edit your expression or its generated Cargo.toml file and copy or save them from the
/// editor or directly from their temporary disk locations.
/// The tab key will show command selections and complete partial matching selections."
enum ReplCommand {
    /// Show the REPL banner
    Banner,
    /// Edit the Rust expression.
    Edit,
    /// Edit the generated Cargo.toml
    Toml,
    /// Edit history
    History,
    /// Show help information
    Help,
    /// Show key bindings
    Keys,
    /// Exit the REPL
    Quit,
}

#[allow(dead_code)]
fn main() -> ThagResult<()> {
    let event_reader = CrosstermEventReader;
    for line in &edit(&event_reader)? {
        log!(Verbosity::Normal, "{line}");
    }
    Ok(())
}

/// Edit the stdin stream.
///
///
/// # Examples
///
/// ```no_run
/// use thag_rs::stdin::edit;
/// use thag_rs::tui_editor::CrosstermEventReader;
/// use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers };
/// use thag_rs::tui_editor::MockEventReader;
///
/// let mut event_reader = MockEventReader::new();
/// event_reader.expect_read_event().return_once(|| {
///     Ok(Event::Key(KeyEvent::new(
///         KeyCode::Char('d'),
///         KeyModifiers::CONTROL,
///     )))
/// });
/// let actual = edit(&event_reader);
/// let buf = vec![""];
/// assert!(matches!(actual, Ok(buf)));
/// ```
/// # Errors
///
/// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
/// # Panics
///
/// If the terminal cannot be reset.
pub fn edit<R: EventReader + Debug>(event_reader: &R) -> ThagResult<Vec<String>> {
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let history_path = PathBuf::from(cargo_home).join("rs_stdin_history.json");
    let mut history = History::load_from_file(&history_path);

    let input = std::io::stdin();

    #[cfg(debug_assertions)]
    debug_log!("input.is_terminal()? {}", input.is_terminal());
    let initial_content = if input.is_terminal() {
        String::new()
    } else {
        crate::stdin::read()?
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
        KeyDisplayLine::new(361, "Ctrl+Alt+s", "Save a copy"),
        KeyDisplayLine::new(371, "F3", "Discard saved and unsaved changes, and exit"),
        KeyDisplayLine::new(372, "F4", "Clear text buffer (Ctrl+y or Ctrl+u to restore)"),
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

/// Prompt for and read Rust source code from stdin.
///
/// # Examples
///
/// ```
/// use thag_rs::stdin::read;
///
/// let hello = String::from("Hello world!");
/// assert!(matches!(read(), Ok(hello)));
/// ```
/// # Errors
///
/// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
pub fn read() -> Result<String, std::io::Error> {
    log!(Verbosity::Normal, "Enter or paste lines of Rust source code at the prompt and press Ctrl-D on a new line when done");
    let buffer = read_to_string(&mut std::io::stdin().lock())?;
    Ok(buffer)
}

/// Read Rust source code into a String from the provided reader (e.g., stdin or a mock reader).
///
/// # Examples
///
/// ```
/// use thag_rs::stdin::read_to_string;
///
/// let stdin = std::io::stdin();
/// let mut input = stdin.lock();
/// let hello = String::from("Hello world!");
/// assert!(matches!(read_to_string(&mut input), Ok(hello)));
/// ```
///
/// # Errors
///
/// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
pub fn read_to_string<R: BufRead>(input: &mut R) -> Result<String, io::Error> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Convert the different newline sequences for Windows and other platforms into the common
/// standard sequence of `"\n"` (backslash + 'n', as opposed to the '\n' (0xa) character for which
/// it stands).
#[must_use]
pub fn normalize_newlines(input: &str) -> String {
    let re: &Regex = regex!(r"\r\n?");

    re.replace_all(input, "\n").to_string()
}

/// Apply highlights to the text depending on the light or dark theme as detected, configured
/// or defaulted, or as toggled by the user with Ctrl-t.
pub fn apply_highlights(scheme: &TuiSelectionBg, textarea: &mut TextArea) {
    match scheme {
        TuiSelectionBg::BlueYellow => {
            // Dark theme-friendly colors
            textarea.set_selection_style(Style::default().bg(Color::Cyan).fg(Color::Black));
            textarea.set_cursor_style(Style::default().bg(Color::LightYellow).fg(Color::Black));
            textarea.set_cursor_line_style(Style::default().bg(Color::DarkGray).fg(Color::White));
        }
        TuiSelectionBg::RedWhite => {
            // Light theme-friendly colors
            textarea.set_selection_style(Style::default().bg(Color::Blue).fg(Color::White));
            textarea.set_cursor_style(Style::default().bg(Color::LightRed).fg(Color::White));
            textarea.set_cursor_line_style(Style::default().bg(Color::Gray).fg(Color::Black));
        }
    }
}

/// Reset the terminal.
///
/// # Errors
///
/// This function will bubble up any `ratatui` or `crossterm` errors encountered.
// TODO: move to shared or tui_editor?
pub fn reset_term(mut term: Terminal<CrosstermBackend<io::StdoutLock<'_>>>) -> ThagResult<()> {
    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}

/// Open the history file in an editor.
/// # Errors
/// Will return `Err` if there is an error editing the file.
#[allow(clippy::unnecessary_wraps)]
pub fn edit_history() -> ThagResult<Option<String>> {
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let history_path = PathBuf::from(cargo_home).join("rs_stdin_history.json");
    println!("history_path={history_path:#?}");
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&history_path)?;
    edit_file(&history_path)?;
    Ok(Some(String::from("End of history file edit")))
}
