//! [dependencies]
//! crossterm = "0.27.0"
//! ratatui = "0.26.1"
//! tui-textarea = { version = "0.4.0", features = ["crossterm", "search"] }

use crossterm::event::read;
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event::Paste,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::borrow::Cow;
use std::env;
use std::fmt::Display;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use tui_textarea::{CursorMove, Input, Key, TextArea};

macro_rules! error {
    ($fmt: expr $(, $args:tt)*) => {{
        Err(io::Error::new(io::ErrorKind::Other, format!($fmt $(, $args)*)))
    }};
}

#[allow(dead_code)]
struct SearchBox<'a> {
    textarea: TextArea<'a>,
    open: bool,
}

impl<'a> Default for SearchBox<'a> {
    fn default() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));
        Self {
            textarea,
            open: false,
        }
    }
}

#[allow(dead_code)]
impl<'a> SearchBox<'a> {
    fn open(&mut self) {
        self.open = true;
    }

    fn close(&mut self) {
        self.open = false;
        // Remove input for next search. Do not recreate `self.textarea` instance to keep undo history so that users can
        // restore previous input easily.
        self.textarea.move_cursor(CursorMove::End);
        self.textarea.delete_line_by_head();
    }

    fn height(&self) -> u16 {
        if self.open {
            3
        } else {
            0
        }
    }

    fn input(&mut self, input: Input) -> Option<&'_ str> {
        match input {
            Input {
                key: Key::Enter, ..
            }
            | Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            } => None, // Disable shortcuts which inserts a newline. See `single_line` example
            input => {
                let modified = self.textarea.input(input);
                modified.then(|| self.textarea.lines()[0].as_str())
            }
        }
    }

    fn set_error(&mut self, err: Option<impl Display>) {
        let b = if let Some(err) = err {
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Search: {}", err))
                .style(Style::default().fg(Color::Red))
        } else {
            Block::default().borders(Borders::ALL).title("Search")
        };
        self.textarea.set_block(b);
    }
}

#[allow(dead_code)]
struct Buffer<'a> {
    textarea: TextArea<'a>,
    path: PathBuf,
    modified: bool,
}

#[allow(dead_code)]
impl<'a> Buffer<'a> {
    fn new(path: PathBuf) -> io::Result<Self> {
        let mut textarea = if let Ok(md) = path.metadata() {
            if md.is_file() {
                let mut textarea: TextArea = io::BufReader::new(fs::File::open(&path)?)
                    .lines()
                    .collect::<io::Result<_>>()?;
                if textarea.lines().iter().any(|l| l.starts_with('\t')) {
                    textarea.set_hard_tab_indent(true);
                }
                textarea
            } else {
                return error!("{:?} is not a file", path);
            }
        } else {
            TextArea::default() // File does not exist
        };
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
        textarea.set_selection_style(Style::default().bg(Color::LightCyan));
        textarea.set_line_number_style(Style::default());
        textarea.set_cursor_style(Style::default().on_yellow());
        textarea.set_cursor_line_style(Style::default().on_light_yellow());
        textarea.set_block(Block::default().borders(Borders::TOP).title("Editor"));

        Ok(Self {
            textarea,
            path,
            modified: false,
        })
    }

    fn save(&mut self) -> io::Result<()> {
        let mut f = io::BufWriter::new(fs::File::create(&self.path)?);
        for line in self.textarea.lines() {
            f.write_all(line.as_bytes())?;
            f.write_all(b"\n")?;
        }
        self.modified = false;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct Output<'a> {
    textarea: TextArea<'a>,
    modified: bool,
}

impl<'a> Output<'a> {
    fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_style(Style::default().fg(Color::DarkGray));
        textarea.set_cursor_style(Style::default().add_modifier(Modifier::HIDDEN));
        // Disable cursor line style
        textarea.set_cursor_line_style(Style::default());
        textarea.set_block(Block::default().borders(Borders::TOP).title("Output"));

        Self {
            textarea,
            modified: true, // For initial display
        }
    }
}

#[allow(dead_code)]
struct Editor<'a> {
    current: usize,
    buffers: Vec<Buffer<'a>>,
    term: Terminal<CrosstermBackend<io::Stdout>>,
    message: Option<Cow<'static, str>>,
    search: SearchBox<'a>,
    output: Output<'a>,
}

#[allow(dead_code)]
impl<'a> Editor<'a> {
    fn new<I>(paths: I) -> io::Result<Self>
    where
        I: Iterator,
        I::Item: Into<PathBuf>,
    {
        let buffers = paths
            .map(|p| Buffer::new(p.into()))
            .collect::<io::Result<Vec<_>>>()?;
        if buffers.is_empty() {
            return error!("USAGE: cargo run --example editor FILE1 [FILE2...]");
        }
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        crossterm::execute!(
            stdout,
            EnterAlternateScreen,
            EnableBracketedPaste,
            EnableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend)?;

        Ok(Self {
            current: 0,
            buffers,
            term,
            message: None,
            search: SearchBox::default(),
            output: Output::new(),
        })
    }

    #[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
    fn run(&mut self) -> io::Result<()> {
        loop {
            let search_height = self.search.height();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(search_height),
                        Constraint::Length(1),
                        Constraint::Min(1),
                        Constraint::Length(1),
                        Constraint::Percentage(25),
                    ]
                    .as_ref(),
                );

