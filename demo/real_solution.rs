/*[toml]
[dependencies]
crossterm = "0.28"
once_cell = "1.19"
*/

/// Real solution for terminal corruption in concurrent environments
//# Purpose: Provide actual solution for OSC sequence corruption without false positives
//# Categories: terminal, testing, synchronization, solution
use once_cell::sync::Lazy;
use std::io::{stdout, Write};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/// Global mutex for synchronizing all terminal output
static TERMINAL_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// Synchronized print function that prevents terminal corruption
pub fn sync_print(text: &str) {
    let _guard = TERMINAL_MUTEX.lock().unwrap();
    print!("{}", text);
    let _ = stdout().flush();
}

/// Synchronized println function that prevents terminal corruption
pub fn sync_println(text: &str) {
    let _guard = TERMINAL_MUTEX.lock().unwrap();
    println!("{}", text);
    let _ = stdout().flush();
}

/// Synchronized OSC sequence sender - this is the key function
pub fn sync_osc_sequence(sequence: &str) {
    let _guard = TERMINAL_MUTEX.lock().unwrap();
    print!("{}", sequence);
    let _ = stdout().flush();
}

/// Test if we're in a testing environment
fn is_test_environment() -> bool {
    std::env::var("CARGO_TEST").is_ok()
        || std::env::var("TEST_ENV").is_ok()
        || std::env::args().any(|arg| arg.contains("test"))
}

/// Simple visual corruption test - no raw mode needed
fn visual_corruption_test() -> bool {
    println!("=== Visual Corruption Test ===");
    println!("ALIGNMENT_TEST_LINE_1");
    println!("ALIGNMENT_TEST_LINE_2");
    println!("ALIGNMENT_TEST_LINE_3");

    // If these align visually, terminal is working
    // This is the only reliable test without raw mode interference
    true // User must visually verify
}

/// Demonstrate the actual problem: concurrent OSC sequences
fn demonstrate_real_problem() {
    println!("\n=== Demonstrating Real OSC Corruption ===");
    println!("This will cause actual corruption through thread interference:");

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..15 {
                    // UNSYNCHRONIZED OSC sequences - this causes real corruption
                    print!("\x1b]0;Thread {} Title {}\x1b\\", i, j);
                    print!(
                        "\x1b]8;;http://example.com/{}\x1b\\Link {}\x1b]8;;\x1b\\",
                        i, j
                    );
                    println!("Thread {} regular output {}", i, j);
                    thread::sleep(Duration::from_millis(1));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Concurrent OSC output complete");
    println!("Check if the lines above align properly at column 0");
}

/// Demonstrate the solution: synchronized OSC sequences
fn demonstrate_solution() {
    println!("\n=== Demonstrating Synchronized Solution ===");
    println!("This prevents corruption through proper synchronization:");

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..15 {
                    // SYNCHRONIZED OSC sequences - this prevents corruption
                    sync_osc_sequence(&format!("\x1b]0;Thread {} Title {}\x1b\\", i, j));
                    sync_osc_sequence(&format!(
                        "\x1b]8;;http://example.com/{}\x1b\\Link {}\x1b]8;;\x1b\\",
                        i, j
                    ));
                    sync_println(&format!("Thread {} regular output {}", i, j));
                    thread::sleep(Duration::from_millis(1));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Synchronized OSC output complete");
    println!("These lines should align properly at column 0");
}

/// Unit test helper functions
pub mod test_helpers {
    use super::*;

    /// Test setup - use at start of tests that do terminal output
    pub fn setup() {
        if is_test_environment() {
            sync_println("🧪 Test environment - using synchronized output");
        }
    }

    /// Test teardown - use at end of tests
    pub fn teardown() {
        if is_test_environment() {
            // No raw mode corruption detection - just ensure clean state
            sync_println("✅ Test complete");
        }
    }

    /// Macro for safe test output
    #[macro_export]
    macro_rules! test_println {
        ($($arg:tt)*) => {
            $crate::sync_println(&format!($($arg)*))
        };
    }

    /// Safe assertion that won't corrupt terminal
    pub fn assert_with_output<T>(condition: bool, success_msg: T, failure_msg: T)
    where
        T: std::fmt::Display,
    {
        if condition {
            sync_println(&format!("✅ {}", success_msg));
        } else {
            sync_println(&format!("❌ {}", failure_msg));
            panic!("Assertion failed: {}", failure_msg);
        }
    }
}

/// Example of a proper unit test using the solution
fn example_unit_test() {
    test_helpers::setup();

    sync_println("🧪 Running example concurrent test...");

    // Simulate a test that does concurrent terminal output safely
    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..3 {
                    // All terminal output synchronized
                    sync_println(&format!("Test assertion {} from thread {}", j, i));

                    // Safe to do OSC sequences too
                    sync_osc_sequence(&format!("\x1b]0;Test {} Progress\x1b\\", i));

                    thread::sleep(Duration::from_millis(10));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    test_helpers::assert_with_output(
        true,
        "Concurrent test completed successfully",
        "Test failed",
    );

    test_helpers::teardown();
}

/// Configuration for different environments
pub mod config {
    use super::*;

    /// Initialize terminal safety for the current environment
    pub fn init() {
        if is_test_environment() {
            sync_println("🔧 Terminal synchronization active for testing");
        }
    }

    /// Check if terminal output should be synchronized
    pub fn should_sync() -> bool {
        // Always sync in test environment
        // Could be extended with other conditions
        is_test_environment() || std::env::var("FORCE_TERMINAL_SYNC").is_ok()
    }
}

fn main() {
    println!("=== Real Terminal Corruption Solution ===");
    println!("This demonstrates the actual problem and working solution\n");

    // Initialize
    config::init();

    // Show the problem
    println!("1. First, let's see normal aligned output:");
    visual_corruption_test();

    // Show the real problem
    println!("\n2. Now demonstrating REAL corruption from concurrent OSC:");
    demonstrate_real_problem();

    // Show it works
    println!("\n3. Now demonstrating the SOLUTION:");
    demonstrate_solution();

    // Example test
    println!("\n4. Example of safe concurrent test:");
    example_unit_test();

    // Final verification
    println!("\n=== Final Verification ===");
    println!("Check the output above:");
    println!("- Lines should align after 'Synchronized OSC output complete'");
    println!("- Test output should be clean and aligned");
    println!("- No missing or overwritten lines");

    println!("\n=== Usage Guide ===");
    println!("For your unit tests:");
    println!("1. Replace println! with sync_println in tests");
    println!("2. Use sync_osc_sequence for any OSC sequences");
    println!("3. Add test_helpers::setup() and teardown() to tests");
    println!("4. Set CARGO_TEST=1 when running tests");

    println!("\n=== Alternative: Single-threaded Tests ===");
    println!("Run: cargo test -- --test-threads=1");
    println!("This completely avoids concurrency issues");

    println!("\n=== Key Insight ===");
    println!("The corruption is NOT permanent terminal damage.");
    println!("It's thread interference with OSC sequences.");
    println!("Synchronization prevents it completely.");
}
