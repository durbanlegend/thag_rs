/*
# [dependencies]
# crokey = "1.1.0"
# crossterm = "0.28.1"
# serde = { version = "1.0.130", features = ["derive"] }
*/
/*[toml]
[dependencies]
crokey = { version = "1.1.0", features = ["default"] }
crossterm = { version = "0.28.1", features = ["bracketed-paste", "default", "use-dev-tty"] }
*/

/// Interactively test dependency inferency. This script was arbitrarily copied from
/// demo/crokey_print_key.rs.
//# Purpose: Test thag manifest module's dependency inference.
//# Categories: crates, technique, testing
use {
    crokey::*,
    crossterm::{
        event::{read, Event},
        style::Stylize,
        terminal,
    },
};

pub fn main() {
    let fmt = KeyCombinationFormat::default();
    let mut combiner = Combiner::default();
    let combines = combiner.enable_combining().unwrap();
    if combines {
        println!("Your terminal supports combining keys");
    } else {
        println!("Your terminal doesn't support combining standard (non modifier) keys");
    }
    println!("Type any key combination (remember that your terminal intercepts many ones)");
    loop {
        terminal::enable_raw_mode().unwrap();
        let e = read();
        terminal::disable_raw_mode().unwrap();
        match e {
            Ok(Event::Key(key_event)) => {
                let Some(key_combination) = combiner.transform(key_event) else {
                    continue;
                };
                let key = fmt.to_string(key_combination);
                println!("Detected {key}");
                match key_combination {
                    key!(ctrl - c) => {
                        println!("Arg! You savagely killed me with a {}", key.red());
                        break;
                    }
                    key!(ctrl - q) | key!(ctrl - q - q - q) => {
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
