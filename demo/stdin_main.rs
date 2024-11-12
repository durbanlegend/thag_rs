/*[toml]
[dependencies]
crossterm = "0.28.1"
lazy_static = "1.4.0"
mockall = "0.13.0"
ratatui = "0.28.1"
regex = "1.10.4"
scopeguard = "1.2.0"
serde = "1.0.210"
serde_json = "1.0.132"
thag_rs = "0.1.7"
tui-textarea = { version = "0.6", features = ["search"] }
*/

#![allow(clippy::uninlined_format_args)]

/// A version of `thag_rs`'s `stdin` module from the `main` `git` branch for the purpose of comparison
/// with the `develop` branch version being debugged.
///
/// E.g. `thag demo/stdin_main.rs`
//# Purpose: Debugging.
use thag_rs::errors::ThagError;
use thag_rs::log;
use thag_rs::logging::Verbosity;

use crossterm::event::{
    DisableMouseCapture,
    EnableBracketedPaste,
    EnableMouseCapture,
    Event::{self, Paste},
    // KeyCode, KeyEvent, KeyModifiers,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use lazy_static::lazy_static;
use mockall::{automock, predicate::str};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin};
use ratatui::prelude::Rect;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::block::Title;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Terminal;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{self, BufRead, IsTerminal};
use std::{collections::VecDeque, fs, path::PathBuf};
use tui_textarea::{CursorMove, Input, Key, TextArea};

#[derive(Default, Serialize, Deserialize)]
struct History {
    entries: VecDeque<String>,
    current_index: Option<usize>,
}

