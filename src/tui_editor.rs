use crate::{
    code_utils::write_source,
    color_support::Level,
    debug_log,
    file_dialog::{DialogMode, FileDialog, Status},
    profile, profile_method, regex,
    stdin::edit_history,
    KeyCombination, Lvl, ThagError, ThagResult,
};
use crokey::key;
use crossterm::event::{
    self, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event::{self, Paste},
    KeyEvent, KeyEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use mockall::automock;
use ratatui::layout::{Constraint, Direction, Layout, Margin};
use ratatui::prelude::{CrosstermBackend, Rect};
use ratatui::style::{Color, Modifier, Style, Styled, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{block::Block, Borders, Clear, Paragraph};
use ratatui::{CompletedFrame, Frame, Terminal};
use regex::Regex;
use scopeguard::{guard, ScopeGuard};
use serde::{Deserialize, Serialize};
use std::{
    self,
    collections::VecDeque,
    convert::Into,
    env::var,
    fmt::{Debug, Display},
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::Duration,
};
use tui_textarea::{CursorMove, Input, TextArea};

pub const TITLE_TOP: &str = "Key bindings - subject to your terminal settings";
pub const TITLE_BOTTOM: &str = "Ctrl+l to hide";

pub type BackEnd<'a> = CrosstermBackend<std::io::StdoutLock<'a>>;
pub type Term<'a> = Terminal<BackEnd<'a>>;
pub type ResetTermClosure<'a> = Box<dyn FnOnce(Term<'a>)>;
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
    fn read_event(&self) -> ThagResult<Event> {
        profile_method!("read_event");
        crossterm::event::read().map_err(Into::<ThagError>::into)
    }

    fn poll(&self, timeout: Duration) -> ThagResult<bool> {
        profile_method!("poll");
        crossterm::event::poll(timeout).map_err(Into::<ThagError>::into)
    }
}

pub struct ManagedTerminal<'a> {
    terminal: TermScopeGuard<'a>,
}

impl ManagedTerminal<'_> {
    /// Draw to the terminal.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue drawing to the terminal.
    pub fn draw<F>(&mut self, f: F) -> std::io::Result<CompletedFrame>
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
pub fn resolve_term<'a>() -> ThagResult<Option<ManagedTerminal<'a>>> {
    profile!("resolve_term");
    if var("TEST_ENV").is_ok() {
        return Ok(None);
    }

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
pub struct Entry {
    pub index: usize,       // Holds the entry's index
    pub lines: Vec<String>, // Holds editor content as lines
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        profile_method!("fmt");
        write!(f, "{:?}: {}", self.index, self.lines.join("\n"))
    }
}

impl Entry {
    pub fn new(index: usize, content: &str) -> Self {
        profile_method!("new");
        Self {
            index,
            lines: content.lines().map(String::from).collect(),
        }
    }

