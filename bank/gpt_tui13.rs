/*[toml]
[dependencies]
#crossterm = { version = "0.27.0", features = ["use-dev-tty", "event-stream"] }
libc = "0.2.80"
ratatui = "0.27.0"
tui-textarea = { version = "0.4.0", features = ["crossterm", "search"] }
*/

use libc;
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute, terminal,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io::{self, stdin, Read};
use tui_textarea::TextArea;
use tui_textarea::{Input, Key};

// Function to read all input from stdin
fn read_from_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

// Function to reset stdin state
fn reset_stdin() {
    #[cfg(unix)]
    {
        use libc;
        use std::os::unix::io::AsRawFd;
        unsafe {
            let filename = "/dev/tty\0".as_ptr() as *const i8;
            let mode = "r\0".as_ptr() as *const i8;
            let file = libc::fdopen(libc::STDIN_FILENO, mode);
            libc::freopen(filename, mode, file);
        }
    }
    #[cfg(windows)]
    {
        use libc;
        unsafe {
            let filename = "CON\0".as_ptr() as *const i8;
            let mode = "r\0".as_ptr() as *const i8;
            let file = libc::fdopen(0, mode);
            libc::freopen(filename, mode, file);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Read from stdin if there's any input
    let initial_content = read_from_stdin().unwrap_or_else(|_| "".to_string());

    // Step 2: Reset stdin state to ensure no leftover input remains
    reset_stdin();

    // Step 3: Setup Crossterm terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Step 4: Setup the TUI editor with the initial content
    let mut textarea = TextArea::default();
    textarea.insert_str(&initial_content);

    // Step 5: Main loop for TUI
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

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

    // Step 6: Restore the terminal to its previous state
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