impl History {
    fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(20),
            current_index: None,
        }
    }

    fn load_from_file(path: &PathBuf) -> Self {
        fs::read_to_string(path).map_or_else(
            |_| Self::default(),
            |data| serde_json::from_str(&data).unwrap_or_else(|_| Self::new()),
        )
    }

    fn save_to_file(&self, path: &PathBuf) {
        if let Ok(data) = serde_json::to_string(self) {
            let _ = fs::write(path, data);
        }
    }

    fn add_entry(&mut self, entry: String) {
        // Remove prior duplicates
        self.entries.retain(|f| f != &entry);
        self.entries.push_front(entry);
    }

    fn get_current(&mut self) -> Option<&String> {
        if self.entries.is_empty() {
            return None;
        }

        self.current_index = self.current_index.map_or(Some(0), |index| Some(index + 1));
        self.entries.front()
    }

    fn get_previous(&mut self) -> Option<&String> {
        if self.entries.is_empty() {
            return None;
        }

        self.current_index = self.current_index.map_or(Some(0), |index| Some(index + 1));
        self.current_index.and_then(|index| self.entries.get(index))
    }

    fn get_next(&mut self) -> Option<&String> {
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

// A trait to allow mocking of the event reader for testing purposes.
#[automock]
pub trait EventReader {
    // Read a `crossterm` event.
    // # Errors
    //
    // If the timeout expires then an error is returned and buf is unchanged.
    fn read_event(&self) -> Result<Event, std::io::Error>;
}

// A struct to implement real-world use of the event reader, as opposed to use in testing.
pub struct CrosstermEventReader;

impl EventReader for CrosstermEventReader {
    fn read_event(&self) -> Result<Event, std::io::Error> {
        crossterm::event::read()
    }
}

#[allow(dead_code)]
fn main() -> Result<(), ThagError> {
    let event_reader = CrosstermEventReader;
    for line in &edit(&event_reader)? {
        log!(Verbosity::Normal, "{line}");
    }
    Ok(())
}

// Edit the stdin stream.
//
//
// # Examples
//
// ```no_run
// use thag_rs::stdin::{edit, CrosstermEventReader};
// use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers };
// # use thag_rs::stdin::MockEventReader;
//
// # let mut event_reader = MockEventReader::new();
// # event_reader.expect_read_event().return_once(|| {
// #     Ok(Event::Key(KeyEvent::new(
// #         KeyCode::Char('d'),
// #         KeyModifiers::CONTROL,
// #     )))
// # });
// let actual = edit(&event_reader);
// let buf = vec![""];
// assert!(matches!(actual, Ok(buf)));
// ```
// # Errors
//
// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
// # Panics
//
// If the terminal cannot be reset.
#[allow(clippy::too_many_lines)]
pub fn edit<R: EventReader>(event_reader: &R) -> Result<Vec<String>, ThagError> {
    let input = std::io::stdin();
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let history_path = PathBuf::from(cargo_home).join("rs_stdin_history.json");

    let mut history = History::load_from_file(&history_path);

    let mut saved_to_history = false;

    let initial_content = if input.is_terminal() {
        String::new()
    } else {
        read()?
    };

    let mut popup = false;
    let mut alt_highlights = false;

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;

    crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )
    .map_err(|e| {
        // println!("Error executing terminal commands: {:?}", e);
        e
    })?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?; // Ensure terminal will get reset when it goes out of scope.
    let mut term = scopeguard::guard(terminal, |term| {
        reset_term(term).expect("Error resetting terminal");
    });

    let mut textarea = TextArea::from(initial_content.lines());

    textarea.set_block(
        Block::default()
            .borders(Borders::NONE)
            .title("Enter / paste / edit Rust script. Ctrl+D: submit  Ctrl+Q: quit  Ctrl+L: keys")
            .title_style(Style::default().fg(Color::Yellow).bold().italic()),
    );
    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
    textarea.set_selection_style(Style::default().bg(Color::Blue));
    textarea.set_cursor_style(Style::default().on_magenta());
    textarea.set_cursor_line_style(Style::default().on_dark_gray());

    textarea.move_cursor(CursorMove::Bottom);

    apply_highlights(alt_highlights, &mut textarea);

    loop {
        term.draw(|f| {
            f.render_widget(&textarea, f.area());
            if popup {
                show_popup(f);
            }
            apply_highlights(alt_highlights, &mut textarea);
        })
        .map_err(|e| {
            println!("Error drawing terminal: {:?}", e);
            e
        })?;
        let event = event_reader.read_event().map_err(|e| {
            println!("Error reading event: {:?}", e);
            e
        })?;
        if let Paste(ref data) = event {
            textarea.insert_str(normalize_newlines(data));
        } else {
            let input = Input::from(event);
            match input {
                Input {
                    key: Key::Char('q'),
                    ctrl: true,
                    ..
                } => {
                    return Err(ThagError::Cancelled);
                }
                Input {
                    key: Key::Char('d'),
                    ctrl: true,
                    ..
                } => {
                    // 6 >5,4,3,2,1 -> 6 >6,5,4,3,2,1
                    history.add_entry(textarea.lines().to_vec().join("\n"));
                    history.current_index = Some(0);
                    history.save_to_file(&history_path);
                    break;
                }
                Input {
                    key: Key::Char('l'),
                    ctrl: true,
                    ..
                } => popup = !popup,
                Input {
                    key: Key::Char('t'),
                    ctrl: true,
                    ..
                } => {
                    alt_highlights = !alt_highlights;
                    term.draw(|_| {
                        apply_highlights(alt_highlights, &mut textarea);
                    })?;
                }
                Input { key: Key::F(1), .. } => {
                    let mut found = false;
                    // 6 5,4,3,2,1 -> >5,4,3,2,1
                    if saved_to_history {
                        if let Some(entry) = history.get_previous() {
                            // 5
                            found = true;
                            textarea.select_all();
                            textarea.cut(); // 6
                            textarea.insert_str(entry); // 5
                        }
                    } else {
                        // println!("Not already saved to history: calling history.get_current()");
                        if let Some(entry) = history.get_current() {
                            found = true;
                            textarea.select_all();
                            textarea.cut(); // 6
                            textarea.insert_str(entry); // 5
                        }
                    }
                    if found && !saved_to_history && !textarea.yank_text().is_empty() {
                        // 5 >5,4,3,2,1 -> 5 6,>5,4,3,2,1
                        history
                            .add_entry(textarea.yank_text().lines().collect::<Vec<_>>().join("\n"));
                        saved_to_history = true;
                    }
                }
                Input { key: Key::F(2), .. } => {
                    // 5 >6,5,4,3,2,1 ->
                    if let Some(entry) = history.get_next() {
                        textarea.select_all();
                        textarea.cut();
                        textarea.insert_str(entry);
                    }
                }
                input => {
                    textarea.input(input);
                }
            }
        }
    }

    Ok(textarea.lines().to_vec())
}

// Prompt for and read Rust source code from stdin.
//
// # Examples
//
// ```
// use thag_rs::stdin::read;
//
// let hello = String::from("Hello world!");
// assert!(matches!(read(), Ok(hello)));
// ```
// # Errors
//
// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
pub fn read() -> Result<String, std::io::Error> {
    log!(Verbosity::Normal, "Enter or paste lines of Rust source code at the prompt and press Ctrl-D on a new line when done");
    let buffer = read_to_string(&mut std::io::stdin().lock())?;
    Ok(buffer)
}

// Read the input from a `BufRead` implementing item into a String.
//
// # Examples
//
// ```
// use thag_rs::stdin::read_to_string;
//
// let stdin = std::io::stdin();
// let mut input = stdin.lock();
// let hello = String::from("Hello world!");
// assert!(matches!(read_to_string(&mut input), Ok(hello)));
// ```
//
// # Errors
//
// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
pub fn read_to_string<R: BufRead>(input: &mut R) -> Result<String, io::Error> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer)?;
    Ok(buffer)
}

// Convert the different newline sequences for Windows and other platforms into the common
// standard sequence of `"\n"` (backslash + 'n', as opposed to the '\n' (0xa) character for which
// it stands).
#[must_use]
pub fn normalize_newlines(input: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\r\n?").unwrap();
    }
    RE.replace_all(input, "\n").to_string()
}

