#![allow(clippy::uninlined_format_args)]

use thag_rs::ThagError;
use thag_rs::{execute, get_args};

use std::cell::RefCell;

pub fn main() -> Result<(), ThagError> {
    let args = RefCell::new(get_args()); // Wrap args in a RefCell

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
