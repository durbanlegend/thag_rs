use crate::{
    KeyCombination, ThagError, ThagResult,
    code_utils::write_source,
    file_dialog::{DialogMode, FileDialog, Status},
    key,
    stdin::edit_history,
};
// use crokey::key;
// use crokey::crossterm::event::KeyEvent;
use crossterm::event::{
    self, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event::{self, Paste},
    KeyCode, KeyEventKind, KeyModifiers,
};
use mockall::automock;
use ratatui::crossterm::{
    event::KeyEvent,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
        is_raw_mode_enabled,
    },
};
use ratatui::layout::{Constraint, Direction, Layout, Margin};
use ratatui::prelude::{CrosstermBackend, Rect};
pub use ratatui::style::Style as RataStyle;
use ratatui::style::{Color, Modifier, Styled, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::{CompletedFrame, Frame, Terminal};
use regex::Regex;
use scopeguard::{ScopeGuard, guard};
use serde::{Deserialize, Serialize};
use std::{
    self,
    collections::VecDeque,
    convert::Into,
    env::var,
    fmt::{Debug, Display, Write as _},
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::Duration,
};
use thag_common::{debug_log, re};
use thag_styling::{Role, ThemedStyle};
// import without risk of name clashing
use thag_profiler::profiled;
use tui_textarea::{CursorMove, Input, TextArea};

/// Title displayed at the top of the key bindings popup
pub const TITLE_TOP: &str = "Key bindings - subject to your terminal settings";
/// Title displayed at the bottom of the key bindings popup
pub const TITLE_BOTTOM: &str = "Ctrl+l to hide";

/// Type alias for the crossterm backend with stdout lock
pub type BackEnd<'a> = CrosstermBackend<std::io::StdoutLock<'a>>;
/// Type alias for a terminal with the backend
pub type Term<'a> = Terminal<BackEnd<'a>>;
/// Type alias for a closure that resets the terminal
pub type ResetTermClosure<'a> = Box<dyn FnOnce(Term<'a>)>;
/// Type alias for a scope guard that manages terminal cleanup
pub type TermScopeGuard<'a> = ScopeGuard<Term<'a>, ResetTermClosure<'a>>;
/// A trait to allow mocking of the event reader for testing purposes.
#[automock]
pub trait EventReader {
    /// Read a terminal event.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
    fn read_event(&self) -> ThagResult<Event>;
    /// Poll for a terminal event.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
    fn poll(&self, timeout: Duration) -> ThagResult<bool>;
}

/// A struct to implement real-world use of the event reader, as opposed to use in testing.
#[derive(Debug)]
pub struct CrosstermEventReader;

impl EventReader for CrosstermEventReader {
    #[profiled]
    fn read_event(&self) -> ThagResult<Event> {
        crossterm::event::read().map_err(Into::<ThagError>::into)
    }

    #[profiled]
    fn poll(&self, timeout: Duration) -> ThagResult<bool> {
        crossterm::event::poll(timeout).map_err(Into::<ThagError>::into)
    }
}

/// A wrapper around a terminal with scope guard for automatic cleanup.
///
/// This struct manages a terminal instance and ensures proper cleanup
/// when the terminal goes out of scope, regardless of how the program exits.
#[derive(Debug)]
pub struct ManagedTerminal<'a> {
    terminal: TermScopeGuard<'a>,
}

impl ManagedTerminal<'_> {
    /// Draw to the terminal.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue drawing to the terminal.
    #[profiled]
    pub fn draw<F>(&mut self, f: F) -> std::io::Result<CompletedFrame<'_>>
    where
        F: FnOnce(&mut Frame<'_>),
    {
        self.terminal.draw(f)
    }
}

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
#[profiled]
pub fn resolve_term<'a>() -> ThagResult<Option<ManagedTerminal<'a>>> {
    if var("TEST_ENV").is_ok() {
        return Ok(None);
    }

    let mut stdout = std::io::stdout().lock();
    enable_raw_mode()?;

    ratatui::crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(Some(ManagedTerminal {
        terminal: guard(
            terminal,
            Box::new(|term| {
                reset_term(term).expect("Error resetting terminal");
            }),
        ),
    }))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Represents a single entry in the edit history.
///
/// An entry contains both an index for ordering and the actual text content
/// stored as individual lines. This structure is used to maintain a history
/// of text edits that can be navigated and restored.
pub struct Entry {
    /// The index of this entry in the history collection
    pub index: usize, // Holds the entry's index
    /// The text content of this entry, stored as separate lines
    pub lines: Vec<String>, // Holds editor content as lines
}

impl Display for Entry {
    #[profiled]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.index, self.lines.join("\n"))
    }
}

impl Entry {
    /// Creates a new Entry with the given index and content.
    ///
    /// The content string is split into individual lines and stored in the `lines` field.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of this entry in the history collection
    /// * `content` - The text content to store, which will be split into lines
    #[profiled]
    pub fn new(index: usize, content: &str) -> Self {
        Self {
            index,
            lines: content.lines().map(String::from).collect(),
        }
    }

    /// Extracts string contents of entry for use in the editor.
    ///
    /// Joins all lines in the entry with newline characters to reconstruct
    /// the original text content.
    ///
    /// # Returns
    ///
    /// A String containing the full text content with lines joined by newlines
    #[must_use]
    #[profiled]
    pub fn contents(&self) -> String {
        self.lines.join("\n")
    }
}

/// Represents the edit history for a text editor.
///
/// This struct maintains a collection of text entries that can be navigated
/// through, similar to command history in a shell. It tracks the current
/// position within the history and provides methods for adding, updating,
/// and navigating through entries.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct History {
    /// The index of the currently selected entry in the history.
    /// `None` indicates no current selection or an empty history.
    pub current_index: Option<usize>,
    /// A double-ended queue containing all history entries.
    /// Entries are stored as `Entry` objects which contain both
    /// an index and the text content as lines.
    pub entries: VecDeque<Entry>, // Now a VecDeque of Entries
}

impl History {
    /// Creates a new empty History instance.
    #[must_use]
    #[profiled]
    pub fn new() -> Self {
        Self {
            current_index: None,
            entries: VecDeque::with_capacity(20),
        }
    }

    /// Loads a History instance from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to load from
    ///
    /// # Returns
    ///
    /// A History instance loaded from the file, or a new empty instance if loading fails
    #[must_use]
    #[profiled]
    pub fn load_from_file(path: &PathBuf) -> Self {
        let mut history = fs::read_to_string(path).map_or_else(
            |_| Self::default(),
            |data| serde_json::from_str(&data).unwrap_or_else(|_| Self::new()),
        );
        debug_log!("Loaded history={history:?}");
        // Remove any blanks - TODO they shouldn't be saved in the first place
        history.entries.retain(|e| !e.contents().trim().is_empty());

        // Reassign indices
        history.reassign_indices();

        // Set current_index to the index of the front entry (most recent one)
        if history.entries.is_empty() {
            history.current_index = None;
        } else {
            history.current_index = Some(history.entries.len() - 1);
        }
        debug_log!("history={history:?}");
        debug_log!(
            "load_from_file({path:?}); current index={:?}",
            history.current_index
        );
        history
    }

    /// Returns true if the current position is at the start of the history.
    #[allow(clippy::unnecessary_map_or)]
    #[must_use]
    #[profiled]
    pub fn at_start(&self) -> bool {
        debug_log!("at_start ...");
        self.current_index
            .map_or(true, |current_index| current_index == 0)
    }

