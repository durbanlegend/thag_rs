/*[toml]
[dependencies]
nu-ansi-term = { version = "0.50.0", features = ["derive_serde_style"] }
strum = { version = "0.26.2", features = ["derive", "strum_macros", "phf"] }
termbg = "0.6.0"
thag_rs = "0.1.6"
*/

#![allow(clippy::implicit_return)]
use nu_ansi_term::{Color, Style};
use std::time::{Duration, Instant};
use strum::IntoEnumIterator;
use termbg::terminal;
use thag_rs::colors::{coloring, ColorSupport, MessageStyle, XtermColor};
use thag_rs::logging::V;
use thag_rs::{cvprtln, vlog, Lvl};

let hash_map = Lvl::iter().map(|variant| (variant.to_string(), Style::from(&variant)))
    .collect::<std::collections::HashMap<String, Style>>();
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
