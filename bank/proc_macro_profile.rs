/*[toml]
[dependencies]
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["full_profiling"] }
# thag_profiler = { version = "0.1", features = ["full_profiling"] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features=["full_profiling"] }

[profile.dev]
debug-assertions = true
*/
use thag_profiler::*;

#[enable_profiling]
fn main() {
    // let _profile = profile!(print_lines, time, mem_detail, unbounded);
    // .expect("Failed to initialize section profile `print_lines`");
    let _profile_section = profile!(transform_snippet, time);
    println!("Section ends on line {}", end_transform_snippet());
    // Some filler lines here
    // Some filler lines here
    end!(transform_snippet);
}
