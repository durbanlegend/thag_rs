use crokey::{key, KeyCombination};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
};
use crossterm::{
    event::{
        DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event::{self, Paste},
        KeyEvent,
    },
    terminal::LeaveAlternateScreen,
};
use firestorm::profile_fn;
use lazy_static::lazy_static;
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
use regex::Regex;
use scopeguard::{guard, ScopeGuard};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::convert::Into;
use std::env::var;
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;
use std::{self, fs};
use tui_textarea::{CursorMove, Input, TextArea};

use crate::colors::coloring;
use crate::{
    colors::{tui_selection_bg, TuiSelectionBg},
    shared::KeyDisplayLine,
};
use crate::{debug_log, MessageLevel, ThagError, ThagResult};

pub type BackEnd = CrosstermBackend<std::io::StdoutLock<'static>>;
pub type Term = Terminal<BackEnd>;
pub type ResetTermClosure = Box<dyn FnOnce(Term)>;
pub type TermScopeGuard = ScopeGuard<Term, ResetTermClosure>;

pub const TITLE_TOP: &str = "Key bindings - subject to your terminal settings";
pub const TITLE_BOTTOM: &str = "Ctrl+l to hide";

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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct History {
    entries: VecDeque<String>,
    pub current_index: Option<usize>,
}

impl History {
    fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(20),
            current_index: None,
        }
    }

    #[must_use]
    pub fn load_from_file(path: &PathBuf) -> Self {
        fs::read_to_string(path).map_or_else(
            |_| Self::default(),
            |data| serde_json::from_str(&data).unwrap_or_else(|_| Self::new()),
        )
    }

    pub fn save_to_file(&self, path: &PathBuf) {
        if let Ok(data) = serde_json::to_string(self) {
            let _ = fs::write(path, data);
        }
    }

    pub fn add_entry(&mut self, entry: String) {
        // Remove prior duplicates
        self.entries.retain(|f| f != &entry);
        self.entries.push_front(entry);
    }

    pub fn get_current(&mut self) -> Option<&String> {
        if self.entries.is_empty() {
            return None;
        }

        self.current_index = self.current_index.map_or(Some(0), |index| Some(index + 1));
        self.entries.front()
    }

    pub fn get_previous(&mut self) -> Option<&String> {
        if self.entries.is_empty() {
            return None;
        }

        self.current_index = self.current_index.map_or(Some(0), |index| Some(index + 1));
        self.current_index.and_then(|index| self.entries.get(index))
    }

    pub fn get_next(&mut self) -> Option<&String> {
        if self.entries.is_empty() {
            return None;
        }

        self.current_index = match self.current_index {
            Some(index) if index > 0 => Some(index - 1),
            Some(index) if index == 0 => Some(index + self.entries.len() - 1),
            _ => Some(self.entries.len() - 1),
        };

        self.current_index.and_then(|index| self.entries.get(index))
    }
}

/// A trait to allow mocking of the event reader for testing purposes.
#[automock]
pub trait EventReader {
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
#[derive(Debug, Default)]
pub struct EditData<'a> {
    pub return_text: bool,
    pub initial_content: &'a str,
    pub save_path: Option<PathBuf>,
    pub history_path: Option<PathBuf>,
    pub history: Option<History>,
}

