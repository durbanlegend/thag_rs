/// Published example from `crossterm` crate.
///
/// The latest version of this example is available in the [examples] folder in the `crossterm`
/// repository. At time of writing you can run it successfully just
/// by invoking its URL with the `thag_url` tool, like this:
///
/// ```bash
/// thag_url https://github.com/crossterm-rs/crossterm/blob/master/examples/event-read-char-line.rs
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
/// Original `crossterm` crate comments:
///
/// Demonstrates how to block read characters or a full line.
/// Just note that crossterm is not required to do this and can be done with `io::stdin()`.
//# Purpose: Demo crossterm reading key events as a line or a single char.
//# Categories: crates
use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal,
};

pub fn read_char() -> io::Result<char> {
    loop {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            return Ok(c);
        }
    }
}

pub fn read_line() -> io::Result<String> {
    let mut line = String::new();
    loop {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            match code {
                KeyCode::Enter => {
                    break;
                }
                KeyCode::Char(c) => {
                    line.push(c);
                }
                _ => {}
            }
        }
    }

    Ok(line)
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;

    println!("read line:\r");
    println!("{:?}\r", read_line());
    println!("read char:\r");
    println!("{:?}\r", read_char());
    println!("read char again:\r");
    println!("{:?}\r", read_char());

    terminal::disable_raw_mode()
}
