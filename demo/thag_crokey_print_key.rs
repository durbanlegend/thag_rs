/*[toml]
[dependencies]
crokey = "1.1.0"
crossterm = "0.28.0"
serde = { version = "1.0.130", features = ["derive"] }
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop" }
toml = "0.5"
*/

/// Published example of KeyCombination from `crokey` crate, modified to use
/// basic `crokey` key combos embedded in `thag_rs` under MIT licence.
//# Purpose: Test for stability and consistency across different platforms and terminals.
use {
    crokey::KeyCombinationFormat,
    crossterm::{
        event::{read, Event},
        style::Stylize,
        terminal,
    },
    thag_rs::{key, KeyCombination},
};

pub fn main() {
    let fmt = KeyCombinationFormat::default();
    println!("Type any key combination (remember that your terminal intercepts many ones)");
    loop {
        terminal::enable_raw_mode().unwrap();
        let e = read();
        terminal::disable_raw_mode().unwrap();
        match e {
            Ok(Event::Key(key_event)) => {
                let key_combination = key_event.into();
                let key = fmt.to_string(key_combination);
                match key_combination {
                    key!(ctrl - c) => {
                        println!("Arg! You savagely killed me with a {}", key.red());
                        break;
                    }
                    key!(ctrl - q) => {
                        println!("You typed {} which gracefully quits", key.green());
                        break;
                    }
                    key!('?') | key!(shift - '?') => {
                        println!("{}", "There's no help on this app".red());
                    }
                    _ => {
                        println!("You typed {}", key.blue());
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
