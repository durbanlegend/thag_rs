/*[toml]
[package]
name = "termbg_latency_win"
features = ["simplelog"]

[dependencies]
crossterm = "0.29"
log = "0.4.22"
simplelog = { version = "0.12.2" }
#termbg = "0.5.2"
termbg = { path = "/Users/donforbes/Documents/GitHub/termbg" }

[features]
debug_logging = []
nightly = []
default = ["simplelog"]
simplelog = []
*/

use log::info;
/// Test changes to latency function.
///
//# Purpose: Test for possible PR to `termbg` crate.
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;
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
            File::create("latency.log").unwrap(),
        ),
    ])
    .unwrap();
    info!("Initialized simplelog");

    println!("Latency={:?}", termbg::latency(Duration::from_secs(2)));
}
