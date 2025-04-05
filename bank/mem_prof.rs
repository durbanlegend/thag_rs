/*[toml]
[dependencies]
# thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["profiling"] }
# thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", default-features = false }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler" }


# If you want to enable profiling, either this or specify the profiling feature on the thag_profiler dependency
[features]
profile = ["thag_profiler/profiling"]
*/

extern crate stats_alloc;

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;
use thag_profiler::{enable_profiling, profile, profiled};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[profiled]
fn example_using_region() {
    let reg = Region::new(GLOBAL);

    println!("Creating profile section with custom name 'vec_1024'");
    let section = profile!("vec_1024", both);

    println!("Allocating vector with capacity 1024");
    let x: Vec<u8> = Vec::with_capacity(1_024);

    println!("Ending profiled section");
    section.end();

    println!("Stats at 1: {:#?}", reg.change());
    // Used here to ensure that the value is not
    // dropped before we check the statistics
    let size = std::mem::size_of_val(&x);
    println!("Size of x: {size}");
}

#[enable_profiling]
fn main() {
    // Check if profiling is enabled
    println!(
        "PROFILING_ENABLED={}",
        thag_profiler::PROFILING_FEATURE_ENABLED
    );

    if cfg!(feature = "profile") {
        println!("Profiling is enabled");
    } else {
        println!("Profiling is disabled");
    }

    example_using_region();

    println!("Program complete");
}
