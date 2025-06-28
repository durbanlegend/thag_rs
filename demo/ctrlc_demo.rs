use ctrlc;
use std::sync::mpsc::channel;

/// Published example from `ctrlc` crate: "Cross platform handling of Ctrl-C signals."
//# Purpose: Demo one option for intercepting Ctrl-C.
//# Categories: crates, technique
fn main() {
    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    println!("Waiting for Ctrl-C...");
    rx.recv().expect("Could not receive from channel.");
    println!("Got it! Exiting...");
}
