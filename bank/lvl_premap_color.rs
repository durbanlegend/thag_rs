/*[toml]
[dependencies]
nu-ansi-term = { version = "0.50.0", features = ["derive_serde_style"] }
strum = { version = "0.26.3", features = ["derive", "strum_macros", "phf"] }
termbg = "0.6.0"
thag_rs = "0.1.9"
*/

#![allow(clippy::implicit_return)]
use nu_ansi_term::{Color, Style};
use std::time::{Duration, Instant};
use strum::IntoEnumIterator;
use termbg::terminal;
use thag_rs::styling::{ColorSupport, TermBgLuma};
use thag_rs::logging::V;
use thag_rs::{cvprtln, vlog, Lvl};

// The colors module was removed, so we'll create a simple style mapping instead
let hash_map = Lvl::iter().map(|variant| {
    let style = match variant {
        Lvl::ERROR => Style::new().fg(Color::Red).bold(),
        Lvl::WARNING => Style::new().fg(Color::Yellow),
        Lvl::EMPHASIS => Style::new().fg(Color::Cyan).bold(),
        Lvl::HEADING => Style::new().fg(Color::Blue).bold(),
        Lvl::SUBHEADING => Style::new().fg(Color::Blue),
        Lvl::BRIGHT => Style::new().fg(Color::White).bold(),
        Lvl::NORMAL => Style::new().fg(Color::White),
        Lvl::DEBUG => Style::new().fg(Color::Green),
        Lvl::GHOST => Style::new().fg(Color::Fixed(8)),
    };
    (variant.to_string(), style)
}).collect::<std::collections::HashMap<String, Style>>();
// println!("hash_map={hash_map:#?}");

let style_keys = Lvl::iter().map(|variant| variant.to_string())
    .collect::<Vec<String>>();
println!("style_keys={style_keys:#?}");

let loops = 100000;

let start = Instant::now();
for i in 1..=loops {
    for variant in Lvl::iter() {
        let style = Style::from(&variant);
    }
}
let dur = start.elapsed();
let msg = format!("Completed {loops} sets of enum translations in {}.{}s", dur.as_secs(), dur.subsec_millis());
println!("{msg}");

let start = Instant::now();
for i in 1..=loops {
    for key in &style_keys {
        let style = hash_map.get(key);
    }
}
let dur = start.elapsed();
let msg = format!("Completed {loops} sets of hash_table lookups in {}.{}s", dur.as_secs(), dur.subsec_millis());
println!("{msg}");
