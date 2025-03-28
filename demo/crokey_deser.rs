/*[toml]
[dependencies]
crokey = "1.0.1"
serde = { version = "1.0.215", features = ["derive"] }
toml = "0.8.19"
*/

/// Published example of serde deserialisation from `crokey` crate.
//# Purpose: Demo loading keybindings from a file.
//# Categories: crates, technique
use {
    crokey::{
        crossterm::{
            event::{read, Event},
            style::Stylize,
            terminal,
        },
        key, KeyCombination, KeyCombinationFormat,
    },
    serde::Deserialize,
    std::collections::HashMap,
    toml,
};

// This is an example of a configuration structure which contains a map from KeyEvent to String.
#[derive(Deserialize)]
struct Config {
    keybindings: HashMap<KeyCombination, String>,
}

// An example of what could be a configuration file
static CONFIG_TOML: &str = r#"
[keybindings]
a = "aardvark"
shift-b = "babirussa"
ctrl-k = "koala"
alt-j = "jaguar"
h = "hexapode"
shift-h = "HEXAPODE"
- = "mandrill"
alt-- = "nasalis" # some terminals don't distinguish between - and alt--
alt-up = "alt-up (native)"
f3 = "toml"
"#;
// esc-[-shift-a
// \[-shift-a
// esc-\[-shift-a

pub fn main() {
    print!("Application configuration:\n{}", CONFIG_TOML.blue());
    let config: Config = toml::from_str(CONFIG_TOML).unwrap();
    let fmt = KeyCombinationFormat::default();
    println!("\nType any key combination");
    loop {
        terminal::enable_raw_mode().unwrap();
        let e = read();
        terminal::disable_raw_mode().unwrap();
        if let Ok(Event::Key(key_event)) = e {
            let key = KeyCombination::from(key_event);
            if key == key!(ctrl - c) || key == key!(ctrl - q) {
                println!("bye!");
                break;
            }
            if let Some(word) = config.keybindings.get(&key) {
                println!(
                    "You hit {} which is mapped to {}",
                    fmt.to_string(key).green(),
                    word.clone().yellow(),
                );
            } else {
                println!("You hit {} which isn't mapped", fmt.to_string(key).red(),);
            }
        }
    }
}
