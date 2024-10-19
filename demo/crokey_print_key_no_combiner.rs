/*[toml]
[dependencies]
crokey = "1.1.0"
crossterm = "0.28.0"
serde = { version = "1.0.130", features = ["derive"] }
toml = "0.5"
*/

/// Published example of KeyCombination from `crokey` crate.
//# Purpose: Demo key combination without Combiner.
use {
    crokey::*,
    crossterm::{
        event::{read, Event, KeyEventKind},
        style::Stylize,
        terminal,
    },
};

pub fn main() {
    let fmt = KeyCombinationFormat::default();
    terminal::enable_raw_mode().unwrap();
    println!("Type any key combination (remember that your terminal intercepts many ones)");
    loop {
        let e = read();
        // terminal::disable_raw_mode().unwrap();
        match e {
            Ok(Event::Key(key_event)) => {
                if key_event.kind == KeyEventKind::Release {
                    continue;
                }
                println!("event={e:?}");
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
                    // key!('?') | key!(shift - '?') => {
                    //     println!("{}", "There's no help on this app".red());
                    // }
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
    terminal::disable_raw_mode().unwrap();
}
