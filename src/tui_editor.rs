use crate::MessageLevel;
use crokey::{key, KeyCombination};
use crossterm::event::{
    DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event::{self, Paste},
    KeyEvent,
};
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use mockall::automock;
use ratatui::prelude::{CrosstermBackend, Rect};
use ratatui::style::{Color, Modifier, Style, Styled};
use ratatui::text::Line;
use ratatui::widgets::{block::Block, Borders, Clear, Paragraph};
use ratatui::Terminal;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::Stylize,
};
use scopeguard::{guard, ScopeGuard};
use serde::{Deserialize, Serialize};
use std;
use std::collections::VecDeque;
use std::convert::Into;
use std::env::var;
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use tui_textarea::{CursorMove, Input, TextArea};

use crate::stdin::{apply_highlights, normalize_newlines, reset_term};
use crate::{
    colors::{TuiSelectionBg, TUI_SELECTION_BG},
    shared::KeyDisplayLine,
};
use crate::{ThagError, ThagResult};

pub type BackEnd = CrosstermBackend<std::io::StdoutLock<'static>>;
pub type Term = Terminal<BackEnd>;
pub type ResetTermClosure = Box<dyn FnOnce(Term)>;
pub type TermScopeGuard = ScopeGuard<Term, ResetTermClosure>;

pub(crate) const TITLE_TOP: &str = "Key bindings - subject to your terminal settings";
pub(crate) const TITLE_BOTTOM: &str = "Ctrl+l to hide";

/// Determine whether a terminal is in use (as opposed to testing or headless CI), and
/// if so, wrap it in a scopeguard in order to reset it regardless of success or failure.
///
/// # Panics
///
/// Panics if a `crossterm` error is encountered resetting the terminal inside a
/// `scopeguard::guard` closure.
///
/// # Errors
///
pub fn resolve_term() -> ThagResult<Option<TermScopeGuard>> {
    let maybe_term = if var("TEST_ENV").is_ok() {
        None
    } else {
        let mut stdout = std::io::stdout().lock();

        enable_raw_mode()?;

        crossterm::execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste
        )?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Box the closure explicitly as `Box<dyn FnOnce>`
        let term = guard(
            terminal,
            Box::new(|term| {
                reset_term(term).expect("Error resetting terminal");
            }) as ResetTermClosure,
        );

        Some(term)
    };
    Ok(maybe_term)
}

#[derive(Default, Serialize, Deserialize)]
pub struct History {
    entries: VecDeque<String>,
    current_index: Option<usize>,
}

/// A trait to allow mocking of the event reader for testing purposes.
#[automock]
pub trait EventReader: Debug {
    /// Read a terminal event.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
    fn read_event(&self) -> ThagResult<Event>;
}

/// A struct to implement real-world use of the event reader, as opposed to use in testing.
#[derive(Debug)]
pub struct CrosstermEventReader;

impl EventReader for CrosstermEventReader {
    fn read_event(&self) -> ThagResult<Event> {
        crossterm::event::read().map_err(Into::<ThagError>::into)
    }
}

// Struct to hold data-related parameters
#[allow(dead_code)]
pub struct EditData<'a> {
    pub return_text: bool,
    pub initial_content: String,
    pub save_path: &'a Option<PathBuf>,
    // pub history_path: &'a Option<PathBuf>,
    // pub history: &'a mut Option<History>,
}

// Struct to hold display-related parameters
pub struct Display<'a> {
    pub title: &'a str,
    pub title_style: Style,
    pub remove_keys: &'a [&'a str],
    pub add_keys: &'a [KeyDisplayLine],
}

#[derive(Debug)]
pub enum KeyAction {
    AbandonChanges,
    Continue, // For other inputs that don't need specific handling
    Quit(bool),
    Save,
    SaveAndExit,
    ShowHelp,
    SaveAndSubmit,
    Submit,
    ToggleHighlight,
    TogglePopup,
}

