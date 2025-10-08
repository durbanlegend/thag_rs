/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["full_profiling"] }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["tui", "simplelog"] }
*/

#![allow(clippy::uninlined_format_args)]
/**
A version of `demo/stdin_main.rs` instrumented for profiling with `thag_profiler`.
**Caution**: For memory profiling of the `Ctrl+L: keys` action, this particular example is painfully slow,
even though the original is not. As detailed memory profiling shows, this is because a great deal of memory
allocation is taking place in the `cassowary` solver algorithm that calculates the layout. All allocations
are less than 4KB and almost half the profiled allocations are under 64B. No fingers are being pointed here
since GUI layout is fiendishly difficult - but it is something out of our control without a radical redesign
- which wouldn't be justified because the normal response is still sub-second. But it does illustrate how
and why memory profiling can be slow in some cases.

E.g. `THAG_PROFILER=both,,none,true thag demo/stdin_main_instr.rs`
*/
//# Purpose: Debugging.
//# Categories: profiling, testing, tui
use edit::edit_file;
use ratatui::style::{Color, Modifier, Style};
use std::{
    fmt::Debug,
    fs::OpenOptions,
    io::{self, BufRead, IsTerminal},
    path::PathBuf,
};
use thag_profiler::{enable_profiling, profiled};
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
#[enable_profiling(runtime)]
fn main() -> ThagResult<()> {
    let event_reader = CrosstermEventReader;
    for line in &edit(&event_reader)? {
        vprtln!(V::N, "{line}");
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
/// use thag_rs::CrosstermEventReader;
/// use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers };
/// use thag_rs::MockEventReader;
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
#[profiled]
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

/// Prompt for and read Rust source code from stdin.
///
/// # Examples
///
/// ``` ignore
/// use thag_rs::stdin::read;
///
/// let hello = String::from("Hello world!");
/// assert!(matches!(read(), Ok(hello)));
/// ```
/// # Errors
///
/// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
#[profiled]
pub fn read() -> Result<String, std::io::Error> {
    if std::io::stdin().is_terminal() {
        vprtln!(V::N, "Enter or paste lines of Rust source code at the prompt and press Ctrl-D on a new line when done");
    }
    let buffer = read_to_string(&mut std::io::stdin().lock())?;
    Ok(buffer)
}

/// Read Rust source code into a String from the provided reader (e.g., stdin or a mock reader).
///
/// # Examples
///
/// ``` ignore
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
#[profiled]
pub fn read_to_string<R: BufRead>(input: &mut R) -> Result<String, io::Error> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Open the history file in an editor.
/// # Errors
/// Will return `Err` if there is an error editing the file.
#[allow(clippy::unnecessary_wraps)]
#[profiled]
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
