/// Published example from the `clap` crate.
///
/// The latest version of this example is available in the [examples] folder in the `clap` repository.
/// At time of writing you can run it successfully just by invoking its URL with the `thag_url` tool
/// and passing the required arguments as normal, like this:
///
/// ```bash
/// thag_url https://github.com/clap-rs/clap/blob/master/examples/demo.rs -- --name "is this the Krusty Krab?"
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
/// Original `clap` crate comments:
///
/// Simple program to greet a person
//# Purpose: Demo building a repl using `clap` directly.
//# Categories: REPL, technique
//
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name);
    }
}
