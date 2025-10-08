/// Published example from `reedline` crate.
///
/// The latest version of this example is available in the [examples] folder in the `reedline`
/// repository. At time of writing you can run it successfully just
/// by invoking its URL with the `thag_url` tool, like this:
///
/// ```bash
/// thag_url https://github.com/nushell/reedline/blob/main/examples/event_listener.rs
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
//# Purpose: demo featured crates.
//# Categories: crates, repl, technique
use {
    crossterm::{
        event::{poll, Event, KeyCode, KeyEvent},
        terminal,
    },
    std::{
        io::{stdout, Write},
        time::Duration,
    },
};

fn main() -> std::io::Result<()> {
    println!("Ready to print events (Abort with ESC):");
    print_events()?;
    println!();
    Ok(())
}

// **For debugging purposes only:** Track the terminal events observed by [`Reedline`] and print them.
//# Categories: crates, repl, technique
pub fn print_events() -> std::io::Result<()> {
    stdout().flush()?;
    terminal::enable_raw_mode()?;
    let result = print_events_helper();
    terminal::disable_raw_mode()?;

    result
}

// this fn is totally ripped off from crossterm's examples
// it's really a diagnostic routine to see if crossterm is
// even seeing the events. if you press a key and no events
// are printed, it's a good chance your terminal is eating
// those events.
fn print_events_helper() -> std::io::Result<()> {
    loop {
        // Wait up to 5s for another event
        if poll(Duration::from_millis(5_000))? {
            // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
            let event = crossterm::event::read()?;

            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                state,
            }) = event
            {
                match code {
                    KeyCode::Char(c) => {
                        println!(
                            "Char: {} code: {:#08x}; Modifier {:?}; Flags {:#08b}; Kind {kind:?}; state {state:?}\r",
                            c,
                            u32::from(c),
                            modifiers,
                            modifiers
                        );
                    }
                    _ => {
                        println!(
                            "Keycode: {code:?}; Modifier {modifiers:?}; Flags {modifiers:#08b}; Kind {kind:?}; state {state:?}\r"
                        );
                    }
                }
            } else {
                println!("Event::{event:?}\r");
            }

            // hit the esc key to git out
            if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }
        } else {
            // Timeout expired, no event for 5s
            println!("Waiting for you to type...\r");
        }
    }

    Ok(())
}