            self.term.draw(|f| {
                let chunks = layout.split(f.size());

                if search_height > 0 {
                    f.render_widget(self.search.textarea.widget(), chunks[0]);
                }

                let buffer = &self.buffers[self.current];
                let textarea = &buffer.textarea;
                let widget = textarea.widget();
                f.render_widget(widget, chunks[2]);

                // Render status line
                let modified = if buffer.modified { " [modified]" } else { "" };
                let slot = format!("[{}/{}]", self.current + 1, self.buffers.len());
                let path = format!(" {}{} ", buffer.path.display(), modified);
                let (row, col) = textarea.cursor();
                let cursor = format!("({},{})", row + 1, col + 1);
                let status_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Length(slot.len() as u16),
                            Constraint::Min(1),
                            Constraint::Length(cursor.len() as u16),
                        ]
                        .as_ref(),
                    )
                    .split(chunks[1]);
                let status_style = Style::default()
                    // .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Blue);
                f.render_widget(Paragraph::new(slot).style(status_style), status_chunks[0]);
                f.render_widget(Paragraph::new(path).style(status_style), status_chunks[1]);
                f.render_widget(Paragraph::new(cursor).style(status_style), status_chunks[2]);

                // Render message at bottom of editor
                let message = if let Some(message) = self.message.take() {
                    Line::from(Span::raw(message))
                } else if search_height > 0 {
                    Line::from(vec![
                        Span::raw("Press "),
                        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to jump to first match and close, "),
                        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to close, "),
                        Span::styled(
                            "^G or ↓ or ^N",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" to search next, "),
                        Span::styled(
                            "M-G or ↑ or ^P",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" to search previous"),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw("Press "),
                        Span::styled("^Q", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to quit, "),
                        Span::styled("^S", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to save, "),
                        Span::styled("^G", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to search, "),
                        Span::styled("^T", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to switch buffer"),
                    ])
                };
                f.render_widget(Paragraph::new(message), chunks[3]);

                // Render output belowx editor
                let textarea = &self.output.textarea;
                let widget = textarea.widget();
                f.render_widget(widget, chunks[4]);
                self.output.modified = false;
            })?;

            if search_height > 0 {
                let textarea = &mut self.buffers[self.current].textarea;
                match read()?.into() {
                    Input {
                        key: Key::Char('g' | 'n'),
                        ctrl: true,
                        alt: false,
                        ..
                    }
                    | Input { key: Key::Down, .. } => {
                        if !textarea.search_forward(false) {
                            self.search.set_error(Some("Pattern not found"));
                        }
                    }
                    Input {
                        key: Key::Char('g'),
                        ctrl: false,
                        alt: true,
                        ..
                    }
                    | Input {
                        key: Key::Char('p'),
                        ctrl: true,
                        alt: false,
                        ..
                    }
                    | Input { key: Key::Up, .. } => {
                        if !textarea.search_back(false) {
                            self.search.set_error(Some("Pattern not found"));
                        }
                    }
                    Input {
                        key: Key::Enter, ..
                    } => {
                        if !textarea.search_forward(true) {
                            self.message = Some("Pattern not found".into());
                        }
                        self.search.close();
                        textarea.set_search_pattern("").unwrap();
                    }
                    Input { key: Key::Esc, .. } => {
                        self.search.close();
                        textarea.set_search_pattern("").unwrap();
                    }
                    input => {
                        if let Some(query) = self.search.input(input) {
                            let maybe_err = textarea.set_search_pattern(query).err();
                            self.search.set_error(maybe_err);
                        }
                    }
                }
            } else {
                let event = read()?;

                if let Paste(data) = event {
                    self.output.textarea.insert_str("Pasting data");
                    self.output.textarea.insert_newline();
                    self.output.modified = true;

                    let buffer = &mut self.buffers[self.current];
                    for line in data.lines() {
                        buffer.textarea.insert_str(line);
                        buffer.textarea.insert_newline();
                        buffer.modified = true;
                    }
                } else {
                    let input = Input::from(event.clone());
                    // if input.ctrl {
                    //     println!("input={input:?}");
                    // }

                    match input {
                        Input {
                            key: Key::Char('q'),
                            ctrl: true,
                            ..
                        } => break,
                        Input {
                            key: Key::Char('t'),
                            ctrl: true,
                            ..
                        } => {
                            self.current = (self.current + 1) % self.buffers.len();
                            let msg: Cow<'static, str> =
                                format!("Switched to buffer #{}", self.current + 1).into();
                            self.message = Some(msg.clone());
                            self.write_output(&msg);
                        }
                        Input {
                            key: Key::Char('s'),
                            ctrl: true,
                            ..
                        } => {
                            let msg = if self.buffers[self.current].modified {
                                self.buffers[self.current].save()?;
                                "Saved!"
                            } else {
                                "No changes to save"
                            };
                            self.message = Some(msg.into());
                            self.write_output(msg);
                        }
                        Input {
                            key: Key::Char('g'),
                            ctrl: true,
                            ..
                        } => {
                            self.search.open();
                        }
                        input => {
                            let buffer = &mut self.buffers[self.current];
                            let just_modified = buffer.textarea.input(input);
                            buffer.modified = buffer.modified || just_modified;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn write_output(&mut self, msg: &str) {
        self.output.textarea.insert_str(msg);
        self.output.textarea.insert_newline();
        self.output.modified = true;
    }
}

impl<'a> Drop for Editor<'a> {
    fn drop(&mut self) {
        self.term.show_cursor().unwrap();
        disable_raw_mode().unwrap();
        crossterm::execute!(
            self.term.backend_mut(),
            LeaveAlternateScreen,
            DisableBracketedPaste,
            DisableMouseCapture
        )
        .unwrap();
    }
}

#[allow(dead_code)]
fn main() -> io::Result<()> {
    Editor::new(env::args_os().skip(1))?.run()
}