    /// Returns true if the current position is at the end of the history.
    #[allow(clippy::unnecessary_map_or)]
    #[must_use]
    #[profiled]
    pub fn at_end(&self) -> bool {
        debug_log!("at_end ...");
        self.current_index.map_or(true, |current_index| {
            current_index == self.entries.len() - 1
        })
    }

    /// Adds a new entry to the history.
    ///
    /// # Arguments
    ///
    /// * `text` - The text content to add to the history
    #[profiled]
    pub fn add_entry(&mut self, text: &str) {
        let new_index = self.entries.len(); // Assign the next index based on current length
        let new_entry = Entry::new(new_index, text);

        // Remove prior duplicates
        self.entries
            .retain(|f| f.contents().trim() != new_entry.contents().trim());
        self.entries.push_back(new_entry);

        // // Reassign indices after pushing the new entry
        // self.reassign_indices();

        // Update current_index to point to the most recent entry (the front)
        self.current_index = Some(self.entries.len() - 1);
        debug_log!("add_entry({text}); current index={:?}", self.current_index);
        debug_log!("history={self:?}");
    }

    /// Updates an existing entry in the history or adds a new one if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the entry to update
    /// * `text` - The new text content for the entry
    #[profiled]
    pub fn update_entry(&mut self, index: usize, text: &str) {
        debug_log!("update_entry for index {index}...");
        // Get a mutable reference to the entry at the specified index
        let current_index = self.current_index;
        if let Some(entry) = self.get_mut(index) {
            // Update the lines if the entry exists
            entry.lines = text.lines().map(String::from).collect::<Vec<String>>();
            debug_log!("... update_entry({entry:?}); current index={current_index:?}");
        } else {
            // If the entry doesn't exist, add it
            self.add_entry(text);
        }
    }

    /// Deletes an entry from the history by index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the entry to delete
    #[profiled]
    pub fn delete_entry(&mut self, index: usize) {
        self.entries.retain(|entry| entry.index != index);

        // Reassign indices after deletion
        self.reassign_indices();

        // Update current_index after deletion, set to most recent entry (the front)
        if self.entries.is_empty() {
            self.current_index = None;
        } else {
            self.current_index = Some(self.entries.len() - 1);
        }
    }

    /// Save history to a file.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered writing the file.
    #[profiled]
    pub fn save_to_file(&mut self, path: &PathBuf) -> ThagResult<()> {
        self.reassign_indices();
        if let Ok(data) = serde_json::to_string(&self) {
            debug_log!("About to write data=({data}");
            if let Ok(metadata) = std::fs::metadata(path) {
                debug_log!("File permissions: {:?}", metadata.permissions());
            }

            // fs::write(path, data)?;
            // fs::write(path, "\n")?;
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true) // This will clear the file before writing
                .open(path)?;

            // Write the data
            file.write_all((data + "\n").into_bytes().as_ref())?;

            // Flush the write to disk
            // Beware of exiting "too early" for writes actually to be flushed despite sync.
            // file.sync_all()?;
            file.sync_data()?;
        } else {
            debug_log!("Could not serialise history: {self:?}");
        }
        debug_log!("save_to_file({path:?}");
        Ok(())
    }

    /// Gets the currently selected entry from the history.
    ///
    /// # Returns
    ///
    /// An optional reference to the current entry, or None if the history is empty
    #[profiled]
    pub fn get_current(&mut self) -> Option<&Entry> {
        if self.entries.is_empty() {
            return None;
        }

        if let Some(index) = self.current_index {
            debug_log!("get_current(); current index={:?}", self.current_index);

            self.get(index)
        } else {
            debug_log!("None");
            None
        }
    }

    /// Gets an entry at the specified index and sets it as the current entry.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the entry to retrieve
    ///
    /// # Returns
    ///
    /// An optional reference to the entry at the specified index
    #[profiled]
    pub fn get(&mut self, index: usize) -> Option<&Entry> {
        debug_log!("get({index})...");
        if !(0..self.entries.len()).contains(&index) {
            return None;
        }
        self.current_index = Some(index);
        debug_log!(
            "...get({:?}); current index={:?}",
            self.entries.get(index),
            self.current_index
        );

        let entry = self.entries.get(index);
        debug_log!("... returning {entry:?}");
        entry
    }

    /// Gets a mutable reference to an entry at the specified index and sets it as the current entry.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the entry to retrieve
    ///
    /// # Returns
    ///
    /// An optional mutable reference to the entry at the specified index
    #[profiled]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Entry> {
        debug_log!("get_mut({index})...");

        if !(0..self.entries.len()).contains(&index) {
            return None;
        }

        self.current_index = Some(index);
        debug_log!(
            "...get_mut({:?}); current index={:?}",
            self.entries.get(index),
            self.current_index
        );

        let entry = self.entries.get_mut(index);
        debug_log!("... returning {entry:?}");

        entry
    }

    /// Returns the previous entry in this [`History`] collection.
    ///
    /// # Panics
    ///
    /// Panics if a logic error is detected, likely when reaching the oldest History entry.
    #[profiled]
    pub fn get_previous(&mut self) -> Option<&Entry> {
        // let this = &mut *self;
        debug_log!("get_previous...");
        if self.entries.is_empty() {
            return None;
        }
        let new_index = self.current_index.map(|index| {
            if index > 0 {
                index - 1
            } else {
                // TODO crossterm terminal beep if and when implemented (issue #806 pull request)
                0
            }
        });
        debug_log!(
            "...old index={:#?};new_index={new_index:?}",
            self.current_index
        );

        self.current_index = new_index;

        self.current_index.map_or_else(
            || {
                panic!(
                    "Logic error: current_index should never be None if there are History records"
                );
            },
            |index| {
                let entry = self.get(index);
                debug_log!("get_previous; new current index={index:?}, entry={entry:?}");
                entry
            },
        )
    }

    /// Returns the next entry in this [`History`] collection.
    ///
    /// # Panics
    ///
    /// Panics if a logic error is detected, likely when reaching the newest History entry.
    #[profiled]
    pub fn get_next(&mut self) -> Option<&Entry> {
        debug_log!("get_next...");
        let this = &mut *self;
        if this.entries.is_empty() {
            return None;
        }
        let new_index = self.current_index.map(|index| {
            let max_index = self.entries.len() - 1;
            if index < max_index {
                index + 1
            } else {
                // crossterm terminal beep if and when implemented (issue #806 pull request)
                max_index
            }
        });
        debug_log!(
            "...old index={:#?};new_index={new_index:?}",
            self.current_index
        );

        self.current_index = new_index;

        self.current_index.map_or_else(
            || {
                panic!(
                    "Logic error: current_index should never be None if there are History records"
                );
            },
            |index| {
                let entry = self.get(index);
                debug_log!("get_next(); current index={index:?}, entry={entry:?}");
                entry
            },
        )
    }

    /// Gets the last (most recent) entry in the history.
    ///
    /// # Returns
    ///
    /// An optional reference to the last entry, or None if the history is empty
    #[profiled]
    pub fn get_last(&mut self) -> Option<&Entry> {
        if self.entries.is_empty() {
            return None;
        }

        self.entries.back()
    }

    /// Reassigns indices so that the newest entry has index 0, and the oldest has len - 1.
    #[profiled]
    fn reassign_indices(&mut self) {
        // let len = self.entries.len();
        for (i, entry) in self.entries.iter_mut().enumerate() {
            entry.index = i;
        }
    }
}

