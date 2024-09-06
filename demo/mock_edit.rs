/*[toml]
[dependencies]
crossterm = "0.28"
mockall = "0.13.0"
thag_rs = "0.1.3"
*/

/// Used to debug a doctest.
//# Purpose: Debugging script.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::Event;
use mockall::{automock, predicate::str};
use thag_rs::stdin::{edit, EventReader, MockEventReader};

pub struct CrosstermEventReader;

impl EventReader for CrosstermEventReader {
    fn read_event(&self) -> Result<crossterm::event::Event, std::io::Error> {
        crossterm::event::read()
    }
}

let mut event_reader = MockEventReader::new();
event_reader.expect_read_event().return_once(|| {
    Ok(Event::Key(KeyEvent::new(
        KeyCode::Char('d'),
        KeyModifiers::CONTROL,
    )))
});
let actual = edit(&event_reader);
let buf = vec![""];
eprintln!("actual=[{actual:#?}]");
assert!(matches!(actual, Ok(buf)));
