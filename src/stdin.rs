/*[toml]
[dependencies]
crossterm = "0.27.0"
lazy_static = "1.4.0"
ratatui = "0.26.3"
regex = "1.10.3"

tui-textarea = { version = "0.4.0", features = ["crossterm", "search"] }
*/

use crossterm::event::{
    DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event::Paste,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use lazy_static::lazy_static;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use regex::Regex;
use std::io;
use tui_textarea::{Input, Key, TextArea};

// use crate::code_utils;

#[allow(dead_code)]
fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::NONE)
            .title("Enter Rust script. Ctrl-D: submit"),
    );
    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
    textarea.set_selection_style(Style::default().bg(Color::LightCyan));
    // textarea.set_line_number_style(Style::default());
    textarea.set_cursor_style(Style::default().on_yellow());
    textarea.set_cursor_line_style(Style::default().on_light_yellow());

    loop {
        term.draw(|f| {
            f.render_widget(textarea.widget(), f.size());
        })?;
        let event = crossterm::event::read()?;
        if let Paste(data) = event {
            for line in data.lines() {
                textarea.insert_str(line);
                textarea.insert_newline();
            }
        } else {
            let input = Input::from(event.clone());
            match input {
                Input {
                    key: Key::Char('c' | 'd'),
                    ctrl: true,
                    ..
                } => break,
                input => {
                    textarea.input(input);
                }
            }
        }
    }
    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    textarea.lines().iter().for_each(|l| println!("{l}"));

    // println!("Lines: {:?}", re_disentangle(&x));
    Ok(())
}

#[allow(dead_code)]
pub(crate) fn re_disentangle(text_wall: &str) -> String {
    use std::fmt::Write;
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?m)(?P<line>.*?)(?:[\\]n|$)").unwrap();
    }

    // We extract the non-greedy capturing group named "line" from each capture of the multi-line mode regex..
    RE.captures_iter(text_wall)
        .map(|c| c.name("line").unwrap().as_str())
        .fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "{b}");
            output
        })
}