type KeyHandlerClosure = dyn Fn(KeyEvent, &mut EditData) -> ThagResult<KeyAction>;

#[derive(Debug, Default, PartialEq)]
/// Define `vim`-style editor states
pub enum EditorMode {
    /// Text editing mode (tui-textarea consumes text input)
    #[default]
    Edit,
    /// Navigation mode (Vim/Helix style chords)
    Vim,
}

#[allow(dead_code)]
/// Struct to hold data-related parameters for the TUI editor
pub struct EditData<'a> {
    /// Whether to return the edited text as part of the result
    pub return_text: bool,
    /// The initial content to display in the editor
    pub initial_content: &'a str,
    /// Optional path where the edited content should be saved
    pub save_path: Option<PathBuf>,
    /// Optional path to the history file for storing edit history
    pub history_path: Option<&'a PathBuf>,
    /// Optional history object for managing edit history
    pub history: Option<History>,
    /// The `TextArea` to be used to edit the content.
    pub textarea: TextArea<'a>,
    /// The wrapped terminal instance
    pub maybe_term: Option<ManagedTerminal<'a>>,
    /// Popup active flag
    pub popup: bool,
    /// Saved flag
    pub saved: bool,
    /// The user_selected styling message role for text highlighting
    pub tui_highlight_fg: Role,
    /// The popup scroll state tracker
    pub popup_scroll: PopupScrollState,
    /// The edit status message
    pub status_message: String,
    /// The preconfigured key display lines
    pub adjusted_mappings: Vec<KeyDisplayLine>,
    /// The display-related parameters for the TUI editor
    pub display: KeyDisplay<'a>,
    /// A preconfigured key event handler to use in the current context
    pub key_handler: Option<Box<KeyHandlerClosure>>,
    /// The `vim`-style navigation or text editing mode.
    pub mode: EditorMode,
    /// Tracks multi-key sequences like `gg` for top of file.
    pub last_char: Option<char>,
}

impl<'a> EditData<'a> {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> ThagResult<KeyAction> {
        match self.mode {
            EditorMode::Edit => {
                // In Insert mode, Esc drops back to Normal/Nav mode
                if key_event.code == KeyCode::Esc {
                    self.mode = EditorMode::Vim;
                } else {
                    // log::debug_log!("key_event={key_event:#?}");
                    let key_combination = KeyCombination::from(key_event); // Derive KeyCombination

                    // Handle scrolling in popup before normal editor keys
                    if self.popup {
                        let max_scroll = self.adjusted_mappings.len().saturating_sub(10);

                        match key_combination {
                            key!(up) => {
                                self.popup_scroll.scroll_offset =
                                    self.popup_scroll.scroll_offset.saturating_sub(1);
                                return Ok(KeyAction::Continue);
                            }
                            key!(down) => {
                                if self.popup_scroll.scroll_offset < max_scroll {
                                    self.popup_scroll.scroll_offset += 1;
                                }
                                return Ok(KeyAction::Continue);
                            }
                            _ => {} // Let other keys fall through to toggle popup
                        }
                    }

                    // If using iterm2, ensure Settings | Profiles | Keys | Left Option key is set to Esc+.
                    #[allow(clippy::unnested_or_patterns)]
                    match key_combination {
                        key!(ctrl - h) | key!(backspace) => {
                            self.textarea.delete_char();
                        }
                        // Not how this works. Intercepting tab and Ctrl-i is counter-productive.
                        // key!(ctrl - i) | key!(tab) => {
                        //     textarea.indent();
                        // }
                        key!(ctrl - m) | key!(enter) => {
                            self.textarea.insert_newline();
                        }
                        key!(ctrl - k) => {
                            self.textarea.delete_line_by_end();
                        }
                        key!(ctrl - j) => {
                            self.textarea.delete_line_by_head();
                        }
                        key!(ctrl - w) | key!(alt - backspace) => {
                            self.textarea.delete_word();
                        }
                        key!(alt - d) => {
                            self.textarea.delete_next_word();
                        }
                        key!(ctrl - u) => {
                            self.textarea.undo();
                        }
                        key!(ctrl - r) => {
                            self.textarea.redo();
                        }
                        key!(ctrl - c) => {
                            self.textarea.copy();
                        }
                        key!(ctrl - x) => {
                            self.textarea.cut();
                        }
                        key!(ctrl - y) => {
                            self.textarea.paste();
                        }
                        key!(ctrl - f) | key!(right) => {
                            if self.textarea.is_selecting() {
                                self.textarea.cancel_selection();
                            }
                            self.textarea.move_cursor(CursorMove::Forward);
                        }
                        key!(ctrl - b) | key!(left) => {
                            if self.textarea.is_selecting() {
                                self.textarea.cancel_selection();
                            }
                            self.textarea.move_cursor(CursorMove::Back);
                        }
                        key!(ctrl - p) | key!(up) => {
                            if self.textarea.is_selecting() {
                                self.textarea.cancel_selection();
                            }
                            self.textarea.move_cursor(CursorMove::Up);
                        }
                        key!(ctrl - n) | key!(down) => {
                            if self.textarea.is_selecting() {
                                self.textarea.cancel_selection();
                            }
                            self.textarea.move_cursor(CursorMove::Down);
                        }
                        key!(alt - f) => {
                            if self.textarea.is_selecting() {
                                self.textarea.cancel_selection();
                            }
                            self.textarea.move_cursor(CursorMove::WordForward);
                        }
                        key!(alt - shift - f) => {
                            self.textarea.move_cursor(CursorMove::WordEnd);
                        }
                        key!(alt - b) => {
                            if self.textarea.is_selecting() {
                                self.textarea.cancel_selection();
                            }
                            self.textarea.move_cursor(CursorMove::WordBack);
                        }
                        key!(alt - p) | key!(alt - ')') | key!(f1) => {
                            if self.textarea.is_selecting() {
                                self.textarea.cancel_selection();
                            }
                            self.textarea.move_cursor(CursorMove::ParagraphBack);
                        }
                        key!(alt - n) | key!(alt - '(') | key!(f2) => {
                            self.textarea.move_cursor(CursorMove::ParagraphForward);
                        }
                        key!(ctrl - e) | key!(end) | key!(ctrl - alt - f) => {
                            self.textarea.move_cursor(CursorMove::End);
                        }
                        key!(ctrl - a) | key!(home) | key!(ctrl - alt - b) => {
                            self.textarea.move_cursor(CursorMove::Head);
                        }
                        key!(f9) => {
                            ratatui::crossterm::execute!(
                                std::io::stdout().lock(),
                                DisableMouseCapture,
                            )?;
                            self.textarea.remove_line_number();
                            self.textarea.set_block(
                                Block::default()
                                    .borders(Borders::NONE)
                                    .title(self.display.title)
                                    .title_style(self.display.title_style),
                            );
                        }
                        key!(f10) => {
                            // eprintln!("key_combination={key_combination:?}");
                            ratatui::crossterm::execute!(
                                std::io::stdout().lock(),
                                EnableMouseCapture,
                            )?;
                            self.textarea
                                .set_line_number_style(RataStyle::themed(Role::Hint));
                            self.textarea.set_block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .title(self.display.title)
                                    .title_style(self.display.title_style),
                            );
                        }
                        key!(alt - '<') | key!(ctrl - alt - p) => {
                            self.textarea.move_cursor(CursorMove::Top);
                        }
                        key!(alt - '>') | key!(ctrl - alt - n) => {
                            self.textarea.move_cursor(CursorMove::Bottom);
                        }
                        key!(alt - c) => {
                            if self.textarea.is_selecting() {
                                self.textarea.cancel_selection();
                            } else {
                                self.textarea.start_selection();
                            }
                        }
                        key!(alt - shift - 'h') => {
                            if !self.textarea.is_selecting() {
                                self.textarea.start_selection();
                            }
                            self.textarea.move_cursor(CursorMove::WordBack);
                        }
                        key!(alt - shift - 'j') => {
                            if !self.textarea.is_selecting() {
                                self.textarea.start_selection();
                            }
                            self.textarea.move_cursor(CursorMove::Down);
                        }
                        key!(alt - shift - 'k') => {
                            if !self.textarea.is_selecting() {
                                self.textarea.start_selection();
                            }
                            self.textarea.move_cursor(CursorMove::Up);
                        }
                        key!(alt - shift - 'l') => {
                            if !self.textarea.is_selecting() {
                                self.textarea.start_selection();
                            }
                            self.textarea.move_cursor(CursorMove::WordEnd);
                        }
                        key!(alt - shift - 'p') => {
                            if !self.textarea.is_selecting() {
                                self.textarea.start_selection();
                            }
                            self.textarea.move_cursor(CursorMove::ParagraphBack);
                        }
                        key!(alt - shift - 'n') => {
                            if !self.textarea.is_selecting() {
                                self.textarea.start_selection();
                            }
                            self.textarea.move_cursor(CursorMove::ParagraphForward);
                        }
                        // key!(alt - shift - c) => {
                        //     textarea.start_selection();
                        // }
                        key!(alt - shift - a) => {
                            self.textarea.select_all();
                        }
                        key!(ctrl - t) => {
                            // Toggle highlighting colours
                            self.tui_highlight_fg = match self.tui_highlight_fg {
                                Role::Emphasis => Role::Info,
                                Role::Info => Role::Error,
                                Role::Error => Role::Warning,
                                Role::Warning => Role::Heading1,
                                Role::Heading1 => Role::Heading2,
                                Role::Heading2 => Role::Heading3,
                                _ => Role::Emphasis,
                            };
                            if var("TEST_ENV").is_err() {
                                #[allow(clippy::option_if_let_else)]
                                if let Some(ref mut term) = self.maybe_term {
                                    term.draw(|_| {
                                        highlight_selection(
                                            &mut self.textarea,
                                            self.tui_highlight_fg,
                                        );
                                    })?;
                                }
                            }
                        }
                        _ => {
                            // Call the key_handler closure to process events
                            // Use `take` to work around the borrow checker
                            if let Some(handler) = self.key_handler.take() {
                                let result = handler(key_event, self);

                                self.key_handler = Some(handler);

                                return result;
                            }
                            // eprintln!("key_action={key_action:?}");
                        }
                    }
                }
            }
            EditorMode::Vim => {
                self.handle_normal_mode(key_event);
            }
        }
        return Ok(KeyAction::Continue);
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        // If we are waiting for a sequence (like 'g' prefix)
        if let Some('g') = self.last_char {
            self.last_char = None; // Reset prefix tracker
            match key.code {
                KeyCode::Char('g') => self.textarea.move_cursor(CursorMove::Top), // Vim 'gg'
                KeyCode::Char('k') => self.textarea.move_cursor(CursorMove::Top), // Helix 'gk'
                KeyCode::Char('j') => self.textarea.move_cursor(CursorMove::Bottom), // Helix 'gj'
                _ => {}
            }
            return;
        }

