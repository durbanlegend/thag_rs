/*[toml]
[package]
name = "termbg_bug"
features = ["simplelog"]

[dependencies]
crossterm = "0.29"
log = "0.4.22"
simplelog = { version = "0.12.2" }
#termbg = "=0.5.2"
termbg = "0.6.0"
#termbg = { path = "/Users/donforbes/Documents/GitHub/termbg" }

[features]
debug-logs = []
nightly = []
default = ["simplelog"]
simplelog = []
*/

use log::info;
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;
use std::io::{self, Read};
use std::time::Duration;
use termbg;

fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("app.log").unwrap(),
        ),
    ])
    .unwrap();
    info!("Initialized simplelog");

    let timeout = Duration::from_millis(100);

    // let term = termbg::terminal();
    let _rgb = termbg::rgb(timeout);
    // let theme = termbg::theme(timeout);

    println!("Type in something and see if first character gets swallowed in Windows Terminal");
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("buffer={buffer:?}");
}
