/*[toml]
[dependencies]
crossterm = "0.28.1"
derivative = "2.2"
log = "0.4.22"
ratatui = { version = "0.28.1", features = ["unstable-widget-ref"] }
ratatui-explorer = "0.1.2"
thag_rs = { git = "https://github.com/durbanlegend/thag_rs" }
tui-textarea = "0.6.0"
*/
/// Original is `https://github.com/flip1995/tui-rs-file-dialog/blob/master/src/lib.rs`
/// Copyright (c) 2023 Philipp Krones
/// Licence: MIT
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, ListState},
    Frame, Terminal,
};
use ratatui_explorer::{FileExplorer, Theme};
use std::{
    cmp,
    ffi::OsString,
    fs,
    io::{self, stdout, Result},
    iter,
    path::{Path, PathBuf},
    sync::OnceLock,
};
use tui_textarea::{Input, TextArea};

use thag_rs::tui_editor::{self, centered_rect};
use thag_rs::{debug_log, key_mappings};
use thag_rs::{shared::KeyDisplayLine, tui_editor::display_popup};

/// File dialog mode to distinguish between Open and Save dialogs
#[derive(Debug, PartialEq, Eq)]
pub enum DialogMode {
    Open,
    Save,
}

/// A pattern that can be used to filter the displayed files.
pub enum FilePattern {
    /// Filter by file extension. This filter is case insensitive.
    Extension(String),
    /// Filter by substring. This filter is case sensitive.
    Substring(String),
}

/// Enum to represent which part of the dialog has focus.
#[derive(Debug, PartialEq, Eq)]
pub enum DialogFocus {
    List,  // Focus on file list
    Input, // Focus on input area
}

pub enum Status {
    Complete,
    Incomplete,
    Quit,
}

/// The file dialog.
///
/// This manages the state of the file dialog. After selecting or specifying a file, the absolute
/// path to that file will be stored in the file dialog.
///
/// The file dialog is opened with the current working directory by default.
// pub struct FileDialog<'a, FilePattern> {
pub struct FileDialog<'a> {
    /// The file that was selected or specified when the file dialog was open the last time.
    pub selected_file: Option<PathBuf>,

    width: u16,
    height: u16,

    filter: Option<FilePattern>,
    open: bool,
    current_dir: PathBuf,
    show_hidden: bool,

    list_state: ListState,
    items: Vec<String>,

    file_explorer: FileExplorer,

    /// Current mode of the dialog (Open or Save)
    mode: DialogMode,

    /// Current focus of the Save dialog (List or Input)
    focus: DialogFocus,

    pub popup: bool,
    // title_bottom: &'a str,
    /// Input for the file name in Input mode
    pub input: TextArea<'a>,
}

// impl<FilePattern> FileDialog<'_, FilePattern> {
impl<'a> FileDialog<'a> {
    /// Create a new file dialog.
    ///
    /// The width and height are the size of the file dialog in percent of the terminal size. They
    /// are clamped to 100%.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered by the `update_entries` method.
    pub fn new(width: u16, height: u16, mode: DialogMode) -> Result<Self> {
        // Create a new file explorer with the default theme and title.
        let theme = Theme::default().add_default_title();
        let file_explorer = FileExplorer::with_theme(theme)?;
        let mut s = Self {
            width: cmp::min(width, 100),
            height: cmp::min(height, 100),
            selected_file: None,
            filter: None,
            open: false,
            current_dir: PathBuf::from(".").canonicalize()?,
            show_hidden: false,
            list_state: ListState::default(),
            items: vec![],
            file_explorer,
            mode,
            focus: DialogFocus::List,
            popup: false, // Start with focus on the file list
            input: TextArea::default(),
            // title_bottom: "Ctrl+l to show keys",
        };
        s.update_entries()?;
        Ok(s)
    }

    /// Sets the directory to open the file dialog in.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is a problem canonicalizing the directory.
    pub fn set_dir(&mut self, dir: &Path) -> Result<()> {
        self.current_dir = dir.canonicalize()?;
        self.update_entries()
    }

    /// Sets the filter to use when browsing files.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered by the `update_entries` method.
    pub fn set_filter(&mut self, filter: FilePattern) -> Result<()> {
        self.filter = Some(filter);
        self.update_entries()
    }