// Struct to hold display-related parameters
#[derive(Debug)]
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
    let maybe_save_file = if let Some(ref save_path) = edit_data.save_path {
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
    let mut tui_highlight_bg = tui_selection_bg(coloring().1);
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
    apply_highlights(&tui_selection_bg(coloring().1), &mut textarea);

    // Event loop for handling key events
    loop {
        maybe_enable_raw_mode()?;
        let test_env = &var("TEST_ENV");
        let event = if test_env.is_ok() {
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
                                MAPPINGS,
                                f,
                                TITLE_TOP,
                                TITLE_BOTTOM,
                                display.remove_keys,
                                display.add_keys,
                            );
                        };
                        apply_highlights(&tui_highlight_bg, &mut textarea);
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
                        TuiSelectionBg::BlueYellow => TuiSelectionBg::RedWhite,
                        TuiSelectionBg::RedWhite => TuiSelectionBg::BlueYellow,
                    };
                    if var("TEST_ENV").is_err() {
                        #[allow(clippy::option_if_let_else)]
                        if let Some(ref mut term) = maybe_term {
                            term.draw(|_| {
                                apply_highlights(&tui_highlight_bg, &mut textarea);
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

/// Enable raw mode, but not if in test mode, because that will cause the dreaded rightward drift
/// in log output due to carriage returns being ignored.
///
/// # Errors
///
/// This function will bubble up any i/o errors encountered by `crossterm::enable_raw_mode`.
pub fn maybe_enable_raw_mode() -> ThagResult<()> {
    let test_env = &var("TEST_ENV");
    #[cfg(debug_assertions)]
    debug_log!("test_env={test_env:?}");
    if !test_env.is_ok() && !is_raw_mode_enabled()? {
        #[cfg(debug_assertions)]
        debug_log!("Enabling raw mode");
        enable_raw_mode()?;
    }
    Ok(())
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

#[must_use]
pub fn centered_rect(max_width: u16, max_height: u16, r: Rect) -> Rect {
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

/// Convert the different newline sequences for Windows and other platforms into the common
/// standard sequence of `"\n"` (backslash + 'n', as opposed to the '\n' (0xa) character for which
/// it stands).
#[must_use]
pub fn normalize_newlines(input: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\r\n?").unwrap();
    }
    RE.replace_all(input, "\n").to_string()
}

/// Apply highlights to the text depending on the light or dark theme as detected, configured
/// or defaulted, or as toggled by the user with Ctrl-t.
pub fn apply_highlights(scheme: &TuiSelectionBg, textarea: &mut TextArea) {
    profile_fn!(apply_highlights);
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

#[macro_export]
macro_rules! key_mappings {
    (
        $(($seq:expr, $keys:expr, $desc:expr)),* $(,)?
    ) => {
        &[
            $(
                KeyDisplayLine {
                    seq: $seq,
                    keys: $keys,
                    desc: $desc,
                }
            ),*
        ]
    };
}

pub const MAPPINGS: &[KeyDisplayLine] = key_mappings![
    (10, "Key bindings", "Description"),
    (
        20,
        "Shift+arrow keys",
        "Select/deselect chars (←→) or lines (↑↓)"
    ),
    (
        30,
        "Shift+Ctrl+arrow keys",
        "Select/deselect words (←→) or paras (↑↓)"
    ),
    (40, "Alt+c", "Cancel selection"),
    (50, "Ctrl+d", "Submit"),
    (60, "Ctrl+q", "Cancel and quit"),
    (70, "Ctrl+h, Backspace", "Delete character before cursor"),
    (80, "Ctrl+i, Tab", "Indent"),
    (90, "Ctrl+m, Enter", "Insert newline"),
    (100, "Ctrl+k", "Delete from cursor to end of line"),
    (110, "Ctrl+j", "Delete from cursor to start of line"),
    (
        120,
        "Ctrl+w, Alt+Backspace",
        "Delete one word before cursor"
    ),
    (130, "Alt+d, Delete", "Delete one word from cursor position"),
    (140, "Ctrl+u", "Undo"),
    (150, "Ctrl+r", "Redo"),
    (160, "Ctrl+c", "Copy (yank) selected text"),
    (170, "Ctrl+x", "Cut (yank) selected text"),
    (180, "Ctrl+y", "Paste yanked text"),
    (
        190,
        "Ctrl+v, Shift+Ins, Cmd+v",
        "Paste from system clipboard"
    ),
    (200, "Ctrl+f, →", "Move cursor forward one character"),
    (210, "Ctrl+b, ←", "Move cursor backward one character"),
    (220, "Ctrl+p, ↑", "Move cursor up one line"),
    (230, "Ctrl+n, ↓", "Move cursor down one line"),
    (240, "Alt+f, Ctrl+→", "Move cursor forward one word"),
    (250, "Alt+Shift+f", "Move cursor to next word end"),
    (260, "Atl+b, Ctrl+←", "Move cursor backward one word"),
    (270, "Alt+) or p, Ctrl+↑", "Move cursor up one paragraph"),
    (280, "Alt+( or n, Ctrl+↓", "Move cursor down one paragraph"),
    (
        290,
        "Ctrl+e, End, Ctrl+Alt+f or → , Cmd+→",
        "Move cursor to end of line"
    ),
    (
        300,
        "Ctrl+a, Home, Ctrl+Alt+b or ← , Cmd+←",
        "Move cursor to start of line"
    ),
    (310, "Alt+<, Ctrl+Alt+p or ↑", "Move cursor to top of file"),
    (
        320,
        "Alt+>, Ctrl+Alt+n or ↓",
        "Move cursor to bottom of file"
    ),
    (330, "PageDown, Cmd+↓", "Page down"),
    (340, "Alt+v, PageUp, Cmd+↑", "Page up"),
    (350, "Ctrl+l", "Toggle keys display (this screen)"),
    (360, "Ctrl+t", "Toggle highlight colours"),
    (370, "F1", "Previous in history"),
    (380, "F2", "Next in history"),
    (
        390,
        "F9",
        "Suspend mouse capture and line numbers for system copy"
    ),
    (400, "F10", "Resume mouse capture and line numbers"),
];
