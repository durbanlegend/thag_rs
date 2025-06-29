/// Published example from the `crossbeam-channel` crate.
///
/// The latest version of this example is available in the [examples] folder
///  in the `crossbeam-channel` repository. At time of writing you can run it successfully simply
/// by invoking its URL with the `thag_url` tool, like this:
///
/// ```bash
/// thag_url https://github.com/crossbeam-rs/crossbeam/blob/master/crossbeam-channel/examples/fibonacci.rs
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
//# Purpose: Demo featured crate.
//# Categories: crates
// An asynchronous fibonacci sequence generator.
use std::thread;

use crossbeam_channel::{bounded, Sender};

// Sends the Fibonacci sequence into the channel until it becomes disconnected.
fn fibonacci(sender: Sender<u64>) {
    let (mut x, mut y) = (0, 1);
    while sender.send(x).is_ok() {
        let tmp = x;
        x = y;
        y += tmp;
    }
}

fn main() {
    let (s, r) = bounded(0);
    thread::spawn(|| fibonacci(s));

    // Print the first 20 Fibonacci numbers.
    for num in r.iter().take(20) {
        println!("{}", num);
    }
}
