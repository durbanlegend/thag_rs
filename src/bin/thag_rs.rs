#![allow(clippy::uninlined_format_args, unused_imports)]

use std::cell::RefCell;
#[cfg(debug_assertions)]
use std::time::Instant;
// use thag_proc_macros::enable_profiling;
use thag_rs::cmd_args::set_verbosity;
#[cfg(debug_assertions)]
use thag_rs::debug_timings;
use thag_rs::logging::configure_log;
use thag_rs::profiling;
use thag_rs::{execute, get_args, ThagResult};

// #[enable_profiling(profile_type = "time")] // default Both
pub fn main() -> ThagResult<()> {
    #[cfg(debug_assertions)]
    let start = Instant::now();
    let cli = RefCell::new(get_args()); // Wrap args in a RefCell

    set_verbosity(&cli.borrow())?;

    configure_log();
    #[cfg(debug_assertions)]
    debug_timings(&start, "Configured logging");

    handle(&cli);
    Ok(())
}

fn handle(cli: &RefCell<thag_rs::Cli>) {
    // Use borrow_mut to get a mutable reference
    let result = execute(&mut cli.borrow_mut());
    match result {
        Ok(()) => (),
        Err(e) => println!("{e}"),
    }
}
