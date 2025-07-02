/*[toml]
[dependencies]
thag_demo_proc_macros = { version = "0.1, thag-auto" }
*/

/// Demo of the generate_tests function-like macro for automatic test generation.
///
/// This macro demonstrates repetitive code generation patterns by creating multiple
/// test functions from a list of test data. It reduces boilerplate in test suites
/// and shows how macros can automate common development tasks.
//# Purpose: Demonstrate automatic test case generation from data
//# Categories: technique, proc_macros, function_like_macros, testing, automation
use thag_demo_proc_macros::generate_tests;

// Example 1: Basic arithmetic tests
generate_tests! {
    test_addition: [
        (1, 2, 3),
        (5, 7, 12),
        (0, 0, 0),
        (-1, 1, 0),
        (100, 200, 300),
    ] => |a, b, expected| assert_eq!(a + b, expected)
}

// Example 2: String manipulation tests
generate_tests! {
    test_string_length: [
        ("hello", 5),
        ("", 0),
        ("rust", 4),
        ("proc_macro", 10),
    ] => |input, expected| assert_eq!(input.len(), expected)
}

// Example 3: Mathematical operations
generate_tests! {
    test_multiplication: [
        (2, 3, 6),
        (0, 5, 0),
        (7, 8, 56),
        (-2, 4, -8),
    ] => |a, b, expected| assert_eq!(a * b, expected)
}

// Example 4: Boolean logic tests
generate_tests! {
    test_logical_and: [
        (true, true, true),
        (true, false, false),
        (false, true, false),
        (false, false, false),
    ] => |a, b, expected| assert_eq!(a && b, expected)
}

// Example 5: String contains tests
generate_tests! {
    test_string_contains: [
        ("hello world", "world", true),
        ("rust programming", "rust", true),
        ("test case", "python", false),
        ("", "test", false),
    ] => |haystack, needle, expected| assert_eq!(haystack.contains(needle), expected)
}

// Example 6: Vector operations
generate_tests! {
    test_vector_sum: [
        (vec![1, 2, 3], 6),
        (vec![0], 0),
        (vec![10, 20, 30, 40], 100),
        (vec![], 0),
    ] => |input, expected| assert_eq!(input.iter().sum::<i32>(), expected)
}

// Example 7: Range tests
generate_tests! {
    test_range_contains: [
        (1..10, 5, true),
        (0..5, 5, false),
        (10..20, 15, true),
        (-5..5, 0, true),
    ] => |range, value, expected| assert_eq!(range.contains(&value), expected)
}

// Helper function for more complex test scenarios
#[allow(dead_code)]
fn is_even(n: i32) -> bool {
    n % 2 == 0
}

// Example 8: Function testing with helper
generate_tests! {
    test_even_numbers: [
        (2, true),
        (3, false),
        (0, true),
        (-4, true),
        (7, false),
    ] => |input, expected| assert_eq!(is_even(input), expected)
}

fn main() {
    println!("Generate Tests Macro Demo");
    println!("========================\n");

    println!("This demo shows the generate_tests! macro that creates multiple");
    println!("test functions from test data, reducing boilerplate code.\n");

    println!("Generated Test Categories:");
    println!("  • Basic arithmetic (addition, multiplication)");
    println!("  • String operations (length, contains)");
    println!("  • Boolean logic (logical AND)");
    println!("  • Vector operations (sum)");
    println!("  • Range operations (contains)");
    println!("  • Custom function testing (even numbers)");

    println!("\nTest Structure:");
    println!("  Each generate_tests! call creates multiple #[test] functions");
    println!("  with names like test_addition_0, test_addition_1, etc.");

    println!("\nExample Generated Code:");
    println!("  generate_tests! {{");
    println!("      test_addition: [(1, 2, 3), (5, 7, 12)]");
    println!("      => |a, b, expected| assert_eq!(a + b, expected)");
    println!("  }}");
    println!("  ");
    println!("  Generates:");
    println!("  #[test]");
    println!("  fn test_addition_0() {{");
    println!("      let a = 1; let b = 2; let expected = 3;");
    println!("      assert_eq!(a + b, expected)");
    println!("  }}");
    println!("  ");
    println!("  #[test]");
    println!("  fn test_addition_1() {{");
    println!("      let a = 5; let b = 7; let expected = 12;");
    println!("      assert_eq!(a + b, expected)");
    println!("  }}");

    println!("\nTo run the generated tests:");
    println!("  cargo test test_addition    # Run all addition tests");
    println!("  cargo test test_string      # Run all string tests");
    println!("  cargo test                  # Run all tests");

    println!("\nKey features demonstrated:");
    println!("  • Automatic test function generation");
    println!("  • Parameter unpacking from tuples");
    println!("  • Repetitive code elimination");
    println!("  • Test data organization");
    println!("  • Closure-based test logic");

    println!("\nUse cases for generate_tests!:");
    println!("  • Mathematical function testing");
    println!("  • String processing validation");
    println!("  • Algorithm correctness verification");
    println!("  • Edge case testing");
    println!("  • Regression test suites");
    println!("  • Property-based testing data");

    println!("\nNote: This is a demo program. The actual tests run with 'cargo test'.");
}

#[cfg(test)]
mod additional_examples {
    use super::*;

    // Example of testing with Result types
    fn safe_divide(a: f64, b: f64) -> Result<f64, &'static str> {
        if b == 0.0 {
            Err("Division by zero")
        } else {
            Ok(a / b)
        }
    }

    generate_tests! {
        test_safe_division: [
            (10.0, 2.0, Ok(5.0)),
            (7.0, 0.0, Err("Division by zero")),
            (0.0, 5.0, Ok(0.0)),
            (-10.0, 2.0, Ok(-5.0)),
        ] => |a, b, expected| assert_eq!(safe_divide(a, b), expected)
    }
}
