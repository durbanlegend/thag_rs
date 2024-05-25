/*[toml]
[dependencies]
crossterm = { version = "0.27.0", features = ["use-dev-tty"] }
ratatui = "0.26.3"
tui-textarea = { version = "0.4.0", features = ["crossterm", "search"] }
*/

use crossterm::event::{
    DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event::Paste,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
// use log::debug;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use std::error::Error;
use std::io;
use tui_textarea::{CursorMove, Input, Key, TextArea};

use crate::code_utils;
use crate::errors::BuildRunError;

// use crate::code_utils;

#[allow(dead_code)]
fn main() -> Result<(), Box<dyn Error>> {
    for line in &read_stdin()? {
        println!("{line}");
    }
    Ok(())
}

pub(crate) fn read_stdin() -> Result<Vec<String>, Box<dyn Error>> {
    let initial_content = code_utils::read_stdin()?;

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
    let mut textarea = TextArea::from(initial_content.lines());

    textarea.set_block(
        Block::default()
            .borders(Borders::NONE)
            .title("Enter / paste / edit Rust script. Ctrl-D: submit")
            .title_style(Style::default().italic()),
    );
    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
    textarea.set_selection_style(Style::default().bg(Color::LightCyan));
    textarea.set_cursor_style(Style::default().on_yellow());
    textarea.set_cursor_line_style(Style::default().on_light_yellow());

    // for line in buffer.lines() {
    //     insert_line(&mut textarea, line);
    // }
    // textarea.insert_str(&initial_content);

    textarea.move_cursor(CursorMove::Bottom);

    loop {
        term.draw(|f| {
            f.render_widget(textarea.widget(), f.size());
        })?;
        let event = crossterm::event::read()?;
        if let Paste(data) = event {
            for line in data.lines() {
                insert_line(&mut textarea, line);
            }
        } else {
            let input = Input::from(event.clone());
            match input {
                Input {
                    key: Key::Char('c'),
                    ctrl: true,
                    ..
                } => {
                    reset_term(term)?;
                    return Err(Box::new(BuildRunError::Cancelled));
                }
                Input {
                    key: Key::Char('d'),
                    ctrl: true,
                    ..
                } => break,
                input => {
                    textarea.input(input);
                }
            }
        }
    }
    reset_term(term)?;

    Ok(textarea.lines().to_vec())
}

fn insert_line(textarea: &mut TextArea, line: &str) {
    textarea.insert_str(line);
    if cfg!(windows) {
        textarea.insert_str("\r");
    }
    textarea.insert_newline();
}

fn reset_term(
    mut term: Terminal<CrosstermBackend<io::StdoutLock<'_>>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}