        match (key.code, key.modifiers) {
            // Mode switching: press 'i' to enter typing mode
            (KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.mode = EditorMode::Edit;
            }

            // --- Micro-Navigation ---
            (KeyCode::Char('h'), KeyModifiers::NONE) => self.textarea.move_cursor(CursorMove::Back),
            (KeyCode::Char('j'), KeyModifiers::NONE) => self.textarea.move_cursor(CursorMove::Down),
            (KeyCode::Char('k'), KeyModifiers::NONE) => self.textarea.move_cursor(CursorMove::Up),
            (KeyCode::Char('l'), KeyModifiers::NONE) => {
                self.textarea.move_cursor(CursorMove::Forward)
            }

            // --- Large Jumps ---
            (KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.last_char = Some('g'); // Stash 'g' to await the next keystroke
            }
            (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                self.textarea.move_cursor(CursorMove::Bottom); // Vim style Bottom
            }

            // --- Paragraph Jumping (Empty line boundaries) ---
            (KeyCode::Char('{'), KeyModifiers::NONE) => {
                self.textarea.move_cursor(CursorMove::ParagraphBack);
            }
            (KeyCode::Char('}'), KeyModifiers::NONE) => {
                self.textarea.move_cursor(CursorMove::ParagraphForward);
            }

            // --- Paging (Ctrl-u / Ctrl-d) ---
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                // tui-textarea doesn't have a native 'page' command,
                // but you can loop standard jumps or use custom window logic
                for _ in 0..15 {
                    self.textarea.move_cursor(CursorMove::Up);
                }
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                for _ in 0..15 {
                    self.textarea.move_cursor(CursorMove::Down);
                }
            }

            _ => {}
        }
    }
}

/// Struct to hold display-related parameters for the TUI editor
#[derive(Debug, Default)]
pub struct KeyDisplay<'a> {
    /// The title to display at the top of the editor
    pub title: &'a str,
    /// The style to apply to the title text
    pub title_style: RataStyle,
    /// Keys to remove from the default key mappings display
    pub remove_keys: &'a [&'a str],
    /// Additional key mappings to add to the display
    pub add_keys: &'a [KeyDisplayLine],
}

/// Tracks the scroll state of the popup help display
#[derive(Debug, Default)]
pub struct PopupScrollState {
    /// Current scroll offset (number of rows scrolled down)
    pub scroll_offset: usize,
}

/// Represents the different actions that can be taken in response to user input in the TUI editor.
///
/// This enum is used to communicate between key handlers and the main editor loop,
/// indicating what action should be taken based on the user's key press.
#[derive(Debug)]
pub enum KeyAction {
    /// Abandon any unsaved changes and exit without saving
    AbandonChanges,
    /// Continue with normal editor operation - no special action needed
    Continue, // For other inputs that don't need specific handling
    /// Quit the editor, with a boolean indicating whether changes have been saved
    Quit(bool),
    /// Save the current content to file
    Save,
    /// Save the current content and then exit the editor
    SaveAndExit,
    /// Show the help screen with key bindings
    ShowHelp,
    /// Save the current content and submit it (e.g., for iterator execution)
    SaveAndSubmit,
    /// Submit the current content without necessarily saving to file
    Submit,
    /// Toggle the syntax highlighting colors
    ToggleHighlight,
    /// Toggle the visibility of the popup help screen
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
#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
#[profiled]
pub fn tui_edit<R>(
    event_reader: &R,
    edit_data: &mut EditData,
) -> ThagResult<(KeyAction, Option<Vec<String>>)>
where
    R: EventReader + Debug,
{
    // Initialize state variables
    edit_data.maybe_term = resolve_term()?;

    // Create the `TextArea` from initial content
    // let mut textarea = TextArea::from(edit_data.initial_content.lines());
    // let mut textarea = &edit_data.textarea;
    edit_data.textarea.set_hard_tab_indent(true);
    // eprintln!("edit_data.textarea.tab_length()={}", edit_data.textarea.tab_length());

    // Set up the display parameters for the `TextArea`
    edit_data.textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(edit_data.display.title)
            .title_style(edit_data.display.title_style),
    );

