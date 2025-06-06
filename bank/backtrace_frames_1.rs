/*[toml]
[dependencies]
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
*/
use thag_profiler::enable_profiling;

#[enable_profiling(memory)]
fn main() {
    use backtrace::{resolve_frame, trace, Backtrace, BacktraceFrame};

    let in_profile_code = false;

    // Pretend to check for profiler code only
    // let mut frames = Vec::new();
    trace(|frame| {
        let mut suppress = false;
        let mut fin = false;

        resolve_frame(frame, |symbol| {
            if let Some(name) = symbol.name() {
                if name.to_string().contains("__rust_begin_short_backtrace") {
                    fin = true;
                }
                if name.to_string().starts_with("backtrace::backtrace::") {
                    suppress = true;
                }
            }
        });
        !fin
    });
    // frames.shrink_to_fit();
}
