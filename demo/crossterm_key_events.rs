//! [dependencies]
//! crossterm = "0.27.0"

use crossterm::event::{read, Event, KeyCode, KeyEventKind};
use std::io::{stdout, Write};

fn main() {
    // Enable raw mode for reading key events
    let mut stdout = stdout();
    crossterm::terminal::disable_raw_mode().unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();

    loop {
        // Read the next event
        match read() {
            Ok(Event::Key(key_event)) => {
                // Check for key press events (some terminals might fire release events)
                if key_event.kind == KeyEventKind::Press {
                    print!(
                        "Key: {:?}, Modifiers: {:?}\n",
                        key_event.code, key_event.modifiers
                    );
                    stdout.flush().unwrap();
                }
            }
            // Ok(Event::Resize(_)) => {} // Ignore resize events
            _ => {} // Handle other events if needed
        }
    }
}
