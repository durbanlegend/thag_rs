#![allow(clippy::uninlined_format_args)]

use thag_rs::logging::{configure_log, set_verbosity};
use thag_rs::{debug_log, debug_timings, ThagError};
use thag_rs::{execute, get_args};

use std::cell::RefCell;
use std::time::Instant;

pub fn main() -> Result<(), ThagError> {
    let start = Instant::now();
    let args = RefCell::new(get_args()); // Wrap args in a RefCell

    set_verbosity(&args.borrow())?;

    configure_log();
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

    // Example of debug logging that will respect both compile-time and runtime flags
    debug_log!("This is another debug log message.");
    Ok(())
}
