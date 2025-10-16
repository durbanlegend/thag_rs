/*[toml]
[dependencies]
crossterm = "0.29"
ratatui = "0.29"
tui-file-dialog = "0.1.0"
*/
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use std::{cmp, ffi::OsString, fs, io::Result, iter, path::PathBuf};

/// A pattern that can be used to filter the displayed files.
pub enum FilePattern {
    /// Filter by file extension. This filter is case insensitive.
    Extension(String),
    /// Filter by substring. This filter is case sensitive.
    Substring(String),
}

/// The file dialog.
///
/// This manages the state of the file dialog. After selecting a file, the absolute path to that
/// file will be stored in the file dialog.
///
/// The file dialog is opened with the current working directory by default. To start the file
/// dialog with a different directory, use [`FileDialog::set_dir`].
pub struct FileDialog {
    /// The file that was selected when the file dialog was open the last time.
    ///
    /// This will reset when re-opening the file dialog.
    pub selected_file: Option<PathBuf>,

    width: u16,
    height: u16,

    filter: Option<FilePattern>,
    open: bool,
    current_dir: PathBuf,
    show_hidden: bool,

    list_state: ListState,
    items: Vec<String>,
}

impl FileDialog {
    /// Create a new file dialog.
    ///
    /// The width and height are the size of the file dialog in percent of the terminal size. They
    /// are clamped to 100%.
    pub fn new(width: u16, height: u16) -> Result<Self> {
        let mut s = Self {
            width: cmp::min(width, 100),
            height: cmp::min(height, 100),

            selected_file: None,

            filter: None,
            open: false,
            current_dir: PathBuf::from(".").canonicalize().unwrap(),
            show_hidden: false,

            list_state: ListState::default(),
            items: vec![],
        };

        s.update_entries()?;

        Ok(s)
    }

    /// The directory to open the file dialog in.
    pub fn set_dir(&mut self, dir: PathBuf) -> Result<()> {
        self.current_dir = dir.canonicalize()?;
        self.update_entries()
    }
    /// Sets the filter to use when browsing files.
    pub fn set_filter(&mut self, filter: FilePattern) -> Result<()> {
        self.filter = Some(filter);
        self.update_entries()
    }
    /// Removes the filter.
    pub fn reset_filter(&mut self) -> Result<()> {
        self.filter.take();
        self.update_entries()
    }
    /// Toggles whether hidden files should be shown.
    ///
    /// This only checks whether the file name starts with a dot.
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
    pub fn is_open(&self) -> bool {
        self.open
    }
    /// Draws the file dialog in the TUI application.
    pub fn draw(&mut self, f: &mut Frame) {
        if self.open {
            let block = Block::default()
                .title(format!("{}", self.current_dir.to_string_lossy()))
                .borders(Borders::ALL);
            let list_items: Vec<ListItem> = self
                .items
                .iter()
                .map(|s| ListItem::new(s.as_str()))
                .collect();

            let list = List::new(list_items).block(block).highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            );

            let area = centered_rect(self.width, self.height, f.size());
            f.render_stateful_widget(list, area, &mut self.list_state);
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
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }
    /// Moves one directory up.
    pub fn up(&mut self) -> Result<()> {
        self.current_dir.pop();
        self.update_entries()
    }

    /// Selects an item in the file list.
    ///
    /// If the item is a directory, the file dialog will move into that directory. If the item is a
    /// file, the file dialog will close and the path to the file will be stored in
    /// [`FileDialog::selected_file`].
    pub fn select(&mut self) -> Result<()> {
        let Some(selected) = self.list_state.selected() else {
            self.next();
            return Ok(());
        };

        let path = self.current_dir.join(&self.items[selected]);
        if path.is_file() {
            self.selected_file = Some(path);
            self.close();
            return Ok(());
        }

        self.current_dir = path;
        self.update_entries()
    }

    /// Updates the entries in the file list. This function is called automatically when necessary.
    fn update_entries(&mut self) -> Result<()> {
        self.items = iter::once("..".to_string())
            .chain(
                fs::read_dir(&self.current_dir)?
                    .flatten()
                    .filter(|e| {
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
}

#[macro_export]
macro_rules! bind_keys {
    ($file_dialog:expr, $e:expr) => {{
        if $file_dialog.is_open() {
            use ::crossterm::event::{self, Event, KeyCode};
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
                    _ => {}
                }
            }
        } else {
            $e
        }
    }};
}

/// Helper function to create a centered rectangle in the TUI app.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
