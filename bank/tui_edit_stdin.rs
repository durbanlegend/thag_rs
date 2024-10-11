/*[toml]
[package]
name = "tui_edit_stdin"
features = ["simplelog"]

[dependencies]
crossterm = "0.28.1"
log = "0.4.22"
ratatui = "0.28.1"
simplelog = { version = "0.12.2" }
#env_logger = { version = "0.11.5", optional = true }
thag_rs = { path = "/Users/donf/projects/thag_rs" }
tui-textarea = "0.6.1"

[features]
debug-logs = []
nightly = []
default = ["simplelog"]
simplelog = []
*/

use log::info;
use ratatui::style::{Color, Modifier, Style};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fmt::Debug;
use std::fs::File;
use std::io::IsTerminal;
use std::path::PathBuf;
use thag_rs::logging::V;
use thag_rs::shared::KeyDisplayLine;
use thag_rs::stdin;
use thag_rs::tui_editor::{
    script_key_handler, tui_edit, CrosstermEventReader, EditData, EventReader, History, KeyAction,
    KeyDisplay,
};
use thag_rs::{debug_log, log, ThagError, ThagResult};

fn main() -> ThagResult<()> {
    // configure_log();

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("/Users/donf/projects/thag_rs/app.log").unwrap(),
        ),
    ])
    .unwrap();
    info!("Initialized simplelog");

    let event_reader = CrosstermEventReader;
    for line in &edit(&event_reader)? {
        log!(V::N, "{line}");
    }
    Ok(())
}

pub fn edit<R: EventReader + Debug>(event_reader: &R) -> ThagResult<Vec<String>> {
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let history_path = PathBuf::from(cargo_home).join("bank_tui_rs_stdin_history.json");
    let mut history = History::load_from_file(&history_path);

    let input = std::io::stdin();

    #[cfg(debug_assertions)]
    debug_log!("input.is_terminal()? {}", input.is_terminal());
    let initial_content = if input.is_terminal() {
        String::new()
    } else {
        crate::stdin::read()?
    };

    if !initial_content.trim().is_empty() {
        history.add_entry(&initial_content);
        history.save_to_file(&history_path)?;
    }

    let mut edit_data = EditData {
        return_text: true,
        initial_content: &initial_content,
        save_path: None,
        history_path: Some(&history_path),
        history: Some(history),
    };
    let add_keys = [
        KeyDisplayLine::new(361, "Ctrl+Alt+s", "Save a copy"),
        KeyDisplayLine::new(371, "F3", "Discard saved and unsaved changes, and exit"),
    ];
    let display = KeyDisplay {
        title: "Enter / paste / edit Rust script.  ^D: submit  ^Q: quit  ^L: keys  ^T: toggle highlighting",
        title_style: Style::from((Color::Yellow, Modifier::BOLD)),
        remove_keys: &[""; 0],
        add_keys: &add_keys,
    };
    let (key_action, maybe_text) = tui_edit(
        event_reader,
        &mut edit_data,
        &display,
        |key_event, maybe_term, /*maybe_save_file,*/ textarea, edit_data, popup, saved| {
            script_key_handler(
                key_event, maybe_term, // maybe_save_file,
                textarea, edit_data, popup, saved,
            )
        },
    )?;
    match key_action {
        KeyAction::Quit(_saved) => Err(ThagError::Cancelled),
        KeyAction::Save
        | KeyAction::ShowHelp
        | KeyAction::ToggleHighlight
        | KeyAction::TogglePopup => Err(ThagError::FromStr(
            format!("Logic error: {key_action:?} should not return from tui_edit").into(),
        )),
        // KeyAction::SaveAndExit => false,
        KeyAction::Submit => {
            std::fs::File::open(&history_path)?.sync_all()?;
            return maybe_text.map_or(Err(ThagError::Cancelled), |v| Ok(v));
        }
        _ => Err(ThagError::FromStr(
            format!("Logic error: {key_action:?} should not return from tui_edit").into(),
        )),
    }
}
