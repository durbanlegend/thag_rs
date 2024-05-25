/*[toml]
[dependencies]
tokio = { version = "1", features = ["full"] }
crossterm = { version = "0.27.0", features = ["use-dev-tty"] }
ratatui = "0.26.3"
tui-textarea = { version = "0.4.0", features = ["crossterm", "search"] }
*/

use std::io;
use tokio::io::AsyncReadExt;
use tokio::time::{timeout, Duration};
use tui_textarea::{Input, Key};

async fn read_from_stdin() -> io::Result<Option<String>> {
    let mut buffer = String::new();
    match timeout(
        Duration::from_millis(100),
        tokio::io::stdin().read_to_string(&mut buffer),
    )
    .await
    {
        Ok(result) => match result {
            Ok(_) => Ok(Some(buffer)), // Read successful, return the buffer
            Err(e) => Err(e),          // Read error, return the error
        },
        Err(_) => Ok(None), // Timeout, return None
    }
}

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute, terminal,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use tui_textarea::TextArea;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Read from stdin with a timeout
    let initial_content = read_from_stdin().await?;

    // Step 2: Setup Crossterm terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Step 3: Setup the TUI editor with the initial content
    let mut textarea = TextArea::default();
    if let Some(content) = initial_content {
        textarea.insert_str(&content);
    }

    // Step 4: Main loop for TUI
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            // textarea.render(f, chunks[0]);
            f.render_widget(textarea.widget(), chunks[0]);
        })?;

        if let Event::Key(key_event) = event::read()? {
            let input = Input::from(key_event.clone());
            match input {
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

    // Step 5: Restore the terminal to its previous state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Output the final content
    let final_content = textarea.lines().to_vec();
    println!("Received input:\n{:#?}", final_content);

    Ok(())
}
