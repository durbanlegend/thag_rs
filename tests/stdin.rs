use clap::CommandFactory;
#[cfg(feature = "simplelog")]
use log::info;
use mockall::Sequence;
use predicates::prelude::predicate;
use ratatui::crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    tty::IsTty,
};
use ratatui::style::{Color, Style};
use sequential_test::sequential;
#[cfg(feature = "simplelog")]
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::{
    env::set_var,
    fs,
    io::{stdout, Write},
    process::{Command, Stdio},
};
#[cfg(feature = "simplelog")]
use std::{fs::File, sync::OnceLock};
use thag_rs::colors::{get_term_theme, TuiSelectionBg};
use thag_rs::stdin::{edit, read_to_string};
use thag_rs::tui_editor::{apply_highlights, normalize_newlines, History};
use thag_rs::{vlog, MockEventReader, ThagResult, TMPDIR, V};
use tui_textarea::TextArea;

// Set environment variables before running tests
fn set_up() {
    init_logger();
    set_var("TEST_ENV", "1");
    #[cfg(windows)]
    {
        set_var("VISUAL", "powershell.exe /C Get-Content");
        set_var("EDITOR", "powershell.exe /C Get-Content");
    }
    #[cfg(not(windows))]
    {
        set_var("VISUAL", "cat");
        set_var("EDITOR", "cat");
    }
}

#[cfg(feature = "simplelog")]
static LOGGER: OnceLock<()> = OnceLock::new();

fn init_logger() {
    // Choose between simplelog and env_logger based on compile feature
    #[cfg(feature = "simplelog")]
    LOGGER.get_or_init(|| {
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
                File::create("app.log").unwrap(),
            ),
        ])
        .unwrap();
        info!("Initialized simplelog");
    });

    #[cfg(not(feature = "simplelog"))] // This will use env_logger if simplelog is not active
    {
        let _ = env_logger::builder().is_test(true).try_init();
    }
}

#[test]
fn test_stdin_edit_stdin_submit() {
    set_up();
    // Check if the test is running in a terminal
    if !stdout().is_tty() {
        println!("Skipping test_edit_stdin_submit as it is not running in a terminal.");
        return;
    }

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

    let result = edit(&mock_reader);

    vlog!(V::N, "\ntest_edit_stdin_submit result={result:#?}");
    assert!(result.is_ok());
    let lines = result.unwrap();
    // Expecting a Vec with one entry: an empty string
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "Hello,");
    assert_eq!(lines[1], "world!");
}

#[test]
fn test_stdin_edit_stdin_quit() {
    set_up();
    // Check if the test is running in a terminal
    if !stdout().is_tty() {
        println!("Skipping test_stdin_edit_stdin_quit as it is not running in a terminal.");
        return;
    }
    let mut mock_reader = MockEventReader::new();

    mock_reader.expect_read_event().return_once(|| {
        Ok(Event::Key(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::CONTROL,
        )))
    });

    let result = edit(&mock_reader);

    assert!(result.is_ok());
}

#[test]
fn test_stdin_history_new() {
    set_up();
    let history = History::new();
    assert!(history.entries.is_empty());
    assert!(history.current_index.is_none());
}

#[test]
fn test_stdin_history_get_current_empty() {
    set_up();
    let mut history = History::new();
    assert!(history.get_current().is_none());
}

#[test]
fn test_stdin_history_get_current() {
    set_up();
    let mut history = History::new();
    history.add_entry("first");
    history.add_entry("second");
    let current = history.get_current();
    assert!(current.is_some());
    assert_eq!(&current.unwrap().contents(), "second");
}

#[test]
fn test_stdin_history_get_previous_empty() {
    set_up();
    let mut history = History::new();
    assert!(history.get_previous().is_none());
}

