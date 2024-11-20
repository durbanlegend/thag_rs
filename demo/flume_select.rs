/*[toml]
[features]
# Use a spinlock internally (may be faster on some platforms)
spin = []
select = []
async = ["futures-sink", "futures-core"]
eventual-fairness = ["select", "nanorand"]
default = ["async", "select", "eventual-fairness"]

[dependencies]
spin1 = { package = "spin", version = "0.9.8", features = ["mutex"] }
futures-sink = { version = "0.3", default_features = false, optional = true }
futures-core = { version = "0.3", default_features = false, optional = true }
nanorand = { version = "0.7", features = ["getrandom"], optional = true }
flume = "0.11.0"
*/

#[cfg(feature = "select")]
use flume::Selector;

/// Published example from the `flume` channel crate.
/// Must be run with --multimain (-m) option to allow multiple main methods.
//# Purpose: demo of async and channel programming and of `flume` in particular.
//# Categories: async, crates, technique
#[cfg(feature = "select")]
fn main() {
    // Create two channels
    let (red_tx, red_rx) = flume::unbounded();
    let (blue_tx, blue_rx) = flume::unbounded();

    // Spawn two threads that each send a message into their respective channel
    std::thread::spawn(move || {
        let _ = red_tx.send("Red");
    });
    std::thread::spawn(move || {
        let _ = blue_tx.send("Blue");
    });

    // Race them to see which one sends their message first
    let winner = Selector::new()
        .recv(&red_rx, |msg| msg)
        .recv(&blue_rx, |msg| msg)
        .wait()
        .unwrap();

    println!("{} won!", winner);
}

#[cfg(not(feature = "select"))]
fn main() {
    println!(r#"Run with flume "select" feature activated in toml block to enable this demo"#);
}
