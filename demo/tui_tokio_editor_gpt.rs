/*[toml]
[dependencies]
ratatui = "0.27.0"
tokio = { version = "1", features = ["full"] }
tui-textarea = { version = "0.5.1", features = ["crossterm", "search"] }
*/

/// GPT-provided demo of a very basic TUI (terminal user interface) editor using
/// `tokio` and the `crossterm` / `ratatui` / `tui-textarea` stack. provides a blank editor
/// screen on which you can capture lines of data. `Ctrl-D` closes the editor and simply
/// prints the captured data.
//# Purpose: Exploring options for editing input. e.g. for a REPL.
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io::{self, IsTerminal};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::time::timeout;
use tui_textarea::TextArea;

async fn read_from_stdin() -> io::Result<Option<String>> {
    let mut buffer = String::new();
    let mut stdin = tokio::io::stdin();
    match timeout(Duration::from_millis(100), async {
        stdin.read_to_string(&mut buffer).await
    })
    .await
    {
        Ok(result) => result.map(|_| Some(buffer)), // Read successful, return the result wrapped in Some
        Err(_) => Ok(None),                         // Timeout reached, return None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::io::stdin();

    // let initial_content = read_from_stdin().await?;
    let initial_content = if input.is_terminal() {
        // No input available
        Some(String::new())
    } else {
        read_from_stdin().await?
    };

    // Step 1: Read from stdin

    // Introduce a small delay to ensure stdin reading is complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Step 2: Set up Crossterm terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Step 3: Set up the TUI editor with the initial content
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

            f.render_widget(&textarea, chunks[0]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                let input = tui_textarea::Input::from(key_event.clone());
                match input {
                    tui_textarea::Input {
                        key: tui_textarea::Key::Char('d'),
                        ctrl: true,
                        ..
                    } => break,
                    input => {
                        textarea.input(input);
                    }
                }
            }
        }
    }

    // Step 5: Restore the terminal to its previous state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Output the final content
    let final_content = textarea.lines().to_vec();
    println!("Received input:\n{:#?}", final_content);

    Ok(())
}
