/*[toml]
[dependencies]
crossterm = "0.27.0"
*/

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, MouseButton, MouseEventKind},
    execute,
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::{self, stdout, Write};
use std::time::Duration;

fn main() -> Result<(), io::Error> {
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();

    // Clear the screen
    execute!(stdout, terminal::Clear(ClearType::All));

    // Write some text to the screen
    write!(
        stdout,
        "Click and drag to select text. Press 'q' to quit.\n\n\
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
        Integer varius fermentum sapien non volutpat. \
        Sed ultricies ipsum ut metus rhoncus, et finibus velit dictum."
    )?;
    stdout.flush()?;

    let mut selecting = false;
    let mut start_pos = (0, 0);

    loop {
        let event = event::read()?;
        println!("event={event:?}");
        match event {
            Event::Mouse(event) => {
                match event.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        selecting = true;
                        start_pos = (event.row, event.column);
                        stdout.execute(cursor::MoveTo(event.row, event.column))?;
                    }
                    MouseEventKind::Up(MouseButton::Left) => {
                        selecting = false;
                        let end_pos = (event.row, event.column);
                        // Handle the selected text here
                        println!("Selected text from {:?} to {:?}", start_pos, end_pos);
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        if selecting {
                            // Handle the dragging here
                            // For simplicity, we're just printing the current position
                            println!("Dragging to position: ({}, {})", event.row, event.column);
                        }
                    }

                    _ => {}
                }
            }
            // Event::Key(KeyEvent) where.code) if code = KeyCode::Char('q') => break,
            Event::Key(_) => break,
            _ => {}
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(_) = event::read()? {
                break;
            }
        }
    }

    // Disable raw mode before exiting
    terminal::disable_raw_mode()?;

    Ok(())
}
