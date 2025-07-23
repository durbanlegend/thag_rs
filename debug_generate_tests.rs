/// Minimal debug case for generate_tests macro to see what's actually generated
//# Purpose: Debug macro expansion visibility issue
use thag_demo_proc_macros::generate_tests;

// Simple test case to debug macro expansion
generate_tests! {
    debug_test: [
        (1, 2),
        (3, 4),
    ] => |a, b| assert_eq!(a + 1, b)
}

fn main() {
    println!("Debug test for generate_tests macro");
    println!("This should generate debug_test_0 and debug_test_1 functions");
}

#[cfg(test)]
mod tests {
    use super::*;

    // Let's also try placing the macro inside a test module
    generate_tests! {
        module_test: [
            (5, 10),
            (7, 14),
        ] => |input, expected| assert_eq!(input * 2, expected)
    }
}
