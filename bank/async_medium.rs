/*[toml]
[dependencies]
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
tokio = { version = "1.36.0", features = ["rt-multi-thread", "macros", "time"] }
*/

use thag_profiler::{enable_profiling, profiled};

#[profiled]
async fn medium_async() {
    for i in 0..10 {
        let _v = vec![i; 100];
        tokio::task::yield_now().await;
    }
}

#[tokio::main]
#[enable_profiling(memory)]
async fn main() {
    println!("Starting...");
    medium_async().await;
    println!("Done.");
}
