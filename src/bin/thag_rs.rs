#![allow(clippy::uninlined_format_args)]

use std::error::Error;
use thag_rs::{execute, get_args};

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = get_args();
    // Check if firestorm profiling is enabled
    if firestorm::enabled() {
        // Profile the `execute` function
        firestorm::bench("./flames/", || {
            execute(args.clone()).unwrap();
        })
        .unwrap();
    } else {
        // Regular execution when profiling is not enabled
        execute(args)?;
    }

    Ok(())
}
