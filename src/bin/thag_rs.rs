#![allow(clippy::uninlined_format_args, unused_imports)]

use std::cell::RefCell;
use std::error::Error;
#[cfg(debug_assertions)]
use std::time::Instant;
use thag_profiler::enable_profiling;
use thag_profiler::profiling;

#[cfg(feature = "build")]
use thag_rs::cmd_args::set_verbosity;

use thag_rs::cvlog;
use thag_rs::cvprtln;
#[cfg(debug_assertions)]
#[cfg(feature = "core")]
use thag_rs::debug_timings;

#[cfg(feature = "core")]
use thag_rs::logging::configure_log;

use thag_rs::Role;
use thag_rs::V;
#[cfg(feature = "build")]
use thag_rs::{execute, get_args, ThagResult};

// use thag_rs::ThagResult;

#[enable_profiling(no)]
pub fn main() {
    #[cfg(feature = "build")]
    {
        let cli = RefCell::new(get_args()); // Wrap args in a RefCell
        let result = handle(&cli);

        if let Err(e) = result {
            cvprtln!(Role::ERR, V::N, "Error running thag: {e}");

            std::process::exit(1);
        }
    }

    #[cfg(not(feature = "build"))]
    {
        println!("Feature `build` not specified - exiting")
    }
}

#[cfg(feature = "build")]
fn handle(cli: &RefCell<thag_rs::Cli>) -> ThagResult<()> {
    use thag_rs::logging;

    #[cfg(debug_assertions)]
    let start = Instant::now();

    set_verbosity(&cli.borrow())?;

    configure_log();
    #[cfg(debug_assertions)]
    debug_timings(&start, "Configured logging");

    execute(&mut cli.borrow_mut())
}