/// Edit content with a TUI
///
/// # Panics
///
/// Panics if a `crossterm` error is encountered resetting the terminal inside a
/// `scopeguard::guard` closure in the call to `resolve_term`.
///
/// # Errors
///
/// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
#[allow(clippy::too_many_lines)]
pub fn tui_edit<R, F>(
    event_reader: &R,
    edit_data: &EditData,
    display: &Display,
    key_handler: F, // closure or function for key handling
) -> ThagResult<(KeyAction, Option<Vec<String>>)>
where
    R: EventReader + Debug,
    F: Fn(
        KeyEvent,
        &mut Option<TermScopeGuard>,
        &Option<File>,
        &mut TextArea,
        &mut bool,
        &mut bool,
    ) -> ThagResult<KeyAction>,
{
    // Initialize save file
    let maybe_save_file = if let Some(save_path) = edit_data.save_path {
        let save_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(save_path)?;
        Some(save_file)
    } else {
        None
    };

    // Initialize state variables
    let mut popup = false;
    let mut tui_highlight_bg = &*TUI_SELECTION_BG;
    let mut saved = false;

    let mut maybe_term = resolve_term()?;

    // Create the TextArea from initial content
    let mut textarea = TextArea::from(edit_data.initial_content.lines());

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
                            show_popup(
                                &get_mappings(),
                                f,
                                TITLE_TOP,
                                TITLE_BOTTOM,
                                display.remove_keys,
                                display.add_keys,
                            );
                        };
                        apply_highlights(tui_highlight_bg, &mut textarea);
                    })
                    .map_err(|e| {
                        eprintln!("Error drawing terminal: {e:?}");
                        e
                    })?;

                    // NB: leave in raw mode until end of session to avoid random appearance of OSC codes on screen
                    let event = event_reader.read_event();
                    event.map_err(Into::<ThagError>::into)
                },
            )?
        };

        if let Paste(ref data) = event {
            textarea.insert_str(normalize_newlines(data));
        } else if let Event::Key(key_event) = event {
            let key_combination = KeyCombination::from(key_event); // Derive KeyCombination

            // If using iterm2, ensure Settings | Profiles | Keys | Left Option key is set to Esc+.
            #[allow(clippy::unnested_or_patterns)]
            match key_combination {
                key!(ctrl - h) | key!(backspace) => {
                    textarea.delete_char();
                }
                key!(ctrl - i) | key!(tab) => {
                    textarea.indent();
                }
                key!(ctrl - m) | key!(enter) => {
                    textarea.insert_newline();
                }
                key!(ctrl - k) => {
                    textarea.delete_line_by_end();
                }
                key!(ctrl - j) => {
                    textarea.delete_line_by_head();
                }
                key!(ctrl - w) | key!(alt - backspace) => {
                    textarea.delete_word();
                }
                key!(alt - d) => {
                    textarea.delete_next_word();
                }
                key!(ctrl - u) => {
                    textarea.undo();
                }
                key!(ctrl - r) => {
                    textarea.redo();
                }
                key!(ctrl - c) => {
                    textarea.yank_text();
                }
                key!(ctrl - x) => {
                    textarea.cut();
                }
                key!(ctrl - y) => {
                    textarea.paste();
                }
                key!(ctrl - f) | key!(right) => {
                    textarea.move_cursor(CursorMove::Forward);
                }
                key!(ctrl - b) | key!(left) => {
                    textarea.move_cursor(CursorMove::Back);
                }
                key!(ctrl - p) | key!(up) => {
                    textarea.move_cursor(CursorMove::Up);
                }
                key!(ctrl - n) | key!(down) => {
                    textarea.move_cursor(CursorMove::Down);
                }
                key!(alt - f) | key!(ctrl - right) => {
                    textarea.move_cursor(CursorMove::WordForward);
                }
                key!(alt - shift - f) => {
                    textarea.move_cursor(CursorMove::WordEnd);
                }
                key!(alt - b) | key!(ctrl - left) => {
                    textarea.move_cursor(CursorMove::WordBack);
                }
                key!(alt - p) | key!(alt - ')') | key!(ctrl - up) => {
                    textarea.move_cursor(CursorMove::ParagraphBack);
                }
                key!(alt - n) | key!(alt - '(') | key!(ctrl - down) => {
                    textarea.move_cursor(CursorMove::ParagraphForward);
                }
                key!(ctrl - e) | key!(end) | key!(ctrl - alt - f) | key!(ctrl - alt - right) => {
                    textarea.move_cursor(CursorMove::End);
                }
                key!(ctrl - a) | key!(home) | key!(ctrl - alt - b) | key!(ctrl - alt - left) => {
                    textarea.move_cursor(CursorMove::Head);
                }
                key!(f9) => {
                    if maybe_term.is_some() {
                        crossterm::execute!(std::io::stdout().lock(), DisableMouseCapture,)?;
                        textarea.remove_line_number();
                    }
                }
                key!(f10) => {
                    if maybe_term.is_some() {
                        crossterm::execute!(std::io::stdout().lock(), EnableMouseCapture,)?;
                        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
                    }
                }

                //
                key!(alt - '<') | key!(ctrl - alt - p) | key!(ctrl - alt - up) => {
                    textarea.move_cursor(CursorMove::Top);
                }
                key!(alt - '>') | key!(ctrl - alt - n) | key!(ctrl - alt - down) => {
                    textarea.move_cursor(CursorMove::Bottom);
                }
                key!(alt - c) => {
                    textarea.cancel_selection();
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
                            })?;
                        }
                    }
                }
                _ => {
                    // Call the key_handler closure to process events
                    let key_action = key_handler(
                        key_event,
                        &mut maybe_term,
                        &maybe_save_file,
                        &mut textarea,
                        &mut popup,
                        &mut saved,
                    )?;
                    // eprintln!("key_action={key_action:?}");
                    match key_action {
                        KeyAction::AbandonChanges => break Ok((key_action, None::<Vec<String>>)),
                        KeyAction::Quit(_)
                        | KeyAction::SaveAndExit
                        | KeyAction::SaveAndSubmit
                        | KeyAction::Submit => {
                            let maybe_text = if edit_data.return_text {
                                Some(textarea.lines().to_vec())
                            } else {
                                None::<Vec<String>>
                            };
                            break Ok((key_action, maybe_text));
                        }
                        KeyAction::Continue
                        | KeyAction::Save
                        | KeyAction::ToggleHighlight
                        | KeyAction::TogglePopup => continue,
                        KeyAction::ShowHelp => todo!(),
                    }
                }
            }
        } else {
            // println!("You typed {} which represents nothing yet", key.blue());
            let input = Input::from(event);
            textarea.input(input);
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::missing_panics_doc)]
pub fn show_popup<'a>(
    mappings: &'a [KeyDisplayLine],
    f: &mut ratatui::prelude::Frame,
    title_top: &str,
    title_bottom: &str,
    remove: &[&str],
    add: &'a [KeyDisplayLine],
) {
    let adjusted_mappings: Vec<KeyDisplayLine> = mappings
        .iter()
        .filter(|&row| !remove.contains(&row.keys))
        .chain(add.iter())
        .cloned()
        .collect();
    let (max_key_len, max_desc_len) =
        adjusted_mappings
            .iter()
            .fold((0_u16, 0_u16), |(max_key, max_desc), row| {
                let key_len = row.keys.len().try_into().unwrap();
                let desc_len = row.desc.len().try_into().unwrap();
                (max_key.max(key_len), max_desc.max(desc_len))
            });
    let num_filtered_rows = adjusted_mappings.len();
    let area = centered_rect(
        max_key_len + max_desc_len + 5,
        num_filtered_rows as u16 + 5,
        f.area(),
    );
    let inner = area.inner(Margin {
        vertical: 2,
        horizontal: 2,
    });
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(title_top).centered())
        .title_bottom(Line::from(title_bottom).centered())
        .add_modifier(Modifier::BOLD)
        .fg(Color::Indexed(u8::from(&MessageLevel::Heading)));
    // this is supposed to clear out the background
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    let row_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            std::iter::repeat(Constraint::Ratio(1, num_filtered_rows as u32))
                .take(num_filtered_rows),
        );
    let rows = row_layout.split(inner);

    for (i, row) in rows.iter().enumerate() {
        let col_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(max_key_len + 1),
                    Constraint::Length(max_desc_len),
                ]
                .as_ref(),
            );
        let cells = col_layout.split(*row);
        let mut widget = Paragraph::new(adjusted_mappings[i].keys);
        if i == 0 {
            widget = widget
                .add_modifier(Modifier::BOLD)
                .fg(Color::Indexed(u8::from(&MessageLevel::Emphasis)));
        } else {
            widget = widget
                .fg(Color::Indexed(u8::from(&MessageLevel::Subheading)))
                .not_bold();
        }
        f.render_widget(widget, cells[0]);
        let mut widget = Paragraph::new(adjusted_mappings[i].desc);

        if i == 0 {
            widget = widget
                .add_modifier(Modifier::BOLD)
                .fg(Color::Indexed(u8::from(&MessageLevel::Emphasis)));
        } else {
            widget = widget.remove_modifier(Modifier::BOLD).set_style(
                Style::default()
                    .fg(Color::Indexed(u8::from(&MessageLevel::Normal)))
                    .not_bold(),
            );
        }
        f.render_widget(widget, cells[1]);
    }
}