    // Extracts string contents of entry for use in the editor
    #[must_use]
    pub fn contents(&self) -> String {
        profile_method!("contents");
        self.lines.join("\n")
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct History {
    pub current_index: Option<usize>,
    pub entries: VecDeque<Entry>, // Now a VecDeque of Entries
}

impl History {
    #[must_use]
    pub fn new() -> Self {
        profile_method!("new");
        Self {
            current_index: None,
            entries: VecDeque::with_capacity(20),
        }
    }

    #[must_use]
    pub fn load_from_file(path: &PathBuf) -> Self {
        profile_method!("load_from_file");
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

    #[must_use]
    pub fn at_start(&self) -> bool {
        profile_method!("at_start");
        debug_log!("at_start ...");
        self.current_index
            .map_or(true, |current_index| current_index == 0)
    }

    #[must_use]
    pub fn at_end(&self) -> bool {
        profile_method!("at_end");
        debug_log!("at_end ...");
        self.current_index.map_or(true, |current_index| {
            current_index == self.entries.len() - 1
        })
    }

    pub fn add_entry(&mut self, text: &str) {
        profile_method!("add_entry");
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

    pub fn update_entry(&mut self, index: usize, text: &str) {
        profile_method!("update_entry");
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

    pub fn delete_entry(&mut self, index: usize) {
        profile_method!("delete_entry");
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
    pub fn save_to_file(&mut self, path: &PathBuf) -> ThagResult<()> {
        profile_method!("save_to_file");
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

    pub fn get_current(&mut self) -> Option<&Entry> {
        profile_method!("get_current");
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

    pub fn get(&mut self, index: usize) -> Option<&Entry> {
        profile_method!("get");
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

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Entry> {
        profile_method!("get_mut");
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
    pub fn get_previous(&mut self) -> Option<&Entry> {
        profile_method!("get_previous");
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
    pub fn get_next(&mut self) -> Option<&Entry> {
        profile_method!("get_next");
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

    pub fn get_last(&mut self) -> Option<&Entry> {
        profile_method!("get_last");
        if self.entries.is_empty() {
            return None;
        }

        self.entries.back()
    }

    // Reassign indices so that the newest entry has index 0, and the oldests has len - 1
    fn reassign_indices(&mut self) {
        profile_method!("reassign_indices");
        // let len = self.entries.len();
        for (i, entry) in self.entries.iter_mut().enumerate() {
            entry.index = i;
        }
    }
}

// Struct to hold data-related parameters
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct EditData<'a> {
    pub return_text: bool,
    pub initial_content: &'a str,
    pub save_path: Option<&'a mut PathBuf>,
    pub history_path: Option<&'a PathBuf>,
    pub history: Option<History>,
}

impl From<&crate::log_color::Style> for Style {
    fn from(style: &crate::log_color::Style) -> Self {
        let mut rata_style = Self::default();

        // Convert the color if present
        if let Some(ref fg) = style.foreground {
            if let Some(i) = fg.index {
                rata_style = rata_style.fg(Color::Indexed(i));
            }
        }

        // Apply modifiers
        let mut modifiers = Modifier::empty();
        if style.bold {
            modifiers |= Modifier::BOLD;
        }
        if style.italic {
            modifiers |= Modifier::ITALIC;
        }
        if style.dim {
            modifiers |= Modifier::DIM;
        }

        rata_style.add_modifier(modifiers)
    }
}

// Implement conversion to ratatui's Color
impl From<&Level> for Color {
    fn from(level: &Level) -> Self {
        Self::Indexed(u8::from(level))
    }
}

// Struct to hold display-related parameters
#[derive(Debug)]
pub struct KeyDisplay<'a> {
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
    edit_data: &mut EditData,
    display: &KeyDisplay,
    key_handler: F, // closure or function for key handling
) -> ThagResult<(KeyAction, Option<Vec<String>>)>
where
    R: EventReader + Debug,
    F: Fn(
        KeyEvent,
        Option<&mut ManagedTerminal>,
        &mut TextArea,
        &mut EditData,
        &mut bool,
        &mut bool,
        &mut String,
    ) -> ThagResult<KeyAction>,
{
    // Initialize state variables
    let mut popup = false;
    let mut tui_highlight_fg: Lvl = Lvl::EMPH;
    let mut saved = false;
    let mut status_message: String = String::default(); // Add status message variable

    let mut maybe_term = resolve_term()?;

    // Create the TextArea from initial content
    let mut textarea = TextArea::from(edit_data.initial_content.lines());

    // Set up the display parameters for the textarea
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(display.title)
            .title_style(display.title_style),
    );

    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
    textarea.move_cursor(CursorMove::Bottom);
    // New line with cursor at EOF for usability
    textarea.move_cursor(CursorMove::End);
    if !textarea.is_empty() {
        textarea.insert_newline();
    }

    // Apply initial highlights
    highlight_selection(&mut textarea, tui_highlight_fg);

    let remove = display.remove_keys;
    let add = display.add_keys;
    // Can't make these OnceLock values, since their configuration depends on the `remove`
    // and `add` values passed in by the caller.
    let mut adjusted_mappings: Vec<KeyDisplayLine> = MAPPINGS
        .iter()
        .filter(|&row| !remove.contains(&row.keys))
        .chain(add.iter())
        .cloned()
        .collect();
    adjusted_mappings.sort();
    let (max_key_len, max_desc_len) =
        adjusted_mappings
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
            maybe_term.as_mut().map_or_else(
                || Err("Logic issue unwrapping term we wrapped ourselves".into()),
                |term| {
                    term.draw(|f| {
                        // Get the size of the available terminal area
                        let area = f.area();

                        // Ensure there's enough height for both the textarea and the status line
                        if area.height > 1 {
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints(
                                    [
                                        Constraint::Min(area.height - 3), // Editor area takes up the rest
                                        Constraint::Length(3),            // Status line gets 1 line
                                    ]
                                    .as_ref(),
                                )
                                .split(area);

                            // Render the textarea in the first chunk
                            f.render_widget(&textarea, chunks[0]);

                            // Render the status line in the second chunk
                            let status_block = Block::default()
                                .borders(Borders::ALL)
                                .title("Status")
                                .style(Style::default().fg(Color::White))
                                .title_style(display.title_style);

                            let status_text = Paragraph::new::<&str>(status_message.as_ref())
                                .block(status_block)
                                .style(Style::default().fg(Color::White));

                            f.render_widget(status_text, chunks[1]);

                            if popup {
                                display_popup(
                                    &adjusted_mappings,
                                    TITLE_TOP,
                                    TITLE_BOTTOM,
                                    max_key_len,
                                    max_desc_len,
                                    f,
                                );
                            };
                            highlight_selection(&mut textarea, tui_highlight_fg);
                            // status_message = String::new();
                        }
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
            // Ignore key release, which creates an unwanted second event in Windows
            if !matches!(key_event.kind, KeyEventKind::Press) {
                continue;
            }
            //
            // log::debug_log!("key_event={key_event:#?}");
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
                    if textarea.is_selecting() {
                        textarea.cancel_selection();
                    }
                    textarea.move_cursor(CursorMove::Forward);
                }
                key!(ctrl - b) | key!(left) => {
                    if textarea.is_selecting() {
                        textarea.cancel_selection();
                    }
                    textarea.move_cursor(CursorMove::Back);
                }
                key!(ctrl - p) | key!(up) => {
                    if textarea.is_selecting() {
                        textarea.cancel_selection();
                    }
                    textarea.move_cursor(CursorMove::Up);
                }
                key!(ctrl - n) | key!(down) => {
                    if textarea.is_selecting() {
                        textarea.cancel_selection();
                    }
                    textarea.move_cursor(CursorMove::Down);
                }
                key!(alt - f) | key!(ctrl - right) => {
                    if textarea.is_selecting() {
                        textarea.cancel_selection();
                    }
                    textarea.move_cursor(CursorMove::WordForward);
                }
                key!(alt - shift - f) => {
                    textarea.move_cursor(CursorMove::WordEnd);
                }
                key!(alt - b) | key!(ctrl - left) => {
                    if textarea.is_selecting() {
                        textarea.cancel_selection();
                    }
                    textarea.move_cursor(CursorMove::WordBack);
                }
                key!(alt - p) | key!(alt - ')') | key!(ctrl - up) => {
                    if textarea.is_selecting() {
                        textarea.cancel_selection();
                    }
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
                    crossterm::execute!(std::io::stdout().lock(), DisableMouseCapture,)?;
                    textarea.remove_line_number();
                }
                key!(f10) => {
                    crossterm::execute!(std::io::stdout().lock(), EnableMouseCapture,)?;
                    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
                }
                key!(alt - '<') | key!(ctrl - alt - p) | key!(ctrl - alt - up) => {
                    textarea.move_cursor(CursorMove::Top);
                }
                key!(alt - '>') | key!(ctrl - alt - n) | key!(ctrl - alt - down) => {
                    textarea.move_cursor(CursorMove::Bottom);
                }
                key!(alt - c) => {
                    if textarea.is_selecting() {
                        textarea.cancel_selection();
                    } else {
                        textarea.start_selection();
                    }
                }
                // key!(alt - shift - c) => {
                //     textarea.start_selection();
                // }
                key!(alt - shift - a) => {
                    textarea.select_all();
                }
                key!(ctrl - t) => {
                    // Toggle highlighting colours
                    tui_highlight_fg = match tui_highlight_fg {
                        Lvl::EMPH => Lvl::BRI,
                        Lvl::BRI => Lvl::ERR,
                        Lvl::ERR => Lvl::WARN,
                        Lvl::WARN => Lvl::HEAD,
                        Lvl::HEAD => Lvl::SUBH,
                        _ => Lvl::EMPH,
                    };
                    if var("TEST_ENV").is_err() {
                        #[allow(clippy::option_if_let_else)]
                        if let Some(ref mut term) = maybe_term {
                            term.draw(|_| {
                                highlight_selection(&mut textarea, tui_highlight_fg);
                            })?;
                        }
                    }
                }
                _ => {
                    // Call the key_handler closure to process events
                    let key_action = key_handler(
                        key_event,
                        maybe_term.as_mut(),
                        &mut textarea,
                        edit_data,
                        &mut popup,
                        &mut saved,
                        &mut status_message,
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
            // println!("You typed {key_combination:?} which represents nothing yet"/*, key.blue()*/);
            let input = Input::from(event);
            textarea.input(input);
        }
    }
}

pub fn highlight_selection(textarea: &mut TextArea<'_>, tui_highlight_fg: crate::MessageLevel) {
    profile!("highlight_selection");
    textarea.set_selection_style(
        Style::default()
            .fg(Color::Indexed(u8::from(&tui_highlight_fg)))
            .bold(),
    );
}

/// Key handler function to be passed into `tui_edit` for editing REPL history.
///
/// # Errors
///
/// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
#[allow(clippy::too_many_lines, clippy::missing_panics_doc)]
pub fn script_key_handler(
    key_event: KeyEvent,
    maybe_term: Option<&mut ManagedTerminal>,
    textarea: &mut TextArea,
    edit_data: &mut EditData,
    popup: &mut bool,
    saved: &mut bool, // TODO decide if we need this
    status_message: &mut String,
) -> ThagResult<KeyAction> {
    profile!("script_key_handler");
    if !matches!(key_event.kind, KeyEventKind::Press) {
        return Ok(KeyAction::Continue);
    }

    let key_combination = KeyCombination::from(key_event); // Derive KeyCombination
                                                           // eprintln!("key_combination={key_combination:?}");

    let history_path = edit_data.history_path.cloned();

    #[allow(clippy::unnested_or_patterns)]
    match key_combination {
        key!(esc) | key!(ctrl - c) | key!(ctrl - q) => Ok(KeyAction::Quit(*saved)),
        key!(ctrl - d) => save_and_submit(history_path.as_ref(), edit_data, textarea),
        key!(ctrl - s) | key!(ctrl - alt - s) => {
            // eprintln!("key_combination={key_combination:?}, maybe_save_path={maybe_save_path:?}");
            if matches!(key_combination, key!(ctrl - s)) {
                save(
                    edit_data,
                    history_path.as_ref(),
                    textarea,
                    saved,
                    status_message,
                )?;
                Ok(KeyAction::Save)
            } else {
                save_as(maybe_term, textarea, saved, status_message)
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
        key!(f4) => {
            // Clear textarea
            textarea.select_all();
            textarea.cut();
            Ok(KeyAction::Continue)
        }
        key!(f5) => {
            // Clear textarea and wipe from history
            if textarea.is_empty() {
                return Ok(KeyAction::Continue);
            }
            wipe_textarea(edit_data, textarea, history_path.as_ref())?;
            Ok(KeyAction::Continue)
        }
        key!(f6) => {
            // Edit history
            edit_history()?;
            Ok(KeyAction::Continue)
        }
        key!(f7) => {
            // Scroll up in history
            prev_hist(edit_data, textarea, history_path.as_ref())?;
            Ok(KeyAction::Continue)
        }
        key!(f8) => {
            // Scroll down in history
            next_hist(edit_data, textarea);
            Ok(KeyAction::Continue)
        }
        _ => {
            // Update the textarea with the input from the key event
            textarea.input(Input::from(key_event)); // Input derived from Event
            Ok(KeyAction::Continue)
        }
    }
}

fn next_hist(edit_data: &mut EditData<'_>, textarea: &mut TextArea<'_>) {
    if let Some(ref mut hist) = edit_data.history {
        // save_if_changed(hist, textarea, &history_path)?;
        if let Some(entry) = hist.get_next() {
            debug_log!("F8 found entry {entry:?}");
            paste_to_textarea(textarea, entry);
        }
    }
}

fn prev_hist(
    edit_data: &mut EditData<'_>,
    textarea: &mut TextArea<'_>,
    history_path: Option<&PathBuf>,
) -> ThagResult<()> {
    if let Some(ref mut hist) = edit_data.history {
        if hist.at_end() && textarea.is_empty() {
            if let Some(entry) = &hist.get_last() {
                debug_log!("F7 (1) found entry {entry:?}");
                paste_to_textarea(textarea, entry);
            }
        } else {
            save_if_changed(hist, textarea, history_path)?;
            if let Some(entry) = &hist.get_previous() {
                debug_log!("F7 (2) found entry {entry:?}");
                paste_to_textarea(textarea, entry);
            }
        }
    }
    Ok(())
}

fn wipe_textarea(
    edit_data: &mut EditData<'_>,
    textarea: &mut TextArea<'_>,
    history_path: Option<&PathBuf>,
) -> ThagResult<()> {
    if let Some(ref mut hist) = edit_data.history {
        let _in_hist = !&hist.at_end();
        let textarea_contents = textarea.lines().to_vec().join("\n");
        textarea.select_all();
        textarea.cut();
        let yank_text = textarea.yank_text();
        assert_eq!(yank_text, textarea_contents);
        if let Some(current_hist_entry) = &hist.get_current() {
            assert_eq!(yank_text, current_hist_entry.contents());
            let index = current_hist_entry.index;
            hist.delete_entry(index);
            hist.entries
                .retain(|f| f.contents().trim() != textarea_contents);
        }
        if let Some(hist_path) = history_path {
            hist.save_to_file(hist_path)?;
        }
    }
    Ok(())
}

fn save_as(
    maybe_term: Option<&mut ManagedTerminal<'_>>,
    textarea: &mut TextArea<'_>,
    saved: &mut bool,
    status_message: &mut String,
) -> ThagResult<KeyAction> {
    if let Some(term) = maybe_term {
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
            status_message.clear();
            status_message.push_str(&format!("Saved to {}", to_rs_path.display()));

            Ok(KeyAction::Save)
        } else {
            Ok(KeyAction::Continue)
        }
    } else {
        Ok(KeyAction::Continue)
    }
}

fn save(
    edit_data: &mut EditData<'_>,
    history_path: Option<&PathBuf>,
    textarea: &mut TextArea<'_>,
    saved: &mut bool,
    status_message: &mut String,
) -> ThagResult<()> {
    if let Some(ref mut save_path) = edit_data.save_path {
        if let Some(hist_path) = history_path {
            let history = &mut edit_data.history;
            if let Some(hist) = history {
                preserve(textarea, hist, hist_path)?;
            };
            let result = save_source_file(save_path, textarea, saved);
            match result {
                Ok(()) => {
                    status_message.clear();
                    status_message.push_str(&format!("Saved to {}", save_path.display()));
                }
                Err(e) => return Err(e),
            }
        }
    }
    Ok(())
}

fn save_and_submit(
    history_path: Option<&PathBuf>,
    edit_data: &mut EditData<'_>,
    textarea: &mut TextArea<'_>,
) -> ThagResult<KeyAction> {
    if let Some(hist_path) = history_path {
        let history = &mut edit_data.history;
        if let Some(hist) = history {
            preserve(textarea, hist, hist_path)?;
        };
    }
    Ok(KeyAction::Submit)
}

/// Enable raw mode, but not if in test mode, because that will cause the dreaded rightward drift
/// in log output due to carriage returns being ignored.
///
/// # Errors
///
/// This function will bubble up any i/o errors encountered by `crossterm::enable_raw_mode`.
pub fn maybe_enable_raw_mode() -> ThagResult<()> {
    profile!("maybe_enable_raw_mode");
    let test_env = &var("TEST_ENV");
    debug_log!("test_env={test_env:?}");
    if !test_env.is_ok() && !is_raw_mode_enabled()? {
        debug_log!("Enabling raw mode");
        enable_raw_mode()?;
    }
    Ok(())
}

pub fn display_popup(
    mappings: &[KeyDisplayLine],
    title_top: &str,
    title_bottom: &str,
    max_key_len: u16,
    max_desc_len: u16,
    f: &mut ratatui::prelude::Frame<'_>,
) {
    profile!("display_popup");
    let num_filtered_rows = mappings.len();
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(title_top).centered())
        .title_bottom(Line::from(title_bottom).centered())
        .add_modifier(Modifier::BOLD)
        .fg(Color::Indexed(u8::from(&Lvl::HEAD)));
    #[allow(clippy::cast_possible_truncation)]
    let area = centered_rect(
        max_key_len + max_desc_len + 5,
        num_filtered_rows as u16 + 5,
        f.area(),
    );
    let inner = area.inner(Margin {
        vertical: 2,
        horizontal: 2,
    });
    // this is supposed to clear out the background
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    #[allow(clippy::cast_possible_truncation)]
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
        let mut widget = Paragraph::new(mappings[i].keys);
        if i == 0 {
            widget = widget
                .add_modifier(Modifier::BOLD)
                .fg(Color::Indexed(u8::from(&Lvl::EMPH)));
        } else {
            widget = widget.fg(Color::Indexed(u8::from(&Lvl::SUBH))).not_bold();
        }
        f.render_widget(widget, cells[0]);
        let mut widget = Paragraph::new(mappings[i].desc);

        if i == 0 {
            widget = widget
                .add_modifier(Modifier::BOLD)
                .fg(Color::Indexed(u8::from(&Lvl::EMPH)));
        } else {
            widget = widget.remove_modifier(Modifier::BOLD).set_style(
                Style::default()
                    .fg(Color::Indexed(u8::from(&Lvl::NORM)))
                    .not_bold(),
            );
        }
        f.render_widget(widget, cells[1]);
    }
}

#[must_use]
pub fn centered_rect(max_width: u16, max_height: u16, r: Rect) -> Rect {
    profile!("centered_rect");
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
    profile!("normalize_newlines");
    let re: &Regex = regex!(r"\r\n?");

    re.replace_all(input, "\n").to_string()
}

/// Reset the terminal.
///
/// # Errors
///
/// This function will bubble up any `ratatui` or `crossterm` errors encountered.
// TODO: move to shared or tui_editor?
pub fn reset_term(mut term: Terminal<CrosstermBackend<std::io::StdoutLock<'_>>>) -> ThagResult<()> {
    profile!("reset_term");
    disable_raw_mode()?;
    crossterm::execute!(
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
pub fn save_if_changed(
    hist: &mut History,
    textarea: &mut TextArea<'_>,
    history_path: Option<&PathBuf>,
) -> ThagResult<()> {
    profile!("save_if_changed");
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
//     profile!("save_if_changed");
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

pub fn paste_to_textarea(textarea: &mut TextArea<'_>, entry: &Entry) {
    profile!("paste_to_textarea");
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
pub fn preserve(
    textarea: &mut TextArea<'_>,
    hist: &mut History,
    history_path: &PathBuf,
) -> ThagResult<()> {
    profile!("preserve");
    debug_log!("preserve...");
    save_if_not_empty(textarea, hist);
    save_history(Some(&mut hist.clone()), Some(history_path))?;
    Ok(())
}

pub fn save_if_not_empty(textarea: &mut TextArea<'_>, hist: &mut History) {
    profile!("save_if_not_empty");
    debug_log!("save_if_not_empty...");

    let text = copy_text(textarea);
    if !text.trim().is_empty() {
        hist.add_entry(&text);
        debug_log!("... added entry");
    }
}

pub fn copy_text(textarea: &mut TextArea<'_>) -> String {
    profile!("copy_text");
    textarea.select_all();
    textarea.copy();
    let text = textarea.yank_text().lines().collect::<Vec<_>>().join("\n");
    text
}

/// Save the history to the backing file.
///
/// # Errors
///
/// This function will bubble up any i/o errors encuntered.
pub fn save_history(
    history: Option<&mut History>,
    history_path: Option<&PathBuf>,
) -> ThagResult<()> {
    profile!("save_history");
    debug_log!("save_history...{history:?}");
    if let Some(hist) = history {
        if let Some(hist_path) = history_path {
            hist.save_to_file(hist_path)?;
            debug_log!("... saved to file");
        }
    }
    Ok(())
}

/// Save Rust source code to a source file.
///
/// # Errors
///
/// This function will bubble up any i/o errors encuntered.
pub fn save_source_file(
    to_rs_path: &PathBuf,
    textarea: &mut TextArea<'_>,
    saved: &mut bool,
) -> ThagResult<()> {
    profile!("save_source_file");
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
    (40, "Alt+a", "Select all"),
    (
        50,
        "Alt+c",
        "Toggle selection mode: start selecting / cancel selection"
    ),
    (60, "Ctrl+d", "Submit"),
    (70, "Ctrl+q", "Cancel and quit"),
    (80, "Ctrl+h, Backspace", "Delete character before cursor"),
    (90, "Ctrl+i, Tab", "Indent"),
    (100, "Ctrl+m, Enter", "Insert newline"),
    (110, "Ctrl+k", "Delete from cursor to end of line"),
    (120, "Ctrl+j", "Delete from cursor to start of line"),
    (
        130,
        "Ctrl+w, Alt+Backspace",
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
        "Ctrl+v, Shift+Ins, Cmd+v",
        "Paste from system clipboard"
    ),
    (210, "Ctrl+f, →", "Move cursor forward one character"),
    (220, "Ctrl+b, ←", "Move cursor backward one character"),
    (230, "Ctrl+p, ↑", "Move cursor up one line"),
    (240, "Ctrl+n, ↓", "Move cursor down one line"),
    (250, "Alt+f, Ctrl+→", "Move cursor forward one word"),
    (260, "Alt+Shift+f", "Move cursor to next word end"),
    (270, "Atl+b, Ctrl+←", "Move cursor backward one word"),
    (280, "Alt+) or p, Ctrl+↑", "Move cursor up one paragraph"),
    (290, "Alt+( or n, Ctrl+↓", "Move cursor down one paragraph"),
    (
        300,
        "Ctrl+e, End, Ctrl+Alt+f or → , Cmd+→",
        "Move cursor to end of line"
    ),
    (
        310,
        "Ctrl+a, Home, Ctrl+Alt+b or ← , Cmd+←",
        "Move cursor to start of line"
    ),
    (320, "Alt+<, Ctrl+Alt+p or ↑", "Move cursor to top of file"),
    (
        330,
        "Alt+>, Ctrl+Alt+n or ↓",
        "Move cursor to bottom of file"
    ),
    (340, "PageDown, Cmd+↓", "Page down"),
    (350, "Alt+v, PageUp, Cmd+↑", "Page up"),
    (360, "Ctrl+l", "Toggle keys display (this screen)"),
    (370, "Ctrl+t", "Toggle selection highlight colours"),
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
        "Suspend mouse capture and line numbers for system copy"
    ),
    (440, "F10", "Resume mouse capture and line numbers"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyDisplayLine {
    pub seq: usize,
    pub keys: &'static str, // Or String if you plan to modify the keys later
    pub desc: &'static str, // Or String for modifiability
}

impl PartialOrd for KeyDisplayLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // profile_method!("partial_cmp");
        Some(self.cmp(other))
    }
}

impl Ord for KeyDisplayLine {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // profile_method!("cmp");
        usize::cmp(&self.seq, &other.seq)
    }
}

impl KeyDisplayLine {
    #[must_use]
    pub const fn new(seq: usize, keys: &'static str, desc: &'static str) -> Self {
        Self { seq, keys, desc }
    }
}