#[test]
fn test_stdin_history_navigate() -> ThagResult<()> {
    set_up();
    let mut history = History::new();
    history.add_entry("first");
    history.add_entry("second");
    history.add_entry("third");
    history.add_entry("fourth");

    eprintln!("History={:#?}", history);

    let current = history.get_current();

    assert!(current.is_some());
    assert_eq!(&current.unwrap().contents(), "fourth");

    let current = history.get_previous();

    eprintln!("Expecting third, current={current:?}");

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

    eprintln!("History={history:#?}");

    let dir_path = &TMPDIR.join("thag_rs_tests");
    let path = dir_path.join("rs_stdin_hist.json");
    eprintln!("path={path:#?}");

    // Ensure REPL subdirectory exists
    fs::create_dir_all(&dir_path)?;

    // Create REPL file if necessary
    let _ = fs::File::create(&path)?;

    let _ = history.save_to_file(&path)?;

    let history = History::load_from_file(&path);
    eprintln!("History (reloaded)={:#?}", history);
    Ok(())
}

#[test]
fn test_stdin_history_get_next_empty() {
    set_up();
    let mut history = History::new();
    assert!(history.get_next().is_none());
}

// #[test]
// fn test_stdin_history_get_next() {
//     set_up();
//     let mut history = History::new();
//     history.add_entry("first");
//     history.add_entry("second");
//     eprintln!("History={:#?}", history);
//     history.get_previous(); // Move to the previous entry
//     let current = history.get_current();
//     assert!(current.is_some());
//     assert_eq!(&current.unwrap().contents(), "first");
// }

#[test]
fn test_stdin_repl_command_print_help() {
    set_up();
    let mut output = Vec::new();
    let mut command = thag_rs::repl::ReplCommand::command();
    command.write_long_help(&mut output).unwrap();
    let help_output = String::from_utf8(output).unwrap();
    assert!(help_output.contains("REPL mode lets you type or paste a Rust expression"));
}

#[test]
fn test_stdin_read_to_string() {
    set_up();
    let string = r#"fn main() {{ println!("Hello, world!"); }}\n"#;
    let mut input = string.as_bytes();
    let result = read_to_string(&mut input).unwrap();
    assert_eq!(result, string);
}

#[test]
#[sequential]
fn test_stdin_read_from_stdin() {
    set_up();
    // Trying an alternative to process::Command.
    // Spawn `cargo run -- -s` using assert_cmd
    let mut cmd = assert_cmd::Command::new("cargo");
    let cmd = cmd // Use assert_cmd to run `cargo`
        .arg("run")
        .arg("--")
        .arg("-qq")
        .arg("-s")
        .write_stdin(Vec::from("13 + 21\n"));
    // Assert the output
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("34"));
}

#[test]
#[sequential]
fn test_stdin_read_stdin() {
    set_up();
    init_logger();
    let string = "Hello, world!";
    let input = format!(
        r#"fn main() {{ println!("{string}"); }}
"#
    );
    println!("input={input}");

    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--features=debug-logs")
        .arg("--")
        .arg("-qq")
        .arg("-s")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("{}\n", string)
    );
}

#[test]
fn test_stdin_normalize_newlines() {
    set_up();
    let input = "Hello\r\nWorld\r!";
    let expected_output = "Hello\nWorld\n!";
    assert_eq!(normalize_newlines(input), expected_output);
}

#[test]
fn test_stdin_apply_highlights() {
    set_up();
    let mut textarea = TextArea::default();

    eprintln!("Theme={}", get_term_theme());

    apply_highlights(&TuiSelectionBg::BlueYellow, &mut textarea);
    assert_eq!(
        textarea.selection_style(),
        Style::default().fg(Color::Black).bg(Color::Cyan)
    );
    assert_eq!(
        textarea.cursor_style(),
        Style::default().fg(Color::Black).bg(Color::LightYellow)
    );
    assert_eq!(
        textarea.cursor_line_style(),
        Style::default().fg(Color::White).bg(Color::DarkGray)
    );

    apply_highlights(&TuiSelectionBg::RedWhite, &mut textarea);
    assert_eq!(
        textarea.selection_style(),
        Style::default().fg(Color::White).bg(Color::Blue)
    );
    assert_eq!(
        textarea.cursor_style(),
        Style::default().fg(Color::White).bg(Color::LightRed)
    );
    assert_eq!(
        textarea.cursor_line_style(),
        Style::default().fg(Color::Black).bg(Color::Gray)
    );
}
