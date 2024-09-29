/*[toml]
[dependencies]
clap = "4.5.18"
crossterm = "0.28.1"
mockall = "0.13.0"
thag_rs = { path = "/Users/donf/projects/thag_rs/" }
*/
use clap::Parser;
#[cfg(not(windows))]
use std::path::PathBuf;
use thag_rs::cmd_args::{Cli, ProcFlags};
use thag_rs::repl::{delete, disp_repl_banner, list, parse_line, run_expr};
#[cfg(not(windows))]
use thag_rs::repl::{edit, edit_history, toml, HISTORY_FILE};
use thag_rs::shared::BuildState;

// Set environment variables before running tests
// fn set_up() {
//     std::env::set_var("TEST_ENV", "1");
//     std::env::set_var("VISUAL", "cat");
//     std::env::set_var("EDITOR", "cat");
// }
use std::fs::read_to_string;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use mockall::Sequence;
use thag_rs::tui_editor::MockEventReader;

// set_up();
let build_state = thag_rs::BuildState {
    cargo_home: PathBuf::from("tests/assets/"),
    ..Default::default()
};

let mut seq = Sequence::new();
let mut mock_reader = MockEventReader::new();

mock_reader
    .expect_read_event()
    .times(1)
    .in_sequence(&mut seq)
    .return_once(|| Ok(Event::Paste("Hello,\nworld".to_string())));

mock_reader
    .expect_read_event()
    .times(1)
    .in_sequence(&mut seq)
    .return_once(|| {
        Ok(Event::Key(KeyEvent::new(
            KeyCode::Char('!'),
            KeyModifiers::NONE,
        )))
    });

mock_reader
    .expect_read_event()
    .times(1)
    .in_sequence(&mut seq)
    .return_once(|| {
        Ok(Event::Key(KeyEvent::new(
            KeyCode::Char('d'),
            KeyModifiers::CONTROL,
        )))
    });

let history_path = build_state.cargo_home.join(HISTORY_FILE);
let history_string =
    read_to_string(&history_path).expect(&format!("Error reading from {history_path:?}"));

let initial_content = history_string;
let staging_path: PathBuf = build_state.cargo_home.join("hist_staging.txt");
let result = edit_history(&initial_content, &staging_path, &mock_reader);
dbg!(&result);
assert!(&result.is_ok());
