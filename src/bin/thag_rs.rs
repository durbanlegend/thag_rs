#![allow(clippy::uninlined_format_args, unused_imports)]

use re_memory::accounting_allocator::tracking_stats;
use re_memory::{accounting_allocator, MemoryUse};
use std::cell::RefCell;
#[cfg(debug_assertions)]
use std::time::Instant;
// use thag_proc_macros::enable_profiling;
use thag_profiler::profiling;
use thag_rs::cmd_args::set_verbosity;
#[cfg(debug_assertions)]
use thag_rs::debug_timings;
use thag_rs::logging::configure_log;
use thag_rs::{execute, get_args, ThagResult};

use re_memory::AccountingAllocator;

#[global_allocator]
static GLOBAL: AccountingAllocator<std::alloc::System> =
    AccountingAllocator::new(std::alloc::System);

// #[enable_profiling(profile_type = "time")] // default Both
pub fn main() -> ThagResult<()> {
    if cfg!(feature = "profiling") {
        println!("Enabling profiling..."); // Debug output
        profiling::enable_profiling(true, profiling::ProfileType::Both)?;
    }

    re_memory::accounting_allocator::set_tracking_callstacks(true);

    #[cfg(debug_assertions)]
    let start = Instant::now();
    let cli = RefCell::new(get_args()); // Wrap args in a RefCell

    set_verbosity(&cli.borrow())?;

    configure_log();
    #[cfg(debug_assertions)]
    debug_timings(&start, "Configured logging");

    handle(&cli);
    let memory_use = MemoryUse::capture();
    println!("memory_use={memory_use:?}");
    let tracking_stats =
        accounting_allocator::tracking_stats().expect("Could not get tracking stats");
    let top_callstacks = tracking_stats.top_callstacks;
    for callstack in top_callstacks {
        println!(
            "real estimate count={}, size={}, callstack={}",
            callstack.stochastic_rate * callstack.extant.count,
            thag_rs::thousands(callstack.stochastic_rate * callstack.extant.size),
            callstack.readable_backtrace
        );
    }
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