    /// Removes the filter.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered by the `update_entries` method.
    pub fn reset_filter(&mut self) -> Result<()> {
        self.filter.take();
        self.update_entries()
    }

    /// Toggles whether hidden files should be shown.
    ///
    /// This only checks whether the file name starts with a dot.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered by the `update_entries` method.
    pub fn toggle_show_hidden(&mut self) -> Result<()> {
        self.show_hidden = !self.show_hidden;
        self.update_entries()
    }

    /// Opens the file dialog.
    pub fn open(&mut self) {
        self.selected_file.take();
        self.open = true;
    }

    /// Closes the file dialog.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Returns whether the file dialog is currently open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// Draws the file dialog in the TUI application.
    pub fn draw(&mut self, f: &mut Frame) {
        if self.open {
            let area = centered_rect(self.width, self.height, f.area());

            // Split the area into two parts: one for the file list and one for the input field.
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),            // Input field area
                        Constraint::Min(self.height - 3), // File list area
                    ]
                    .as_ref(),
                )
                .split(area);

            /*
            // Determine if the file list has focus
            let file_list_focus = matches!(self.focus, DialogFocus::List);

            // Render the file list with conditional style based on focus
            let list_style = if file_list_focus {
                Style::default()
                    .bg(Color::LightMagenta)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Gray)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::DIM)
            };

            let block = Block::default()
                .title(format!("{}", self.current_dir.to_string_lossy()))
                .borders(Borders::ALL)
                .border_style(if file_list_focus {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray).dim()
                })
                .title_bottom(Line::from(self.title_bottom).centered());
            let list_items: Vec<ListItem> = self
                .items
                .iter()
                .map(|s| ListItem::new(s.as_str()))
                .collect();
            let list = List::new(list_items)
                .block(block)
                .highlight_style(list_style)
                .style(if file_list_focus {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray).dim()
                });
            */
            f.render_widget(&self.file_explorer.widget(), chunks[1]);

            // Render the input box for the filename
            let input_focus = matches!(self.focus, DialogFocus::Input);
            if self.mode == DialogMode::Save {
                // Create a Block for the input area with borders and background
                let input_style = if input_focus {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray).dim()
                };
                let input_block = Block::default()
                    .title("File Name")
                    .borders(Borders::ALL)
                    .style(input_style)
                    .border_style(input_style);

                let input_area = input_block.inner(chunks[0]); // Adjusts area to fit within borders
                f.render_widget(input_block, chunks[0]);

                // Determine if the filename input has focus

                // Conditionally show the cursor only if the input box has focus
                self.input.set_cursor_line_style(if input_focus {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray) // Cursor won't be visible when not focused
                });

                if input_focus {
                    self.input.set_style(Style::default());
                    self.input
                        .set_selection_style(Style::default().bg(Color::Blue));
                    self.input.set_cursor_style(Style::default().on_magenta());
                    self.input
                        .set_cursor_line_style(Style::default().on_dark_gray());
                } else {
                    self.input.set_style(Style::default().dim());
                    self.input.set_selection_style(Style::default().hidden());
                    self.input.set_cursor_style(Style::default().hidden());
                    self.input.set_cursor_line_style(Style::default().hidden());
                }
                f.render_widget(&self.input, input_area); // Renders the input widget inside the block
            }

            if self.popup {
                let title_bottom = tui_editor::TITLE_BOTTOM;
                let (max_key_len, max_desc_len) = get_max_lengths(MAPPINGS);
                display_popup(
                    MAPPINGS,
                    "Key bindings",
                    title_bottom,
                    max_key_len,
                    max_desc_len,
                    f,
                );
            };
        }
    }

    /// Goes to the next item in the file list.
    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => cmp::min(self.items.len() - 1, i + 1),
            None => cmp::min(self.items.len().saturating_sub(1), 1),
        };
        self.list_state.select(Some(i));
    }
    /// Goes to the previous item in the file list.
    pub fn previous(&mut self) {
        let i = self
            .list_state
            .selected()
            .map_or(0, |i| i.saturating_sub(1));
        self.list_state.select(Some(i));
    }

    /// Goes to the parent directory.
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered by the `update_entries` method.
    pub fn up(&mut self) -> Result<()> {
        self.current_dir.pop();
        self.update_entries()
    }

    /// Selects an item or sets a file name (for Save mode).
    ///
    /// If the item is a directory, the file dialog will move into that directory. If the item is a
    /// file, the file dialog will close and the path to the file will be stored in
    /// [`FileDialog::selected_file`].
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered by the `update_entries` method.
    pub fn select(&mut self) -> Result<()> {
        // Open mode logic (already correct)
        debug_log!("In select()");
        let Some(selected) = self.list_state.selected() else {
            self.next();
            debug_log!("Returning Ok(())");
            return Ok(());
        };

        let path = self.current_dir.join(&self.items[selected]);
        debug_log!(
            "current_dir={:?}; path={path:?}; is_file? {}; mode={:?}",
            self.current_dir,
            path.is_file(),
            self.mode
        );

        if path.is_dir() {
            self.current_dir.clone_from(&path);
            self.update_entries()?;
        }
        // if matches!(self.mode, DialogMode::Save) {
        if self.focus == DialogFocus::Input {
            // Save mode logic to use the entered filename
            let file_name = self.input.lines().join(""); // Get the input from TextArea
            debug_log!("file_name={file_name}");
            if !file_name.is_empty() {
                let path = self.current_dir.join(file_name);
                self.selected_file = Some(path); // Set the selected file
                self.close(); // Close the dialog
            }
        } else if path.is_file() {
            self.selected_file = Some(path);
            self.close();
            // return Ok(());
        }
        debug_log!("self.selected_file={:?}", self.selected_file);
        Ok(())
    }

    /// Updates the entries in the file list. This function is called automatically when necessary.
    ///
    /// # Panics
    ///
    /// Panics if there is a logic error comparing two strings.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered.
    fn update_entries(&mut self) -> Result<()> {
        self.items = iter::once("..".to_string())
            .chain(
                fs::read_dir(&self.current_dir)?
                    .flatten()
                    .filter(|e| -> bool {
                        let e = e.path();
                        if e.file_name()
                            .map_or(false, |n| n.to_string_lossy().starts_with('.'))
                        {
                            return self.show_hidden;
                        }
                        if e.is_dir() || self.filter.is_none() {
                            return true;
                        }
                        match self.filter.as_ref().unwrap() {
                            FilePattern::Extension(ext) => e.extension().map_or(false, |e| {
                                e.to_ascii_lowercase() == OsString::from(ext.to_ascii_lowercase())
                            }),
                            FilePattern::Substring(substr) => e
                                .file_name()
                                .map_or(false, |n| n.to_string_lossy().contains(substr)),
                        }
                    })
                    .map(|file| {
                        let file_name = file.file_name();
                        if matches!(file.file_type(), Ok(t) if t.is_dir()) {
                            format!("{}/", file_name.to_string_lossy())
                        } else {
                            file_name.to_string_lossy().to_string()
                        }
                    }),
            )
            .collect();
        self.items.sort_by(|a, b| {
            if a == ".." {
                return cmp::Ordering::Less;
            }
            if b == ".." {
                return cmp::Ordering::Greater;
            }
            match (a.chars().last().unwrap(), b.chars().last().unwrap()) {
                ('/', '/') => a.cmp(b),
                ('/', _) => cmp::Ordering::Less,
                (_, '/') => cmp::Ordering::Greater,
                _ => a.cmp(b),
            }
        });
        self.list_state.select(None);
        self.next();
        Ok(())
    }

    /// Handle keyboard input while the file dialog is open.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered by the `select` method.
    pub fn handle_input(&mut self, event: Event) -> Result<Status> {
        if let Event::Key(key_event) = event {
            // Make sure for Windows
            if matches!(key_event.kind, KeyEventKind::Press) {
                debug_log!("key_event={key_event:#?}");
                debug_log!("self.focus={:#?}", self.focus);
                let key_code = key_event.code;
                match key_code {
                    KeyCode::Esc => return Ok(Status::Quit),
                    KeyCode::Char('l') if key_event.modifiers == KeyModifiers::CONTROL => {
                        self.popup = !self.popup;
                    }
                    _ => match self.focus {
                        DialogFocus::List => {
                            // Handle keys for navigating the file list
                            match key_code {
                                //     KeyCode::Char('u') => self.up()?,
                                //     KeyCode::Up | KeyCode::Char('k') => self.previous(),
                                //     KeyCode::Down | KeyCode::Char('j') => self.next(),
                                //     KeyCode::Enter => self.select()?,
                                KeyCode::Tab | KeyCode::BackTab => {
                                    debug_log!("Matched Tab / Backtab in {:#?} mode", self.focus);
                                    self.focus = DialogFocus::Input;
                                    let _ = execute!(std::io::stdout().lock(), Show,);
                                } // Switch to input area
                                //     KeyCode::Char('I') => self.toggle_show_hidden()?,
                                _ => self.file_explorer.handle(&event)?,
                            }
                        }
                        DialogFocus::Input => {
                            match key_code {
                                KeyCode::Enter => {
                                    self.select()?;
                                    return Ok(Status::Complete);
                                }
                                KeyCode::Tab | KeyCode::BackTab => {
                                    debug_log!("Matched tab in {:#?} mode", self.focus);
                                    self.focus = DialogFocus::List;
                                    let _ = execute!(std::io::stdout().lock(), Hide,);
                                } // Switch back to list
                                _ => {
                                    // Handle keys for the input area (filename)
                                    handle_save_input(&mut self.input, key_event);
                                }
                            }
                        }
                    },
                }
            }
        }
        Ok(Status::Incomplete)
    }
}

