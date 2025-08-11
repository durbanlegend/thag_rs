/// Original is `https://github.com/flip1995/tui-rs-file-dialog/blob/master/src/lib.rs`
/// Copyright (c) 2023 Philipp Krones
/// Licence: MIT
use crate::{
    key, key_mappings,
    tui_editor::{self, centered_rect, display_popup, KeyDisplayLine},
    KeyCombination,
};
use ratatui::crossterm::{
    cursor::{Hide, Show},
    event::{KeyCode, KeyEvent, KeyEventKind},
    execute,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
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
use thag_common::{debug_log, lazy_static_var};
use thag_profiler::profiled;
use thag_styling::Role;
use tui_textarea::{Input, TextArea};

/// File dialog mode to distinguish between Open and Save dialogs
#[derive(Debug, PartialEq, Eq)]
pub enum DialogMode {
    /// Open file dialog mode for selecting existing files
    Open,
    /// Save file dialog mode for specifying file names to save
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
    /// Focus on file list
    List,
    /// Focus on input area
    Input,
}

/// Status enum to represent the completion state of file dialog operations.
pub enum Status {
    /// Operation completed successfully
    Complete,
    /// Operation is still in progress
    Incomplete,
    /// User requested to quit the operation
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

    /// Whether to show the popup with key bindings
    pub popup: bool,
    title_bottom: &'a str,

    /// Input for the file name in Input mode
    pub input: TextArea<'a>,

    /// Buffer for search/filter functionality
    pub buf: String,
    // term_attrs: &'static TermAttributes,
}

// impl<FilePattern> FileDialog<'_, FilePattern> {
impl FileDialog<'_> {
    /// Create a new file dialog.
    ///
    /// The width and height are the size of the file dialog in percent of the terminal size. They
    /// are clamped to 100%.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered by the `update_entries` method.
    #[profiled]
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
            buf: String::new(),
            // term_attrs: TermAttributes::get_or_init(),
        };
        s.update_entries()?;
        Ok(s)
    }

    /// Sets the directory to open the file dialog in.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is a problem canonicalizing the directory.
    #[profiled]
    pub fn set_dir(&mut self, dir: &Path) -> Result<()> {
        self.current_dir = dir.canonicalize()?;
        self.update_entries()
    }

    /// Sets the filter to use when browsing files.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered by the `update_entries` method.
    #[profiled]
    pub fn set_filter(&mut self, filter: FilePattern) -> Result<()> {
        self.filter = Some(filter);
        self.update_entries()
    }

    /// Removes the filter.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered by the `update_entries` method.
    #[profiled]
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
    #[profiled]
    pub fn toggle_show_hidden(&mut self) -> Result<()> {
        self.show_hidden = !self.show_hidden;
        self.update_entries()
    }

    /// Opens the file dialog.
    #[profiled]
    pub fn open(&mut self) {
        self.selected_file.take();
        self.open = true;
    }

    /// Closes the file dialog.
    #[allow(clippy::missing_const_for_fn)]
    #[profiled]
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Returns whether the file dialog is currently open.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Draws the file dialog in the TUI application.
    #[profiled]
    pub fn draw(&mut self, f: &mut Frame) {
        if self.open {
            let area = centered_rect(self.width, self.height, f.area());

            // Split the area into two parts: one for the file list and one for the input field.
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints::<&[Constraint]>(
                    &[
                        Constraint::Length(3),            // Input field area
                        Constraint::Min(self.height - 3), // File list area
                    ], // .as_ref(),
                )
                .split(area);

            // Determine if the file list has focus
            let file_list_focus = matches!(self.focus, DialogFocus::List);

            let title = format!("{}", self.current_dir.display());
            let block = Block::default()
                .title(title.clone())
                .borders(Borders::ALL)
                .border_style(if file_list_focus {
                    ratatui::style::Style::from(&crate::Style::for_role(Role::Heading1))
                } else {
                    ratatui::style::Style::default()
                        .fg(ratatui::style::Color::DarkGray)
                        .dim()
                })
                .title_bottom(Line::from(self.title_bottom).centered());
            let list_items: Vec<ListItem> = self
                .items
                .iter()
                .map(|s| ListItem::new(s.as_str()))
                .collect();
            let list = List::new(list_items)
                .block(block)
                .highlight_style(
                    Style::default()
                        .fg(Color::Indexed(u8::from(&Role::EMPH)))
                        .bold(),
                )
                .style(if file_list_focus {
                    Style::default()
                        .fg(Color::Indexed(u8::from(&Role::HD2)))
                        .not_bold()
                } else {
                    Style::default().fg(Color::DarkGray).dim()
                });
            f.render_stateful_widget(list, chunks[1], &mut self.list_state);

            // Render the input box for the filename
            let input_focus = matches!(self.focus, DialogFocus::Input);
            if self.mode == DialogMode::Save {
                // Create a Block for the input area with borders and background
                let input_style = if input_focus {
                    Style::default()
                        .fg(Color::Indexed(u8::from(&Role::HD1)))
                        .bold()
                } else {
                    Style::default().fg(Color::DarkGray).dim()
                };
                let input_block = Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .style(input_style)
                    .border_style(input_style);

                let input_area = input_block.inner(chunks[0]); // Adjusts area to fit within borders
                f.render_widget(input_block, chunks[0]);

                // Conditionally show the cursor only if the input box has focus
                self.input
                    .set_cursor_line_style(Style::default().fg(Color::DarkGray));
                if input_focus {
                    self.input.set_style(Style::default());
                    self.input
                        .set_selection_style(Style::default().bg(Color::Blue));
                    self.input.set_cursor_style(Style::default()); //.on_magenta());
                    self.input.set_cursor_line_style(
                        Style::default()
                            .fg(Color::Indexed(u8::from(&Role::EMPH)))
                            .bold(),
                    );
                } else {
                    self.input.set_style(Style::default().dim());
                    self.input.set_selection_style(Style::default().hidden());
                    self.input.set_cursor_style(Style::default().hidden());
                    self.input.set_cursor_line_style(Style::default().hidden());
                }
                f.render_widget(&self.input, input_area); // Renders the input widget inside the block
            }

            if self.popup {
                let mappings: &[KeyDisplayLine] = match self.focus {
                    DialogFocus::List => LIST_MAPPINGS,
                    DialogFocus::Input => INPUT_MAPPINGS,
                };
                let (max_key_len, max_desc_len) = get_max_lengths(mappings);
                let title_bottom = tui_editor::TITLE_BOTTOM;
                display_popup(
                    mappings,
                    "Key bindings",
                    title_bottom,
                    max_key_len,
                    max_desc_len,
                    f,
                );
            }
        }
    }

    /// Goes to the next item in the file list.
    #[profiled]
    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => cmp::min(self.items.len() - 1, i + 1),
            None => cmp::min(self.items.len().saturating_sub(1), 1),
        };
        self.list_state.select(Some(i));
    }
    /// Goes to the previous item in the file list.
    #[profiled]
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
    #[profiled]
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
    #[profiled]
    pub fn select(&mut self) -> Result<()> {
        // Open mode logic (already correct)
        debug_log!("In select()");
        let Some(selected) = self.list_state.selected() else {
            self.next();
            debug_log!("Returning Ok(())");
            return Ok(());
        };

        // if matches!(self.mode, DialogMode::Save) {
        if self.focus == DialogFocus::Input {
            // Save mode logic to use the entered filename
            let file_name = self.input.lines().join(""); // Get the input from TextArea
            debug_log!("file_name={file_name}");
            if !file_name.is_empty() {
                let path = self.current_dir.join(file_name);
                self.selected_file = Some(path); // Set the selected file
                debug_log!("{:?}: selected_file={:?}", self.focus, self.selected_file);
                self.close(); // Close the dialog
            }
        } else {
            let path = if &self.items[selected] == ".." {
                self.current_dir.pop();
                self.current_dir.clone()
            } else {
                self.current_dir.join(&self.items[selected])
            };
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
            debug_log!(
                "Updated:current_dir={:?}; path={path:?}; is_file? {}; mode={:?}",
                self.current_dir,
                path.is_file(),
                self.mode
            );
            if path.is_file() {
                self.selected_file = Some(path);
                debug_log!("{:?}: selected_file={:?}", self.focus, self.selected_file);
                self.close();
                // return Ok(());
            }
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
    #[profiled]
    fn update_entries(&mut self) -> Result<()> {
        self.items = iter::once("..".to_string())
            .chain(
                fs::read_dir(&self.current_dir)?
                    .flatten()
                    .filter(|e| -> bool {
                        let e = e.path();
                        if e.file_name()
                            .is_some_and(|n| n.to_string_lossy().starts_with('.'))
                        {
                            return self.show_hidden;
                        }
                        if self.filter.is_none()
                            || matches!(self.filter, Some(FilePattern::Extension(_)))
                        {
                            return true;
                        }
                        match self.filter.as_ref().unwrap() {
                            FilePattern::Extension(ext) => e.extension().is_some_and(|e| {
                                e.to_ascii_lowercase() == OsString::from(ext.to_ascii_lowercase())
                            }),
                            FilePattern::Substring(substr) => e
                                .file_name()
                                .is_some_and(|n| n.to_string_lossy().contains(substr)),
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
    /// # Panics
    ///
    /// Panics if there is a logic error popping the last character out of the search buffer.
    #[profiled]
    pub fn handle_input(&mut self, key_event: KeyEvent) -> Result<Status> {
        // Make sure for Windows
        if matches!(key_event.kind, KeyEventKind::Press) {
            debug_log!("key_event={key_event:#?}");
            debug_log!("self.focus={:#?}", self.focus);
            let key_code = key_event.code;
            let key_combination = KeyCombination::from(key_event);
            match key_combination {
                key!(ctrl - l) => self.popup = !self.popup,
                key!(ctrl - q) | key!(Esc) => return Ok(Status::Quit),
                key!(Enter) => {
                    debug_log!("In handle_input: Enter pressed, about to call self.select()");
                    self.select()?;
                    self.buf.clear();
                    self.reset_filter()?;
                }
                _ => match self.focus {
                    DialogFocus::List => {
                        // Handle keys for navigating the file list
                        #[allow(clippy::unnested_or_patterns)]
                        match key_combination {
                            key!(tab) | key!(backtab) => {
                                debug_log!("Matched Tab / Backtab in {:#?} mode", self.focus);
                                self.focus = DialogFocus::Input;
                                let _ = execute!(std::io::stdout().lock(), Show,);
                            } // Switch to input area
                            key!(down) | key!(ctrl - j) => self.next(),
                            key!(up) | key!(ctrl - k) => self.previous(),
                            key!(ctrl - u) => {
                                self.buf.clear();
                                self.reset_filter()?;
                                self.up()?;
                            }
                            key!(ctrl - h) => self.toggle_show_hidden()?,
                            key!(backspace) => {
                                if !self.buf.is_empty() {
                                    self.buf.pop().expect("Logic error updating search buffer");
                                }
                                self.set_filter(FilePattern::Substring(self.buf.clone()))?;
                            }
                            _ => {
                                if let KeyCode::Char(c) = key_code {
                                    self.buf.push(c);
                                    debug_log!("self.buf={}", self.buf);
                                    self.set_filter(FilePattern::Substring(self.buf.clone()))?;
                                }
                            }
                        }
                    }
                    DialogFocus::Input => {
                        #[allow(clippy::unnested_or_patterns)]
                        match key_combination {
                            key!(tab) | key!(backtab) => {
                                debug_log!("Matched Tab / Backtab in {:#?} mode", self.focus);
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
        Ok(Status::Incomplete)
    }
}

#[profiled]
fn get_max_lengths(mappings: &[KeyDisplayLine]) -> (u16, u16) {
    lazy_static_var!(
        (u16, u16),
        deref,
        mappings
            .iter()
            .fold((0_u16, 0_u16), |(max_key, max_desc), row| {
                let key_len = row.keys.len().try_into().unwrap();
                let desc_len = row.desc.len().try_into().unwrap();
                (max_key.max(key_len), max_desc.max(desc_len))
            })
    )
}

/// Handle input in Save mode (for typing file name).
#[profiled]
fn handle_save_input(text_area: &mut TextArea, key: KeyEvent) {
    // Convert the KeyEvent into an Input that TextArea can handle
    let input = Input::from(key);
    text_area.input(input);
}

const LIST_MAPPINGS: &[KeyDisplayLine] = key_mappings![
    (10, "Key bindings", "Description"),
    (20, "Esc, Ctrl+q", "Close the file dialog"),
    (30, "↓, Ctrl+j", "Move down in file list view"),
    (40, "↑, Ctrl+k", "Move up in file list view"),
    (50, "Enter", "Choose the current selection"),
    (60, "Ctrl+u", "Go up to parent directory"),
    (70, "Ctrl+h", "Toggle showing hidden files"),
    (
        80,
        "Tab, BackTab",
        "Toggle between directory list and file name input"
    ),
    (90, "a-z, A-Z", "Append character to match string"),
    (100, "Backspace", "Remove last character from match string"),
    (110, "Ctrl+l", "Toggle keys display (this screen)"),
];

const INPUT_MAPPINGS: &[KeyDisplayLine] = key_mappings![
    (10, "Key bindings", "Description"),
    (20, "Esc, Ctrl+q", "Close the file dialog"),
    (30, "Enter", "Save file"),
    (
        40,
        "Tab, BackTab",
        "Toggle between directory list and file name input"
    ),
    (50, "Ctrl+l", "Toggle keys display (this screen)"),
];
