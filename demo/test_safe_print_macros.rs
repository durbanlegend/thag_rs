/*[toml]
[dependencies]
# thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["core", "simplelog"] }
thag_proc_macros = { version = "0.2, thag-auto" }
*/

/// Demo script to test the new safe print macros for terminal synchronization
//# Purpose: Test safe print macros that prevent terminal corruption in concurrent environments
//# Categories: terminal, testing, macros, synchronization
use std::thread;
use std::time::Duration;
use thag_proc_macros::{safe_eprint, safe_eprintln, safe_osc, safe_print, safe_println};

/// Test basic safe print functionality
fn test_basic_safe_prints() {
    safe_println!("=== Testing Basic Safe Print Macros ===");

    safe_print!("This is safe_print! ");
    safe_println!("followed by safe_println!");

    safe_println!("Testing with formatting: {} + {} = {}", 2, 3, 2 + 3);

    safe_eprint!("This is safe_eprint! ");
    safe_eprintln!("followed by safe_eprintln!");

    safe_eprintln!("Error formatting: code {}, message '{}'", 404, "Not Found");

    safe_println!("Basic tests complete\n");
}

/// Test OSC sequences with safe_osc macro
fn test_safe_osc_sequences() {
    safe_println!("=== Testing Safe OSC Sequences ===");

    // Set terminal title
    safe_osc!("\x1b]0;Safe OSC Test\x1b\\");
    safe_println!("Terminal title set (may not be visible in all terminals)");

    // Test hyperlink OSC sequence
    let url = "https://github.com/durbanlegend/thag_rs";
    let text = "thag_rs repository";
    safe_osc!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text);
    safe_println!(" <- That should be a clickable link in supported terminals");

    safe_println!("OSC sequence tests complete\n");
}

/// Test concurrent safe prints (the main use case)
fn test_concurrent_safe_prints() {
    safe_println!("=== Testing Concurrent Safe Prints ===");
    safe_println!("Running 5 threads, each outputting 10 lines...");

    let handles: Vec<_> = (0..5)
        .map(|thread_id| {
            thread::spawn(move || {
                for i in 0..10 {
                    safe_println!("Thread {} message {}", thread_id, i);

                    // Add some OSC sequences to test the problematic case
                    safe_osc!("\x1b]0;Thread {} Progress {}/10\x1b\\", thread_id, i + 1);

                    // Small delay to allow interleaving
                    thread::sleep(Duration::from_millis(5));
                }
                safe_println!("Thread {} completed", thread_id);
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    safe_println!("All threads completed - check if output is properly aligned");
    safe_println!("Concurrent tests complete\n");
}

/// Test mixing safe and unsafe prints (demonstration)
fn test_mixed_output() {
    safe_println!("=== Testing Mixed Safe/Unsafe Output ===");
    safe_println!("This section shows the difference between safe and unsafe output");

    safe_println!("Starting mixed output test with 3 threads...");

    let handles: Vec<_> = (0..3)
        .map(|thread_id| {
            thread::spawn(move || {
                for i in 0..5 {
                    if thread_id == 0 {
                        // Thread 0 uses safe prints
                        safe_println!("SAFE Thread {} message {}", thread_id, i);
                        safe_osc!("\x1b]0;Safe Thread {}\x1b\\", thread_id);
                    } else {
                        // Other threads use unsafe prints (may cause corruption)
                        println!("UNSAFE Thread {} message {}", thread_id, i);
                        print!("\x1b]0;Unsafe Thread {}\x1b\\", thread_id);
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    safe_println!("Mixed output test complete");
    safe_println!("Notice: SAFE lines should align properly, UNSAFE may not\n");
}

/// Test error output scenarios
fn test_error_output() {
    safe_println!("=== Testing Error Output ===");

    let handles: Vec<_> = (0..3)
        .map(|thread_id| {
            thread::spawn(move || {
                for i in 0..5 {
                    safe_eprintln!("Error from thread {}: issue #{}", thread_id, i);
                    thread::sleep(Duration::from_millis(8));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    safe_println!("Error output test complete\n");
}

/// Demonstrate unit test usage pattern
fn demonstrate_unit_test_pattern() {
    safe_println!("=== Unit Test Pattern Demonstration ===");

    // Simulate a unit test that does concurrent operations
    let test_name = "concurrent_operation_test";
    safe_println!("🧪 Starting test: {}", test_name);

    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                safe_println!("  ✓ Test assertion {} passed", i);
                safe_println!("  → Running sub-test {}", i);

                // Simulate some test output that might include OSC sequences
                safe_osc!("\x1b]0;Running Test {}\x1b\\", i);

                thread::sleep(Duration::from_millis(20));
                safe_println!("  ✅ Sub-test {} completed successfully", i);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    safe_println!("✅ Test {} completed successfully", test_name);
    safe_println!("Unit test pattern demonstration complete\n");
}

fn main() {
    safe_println!("=== Safe Print Macros Test Suite ===");
    safe_println!("Testing synchronized terminal output to prevent corruption\n");

    // Run all tests
    test_basic_safe_prints();
    test_safe_osc_sequences();
    test_concurrent_safe_prints();
    test_mixed_output();
    test_error_output();
    demonstrate_unit_test_pattern();

    safe_println!("=== Test Suite Complete ===");
    safe_println!("All output above should be properly aligned and readable.");
    safe_println!("If you see misaligned text, there may be an issue with the macros.");

    safe_println!("\n=== Usage Instructions ===");
    safe_println!("In your unit tests, replace:");
    safe_println!("  println!(...) → safe_println!(...)");
    safe_println!("  print!(...) → safe_print!(...)");
    safe_println!("  eprintln!(...) → safe_eprintln!(...)");
    safe_println!("  eprint!(...) → safe_eprint!(...)");
    safe_println!("  OSC sequences → safe_osc!(...)");

    safe_println!("\nThis will prevent terminal corruption in concurrent test environments.");
}
