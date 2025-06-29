/// Published example of KeyCombination from the `crokey` crate.
///
/// The latest version of this example is available in the [examples] folder
///  in the `crokey` repository. At time of writing you can run it successfully simply
/// by invoking its URL with the `thag_url` tool, like this:
///
/// ```bash
/// thag_url https://github.com/Canop/crokey/blob/main/examples/print_key_no_combiner/src/main.rs
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
//# Purpose: Demo key combination without Combiner.
//# Categories: crates, technique
use crokey::{
    crossterm::{
        event::{read, Event, KeyEventKind},
        style::Stylize,
        terminal,
    },
    {key, KeyCombinationFormat},
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