    edit_data
        .textarea
        .set_line_number_style(RataStyle::themed(Role::Hint));
    edit_data.textarea.move_cursor(CursorMove::Bottom);
    // New line with cursor at EOF for usability
    edit_data.textarea.move_cursor(CursorMove::End);
    if !edit_data.textarea.is_empty() {
        edit_data.textarea.insert_newline();
    }

    // Apply initial highlights
    highlight_selection(&mut edit_data.textarea, edit_data.tui_highlight_fg);

    let remove = edit_data.display.remove_keys;
    let add = edit_data.display.add_keys;
    // Track popup scroll state
    // let mut popup_scroll = PopupScrollState::default();

    // Can't make these OnceLock values, since their configuration depends on the `remove`
    // and `add` values passed in by the caller.
    edit_data.adjusted_mappings = MAPPINGS
        .iter()
        .filter(|&row| !remove.contains(&row.keys))
        .chain(add.iter())
        .cloned()
        .collect();
    edit_data.adjusted_mappings.sort();
    let (max_key_len, max_desc_len) =
        edit_data
            .adjusted_mappings
            .iter()
            .fold((0_u16, 0_u16), |(max_key, max_desc), row| {
                let key_len = row.keys.len().try_into().unwrap();
                let desc_len = row.desc.len().try_into().unwrap();
                (max_key.max(key_len), max_desc.max(desc_len))
            });

