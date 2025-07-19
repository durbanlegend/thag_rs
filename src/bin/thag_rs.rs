#![allow(clippy::uninlined_format_args, unused_imports)]

use std::cell::RefCell;
use std::error::Error;
#[cfg(debug_assertions)]
use std::time::Instant;
use thag_profiler::enable_profiling;
use thag_profiler::profiling;

#[cfg(feature = "build")]
use thag_rs::cmd_args::set_verbosity;

#[cfg(debug_assertions)]
#[cfg(feature = "core")]
use thag_rs::debug_timings;

#[cfg(feature = "core")]
use thag_rs::logging::configure_log;

#[cfg(feature = "build")]
use thag_rs::{execute, get_args, ThagResult};

// use thag_rs::ThagResult;

#[enable_profiling(no)]
pub fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "build")]
    {
        #[cfg(debug_assertions)]
        let start = Instant::now();
        let cli = RefCell::new(get_args()); // Wrap args in a RefCell

        set_verbosity(&cli.borrow())?;

        configure_log();
        #[cfg(debug_assertions)]
        debug_timings(&start, "Configured logging");

        handle(&cli)?;
        Ok(())
    }

    #[cfg(not(feature = "build"))]
    {
        Ok(())
    }
}

#[cfg(feature = "build")]
fn handle(cli: &RefCell<thag_rs::Cli>) -> ThagResult<()> {
    // Use borrow_mut to get a mutable reference
    // let result = execute(&mut cli.borrow_mut());
    // match result {
    //     Ok(()) => (),
    //     Err(e) => println!("{e}"),
    // }

    execute(&mut cli.borrow_mut())
}
