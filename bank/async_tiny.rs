/*[toml]
[dependencies]
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling", "tls_allocator"] }
tokio = { version = "1.36.0", features = ["rt-multi-thread", "macros", "time"] }
*/

use thag_profiler::{enable_profiling, profiled};

#[profiled]
async fn tiny_async() {
    // Single small allocation
    let _v = vec![1; 10];
}

#[tokio::main]
#[enable_profiling(memory)]
async fn main() {
    println!("Starting...");
    tiny_async().await;
    println!("Done.");
}
