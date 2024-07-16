/*[toml]
[features]
# Use a spinlock internally (may be faster on some platforms)
spin = []
select = []
async = ["futures-sink", "futures-core"]
eventual-fairness = ["select", "nanorand"]
default = ["async", "select", "eventual-fairness"]

[dependencies]
async-std = { version = "1.12.0", features = ["attributes"] }
spin1 = { package = "spin", version = "0.9.8", features = ["mutex"] }
futures-sink = { version = "0.3", default_features = false, optional = true }
futures-core = { version = "0.3", default_features = false, optional = true }
nanorand = { version = "0.7", features = ["getrandom"], optional = true }
flume = "0.11.0"
*/

/// Published example from the `flume` channel crate.
/// Must be run with --multimain (-m) option to allow multiple main methods.
//# Purpose: demo of async and channel programming and of `flume` in particular.
use flume;

#[cfg(feature = "async")]
#[async_std::main]
async fn main() {
    let (tx, rx) = flume::bounded(1);

    let t = async_std::task::spawn(async move {
        while let Ok(msg) = rx.recv_async().await {
            println!("Received: {}", msg);
        }
    });

    tx.send_async("Hello, world!").await.unwrap();
    tx.send_async("How are you today?").await.unwrap();

    drop(tx);

    t.await;
}

#[cfg(not(feature = "async"))]
fn main() {
    println!(r#"Run with flume "async" feature activated in toml block to enable this demo"#);
}
