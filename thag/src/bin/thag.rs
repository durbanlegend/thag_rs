#![allow(clippy::uninlined_format_args)]

use std::cell::RefCell;
#[cfg(debug_assertions)]
use std::time::Instant;
#[cfg(debug_assertions)]
use thag::debug_timings;
use thag::logging::{configure_log, set_verbosity};
use thag::{execute, get_args, BuildResult};
use thag_core::profiling;

pub fn main() -> BuildResult<()> {
    #[cfg(debug_assertions)]
    let start = Instant::now();
    let cli = RefCell::new(get_args()); // Wrap args in a RefCell

    set_verbosity(&cli.borrow())?;

    configure_log();
    #[cfg(debug_assertions)]
    debug_timings(&start, "Configured logging");

    if cfg!(feature = "profile") {
        println!("Enabling profiling..."); // Debug output
        profiling::enable_profiling(true)?;
    }

    // Check if firestorm profiling is enabled
    // if firestorm::enabled() {
    //     // Profile the `execute` function
    //     firestorm::bench("./flames/", || {
    //         handle(&cli);
    //     })?;
    // } else {
    // Regular execution when profiling is not enabled
    handle(&cli);
    // }
    Ok(())
}

fn handle(cli: &RefCell<thag::Cli>) {
    // Use borrow_mut to get a mutable reference
    let result = execute(&mut cli.borrow_mut());
    match result {
        Ok(()) => (),
        Err(e) => println!("{e}"),
    }
}