// fn adjust_mappings() -> &'static Vec<KeyDisplayLine> {
//     static ADJUSTED_MAPPINGS: OnceLock<Vec<KeyDisplayLine>> = OnceLock::new();
//     let remove: &[&str] = &[];
//     let add: &'static [KeyDisplayLine] = &[];
//     ADJUSTED_MAPPINGS.get_or_init(|| {
//         MAPPINGS
//             .iter()
//             .filter(|&row| !remove.contains(&row.keys))
//             .chain(add.iter())
//             .cloned()
//             .collect()
//     })
// }

fn get_max_lengths(adjusted_mappings: &[KeyDisplayLine]) -> (u16, u16) {
    static MAX_LENGTHS: OnceLock<(u16, u16)> = OnceLock::new();
    let (max_key_len, max_desc_len) = *MAX_LENGTHS.get_or_init(|| {
        adjusted_mappings
            .iter()
            .fold((0_u16, 0_u16), |(max_key, max_desc), row| {
                let key_len = row.keys.len().try_into().unwrap();
                let desc_len = row.desc.len().try_into().unwrap();
                (max_key.max(key_len), max_desc.max(desc_len))
            })
    });
    (max_key_len, max_desc_len)
}

/// Handle input in Save mode (for typing file name).
fn handle_save_input(text_area: &mut TextArea, key: KeyEvent) {
    // Convert the KeyEvent into an Input that TextArea can handle
    let input = Input::from(key);
    text_area.input(input);
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut save_dialog: FileDialog<'_> = FileDialog::new(60, 40, DialogMode::Save)?;
    save_dialog.open();
    let mut status = Status::Incomplete;
    while matches!(status, Status::Incomplete) && save_dialog.selected_file.is_none() {
        terminal.draw(|f| save_dialog.draw(f))?;
        let event = event::read()?;
        if let Event::Key(_) = event {
            status = save_dialog.handle_input(event)?;
        }
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

const MAPPINGS: &[KeyDisplayLine] = key_mappings![
    (10, "Key bindings", "Description"),
    (20, "q, Esc", "Close the file dialog"),
    (30, "j, ↓", "Move down in file list view"),
    (40, "k, ↑", "Move up in file list view"),
    (50, "Enter", "Choose the current selection"),
    (60, "u", "Go up to parent directory (list view)"),
    (70, "I", "Toggle showing hidden files"),
    (
        80,
        "Tab, BackTab",
        "Toggle between directory list and file name input"
    ),
    (90, "Ctrl+l", "Toggle keys display (this screen)"),
];