    // Event loop for handling key events
    loop {
        maybe_enable_raw_mode()?;
        let test_env = &var("TEST_ENV");
        let event = if test_env.is_ok() {
            // Testing or CI
            event_reader.read_event()?
        } else {
            // Real-world interaction
            edit_data.maybe_term.as_mut().map_or_else(
                || Err("Logic issue unwrapping term we wrapped ourselves".into()),
                |term| {
                    term.draw(|f| {
                        // Get the size of the available terminal area
                        let area = f.area();

                        // Ensure there's enough height for both the `TextArea` and the status line
                        if area.height > 1 {
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints::<&[Constraint]>(&[
                                    Constraint::Min(area.height - 3), // Editor area takes up the rest
                                    Constraint::Length(3),            // Status line gets 1 line
                                ])
                                .split(area);

                            // Render the `TextArea` in the first chunk
                            f.render_widget(&edit_data.textarea, chunks[0]);

                            // Render the status line in the second chunk
                            let status_block = Block::default()
                                .borders(Borders::ALL)
                                .title("Status")
                                .style(RataStyle::themed(Role::Success))
                                .title_style(edit_data.display.title_style)
                                .padding(ratatui::widgets::Padding::horizontal(1));

                            let status_text =
                                Paragraph::new::<&str>(edit_data.status_message.as_ref())
                                    .block(status_block)
                                    .style(RataStyle::themed(Role::Info));

                            f.render_widget(status_text, chunks[1]);

                            if edit_data.popup {
                                display_popup(
                                    &edit_data.adjusted_mappings,
                                    TITLE_TOP,
                                    TITLE_BOTTOM,
                                    max_key_len,
                                    max_desc_len,
                                    &mut edit_data.popup_scroll,
                                    f,
                                );
                            }
                            highlight_selection(
                                &mut edit_data.textarea,
                                edit_data.tui_highlight_fg,
                            );
                            // status_message = String::new();
                        }
                    })
                    .map_err(|e| {
                        eprintln!("Error drawing terminal: {e:?}");
                        e
                    })?;

                    // NB: leave in raw mode until end of session to avoid random appearance of OSC codes on screen
                    event_reader.read_event()
                },
            )?
        };

        if let Paste(ref data) = event {
            edit_data.textarea.insert_str(normalize_newlines(data));
        } else if let Event::Mouse(mouse_event) = event {
            // Handle mouse scrolling in popup
            if edit_data.popup {
                use ratatui::crossterm::event::MouseEventKind;
                match mouse_event.kind {
                    MouseEventKind::ScrollDown => {
                        if edit_data.popup_scroll.scroll_offset + 1
                            < edit_data.adjusted_mappings.len()
                        {
                            edit_data.popup_scroll.scroll_offset += 1;
                        }
                    }
                    MouseEventKind::ScrollUp => {
                        edit_data.popup_scroll.scroll_offset =
                            edit_data.popup_scroll.scroll_offset.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        } else if let Event::Key(key_event) = event {
            // Ignore key release, which creates an unwanted second event in Windows
            if !matches!(key_event.kind, KeyEventKind::Press) {
                continue;
            }

            let key_action = edit_data.handle_key_event(key_event)?;
            match key_action {
                KeyAction::AbandonChanges => {
                    return Ok((key_action, None::<Vec<String>>));
                }
                KeyAction::Quit(_)
                | KeyAction::SaveAndExit
                | KeyAction::SaveAndSubmit
                | KeyAction::Submit => {
                    let maybe_text = if edit_data.return_text {
                        Some(edit_data.textarea.lines().to_vec())
                    } else {
                        None::<Vec<String>>
                    };
                    return Ok((key_action, maybe_text));
                }
                KeyAction::Continue | KeyAction::Save | KeyAction::ToggleHighlight => (),
                KeyAction::TogglePopup => {
                    // Reset scroll position when popup is opened
                    if edit_data.popup {
                        edit_data.popup_scroll.scroll_offset = 0;
                    }
                }
                KeyAction::ShowHelp => todo!(),
            }
        } else if edit_data.mode == EditorMode::Edit {
            // Otherwise, tui-textarea handles typing natively
            let input = tui_textarea::Input::from(event);
            edit_data.textarea.input(input);
        }
    }
}

/// Highlight the selected text in the `TextArea` with the specified color role.
///
/// This function applies styling to the selected text in the `TextArea`, setting
/// the foreground color based on the provided `Role` and making it bold.
///
/// # Arguments
///
/// * `textarea` - A mutable reference to the `TextArea` to apply highlighting to
/// * `tui_highlight_fg` - The `Role` that determines the foreground color for highlighting
#[profiled]
pub fn highlight_selection(textarea: &mut TextArea<'_>, tui_highlight_fg: Role) {
    textarea.set_selection_style(RataStyle::themed(tui_highlight_fg).bold());
}

/// Key handler function to be passed into `tui_edit` for editing iterator history.
///
/// # Errors
///
/// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
#[allow(clippy::too_many_lines, clippy::missing_panics_doc)]
#[profiled]
pub fn script_key_handler(key_event: KeyEvent, edit_data: &mut EditData) -> ThagResult<KeyAction> {
    if !matches!(key_event.kind, KeyEventKind::Press) {
        return Ok(KeyAction::Continue);
    }

    let key_combination = KeyCombination::from(key_event); // Derive KeyCombination
    // eprintln!("key_combination={key_combination:?}");

    // let history_path = edit_data.history_path.cloned();

    #[allow(clippy::unnested_or_patterns)]
    match key_combination {
        key!(esc) | key!(ctrl - q) => Ok(KeyAction::Quit(edit_data.saved)),
        key!(ctrl - d) => save_and_submit(edit_data),
        key!(ctrl - s) | key!(ctrl - alt - s) | key!(f12) => {
            if matches!(key_combination, key!(ctrl - s)) && edit_data.save_path.is_some() {
                // eprintln!("key_combination matches ctrl-s");
                save(edit_data)
            } else {
                let key_action = save_as(edit_data)?;
                Ok(key_action)
            }
        }
        key!(ctrl - l) => {
            // Toggle popup
            edit_data.popup = !edit_data.popup;
            Ok(KeyAction::TogglePopup)
        }
        key!(f3) => {
            // Ask to revert
            Ok(KeyAction::AbandonChanges)
        }
        key!(f4) => {
            // Clear textarea
            edit_data.textarea.select_all();
            edit_data.textarea.cut();
            Ok(KeyAction::Continue)
        }
        key!(f5) => {
            // Clear textarea and wipe from history
            if edit_data.textarea.is_empty() {
                return Ok(KeyAction::Continue);
            }
            wipe_textarea(edit_data)?;
            Ok(KeyAction::Continue)
        }
        key!(f6) => {
            // Edit history
            edit_history()?;
            Ok(KeyAction::Continue)
        }
        key!(f7) => {
            // Scroll up in history
            prev_hist(edit_data)?;
            Ok(KeyAction::Continue)
        }
        key!(f8) => {
            // Scroll down in history
            next_hist(edit_data);
            Ok(KeyAction::Continue)
        }
        _ => {
            // Update the `TextArea` with the input from the key event
            edit_data.textarea.input(Input::from(key_event)); // Input derived from Event
            Ok(KeyAction::Continue)
        }
    }
}

#[profiled]
fn next_hist(edit_data: &mut EditData<'_>) {
    if let Some(ref mut hist) = edit_data.history {
        if let Some(entry) = hist.get_next() {
            debug_log!("F8 found entry {entry:?}");
            paste_to_textarea(&mut edit_data.textarea, entry);
        }
    }
}

#[profiled]
fn prev_hist(edit_data: &mut EditData<'_>) -> ThagResult<()> {
    if let Some(ref mut hist) = edit_data.history {
        if hist.at_end() && edit_data.textarea.is_empty() {
            if let Some(entry) = &hist.get_last() {
                debug_log!("F7 (1) found entry {entry:?}");
                paste_to_textarea(&mut edit_data.textarea, entry);
            }
        } else {
            save_if_changed(hist, &mut edit_data.textarea, edit_data.history_path)?;
            if let Some(entry) = &hist.get_previous() {
                debug_log!("F7 (2) found entry {entry:?}");
                paste_to_textarea(&mut edit_data.textarea, entry);
            }
        }
    }
    Ok(())
}

#[profiled]
fn wipe_textarea(edit_data: &mut EditData<'_>) -> ThagResult<()> {
    if let Some(ref mut hist) = edit_data.history {
        let _in_hist = !&hist.at_end();
        let textarea_contents = edit_data.textarea.lines().to_vec().join("\n");
        edit_data.textarea.select_all();
        edit_data.textarea.cut();
        let yank_text = edit_data.textarea.yank_text();
        assert_eq!(yank_text, textarea_contents);
        if let Some(current_hist_entry) = &hist.get_current() {
            assert_eq!(yank_text, current_hist_entry.contents());
            let index = current_hist_entry.index;
            hist.delete_entry(index);
            hist.entries
                .retain(|f| f.contents().trim() != textarea_contents);
        }
        if let Some(hist_path) = edit_data.history_path {
            hist.save_to_file(hist_path)?;
        }
    }
    Ok(())
}

#[profiled]
fn save_as(edit_data: &mut EditData<'_>) -> ThagResult<KeyAction> {
    if let Some(ref mut term) = edit_data.maybe_term {
        let mut save_dialog: FileDialog<'_> = FileDialog::new(60, 20, DialogMode::Save)?;
        save_dialog.open();
        let mut status = Status::Incomplete;
        while matches!(status, Status::Incomplete) && save_dialog.selected_file.is_none() {
            term.draw(|f| save_dialog.draw(f))?;
            if let Event::Key(key) = event::read()? {
                status = save_dialog.handle_input(key)?;
            }
        }

        edit_data.status_message.clear();
        if let Some(ref to_rs_path) = save_dialog.selected_file {
            save_source_file(to_rs_path, &mut edit_data.textarea, &mut edit_data.saved)?;
            let _ = write!(
                edit_data.status_message,
                "Saved to {}",
                to_rs_path.display()
            );
            edit_data.save_path = Some(to_rs_path.clone());
            Ok(KeyAction::Save)
        } else {
            let _ = write!(edit_data.status_message, "Failed to save file");
            Ok(KeyAction::Continue)
        }
    } else {
        let _ = write!(
            edit_data.status_message,
            "No terminal to display file save dialog"
        );
        Ok(KeyAction::Continue)
    }
}

#[profiled]
fn save(edit_data: &mut EditData<'_>) -> ThagResult<KeyAction> {
    if let Some(ref save_path) = edit_data.save_path {
        if let Some(hist_path) = edit_data.history_path {
            let history = &mut edit_data.history;
            if let Some(hist) = history {
                preserve(&mut edit_data.textarea, hist, hist_path)?;
            }
        }
        let result = save_source_file(save_path, &mut edit_data.textarea, &mut edit_data.saved);
        // eprintln!("result={result:?}");
        match result {
            Ok(()) => {
                edit_data.status_message.clear();
                let _ = write!(edit_data.status_message, "Saved to {}", save_path.display());
                Ok(KeyAction::Save)
            }
            Err(e) => Err(e),
        }
    } else {
        edit_data.status_message.clear();
        let _ = write!(
            edit_data.status_message,
            "No save path: edit_data.save_path={:?}",
            edit_data.save_path
        );
        Ok(KeyAction::Continue)
    }
}

#[profiled]
fn save_and_submit(edit_data: &mut EditData<'_>) -> ThagResult<KeyAction> {
    if let Some(hist_path) = edit_data.history_path {
        let history = &mut edit_data.history;
        if let Some(hist) = history {
            preserve(&mut edit_data.textarea, hist, hist_path)?;
        }
    }
    Ok(KeyAction::Submit)
}

/// Enable raw mode, but not if in test mode, because that will cause the dreaded rightward drift
/// in log output due to carriage returns being ignored.
///
/// # Errors
///
/// This function will bubble up any i/o errors encountered by `crossterm::enable_raw_mode`.
#[profiled]
pub fn maybe_enable_raw_mode() -> ThagResult<()> {
    let test_env = &var("TEST_ENV");
    debug_log!("test_env={test_env:?}");
    if !test_env.is_ok() && !is_raw_mode_enabled()? {
        // Check if stdout is a terminal before enabling raw mode
        if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            debug_log!("Enabling raw mode");
            enable_raw_mode()?;
        } else {
            debug_log!("Skipping raw mode - not a terminal");
        }
    }
    Ok(())
}

/// Display a popup with key mappings and descriptions.
///
/// This function renders a centered popup window containing a list of key bindings
/// and their descriptions. The popup is styled with borders and titles, and each
/// key mapping is displayed in a two-column layout.
///
/// # Arguments
///
/// * `mappings` - A slice of `KeyDisplayLine` structs containing the key mappings to display
/// * `title_top` - The title text to display at the top of the popup
/// * `title_bottom` - The title text to display at the bottom of the popup
/// * `max_key_len` - The maximum length of key strings for column width calculation
/// * `max_desc_len` - The maximum length of description strings for column width calculation
/// * `f` - A mutable reference to the ratatui Frame for rendering
#[profiled]
#[allow(clippy::cast_possible_truncation)]
pub fn display_popup(
    mappings: &[KeyDisplayLine],
    title_top: &str,
    title_bottom: &str,
    max_key_len: u16,
    max_desc_len: u16,
    scroll_state: &mut PopupScrollState,
    f: &mut ratatui::prelude::Frame<'_>,
) {
    let total_rows = mappings.len();

    // Calculate available height for content
    let max_height = f.area().height.saturating_sub(6); // Reserve space for borders and titles
    let content_height = max_height.min(total_rows as u16);

    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(title_top).centered())
        .title_bottom(Line::from(format!("{title_bottom} (scroll with mouse wheel)")).centered())
        .add_modifier(Modifier::BOLD)
        .fg(Color::themed(Role::HD1));

    #[allow(clippy::cast_possible_truncation)]
    let area = centered_rect(max_key_len + max_desc_len + 5, content_height + 5, f.area());

    let inner = area.inner(Margin {
        vertical: 2,
        horizontal: 2,
    });

    // Clear background and render block
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    // Calculate visible range based on scroll offset
    let visible_rows = inner.height as usize;
    let max_scroll = total_rows.saturating_sub(visible_rows);
    scroll_state.scroll_offset = scroll_state.scroll_offset.min(max_scroll);

    let start_idx = scroll_state.scroll_offset;
    let end_idx = (start_idx + visible_rows).min(total_rows);
    let visible_mappings = &mappings[start_idx..end_idx];

    // Create layout for visible rows
    #[allow(clippy::cast_possible_truncation)]
    let row_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(std::iter::repeat_n(
            Constraint::Length(1),
            visible_mappings.len(),
        ));
    let rows = row_layout.split(inner);

    for (i, row) in rows.iter().enumerate() {
        let actual_idx = start_idx + i;
        let col_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints::<&[Constraint]>(&[
                Constraint::Length(max_key_len + 1),
                Constraint::Length(max_desc_len),
            ]);
        let cells = col_layout.split(*row);

        let mut widget = Paragraph::new(visible_mappings[i].keys);
        if actual_idx == 0 {
            widget = widget
                .add_modifier(Modifier::BOLD)
                .fg(Color::themed(Role::EMPH));
        } else {
            widget = widget.fg(Color::themed(Role::HD2)).not_bold();
        }
        f.render_widget(widget, cells[0]);

        let mut widget = Paragraph::new(visible_mappings[i].desc);
        if actual_idx == 0 {
            widget = widget
                .add_modifier(Modifier::BOLD)
                .fg(Color::themed(Role::EMPH));
        } else {
            widget = widget
                .remove_modifier(Modifier::BOLD)
                .set_style(RataStyle::themed(Role::INFO).not_bold());
        }
        f.render_widget(widget, cells[1]);
    }
}

#[must_use]
/// Creates a centered rectangle within the given area with the specified maximum dimensions.
///
/// This function creates a popup-style rectangle that is centered both horizontally
/// and vertically within the provided area, constrained by the given maximum width
/// and height.
///
/// # Arguments
///
/// * `max_width` - The maximum width of the centered rectangle
/// * `max_height` - The maximum height of the centered rectangle
/// * `r` - The area within which to center the rectangle
///
/// # Returns
///
/// A `Rect` representing the centered rectangle
#[profiled]
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
#[profiled]
pub fn normalize_newlines(input: &str) -> String {
    let re: &Regex = re!(r"\r\n?");

    re.replace_all(input, "\n").to_string()
}

/// Reset the terminal.
///
/// # Errors
///
/// This function will bubble up any `ratatui` or `crossterm` errors encountered.
// TODO: move to shared or tui_editor?
#[profiled]
pub fn reset_term(mut term: Terminal<CrosstermBackend<std::io::StdoutLock<'_>>>) -> ThagResult<()> {
    disable_raw_mode()?;
    ratatui::crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}

/// Save a `TextArea` to history if it has changed.
///
/// # Errors
///
/// This function will bubble up any i/o errors encuntered.
#[profiled]
pub fn save_if_changed(
    hist: &mut History,
    textarea: &mut TextArea<'_>,
    history_path: Option<&PathBuf>,
) -> ThagResult<()> {
    debug_log!("save_if_changed...");
    if textarea.is_empty() {
        debug_log!("nothing to save(1)...");
        return Ok(());
    }
    if let Some(entry) = &hist.get_current() {
        let index = entry.index;
        let copy_text = copy_text(textarea);
        // In case they entered blanks
        if copy_text.trim().is_empty() {
            debug_log!("nothing to save(2)...");
            return Ok(());
        }
        if entry.contents() != copy_text {
            hist.update_entry(index, &copy_text);
            if let Some(hist_path) = history_path {
                hist.save_to_file(hist_path)?;
            }
        }
    }
    Ok(())
}

// Save a `TextArea` to history if it has changed.
//
// # Errors
//
// This function will bubble up any i/o errors encuntered.
// pub fn remove_current_from_history(
//     hist: &mut History,
//     textarea: &mut TextArea<'_>,
//     history_path: &Option<PathBuf>,
// ) -> ThagResult<()> {
//     debug_log!("save_if_changed...");
//     if textarea.is_empty() {
//         debug_log!("nothing to save(1)...");
//         return Ok(());
//     }
//     if let Some(entry) = &hist.get_current() {
//         let index = entry.index;
//         let copy_text = copy_text(textarea);
//         // In case they entered blanks
//         if copy_text.trim().is_empty() {
//             debug_log!("nothing to save(2)...");
//             return Ok(());
//         }
//         if entry.contents() != copy_text {
//             hist.update_entry(index, &copy_text);
//             if let Some(ref hist_path) = history_path {
//                 hist.save_to_file(hist_path)?;
//             }
//         }
//     }
//     Ok(())
// }

/// Paste the contents of a history entry into a text area.
///
/// This function clears the current content of the `TextArea` by selecting all
/// and cutting it, then inserts the content from the provided history entry.
///
/// # Arguments
///
/// * `textarea` - A mutable reference to the `TextArea` to paste into
/// * `entry` - The history entry containing the content to paste
#[profiled]
pub fn paste_to_textarea(textarea: &mut TextArea<'_>, entry: &Entry) {
    textarea.select_all();
    textarea.cut();
    // 6
    textarea.insert_str(entry.contents());
}

/// Save a `TextArea` to history and the history to the backing file.
///
/// # Errors
///
/// This function will bubble up any i/o errors encuntered.
#[profiled]
pub fn preserve(
    textarea: &mut TextArea<'_>,
    hist: &mut History,
    history_path: &PathBuf,
) -> ThagResult<()> {
    debug_log!("preserve...");
    save_if_not_empty(textarea, hist);
    save_history(Some(&mut hist.clone()), Some(history_path))?;
    Ok(())
}

/// Save content from textarea to history if it's not empty.
///
/// This function copies the text content from the `TextArea` and adds it to the history
/// collection if the content is not empty (after trimming whitespace).
///
/// # Arguments
///
/// * `textarea` - A mutable reference to the `TextArea` to copy from
/// * `hist` - A mutable reference to the History to add the entry to
#[profiled]
pub fn save_if_not_empty(textarea: &mut TextArea<'_>, hist: &mut History) {
    debug_log!("save_if_not_empty...");

    let text = copy_text(textarea);
    if !text.trim().is_empty() {
        hist.add_entry(&text);
        debug_log!("... added entry");
    }
}

/// Copy the entire text content from a `TextArea`.
///
/// This function selects all text in the `TextArea`, copies it, and returns
/// the content as a single string with newlines preserved.
///
/// # Arguments
///
/// * `textarea` - A mutable reference to the `TextArea` to copy from
///
/// # Returns
///
/// A String containing the entire text content of the `TextArea`
#[profiled]
pub fn copy_text(textarea: &mut TextArea<'_>) -> String {
    textarea.select_all();
    textarea.copy();
    textarea.yank_text().lines().collect::<Vec<_>>().join("\n")
}

/// Save the history to the backing file.
///
/// # Errors
///
/// This function will bubble up any i/o errors encuntered.
#[profiled]
pub fn save_history(
    history: Option<&mut History>,
    history_path: Option<&PathBuf>,
) -> ThagResult<()> {
    debug_log!("save_history...{history:?}");
    if let Some(hist) = history
        && let Some(hist_path) = history_path
    {
        hist.save_to_file(hist_path)?;
        debug_log!("... saved to file");
    }
    Ok(())
}

/// Save Rust source code to a source file.
///
/// # Errors
///
/// This function will bubble up any i/o errors encuntered.
#[profiled]
pub fn save_source_file(
    to_rs_path: &PathBuf,
    textarea: &mut TextArea<'_>,
    saved: &mut bool,
) -> ThagResult<()> {
    // Ensure newline at end
    textarea.move_cursor(CursorMove::Bottom);
    textarea.move_cursor(CursorMove::End);
    if textarea.cursor().1 != 0 {
        textarea.insert_newline();
    }
    let _write_source = write_source(to_rs_path, textarea.lines().join("\n").as_str())?;
    *saved = true;
    Ok(())
}

/// Key mappings for display purposes via (Ctrl-l) in TUI editor and file dialog.
///
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

/// Key mappings for display purposes via (Ctrl-l) in TUI editor and file dialog.
pub const MAPPINGS: &[KeyDisplayLine] = key_mappings![
    (10, "Key bindings", "Description"),
    (
        20,
        "Shift+arrow keys",
        "Select/deselect chars (←→) or lines (↑↓)"
    ),
    (
        30,
        "Alt+shift+ h/j/k/l",
        "Select/deselect words (←h l→) or lines (↑k j↓)"
    ),
    (35, "Alt+shift+ p/n", "Select/deselect paras (↑p n↓)"),
    (40, "Alt+Shift+a", "Select all"),
    (50, "Alt+c", "Cancel selection"),
    (60, "Ctrl+d", "Submit"),
    (70, "Ctrl+q", "Cancel and quit"),
    (80, "Ctrl+h, Backspace", "Delete character before cursor"),
    (90, "Ctrl+i, Tab", "Indent"),
    (100, "Ctrl+m, Enter", "Insert newline"),
    (110, "Ctrl+k", "Delete from cursor to end of line"),
    (120, "Ctrl+j", "Delete from cursor to start of line"),
    (
        130,
        "Ctrl+w, Alt+Backspace",
        "Delete one word before cursor"
    ),
    (140, "Alt+d, Delete", "Delete one word from cursor position"),
    (150, "Ctrl+u", "Undo"),
    (160, "Ctrl+r", "Redo"),
    (170, "Ctrl+c", "Copy (yank) selected text"),
    (180, "Ctrl+x", "Cut (yank) selected text"),
    (190, "Ctrl+y", "Paste yanked text"),
    (
        200,
        "Ctrl+v, Shift+Ins, Cmd+v",
        "Paste from system clipboard according to platform"
    ),
    (210, "Ctrl+f, →", "Move cursor forward one character"),
    (220, "Ctrl+b, ←", "Move cursor backward one character"),
    (230, "Ctrl+p, ↑", "Move cursor up one line"),
    (240, "Ctrl+n, ↓", "Move cursor down one line"),
    (250, "Alt+f", "Move cursor forward one word"),
    (260, "Alt+Shift+f", "Move cursor to next word end"),
    (270, "Atl+b", "Move cursor backward one word"),
    (280, "Alt+p", "Move cursor up one paragraph"),
    (290, "Alt+n", "Move cursor down one paragraph"),
    (300, "Ctrl+e, End, Ctrl+Alt+f", "Move cursor to end of line"),
    (
        310,
        "Ctrl+a, Home, Ctrl+Alt+b",
        "Move cursor to start of line"
    ),
    (320, "Alt+<, Ctrl+Alt+p", "Move cursor to top of file"),
    (330, "Alt+>, Ctrl+Alt+n", "Move cursor to bottom of file"),
    (340, "Ctrl+l", "Toggle keys display (this screen)"),
    (350, "Ctrl+t", "Toggle selection highlight colours"),
    (360, "Alt+v, PageUp, F1", "Page up"),
    (370, "PageDown, F2", "Page down"),
    (380, "F4", "Clear text buffer (Ctrl+y or Ctrl+u to restore)"),
    (
        390,
        "F5",
        "Clear and wipe from history (Ctrl+y or Ctrl+u to restore text buffer)"
    ),
    (400, "F6", "Edit history"),
    (410, "F7", "Previous in history"),
    (420, "F8", "Next in history"),
    (
        430,
        "F9",
        "Enter `copy to system clipboard` mode with mouse selection and OS keys"
    ),
    (440, "F10", "Exit `copy to system clipboard` mode"),
    (450, "F12", "Save as..."),
];

#[derive(Clone, Debug, PartialEq, Eq)]
/// A struct representing a line in the key display help screen.
/// Contains information about key bindings and their descriptions.
pub struct KeyDisplayLine {
    /// Sequence number for ordering the display lines
    pub seq: usize,
    /// The key combination string to display
    pub keys: &'static str, // Or String if you plan to modify the keys later
    /// The description of what the key combination does
    pub desc: &'static str, // Or String for modifiability
}

impl PartialOrd for KeyDisplayLine {
    #[profiled]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // profile_method!("partial_cmp");
        Some(self.cmp(other))
    }
}

impl Ord for KeyDisplayLine {
    #[profiled]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // profile_method!("cmp");
        usize::cmp(&self.seq, &other.seq)
    }
}

impl KeyDisplayLine {
    /// Creates a new `KeyDisplayLine` with the specified sequence number, key combination, and description.
    ///
    /// # Arguments
    ///
    /// * `seq` - The sequence number for ordering the display lines
    /// * `keys` - The key combination string to display
    /// * `desc` - The description of what the key combination does
    #[must_use]
    pub const fn new(seq: usize, keys: &'static str, desc: &'static str) -> Self {
        Self { seq, keys, desc }
    }
}
