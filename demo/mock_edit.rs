/*[toml]
[dependencies]
crossterm = "0.28"
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["tui", "simplelog"] }
*/

/// Used to debug a doctest.
//# Purpose: Debugging script.
//# Categories: crates, technique, testing
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use mockall::{automock, predicate::str};
use std::time::Duration;
use thag_rs::{stdin::edit, EventReader, MockEventReader, ThagResult, ThagError};

pub struct CrosstermEventReader;

impl EventReader for CrosstermEventReader {
    fn read_event(&self) -> ThagResult<Event> {
         Ok(crossterm::event::read()?)
    }

    fn poll(&self, timeout: Duration) -> ThagResult<bool> {
            crossterm::event::poll(timeout).map_err(Into::<ThagError>::into)
    }
}

let mut event_reader = MockEventReader::new();
event_reader.expect_read_event().return_once(|| {
    Ok(Event::Key(KeyEvent::new(
        KeyCode::Char('d'),
        KeyModifiers::CONTROL,
    )))
});
let actual = edit(&event_reader).unwrap();
let buf = vec![""];
eprintln!("actual=[{}]", actual.join("\n"));
assert!(matches!(&actual, buf));
eprintln!("Fin");
