/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features=["full_profiling", "debug_logging"] }
*/

/// Demo of the generate_tests function-like macro for automatic test generation.
///
/// This macro demonstrates repetitive code generation patterns by creating multiple
/// test functions from a list of test data. It reduces boilerplate in test suites
/// and shows how macros can automate common development tasks.
///
/// Note that the expansion is not picked up by `cargo expand`, for reasons unknown.
/// To compensate, the proc macro `generate_tests` prints the test source to `stderr`.
///
/// Also, the expansions of the individual `generate_tests!` invocations are visible
/// if the `expand` argument of the call to fn `maybe_expand_proc_macro` from the proc
/// macro function fn `generate_tests` in `lib.rs` iis set to `true`. So if you prefer
/// to use this, you can remove the hard-coded debugging from `generate_tests.rs`.
///
/// To perform the tests and see the results, simply run:
///
/// ```bash
/// thag demo/proc_macro_generate_tests.rs --testing   # Short form: -T
///
/// ```
///
/// # Alternatively: you can run the tests via `thag_cargo`. Choose the script and the `test` subcommand.
///
/// See also: `demo/proc_macro_generate_tests.rs`
//# Purpose: Demonstrate automatic test case generation from data
//# Categories: technique, proc_macros, function_like_macros, testing, automation
use thag_demo_proc_macros::generate_tests;
use thag_profiler::extract_filename_timestamp;

generate_tests! {
    test_extract_filename_timestamp: [
        (extract_filename_timestamp("fib_dashu_snippet-20250722-110328.folded").format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-07-22 11:03:28".to_string()),
    ] => |date_str, expected| {
        println!("date_str={date_str}");
        assert_eq!(date_str, expected);
    }
}
