/*[toml]
[dependencies]
crossterm = "0.28.0"
serde = { version = "1.0.130", features = ["derive"] }
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop" }
toml = "0.5"
*/

/// Published example of KeyCombination from `crokey` crate, modified to use
/// basic `crokey` key combos embedded in `thag_rs` under MIT licence.
//# Purpose: Test for stability and consistency across different platforms and terminals.
use {
    crossterm::{
        event::{read, Event, KeyEventKind},
        terminal,
    },
    thag_rs::key,
};

pub fn main() {
    println!("Type any key combination (remember that your terminal intercepts many ones)");
    loop {
        terminal::enable_raw_mode().unwrap();
        let e = read();
        terminal::disable_raw_mode().unwrap();
        match e {
            Ok(Event::Key(key_event)) => {
                if !matches!(key_event.kind, KeyEventKind::Press) {
                    continue;
                }
                let key_combination = key_event.into();
                match key_combination {
                    key!(ctrl - c) => {
                        println!("Arg! You savagely killed me with a {key_combination:?}");
                        break;
                    }
                    key!(ctrl - q) => {
                        // println!("You typed {key_combination:?} which gracefully quits", key.green());
                        break;
                    }
                    _ => {
                        println!("You typed {key_combination:?}");
                    }
                }
            }
            e => {
                // any other event, for example a resize, we quit
                eprintln!("Quitting on {:?}", e);
                break;
            }
        }
    }
}
