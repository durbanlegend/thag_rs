/*[toml]
[dependencies]
log = "0.4"
simplelog = "0.12"
thag_profiler = { version = "1, thag-auto", features = ["time_profiling", "debug_logging"] }
thag_styling = { version = "1, thag-auto", features = ["inquire_theming"] }

[features]
time_profiling = ["thag_profiler/time_profiling"]
debug_logging = ["thag_profiler/debug_logging"]
default = ["time_profiling", "debug_logging"]
*/

/// Tool to post_process `.profraw` files from `thag_profiler`, e.g. if the process didn't complete at runtime.
/// E.g.:
///
/// `THAG_PROFILER=time,,announce thag demo/thag_profiler_fold.rs -- test`
///
//# Purpose: Post_process `.profraw` files from `thag_profiler`
//# Categories: profiling, tools
use std::error::Error;
use std::io::Write;
// use std::path::PathBuf;
use thag_profiler::{profiling, DebugLogger};
use thag_styling::{
    auto_help, file_navigator, help_system::check_help_and_exit, themed_inquire_config,
};

file_navigator! {}

fn main() -> Result<(), Box<dyn Error>> {
    let help = auto_help!();
    check_help_and_exit(&help);

    let logger = DebugLogger::get();
    assert!(logger.is_some(), "Logger should be available");

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: No folded name stem provided");
        std::process::exit(1);
    }

    let folded_stem = &args[1];

    let folded_file = format!("{folded_stem}.inclusive.folded");

    let selected_file = if args.len() >= 3 {
        args[2].to_string()
    } else {
        inquire::set_global_render_config(themed_inquire_config());

        let mut navigator = FileNavigator::new();
        select_file(&mut navigator, Some("profraw"), false)
            .unwrap()
            .to_string_lossy()
            .to_string()
    };

    dbg!(&selected_file);
    profiling::process_profraw_to_folded(&selected_file, &folded_file)?;

    // Flush logs directly
    if let Some(logger) = DebugLogger::get() {
        let _ = logger.lock().flush();
    }

    Ok(())
}
