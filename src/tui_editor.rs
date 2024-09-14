use crokey::{key, KeyCombination};
use crossterm::event::Event;
use crossterm::terminal;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::convert::Into;
use std::env::var;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::path::PathBuf;
use tui_textarea::{CursorMove, Input, TextArea};

use crate::colors::{TuiSelectionBg, TUI_SELECTION_BG};
use crate::repl::{resolve_term, stage_history};
use crate::stdin::{apply_highlights, show_popup};
use crate::ThagError;

#[derive(Default, Serialize, Deserialize)]
struct History {
    entries: VecDeque<String>,
    current_index: Option<usize>,
}

pub trait EventReader: Debug {
    /// Read a terminal event.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
    fn read_event(&self) -> Result<Event, ThagError>;
}

// Struct to hold data-related parameters
#[allow(dead_code)]
pub struct EditData<'a> {
    initial_content: &'a str,
    save_path: PathBuf,
    history_path: PathBuf,
    history: &'a mut History,
}

// Struct to hold display-related parameters
pub struct Display<'a> {
    title: &'a str,
    title_style: Style,
    remove_keys: &'a [&'a str; 1],
    add_keys: &'a [&'a [&'a str; 2]],
}

pub enum KeyAction {
    AbandonChanges,
    Continue, // For other inputs that don't need specific handling
    Quit(bool),
    Save,
    SaveAndExit,
    ShowHelp,
    ToggleHighlight,
    TogglePopup,
}

/// Edit content with a TUI
///
/// # Errors
///
/// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
pub fn edit<R, F>(
    event_reader: &R,
    data: &EditData,
    display: &Display,
    key_handler: F, // closure or function for key handling
) -> Result<KeyAction, ThagError>
where
    R: EventReader + Debug,
    F: Fn(Event, &EditData, &mut TextArea, &mut bool, &mut bool) -> Result<KeyAction, ThagError>,
{
    // Initialize state variables
    let mut popup = false;
    // let mut tui_highlight_bg = &*TUI_SELECTION_BG;
    let mut saved = false;

    let mut maybe_term = resolve_term()?;

    // Create the TextArea from initial content
    let mut textarea = TextArea::from(data.initial_content.lines());

    // Set up the display parameters for the textarea
    textarea.set_block(
        Block::default()
            .borders(Borders::NONE)
            .title(display.title)
            .title_style(display.title_style),
    );
    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
    textarea.move_cursor(CursorMove::Bottom);

    // Apply initial highlights
    apply_highlights(&TUI_SELECTION_BG, &mut textarea);

    // Event loop for handling key events
    loop {
        let event = if var("TEST_ENV").is_ok() {
            // Testing or CI
            event_reader.read_event()?
        } else {
            // Real-world interaction
            maybe_term.as_mut().map_or_else(
                || Err("Logic issue unwrapping term we wrapped ourselves".into()),
                |term| {
                    term.draw(|f| {
                        f.render_widget(&textarea, f.area());
                        if popup {
                            show_popup(f, display.remove_keys, display.add_keys);
                        };
                        apply_highlights(&TUI_SELECTION_BG, &mut textarea);
                    })
                    .map_err(|e| {
                        println!("Error drawing terminal: {e:?}");
                        e
                    })?;

                    terminal::enable_raw_mode()?;
                    let event = event_reader.read_event();
                    terminal::disable_raw_mode()?;
                    event.map_err(Into::<ThagError>::into)
                },
            )?
        };

        // Call the key_handler closure to process events
        let key_action = key_handler(event, data, &mut textarea, &mut popup, &mut saved)?;

        match key_action {
            KeyAction::AbandonChanges | KeyAction::Quit(_) | KeyAction::SaveAndExit => {
                break (Ok(key_action))
            }
            KeyAction::Continue
            | KeyAction::Save
            | KeyAction::ToggleHighlight
            | KeyAction::TogglePopup => continue,
            KeyAction::ShowHelp => todo!(),
        }
    }
}

/// Example of a key handler function that could be passed into `edit`
///
/// # Errors
///
/// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
pub fn history_key_handler(
    event: Event,
    mut maybe_term: Option<
        scopeguard::ScopeGuard<
            Terminal<CrosstermBackend<std::io::StdoutLock<'static>>>,
            impl FnOnce(Terminal<CrosstermBackend<std::io::StdoutLock<'static>>>),
        >,
    >,
    data: EditData,
    mut textarea: TextArea,
    popup: &mut bool,
    saved: &mut bool,
) -> Result<KeyAction, ThagError> {
    let mut tui_highlight_bg = &*TUI_SELECTION_BG;
    if let Event::Key(key_event) = event {
        let key_combination = KeyCombination::from(key_event); // Derive KeyCombination
        let save_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(data.save_path)?;

        match key_combination {
            #[allow(clippy::unnested_or_patterns)]
            key!(ctrl - c) | key!(ctrl - q) => Ok(KeyAction::Quit(*saved)),
            key!(ctrl - d) => {
                // Save logic
                stage_history(&save_file, &textarea)?;
                println!("Saved");
                Ok(KeyAction::SaveAndExit)
            }
            key!(ctrl - s) => {
                // Save logic
                stage_history(&save_file, &textarea)?;
                println!("Saved");
                *saved = true;
                Ok(KeyAction::Save)
            }
            key!(ctrl - l) => {
                // Toggle popup
                *popup = !*popup;
                Ok(KeyAction::TogglePopup)
            }
            key!(ctrl - t) => {
                // Toggle highlighting colours
                tui_highlight_bg = match tui_highlight_bg {
                    TuiSelectionBg::BlueYellow => &TuiSelectionBg::RedWhite,
                    TuiSelectionBg::RedWhite => &TuiSelectionBg::BlueYellow,
                };
                if var("TEST_ENV").is_err() {
                    #[allow(clippy::option_if_let_else)]
                    if let Some(ref mut term) = maybe_term {
                        term.draw(|_| {
                            apply_highlights(tui_highlight_bg, &mut textarea);
                        })
                        .map_err(Into::into)
                        .map(|_| KeyAction::Continue)
                    } else {
                        Ok(KeyAction::Continue)
                    }
                } else {
                    Ok(KeyAction::Continue)
                }
            }
            key!(f3) => {
                // Ask to revert
                Ok(KeyAction::AbandonChanges)
            }
            _ => {
                // Update the textarea with the input from the key event
                textarea.input(Input::from(event)); // Input derived from Event
                Ok(KeyAction::Continue)
            }
        }
    } else {
        Ok(KeyAction::Continue)
    }
}
