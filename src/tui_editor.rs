use crokey::{key, KeyCombination, KeyCombinationFormat};
use crossterm::event::Event;
use crossterm::terminal;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, Borders};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::env::var;
use std::fmt::{self, Debug};
use std::fs::OpenOptions;
use std::path::PathBuf;
use tui_textarea::{CursorMove, Input, TextArea};

use crate::colors::TUI_SELECTION_BG;
use crate::repl::resolve_term;
use crate::stdin::{apply_highlights, show_popup};
use crate::ThagError;

#[derive(Default, Serialize, Deserialize)]
struct History {
    entries: VecDeque<String>,
    current_index: Option<usize>,
}

pub trait EventReader: Debug {
    fn read_event(&self) -> Result<Event, ThagError>;
}

// Struct to hold data-related parameters
struct EditData<'a> {
    initial_content: &'a str,
    save_path: PathBuf,
    history_path: PathBuf,
    history: &'a mut History,
}

// Struct to hold display-related parameters
struct Display<'a> {
    title: &'a str,
    title_style: Style,
    remove_keys: &'a [&'a str; 1],
    add_keys: &'a [&'a [&'a str; 2]],
}

pub enum KeyAction {
    AbandonChanges,
    Continue, // For other inputs that don't need specific handling
    Quit,
    Save,
    SaveAndExit,
    ShowHelp,
    ToggleHighlight,
    TogglePopup,
}

// Main function to edit content with TUI
pub fn edit<R, F>(
    event_reader: &R,
    data: EditData,
    display: Display,
    key_handler: F, // closure or function for key handling
) -> Result<KeyAction, ThagError>
where
    R: EventReader + Debug,
    F: Fn(Event, &mut TextArea, &mut bool, &mut bool) -> Result<KeyAction, ThagError>,
{
    // Reading the initial content and setting up file for saving
    let initial_content = data.initial_content;
    let save_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(data.save_path)?;

    // Initialize state variables
    let mut popup = false;
    let mut tui_highlight_bg = &*TUI_SELECTION_BG;
    let mut saved = false;

    let mut maybe_term = resolve_term()?;

    // Create the TextArea from initial content
    let mut textarea = TextArea::from(initial_content.lines());

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

    let fmt = KeyCombinationFormat::default();

    // Event loop for handling key events
    loop {
        let event = if var("TEST_ENV").is_ok() {
            event_reader.read_event()?
        } else {
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
                        println!("Error drawing terminal: {:?}", e);
                        e
                    })?;

                    terminal::enable_raw_mode().unwrap();
                    let event = event_reader.read_event();
                    terminal::disable_raw_mode().unwrap();
                    event.map_err(Into::<ThagError>::into)
                },
            )?
        };

        // Call the key_handler closure to process events
        if let Ok(action) = key_handler(&event, &mut textarea) {
            match action {
                KeyAction::AbandonChanges => todo!(),
                KeyAction::Continue => todo!(),
                KeyAction::Quit => todo!(),
                KeyAction::Save => todo!(),
                KeyAction::SaveAndExit => todo!(),
                KeyAction::ShowHelp => todo!(),
                KeyAction::ToggleHighlight => {
                    tui_highlight_bg = match tui_highlight_bg {
                        crate::colors::TuiSelectionBg::BlueYellow => &TuiSelectionBg::RedWhite,
                        crate::colors::TuiSelectionBg::RedWhite => &TuiSelectionBg::BlueYellow,
                    };
                }
                KeyAction::TogglePopup => popup = !popup,
            }
            return Ok(action);
        }
    }
}

// Example of a key handler function that could be passed into `edit`
pub fn default_key_handler(
    event: Event,
    textarea: &mut TextArea,
    popup: &mut bool,
    saved: &mut bool,
) -> Result<KeyAction, ThagError> {
    if let Event::Key(key_event) = event {
        let key_combination = KeyCombination::from(key_event); // Derive KeyCombination

        match key_combination {
            key!(ctrl - c) | key!(ctrl - q) => Ok(KeyAction::Quit),
            key!(ctrl - d) => Ok(KeyAction::SaveAndExit),
            key!(ctrl - s) => {
                // Save logic
                println!("Save");
                *saved = true;
                Ok(KeyAction::Save)
            }
            key!(ctrl - l) => {
                // Toggle popup
                *popup = !*popup;
                Ok(KeyAction::TogglePopup)
            }
            _ => {
                textarea.input(Input::from(event)); // Input derived from Event
                Ok(KeyAction::Continue)
            }
        }
    } else {
        Ok(KeyAction::Continue)
    }
}
