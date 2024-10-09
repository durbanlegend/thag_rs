/*[toml]
[dependencies]
crossterm = "0.28.1"
log = "0.4.22"
ratatui = "0.28.1"
thag_rs = { path = "/Users/donf/projects/thag_rs" }
tui-textarea = "0.6.1"
*/

use crossterm::event::{self, Event, KeyEvent};
use log::debug;
use ratatui::style::{Style, Stylize};
use std::fmt::Debug;
use std::io::IsTerminal;
use std::path::PathBuf;
use thag_rs::file_dialog::{DialogMode, FileDialog, Status};
use thag_rs::keys::KeyCombination;
use thag_rs::logging::V;
use thag_rs::shared::KeyDisplayLine;
use thag_rs::stdin;
use thag_rs::tui_editor::{
    paste_to_textarea, preserve, save_if_changed, save_source_file, tui_edit, CrosstermEventReader,
    EditData, EventReader, History, KeyAction, KeyDisplay, TermScopeGuard,
};
use thag_rs::{debug_log, key, log, Lvl, ThagError, ThagResult};
use tui_textarea::TextArea;

fn main() -> ThagResult<()> {
    let event_reader = CrosstermEventReader;
    for line in &edit(&event_reader)? {
        log!(V::N, "{line}");
    }
    Ok(())
}

pub fn edit<R: EventReader + Debug>(event_reader: &R) -> ThagResult<Vec<String>> {
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let history_path = PathBuf::from(cargo_home).join("bank_tui_rs_stdin_history.json");
    let history = History::load_from_file(&history_path);
    let input = std::io::stdin();

    #[cfg(debug_assertions)]
    debug_log!("input.is_terminal()? {}", input.is_terminal());
    let initial_content = if input.is_terminal() {
        String::new()
    } else {
        crate::stdin::read()?
    };

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
    ];
    let display = KeyDisplay {
        title: "Enter / paste / edit Rust script.  ^D: submit  ^Q: quit  ^L: keys  ^T: toggle highlighting",
        title_style: Style::from(&Lvl::EMPH).bold(),
        remove_keys: &[""; 0],
        add_keys: &add_keys,
    };
    let (key_action, maybe_text) = tui_edit(
        event_reader,
        &mut edit_data,
        &display,
        |key_event, maybe_term, /*maybe_save_file,*/ textarea, edit_data, popup, saved| {
            script_key_handler(
                key_event, maybe_term, // maybe_save_file,
                textarea, edit_data, popup, saved,
            )
        },
    )?;
    match key_action {
        KeyAction::Quit(_saved) => Err(ThagError::Cancelled),
        KeyAction::Save
        | KeyAction::ShowHelp
        | KeyAction::ToggleHighlight
        | KeyAction::TogglePopup => Err(ThagError::FromStr(
            format!("Logic error: {key_action:?} should not return from tui_edit").into(),
        )),
        // KeyAction::SaveAndExit => false,
        KeyAction::Submit => {
            return maybe_text.map_or(Err(ThagError::Cancelled), |v| Ok(v));
        }
        _ => Err(ThagError::FromStr(
            format!("Logic error: {key_action:?} should not return from tui_edit").into(),
        )),
    }
}

/// Key handler function to be passed into `tui_edit` for editing REPL history.
///
/// # Errors
///
/// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
#[allow(clippy::too_many_lines)]
pub fn script_key_handler(
    key_event: KeyEvent,
    maybe_term: &mut Option<&mut TermScopeGuard>,
    textarea: &mut TextArea,
    edit_data: &mut EditData,
    popup: &mut bool,
    saved: &mut bool, // TODO decide if we need this
) -> ThagResult<KeyAction> {
    let history_path = edit_data.history_path.cloned();
    let key_combination = KeyCombination::from(key_event); // Derive KeyCombination
                                                           // eprintln!("key_combination={key_combination:?}");

    #[allow(clippy::unnested_or_patterns)]
    match key_combination {
        key!(esc) | key!(ctrl - c) | key!(ctrl - q) => Ok(KeyAction::Quit(*saved)),
        key!(ctrl - d) => {
            if let Some(ref hist_path) = history_path {
                let history = &mut edit_data.history;
                if let Some(hist) = history {
                    preserve(textarea, hist, hist_path)?;
                };
            }
            Ok(KeyAction::Submit)
        }
        key!(ctrl - s) | key!(ctrl - alt - s) => {
            // eprintln!("key_combination={key_combination:?}, maybe_save_path={maybe_save_path:?}");
            if matches!(key_combination, key!(ctrl - s)) && edit_data.save_path.is_some() {
                if let Some(ref hist_path) = history_path {
                    let history = &mut edit_data.history;
                    if let Some(hist) = history {
                        preserve(textarea, hist, hist_path)?;
                    };
                }
                let result = edit_data
                    .save_path
                    .as_mut()
                    .map(|p| save_source_file(p, textarea, saved));
                match result {
                    Some(Ok(())) => {}
                    Some(Err(e)) => return Err(e),
                    None => return Err(ThagError::Logic(
                        "Should be testing for maybe_save_path.is_some() before calling map on it.",
                    )),
                }
                Ok(KeyAction::Save)
            } else if let Some(term) = maybe_term {
                let mut save_dialog: FileDialog<'_> = FileDialog::new(60, 40, DialogMode::Save)?;
                save_dialog.open();
                let mut status = Status::Incomplete;
                while matches!(status, Status::Incomplete) && save_dialog.selected_file.is_none() {
                    term.draw(|f| save_dialog.draw(f))?;
                    if let Event::Key(key) = event::read()? {
                        status = save_dialog.handle_input(key)?;
                    }
                }

                if let Some(ref to_rs_path) = save_dialog.selected_file {
                    save_source_file(to_rs_path, textarea, saved)?;
                    Ok(KeyAction::Save)
                } else {
                    Ok(KeyAction::Continue)
                }
            } else {
                Ok(KeyAction::Continue)
            }
        }
        key!(ctrl - l) => {
            // Toggle popup
            *popup = !*popup;
            Ok(KeyAction::TogglePopup)
        }
        key!(f3) => {
            // Ask to revert
            Ok(KeyAction::AbandonChanges)
        }
        key!(f7) => {
            if let Some(ref mut hist) = edit_data.history {
                save_if_changed(hist, textarea, &history_path)?;
                if let Some(entry) = &hist.get_previous() {
                    #[cfg(debug_assertions)]
                    debug!("F7 found entry {entry:?}");
                    paste_to_textarea(textarea, entry);
                }
            }
            Ok(KeyAction::Continue)
        }
        key!(f8) => {
            if let Some(ref mut hist) = edit_data.history {
                save_if_changed(hist, textarea, &history_path)?;
                if let Some(entry) = hist.get_next() {
                    #[cfg(debug_assertions)]
                    debug!("F8 found entry {entry:?}");
                    paste_to_textarea(textarea, entry);
                }
            }
            Ok(KeyAction::Continue)
        }
        _ => {
            // Update the textarea with the input from the key event
            textarea.input(tui_textarea::Input::from(key_event)); // Input derived from Event
            Ok(KeyAction::Continue)
        }
    }
}
