#[cfg(feature = "simplelog")]
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::{env::set_var, fs, sync::Once};
use thag_proc_macros::safe_eprintln;
use thag_rs::tui_editor::{normalize_newlines, History};
use thag_rs::{ThagResult, TMPDIR};

#[cfg(feature = "simplelog")]
use thag_rs::debug_log;

// Set environment variables before running tests
fn set_up() {
    static INIT: Once = Once::new();
    #[cfg(windows)]
    {
        INIT.call_once(|| unsafe {
            set_var("TEST_ENV", "1");
            set_var("VISUAL", "powershell.exe /C Get-Content");
            set_var("EDITOR", "powershell.exe /C Get-Content");
            init_logger();
        });
    }
    #[cfg(not(windows))]
    {
        INIT.call_once(|| unsafe {
            set_var("TEST_ENV", "1");
            set_var("VISUAL", "cat");
            set_var("EDITOR", "cat");
            init_logger();
        });
    }
}

#[cfg(feature = "simplelog")]
use std::{fs::File, sync::OnceLock};

#[cfg(feature = "simplelog")]
static LOGGER: OnceLock<()> = OnceLock::new();

fn init_logger() {
    // Choose between simplelog and env_logger based on compile feature
    #[cfg(feature = "simplelog")]
    LOGGER.get_or_init(|| {
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Debug,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                LevelFilter::Debug,
                Config::default(),
                File::create("app.log").unwrap(),
            ),
        ])
        .unwrap();
        debug_log!("Initialized simplelog");
    });

    #[cfg(not(feature = "simplelog"))] // This will use env_logger if simplelog is not active
    {
        let _ = env_logger::builder().is_test(true).try_init();
    }
}

#[test]
fn test_tui_editor_normalize_newlines() {
    set_up();
    let input = "Hello\r\nWorld\r!";
    let expected_output = "Hello\nWorld\n!";
    assert_eq!(normalize_newlines(input), expected_output);
}

#[test]
fn test_tui_editor_history_new() {
    set_up();
    let history = History::new();
    assert!(history.entries.is_empty());
    assert!(history.current_index.is_none());
}

#[test]
fn test_tui_editor_history_get_current_empty() {
    set_up();
    let mut history = History::new();
    assert!(history.get_current().is_none());
}

#[test]
fn test_tui_editor_history_get_current() {
    set_up();
    let mut history = History::new();
    history.add_entry("first");
    history.add_entry("second");
    let current = history.get_current();
    assert!(current.is_some());
    assert_eq!(&current.unwrap().contents(), "second");
}

#[test]
fn test_tui_editor_history_get_previous_empty() {
    set_up();
    let mut history = History::new();
    assert!(history.get_previous().is_none());
}

#[test]
fn test_tui_editor_history_navigate() -> ThagResult<()> {
    set_up();
    let mut history = History::new();
    history.add_entry("first");
    history.add_entry("second");
    history.add_entry("third");
    history.add_entry("fourth");

    safe_eprintln!("History={:#?}", history);

    let current = history.get_current();

    assert!(current.is_some());
    assert_eq!(&current.unwrap().contents(), "fourth");

    let current = history.get_previous();

    safe_eprintln!("Expecting third, current={current:?}");

    assert!(current.is_some());
    assert_eq!(&current.unwrap().contents(), "third");

    let current = history.get_previous();
    assert!(current.is_some());
    assert_eq!(&current.unwrap().contents(), "second");

    let current = history.get_previous();
    assert!(current.is_some());
    assert_eq!(&current.unwrap().contents(), "first");

    let current = history.get_previous();
    assert_eq!(&current.unwrap().contents(), "first");

    let current = history.get_next();
    assert!(current.is_some());
    assert_eq!(&current.unwrap().contents(), "second");

    let current = history.get_next();
    assert!(current.is_some());
    assert_eq!(&current.unwrap().contents(), "third");

    let current = history.get_next();
    assert!(current.is_some());
    assert_eq!(&current.unwrap().contents(), "fourth");

    let current = history.get_next();
    assert_eq!(&current.unwrap().contents(), "fourth");

    safe_eprintln!("History={history:#?}");

    let dir_path = &TMPDIR.join("thag_rs_tests");
    let path = dir_path.join("rs_stdin_hist.json");
    safe_eprintln!("path={path:#?}");

    // Ensure REPL subdirectory exists
    fs::create_dir_all(&dir_path)?;

    // Create REPL file if necessary
    let _ = fs::File::create(&path)?;

    let _ = history.save_to_file(&path)?;

    let history = History::load_from_file(&path);
    safe_eprintln!("History (reloaded)={:#?}", history);
    Ok(())
}

#[test]
fn test_tui_editor_history_get_next_empty() {
    set_up();
    let mut history = History::new();
    assert!(history.get_next().is_none());
}

// #[test]
// fn test_tui_editor_history_get_next() {
//     set_up();
//     let mut history = History::new();
//     history.add_entry("first");
//     history.add_entry("second");
//     safe_eprintln!("History={:#?}", history);
//     history.get_previous(); // Move to the previous entry
//     let current = history.get_current();
//     assert!(current.is_some());
//     assert_eq!(&current.unwrap().contents(), "first");
// }
