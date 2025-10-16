#[cfg(feature = "simplelog")]
use log::info;
use mockall::Sequence;
use predicates::prelude::predicate;
use ratatui::crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    tty::IsTty,
};
use serial_test::serial;
use std::{
    env::set_var,
    io::{stdout, Write},
    process::{Command, Stdio},
    sync::Once,
};
use thag_rs::stdin::{edit, read_to_string};
use thag_rs::{vprtln, MockEventReader, V};

#[cfg(feature = "simplelog")]
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
#[cfg(feature = "simplelog")]
use std::{fs::File, sync::OnceLock};

// Set environment variables before running tests
fn set_up() {
    static INIT: Once = Once::new();
    init_logger();
    #[cfg(windows)]
    {
        INIT.call_once(|| unsafe {
            set_var("TEST_ENV", "1");
            set_var("VISUAL", "powershell.exe /C Get-Content");
            set_var("EDITOR", "powershell.exe /C Get-Content");
        });
    }
    #[cfg(not(windows))]
    {
        INIT.call_once(|| unsafe {
            set_var("TEST_ENV", "1");
            set_var("VISUAL", "cat");
            set_var("EDITOR", "cat");
        });
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

    vprtln!(V::N, "\ntest_edit_stdin_submit result={result:#?}");
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
fn test_stdin_read_to_string() {
    set_up();
    let string = r#"fn main() {{ println!("Hello, world!"); }}\n"#;
    let mut input = string.as_bytes();
    let result = read_to_string(&mut input).unwrap();
    assert_eq!(result, string);
}

#[test]
#[serial]
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
#[serial]
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
        // .arg("--features=debug_logging")
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
