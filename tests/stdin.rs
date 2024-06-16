use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use ratatui::style::{Color, Style, Stylize};
use rs_script::stdin::MockEventReader;
use rs_script::{
    edit_stdin,
    stdin::{apply_highlights, normalize_newlines, read_to_string},
};
use tui_textarea::TextArea;

#[test]
fn test_edit_stdin() {
    let mut mock_reader = MockEventReader::new();

    mock_reader.expect_read_event().return_once(|| {
        Ok(Event::Key(KeyEvent::new(
            KeyCode::Char('d'),
            KeyModifiers::CONTROL,
        )))
    });

    let result = edit_stdin(mock_reader);

    assert!(result.is_ok());
    let lines = result.unwrap();
    println!("\nlines={lines:#?}");
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].len(), 0);
}

fn init_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn test_read_to_string() {
    let string = r#"fn main() {{ println!("Hello, world!"); }}\n"#;
    let input = string.as_bytes();
    let mut input = &input[..];
    let result = read_to_string(&mut input).unwrap();
    assert_eq!(result, string);
}

use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_read_stdin() {
    init_logger();
    let string = "Hello, world!";
    let input = format!(
        r#"fn main() {{ println!("{string}"); }}
"#
    );
    println!("input={input}");

    let mut child = Command::new("cargo")
        .arg("run")
        // .arg("--bin")
        // .arg("rs_script")
        .arg("--")
        // .arg("--features=debug-logs")
        .arg("-q")
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
fn test_normalize_newlines() {
    let input = "Hello\r\nWorld\r!";
    let expected_output = "Hello\nWorld\n!";
    assert_eq!(normalize_newlines(input), expected_output);
}

#[test]
fn test_apply_highlights() {
    let mut textarea = TextArea::default();

    apply_highlights(true, &mut textarea);
    assert_eq!(
        textarea.selection_style(),
        Style::default().bg(Color::LightRed)
    );
    assert_eq!(textarea.cursor_style(), Style::default().on_yellow());
    assert_eq!(
        textarea.cursor_line_style(),
        Style::default().on_light_yellow()
    );

    apply_highlights(false, &mut textarea);
    assert_eq!(
        textarea.selection_style(),
        Style::default().bg(Color::Green)
    );
    assert_eq!(textarea.cursor_style(), Style::default().on_magenta());
    assert_eq!(
        textarea.cursor_line_style(),
        Style::default().on_dark_gray()
    );
}

// #[test]
// fn test_edit_stdin_quit() {
//     let stdin_mock = Arc::new(Mutex::new(vec![KeyEvent::new(
//         KeyCode::Char('q'),
//         KeyModifiers::CONTROL,
//     )]));

//     // Mock crossterm's event reading function
//     crossterm::event::mock::with(mock_input(stdin_mock.clone()));

//     let result = edit_stdin();
//     assert!(result.is_err());
//     assert!(matches!(
//         result.err().unwrap().downcast_ref::<BuildRunError>(),
//         Some(&BuildRunError::Cancelled)
//     ));
// }
// #[test]
// fn test_edit_stdin_submit() {
//     let stdin_mock = Arc::new(Mutex::new(vec![KeyEvent::new(
//         KeyCode::Char('d'),
//         KeyModifiers::CONTROL,
//     )]));

//     // Mock crossterm's event reading function
//     crossterm::event::mock::with(mock_input(stdin_mock.clone()));

//     let result = edit_stdin();
//     assert!(result.is_ok());
//     assert_eq!(result.unwrap(), vec![]);
// }

// fn mock_input(
//     stdin_mock: Arc<Mutex<Vec<KeyEvent>>>,
// ) -> impl Fn() -> Option<crossterm::event::Event> {
//     let stdin_mock = stdin_mock.clone();
//     move || {
//         let mut stdin = stdin_mock.lock().unwrap();
//         if stdin.is_empty() {
//             None
//         } else {
//             Some(crossterm::event::Event::Key(stdin.remove(0)))
//         }
//     }
// }
