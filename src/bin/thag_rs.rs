#![allow(clippy::uninlined_format_args)]

#[cfg(debug_assertions)]
use thag_rs::debug_timings;
use thag_rs::logging::{configure_log, set_verbosity};
use thag_rs::{execute, get_args, ThagResult};

use std::cell::RefCell;
#[cfg(debug_assertions)]
use std::time::Instant;

pub fn main() -> ThagResult<()> {
    #[cfg(debug_assertions)]
    let start = Instant::now();
    let args = RefCell::new(get_args()); // Wrap args in a RefCell

    set_verbosity(&args.borrow())?;

    configure_log();
    #[cfg(debug_assertions)]
    debug_timings(&start, "Configured logging");

    // Check if firestorm profiling is enabled
    if firestorm::enabled() {
        // Profile the `execute` function
        // Use borrow_mut to get a mutable reference
        firestorm::bench("./flames/", || {
            execute(&mut args.borrow_mut()).expect("Error calling execute() in firestorm profiler");
        })?;
    } else {
        // Regular execution when profiling is not enabled
        execute(&mut args.borrow_mut())?; // Use borrow_mut to get a mutable reference
    }

    Ok(())
}
