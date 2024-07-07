/*[toml]
[dependencies]
ratatui = "0.27.0"
scopeguard = "1.2.0"
tui-textarea = { version = "0.5.0", features = ["crossterm", "search"] }
*/

/// Demo a very minimal and not very useful TUI (text user interface) editor based on the featured crates.
//# Purpose: Demo TUI editor and featured crates, including `crossterm`, and the use of the `scopeguard` crate to reset the terminal when it goes out of scope.
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event::Paste,
};
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use std::error::Error;
use std::io;
use tui_textarea::{Input, Key, TextArea};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    ratatui::crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    let backend = CrosstermBackend::new(stdout);
    let term = Terminal::new(backend)?;

    // Ensure terminal will get reset when it goes out of scope.
    // This can't protect against catastrophic events like process::exit
    // where destructors don't run.
    let mut term = scopeguard::guard(term, |mut t| {
        reset_term(&mut t).expect("Error resetting terminal")
    });

    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::NONE)
            .title("Enter Rust script. Ctrl-D: end  Ctrl-C: quit"),
    );

    loop {
        term.draw(|f| {
            f.render_widget(textarea.widget(), f.size());
        })?;
        let event = event::read()?;
        if let Paste(data) = event {
            for line in data.lines() {
                textarea.insert_str(line);
                textarea.insert_newline();
            }
        } else {
            let input = Input::from(event.clone());
            match input {
                Input {
                    key: Key::Char('c'),
                    ctrl: true,
                    ..
                } => {
                    reset_term(&mut *term).expect("Error resetting terminal");
                    std::process::exit(0);
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
    disable_raw_mode()?;
    ratatui::crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    println!("Lines: {:?}", textarea.lines());
    Ok(())
}

fn reset_term(
    term: &mut Terminal<CrosstermBackend<io::StdoutLock<'_>>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    ratatui::crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}
