/*[toml]
[dependencies]
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
*/
use thag_profiler::enable_profiling;

#[enable_profiling(memory)]
fn main() {
    use backtrace::{resolve_frame, trace, Backtrace, BacktraceFrame};

    let mut frames = Vec::new();
    trace(|frame| {
        let mut suppress = false;
        let mut fin = false;

        resolve_frame(frame, |symbol| {
            if let Some(name) = symbol.name() {
                let name = name.to_string();
                if name.contains("__rust_begin_short_backtrace") {
                    fin = true;
                }
                if name.starts_with("backtrace::backtrace::") {
                    suppress = true;
                }
                if !suppress {
                    eprintln!(
                        "ip={:?}, name={name}, filename={}, lineno={}",
                        frame.ip(),
                        symbol
                            .filename()
                            .map_or_else(|| "N/A".to_string(), |f| f.display().to_string()),
                        symbol.lineno().unwrap_or_else(|| 0),
                    );
                }
            }
        });
        if !suppress {
            let mut btf = BacktraceFrame::from(frame.clone());
            btf.resolve();
            frames.push(btf);
        }
        !fin
    });
    frames.shrink_to_fit();

    eprintln!("frames={frames:#?}");

    eprintln!("Backtrace::new()={:#?}", Backtrace::new());
}
