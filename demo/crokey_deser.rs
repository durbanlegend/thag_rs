/// Published example of serde deserialisation from the `crokey` crate.
///
/// The latest version of this example is available in the [examples] folder
///  in the `crokey` repository. At time of writing you can run it successfully simply
/// by invoking its URL with the `thag_url` tool, like this:
///
/// ```bash
/// thag_url https://github.com/Canop/crokey/blob/main/examples/deser_keybindings/src/main.rs
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
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
