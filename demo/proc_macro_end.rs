/// Proof of concept of a function-like proc macro to provide its line number to
/// a caller higher up in the same function. The `end!("x")` proc macro replaces itself
/// in the calling source code by a tiny function `end_x() -> u32` that evaluates the
/// Rust standard `line!()` macro to determine its line number and returns this value
/// to callers. This is to allow the `thag_profiler` `profile!` declarative macro, which
/// profiles a section, to determine the end line number of the section in addition to
/// the starting line which it determines itself, so that memory allocations made within
/// the section can be correctly attributed to the section by the tracking allocator
/// doing a backtrace and matching up the module, function and line number from
/// the backtrace with the registered profiles.
///
//# Purpose: Prototype a technique to facilitate section profiling.
//# Categories: proc_macros, profiling, prototype, technique
// "use thag_demo_proc_macros..." is a magic import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::end;

// Generate a function named "end_my_section" that returns line!()

fn main() {
    println!("Section ends on line {}", end_my_section());
    // Some filler lines here
    // Some filler lines here
    // Some filler lines here
    // Some filler lines here
    // Some filler lines here
    // Some filler lines here
    end!("my_section");
}
