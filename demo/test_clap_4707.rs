/*[toml]
[dependencies]
clap = { version = "4.5.16", features = ["cargo", "derive"] }
*/

/// Minimal reproducible code posted by user `mkeeter` to demonstrate `clap` issue 4707
/// which we are experiencing at time of creation of this script.
///
/// To reproduce the error, run `cargo run demo/test_clap_4707.rs -- --write --show-hex`
//# Purpose: test if the error exists, then periodically to see if it persists.
use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[clap(group = ArgGroup::new("command").multiple(false))]
struct Args {
    #[clap(long, group = "command")]
    read: bool,

    #[clap(long, group = "command")]
    write: bool,

    #[clap(long, requires = "read")]
    show_hex: bool,
}

fn main() {
    let args = Args::parse();
    println!("{args:?}");
}
