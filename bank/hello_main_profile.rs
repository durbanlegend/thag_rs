/*[toml]
[dependencies]
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["full_profiling"] }
# thag_profiler = { version = "0.1", features = ["full_profiling"] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
*/
/// Hello World as a program (posh Winnie-the-Pooh version)
//# Purpose: Demo Hello World as a program
//# Categories: basic
#[thag_profiler::enable_profiling(runtime)]
fn main() {
    let other = "World 🌍";
    println!("Hello, {other}!");
}