/// Apply highlights to the text depending on the light or dark theme as detected, configured
/// or defaulted, or as toggled by the user with Ctrl-t.
pub fn apply_highlights(alt_highlights: bool, textarea: &mut TextArea) {
    if alt_highlights {
        // Dark theme-friendly colors
        textarea.set_selection_style(Style::default().bg(Color::Cyan).fg(Color::Black));
        textarea.set_cursor_style(Style::default().bg(Color::LightYellow).fg(Color::Black));
        textarea.set_cursor_line_style(Style::default().bg(Color::DarkGray).fg(Color::White));
    } else {
        // Light theme-friendly colors
        textarea.set_selection_style(Style::default().bg(Color::Blue).fg(Color::White));
        textarea.set_cursor_style(Style::default().bg(Color::LightRed).fg(Color::White));
        textarea.set_cursor_line_style(Style::default().bg(Color::Gray).fg(Color::Black));
    }
}

fn reset_term(mut term: Terminal<CrosstermBackend<io::StdoutLock<'_>>>) -> Result<(), ThagError> {
    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
fn show_popup(f: &mut ratatui::prelude::Frame) {
    let area = centered_rect(90, NUM_ROWS as u16 + 5, f.area());
    let inner = area.inner(Margin {
        vertical: 2,
        horizontal: 2,
    });
    let block = Block::default()
        .borders(Borders::ALL)
        .title(
            Title::from("Platform-dependent key mappings (YMMV)")
                .alignment(ratatui::layout::Alignment::Center),
        )
        .title(Title::from("(Ctrl+L to toggle)").alignment(Alignment::Center))
        .add_modifier(Modifier::BOLD);
    f.render_widget(Clear, area);
    //this clears out the background
    f.render_widget(block, area);
    let row_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(std::iter::repeat(Constraint::Ratio(1, NUM_ROWS as u32)).take(NUM_ROWS));
    let rows = row_layout.split(inner);

    for (i, row) in rows.iter().enumerate() {
        let col_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(45), Constraint::Length(43)].as_ref());
        let cells = col_layout.split(*row);
        for n in 0..=1 {
            let mut widget = Paragraph::new(MAPPINGS[i][n]);
            if i == 0 {
                widget = widget.add_modifier(Modifier::BOLD);
            } else {
                widget = widget.remove_modifier(Modifier::BOLD);
            }
            f.render_widget(widget, cells[n]);
        }
    }
}

fn centered_rect(max_width: u16, max_height: u16, r: Rect) -> Rect {
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

const MAPPINGS: &[[&str; 2]; 35] = &[
    ["Key bindings", "Description"],
    ["Shift+arrow keys", "Select/deselect ← chars→  / ↑ lines↓"],
    [
        "Shift+Ctrl+arrow keys",
        "Select/deselect ← words→  / ↑ paras↓",
    ],
    ["Ctrl+D", "Submit"],
    ["Ctrl+Q", "Cancel and quit"],
    ["Ctrl+H, Backspace", "Delete character before cursor"],
    ["Ctrl+I, Tab", "Indent"],
    ["Ctrl+M, Enter", "Insert newline"],
    ["Ctrl+K", "Delete from cursor to end of line"],
    ["Ctrl+J", "Delete from cursor to start of line"],
    ["Ctrl+W, Alt+<, Backspace", "Delete one word before cursor"],
    ["Alt+D, Delete", "Delete one word from cursor position"],
    ["Ctrl+U", "Undo"],
    ["Ctrl+R", "Redo"],
    ["Ctrl+C", "Copy (yank) selected text"],
    ["Ctrl+X", "Cut (yank) selected text"],
    ["Ctrl+Y", "Paste yanked text"],
    ["Ctrl+V, Shift+Ins, Cmd+V", "Paste from system clipboard"],
    ["Ctrl+F, →", "Move cursor forward one character"],
    ["Ctrl+B, ←", "Move cursor backward one character"],
    ["Ctrl+P, ↑", "Move cursor up one line"],
    ["Ctrl+N, ↓", "Move cursor down one line"],
    ["Alt+F, Ctrl+→", "Move cursor forward one word"],
    ["Atl+B, Ctrl+←", "Move cursor backward one word"],
    ["Alt+] or P, Ctrl+↑", "Move cursor up one paragraph"],
    ["Alt+[ or N, Ctrl+↓", "Move cursor down one paragraph"],
    [
        "Ctrl+E, End, Ctrl+Alt+F or → , Cmd+→",
        "Move cursor to end of line",
    ],
    [
        "Ctrl+A, Home, Ctrl+Alt+B or ← , Cmd+←",
        "Move cursor to start of line",
    ],
    ["Alt+<, Ctrl+Alt+P or ↑", "Move cursor to top of file"],
    ["Alt+>, Ctrl+Alt+N or↓", "Move cursor to bottom of file"],
    ["PageDown, Cmd+↓", "Page down"],
    ["Alt+V, PageUp, Cmd+↑", "Page up"],
    ["Ctrl+T", "Toggle selection highlight colours"],
    ["F1", "Previous in history"],
    ["F2", "Next in history"],
];
const NUM_ROWS: usize = MAPPINGS.len();
