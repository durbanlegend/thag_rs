/*[toml]
[dependencies]
crossterm = { version = "0.27.0", features = ["use-dev-tty"] }
ratatui = "0.26.3"
tokio = { version = "1", features = ["full"] }
tui-textarea = { version = "0.4.0", features = ["crossterm", "search"] }
*/

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::io::{self as tokio_io, AsyncReadExt};
use tui_textarea::{Input, Key, TextArea};

enum Event<I> {
    Input(I),
    Tick,
}

async fn read_from_stdin() -> io::Result<Option<String>> {
    let mut buffer = String::new();
    let mut stdin = tokio_io::stdin();
    tokio::select! {
        result = stdin.read_to_string(&mut buffer) => {
            match result {
                Ok(_) if buffer.is_empty() => Ok(None),
                Ok(_) => Ok(Some(buffer)),
                Err(e) => Err(e),
            }
        },
        _ = tokio::time::sleep(Duration::from_millis(50)) => Ok(None),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Read from stdin asynchronously
    let initial_content = read_from_stdin().await?;

    // Step 2: Setup Crossterm terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Flush any pending events
    while event::poll(Duration::from_millis(0))? {
        event::read()?;
    }

    // Create a channel to communicate between the input handling thread and the main thread
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(250);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // Poll for tick rate duration, if no events, send tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

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

            f.render_widget(textarea.widget(), chunks[0]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('d')
                    if event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    break
                }
                _ => {
                    let input = Input::from(event);
                    textarea.input(input);
                }
            },
            Event::Tick => {}
        }
    }

    // Step 5: Restore the terminal to its previous state
    terminal::disable_raw_mode()?;
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
