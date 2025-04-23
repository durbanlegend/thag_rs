/*[toml]
[features]
# Use a spinlock internally (may be faster on some platforms)
spin = []
select = []
async = ["futures-sink", "futures-core"]
eventual-fairness = ["select", "nanorand"]
default = ["async", "select", "eventual-fairness"]
profiling = ["thag_profiler/full_profiling"]

[dependencies]
async-std = { version = "1.13.0", features = ["attributes"] }
spin1 = { package = "spin", version = "0.9.8", features = ["mutex"] }
futures-sink = { version = "0.3", default-features = false, optional = true }
futures-core = { version = "0.3", default-features = false, optional = true }
nanorand = { version = "0.7", features = ["getrandom"], optional = true }
# flume = "0.11"
flume = { version = "0.11", features = ["async"] }
rustix = "0.37.19"
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["full_profiling"] }
# thag_profiler = { version = "0.1", features = ["full_profiling"] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler" }
*/

/// Published example from the `flume` channel crate.
/// Must be run with --multimain (-m) option to allow multiple main methods.
///
/// Refactored and profiled to test and demonstrate profiling of non-tokio
/// async functions with `thag_profiler`.
//# Purpose: demo and test profiling of non-tokio async functions with `thag_profiler`.
//# Categories: async, crates, proc_macros, profiling, technique
use flume;

use thag_profiler::*;

#[cfg(feature = "async")]
#[async_std::main]
#[enable_profiling]
async fn main() {
    // enable_profiling(true, ProfileType::Both).expect("Failed to enable profiling");

    // Check if profiling is enabled
    println!(
        "is_profiling_enabled() = {}",
        thag_profiler::is_profiling_enabled()
    );

    if cfg!(feature = "profiling") {
        println!("Profiling is enabled");
    } else {
        println!("Profiling is disabled");
    }

    let _ = perform().await;
}

#[profiled]
async fn perform() {
    let (tx, rx) = flume::bounded(1);

    profile!("outer_async_operation", time, mem_summary, async_fn);
    let t = async_std::task::spawn(async move {
        while let Ok(msg) = rx.recv_async().await {
            println!("Received: {}", msg);
        }
    });
    end!("outer_async_operation");

    profile!("send_async", time, mem_detail, async_fn);
    tx.send_async("Hello, world!").await.unwrap();
    tx.send_async("How are you today?").await.unwrap();
    end!("send_async");

    drop(tx);

    t.await;
}

#[cfg(not(feature = "async"))]
#[enable_profiling]
fn main() {
    println!(r#"Run with flume "async" feature activated in toml block to enable this demo"#);
}
