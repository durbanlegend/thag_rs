/// Original is `https://github.com/flip1995/tui-rs-file-dialog/blob/master/src/lib.rs`
/// Copyright (c) 2023 Philipp Krones
/// Licence: MIT
use crossterm::{
    cursor::{Hide, Show},
    event::{KeyCode, KeyEvent, KeyModifiers},
    execute,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use std::{
    cmp,
    ffi::OsString,
    fs,
    io::Result,
    iter,
    path::{Path, PathBuf},
};
use tui_textarea::{Input, TextArea};

use crate::tui_editor::show_popup;
use crate::{shared::KeyDisplayLine, tui_editor};

/// File dialog mode to distinguish between Open and Save dialogs
#[derive(PartialEq, Eq)]
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

    /// Current mode of the dialog (Open or Save)
    mode: DialogMode,

    /// Current focus of the Save dialog (List or Input)
    focus: DialogFocus,

    pub popup: bool,
    title_bottom: &'a str,

    /// Input for the file name in Save mode
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
            mode,
            focus: DialogFocus::List,
            popup: false, // Start with focus on the file list
            input: TextArea::default(),
            title_bottom: "Ctrl+l to show keys",
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
                        Constraint::Min(self.height - 3), // File list area
                        Constraint::Length(3),            // Input field area
                    ]
                    .as_ref(),
                )
                .split(area);

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
            f.render_stateful_widget(list, chunks[0], &mut self.list_state);

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

                let input_area = input_block.inner(chunks[1]); // Adjusts area to fit within borders
                f.render_widget(input_block, chunks[1]);

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
                show_popup(
                    &get_mappings(),
                    f,
                    "Key bindings",
                    tui_editor::TITLE_BOTTOM,
                    &[],
                    &[],
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
        let Some(selected) = self.list_state.selected() else {
            self.next();
            return Ok(());
        };

        let path = self.current_dir.join(&self.items[selected]);

        if path.is_dir() {
            self.current_dir.clone_from(&path);
            self.update_entries()?;
        }
        // if matches!(self.mode, DialogMode::Save) {
        if self.mode == DialogMode::Save {
            // Save mode logic to use the entered filename
            let file_name = self.input.lines().join(""); // Get the input from TextArea
                                                         // eprintln!("file_name={file_name}");
            if !file_name.is_empty() {
                let path = self.current_dir.join(file_name);
                self.selected_file = Some(path); // Set the selected file
                self.close(); // Close the dialog
            }
        } else if path.is_file() {
            self.selected_file = Some(path);
            self.close();
            return Ok(());
        }
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
    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Status> {
        let key_code = key.code;
        match key_code {
            KeyCode::Esc => return Ok(Status::Quit),
            KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                self.popup = !self.popup;
            }
            _ => match self.focus {
                DialogFocus::List => {
                    // Handle keys for navigating the file list
                    match key_code {
                        KeyCode::Char('u') => self.up()?,
                        KeyCode::Up | KeyCode::Char('k') => self.previous(),
                        KeyCode::Down | KeyCode::Char('j') => self.next(),
                        KeyCode::Enter => self.select()?,
                        KeyCode::Tab => {
                            self.focus = DialogFocus::Input;
                            let _ = execute!(std::io::stdout().lock(), Show,);
                        } // Switch to input area
                        _ => {}
                    }
                }
                DialogFocus::Input => {
                    match key_code {
                        KeyCode::Enter => {
                            self.select()?;
                            return Ok(Status::Complete);
                        }
                        KeyCode::Tab => {
                            self.focus = DialogFocus::List;
                            let _ = execute!(std::io::stdout().lock(), Hide,);
                        } // Switch back to list
                        _ => {
                            // Handle keys for the input area (filename)
                            handle_save_input(&mut self.input, key);
                        }
                    }
                }
            },
        }
        Ok(Status::Incomplete)
    }
}

/// Handle input in Save mode (for typing file name).
fn handle_save_input(text_area: &mut TextArea, key: KeyEvent) {
    // Convert the KeyEvent into an Input that TextArea can handle
    let input = Input::from(key);
    text_area.input(input);
}

/// Macro to automatically overwrite the default key bindings of the app, when the file dialog is
/// open.
///
/// This macro only works inside of a function that returns a [`std::io::Result`] or a result that
/// has an error type that implements [`From<std::io::Error>`].
///
/// Default bindings:
///
/// | Key         | Action                       |
/// |-------------|------------------------------|
/// | `q`, `Esc`  | Close the file dialog.       |
/// | `j`, `Down` | Move down in the file list.  |
/// | `k`, `Up`   | Move up in the file list.    |
/// | `Enter`     | Select the current item.     |
/// | `u`         | Move one directory up.       |
/// | `I`         | Toggle showing hidden files. |
///
/// ## Example
///
/// ```ignore
/// bind_keys!(
///     // Expression to use to access the file dialog.
///     app.file_dialog,
///     // Event handler of the app, when the file dialog is closed.
///     if let Event::Key(key) = event::read()? {
///         match key.code {
///             KeyCode::Char('q') => {
///                 return Ok(());
///             }
///             _ => {}
///         }
///     }
/// )
/// ```
#[macro_export]
macro_rules! bind_keys {
    ($file_dialog:expr, $e:expr) => {{
        if $file_dialog.is_open() {
            use crossterm::event::{self, Event, KeyCode};
            // File dialog events
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        $file_dialog.close();
                    }
                    KeyCode::Char('I') => $file_dialog.toggle_show_hidden()?,
                    KeyCode::Enter => {
                        $file_dialog.select()?;
                    }
                    KeyCode::Char('u') => {
                        $file_dialog.up()?;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        $file_dialog.previous();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        $file_dialog.next();
                    }
                    KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                        $file_dialog.popup = !$file_dialog.popup
                    }
                    _ => {}
                }
            }
        } else {
            $e
        }
    }};
}

/// Helper function to create a centered rectangle in the TUI app.
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

fn get_mappings() -> Vec<KeyDisplayLine> {
    vec![
        KeyDisplayLine::new(10, "Keys", "Description"),
        KeyDisplayLine::new(20, "q, Esc", "Close the file dialog"),
        KeyDisplayLine::new(30, "j, ↓", "Move down in the file list"),
        KeyDisplayLine::new(40, "k, ↑", "Move up in the file list"),
        KeyDisplayLine::new(50, "Enter", "Select the current item"),
        KeyDisplayLine::new(60, "u", "Move one directory up"),
        KeyDisplayLine::new(70, "I", "Toggle showing hidden files"),
        KeyDisplayLine::new(80, "Tab", "Toggle between file list and file name"),
    ]
}