pub(crate) fn centered_rect(max_width: u16, max_height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Max(max_height),
        Constraint::Fill(1),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Max(max_width),
        Constraint::Fill(1),
    ])
    .split(popup_layout[1])[1]
}

pub(crate) fn get_mappings() -> Vec<KeyDisplayLine> {
    vec![
        KeyDisplayLine::new(10, "Key Bindings", "Description"),
        KeyDisplayLine::new(
            20,
            "Shift+arrow keys",
            "Select/deselect chars (←→) or lines (↑↓)",
        ),
        KeyDisplayLine::new(
            30,
            "Shift+Ctrl+arrow keys",
            "Select/deselect words (←→) or paras (↑↓)",
        ),
        KeyDisplayLine::new(40, "Alt+c", "Cancel selection"),
        KeyDisplayLine::new(50, "Ctrl+d", "Submit"),
        KeyDisplayLine::new(60, "Ctrl+q", "Cancel and quit"),
        KeyDisplayLine::new(70, "Ctrl+h, Backspace", "Delete character before cursor"),
        KeyDisplayLine::new(80, "Ctrl+i, Tab", "Indent"),
        KeyDisplayLine::new(90, "Ctrl+m, Enter", "Insert newline"),
        KeyDisplayLine::new(100, "Ctrl+k", "Delete from cursor to end of line"),
        KeyDisplayLine::new(110, "Ctrl+j", "Delete from cursor to start of line"),
        KeyDisplayLine::new(
            120,
            "Ctrl+w, Alt+Backspace",
            "Delete one word before cursor",
        ),
        KeyDisplayLine::new(130, "Alt+d, Delete", "Delete one word from cursor position"),
        KeyDisplayLine::new(140, "Ctrl+u", "Undo"),
        KeyDisplayLine::new(150, "Ctrl+r", "Redo"),
        KeyDisplayLine::new(
            160,
            "Ctrl+c",
            "Copy KeyDisplayLine::new(yank) selected text",
        ),
        KeyDisplayLine::new(170, "Ctrl+x", "Cut KeyDisplayLine::new(yank) selected text"),
        KeyDisplayLine::new(180, "Ctrl+y", "Paste yanked text"),
        KeyDisplayLine::new(
            190,
            "Ctrl+v, Shift+Ins, Cmd+v",
            "Paste from system clipboard",
        ),
        KeyDisplayLine::new(200, "Ctrl+f, →", "Move cursor forward one character"),
        KeyDisplayLine::new(210, "Ctrl+b, ←", "Move cursor backward one character"),
        KeyDisplayLine::new(220, "Ctrl+p, ↑", "Move cursor up one line"),
        KeyDisplayLine::new(230, "Ctrl+n, ↓", "Move cursor down one line"),
        KeyDisplayLine::new(240, "Alt+f, Ctrl+→", "Move cursor forward one word"),
        KeyDisplayLine::new(250, "Alt+Shift+f", "Move cursor to next word end"),
        KeyDisplayLine::new(260, "Atl+b, Ctrl+←", "Move cursor backward one word"),
        KeyDisplayLine::new(270, "Alt+) or p, Ctrl+↑", "Move cursor up one paragraph"),
        KeyDisplayLine::new(
            280,
            "Alt+KeyDisplayLine::new( or n, Ctrl+↓",
            "Move cursor down one paragraph",
        ),
        KeyDisplayLine::new(
            290,
            "Ctrl+e, End, Ctrl+Alt+f or → , Cmd+→",
            "Move cursor to end of line",
        ),
        KeyDisplayLine::new(
            300,
            "Ctrl+a, Home, Ctrl+Alt+b or ← , Cmd+←",
            "Move cursor to start of line",
        ),
        KeyDisplayLine::new(310, "Alt+<, Ctrl+Alt+p or ↑", "Move cursor to top of file"),
        KeyDisplayLine::new(
            320,
            "Alt+>, Ctrl+Alt+n or ↓",
            "Move cursor to bottom of file",
        ),
        KeyDisplayLine::new(330, "PageDown, Cmd+↓", "Page down"),
        KeyDisplayLine::new(340, "Alt+v, PageUp, Cmd+↑", "Page up"),
        KeyDisplayLine::new(
            350,
            "Ctrl+l",
            "Toggle keys display KeyDisplayLine::new(this screen)",
        ),
        KeyDisplayLine::new(360, "Ctrl+t", "Toggle highlight colours"),
        KeyDisplayLine::new(370, "F1", "Previous in history"),
        KeyDisplayLine::new(380, "F2", "Next in history"),
        KeyDisplayLine::new(
            390,
            "F9",
            "Suspend mouse capture and line numbers for system copy",
        ),
        KeyDisplayLine::new(400, "F10", "Resume mouse capture and line numbers"),
    ]
}
