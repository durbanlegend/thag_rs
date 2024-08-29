#![allow(clippy::uninlined_format_args)]

use std::error::Error;
use thag_rs::{execute, get_args};

use std::cell::RefCell;

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = RefCell::new(get_args()); // Wrap args in a RefCell

    // Check if firestorm profiling is enabled
    if firestorm::enabled() {
        // Profile the `execute` function
        firestorm::bench("./flames/", || {
            execute(&mut args.borrow_mut()).unwrap(); // Use borrow_mut to get a mutable reference
        })
        .unwrap();
    } else {
        // Regular execution when profiling is not enabled
        execute(&mut args.borrow_mut())?; // Use borrow_mut to get a mutable reference
    }

    Ok(())
}
