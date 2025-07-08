/// Practical terminal synchronization solution for unit tests
//# Purpose: Demonstrate synchronized terminal output to prevent OSC corruption in tests
//# Categories: terminal, testing, synchronization, OSC
use crossterm::{
    cursor::position,
    terminal::{disable_raw_mode, enable_raw_mode},
};
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

/// Synchronized OSC sequence sender
pub fn sync_osc_sequence(sequence: &str) {
    let _guard = TERMINAL_MUTEX.lock().unwrap();
    print!("{}", sequence);
    let _ = stdout().flush();
}

/// Test if terminal corruption is present
fn detect_corruption() -> Result<bool, Box<dyn std::error::Error>> {
    let _guard = TERMINAL_MUTEX.lock().unwrap();

    enable_raw_mode()?;

    // Clear any pending output
    stdout().flush()?;

    // Test sequence: print text, newline, check cursor position
    print!("TEST_CORRUPTION_DETECTION");
    stdout().flush()?;
    print!("\n");
    stdout().flush()?;

    let pos = position()?;
    disable_raw_mode()?;

    // If cursor is not at column 0, we have corruption
    Ok(pos.0 != 0)
}

/// Environment detection for test context
fn is_test_environment() -> bool {
    std::env::var("CARGO_TEST").is_ok()
        || std::env::var("TEST_ENV").is_ok()
        || std::env::args().any(|arg| arg.contains("test"))
}

/// Test setup function that checks and reports terminal state
pub fn test_setup() -> Result<(), Box<dyn std::error::Error>> {
    if is_test_environment() {
        sync_println("🧪 Test environment detected - using synchronized output");

        if detect_corruption()? {
            sync_println("⚠️  Terminal corruption detected at test start");
            return Err("Terminal already corrupted".into());
        }
    }
    Ok(())
}

/// Test teardown function that verifies terminal state
pub fn test_teardown() -> Result<(), Box<dyn std::error::Error>> {
    if is_test_environment() {
        if detect_corruption()? {
            sync_println("❌ Terminal corruption detected after test");
            return Err("Test caused terminal corruption".into());
        } else {
            sync_println("✅ Terminal state OK after test");
        }
    }
    Ok(())
}

/// Demonstrate unsynchronized output that causes corruption
fn demo_unsync_corruption() {
    sync_println("\n=== Demonstrating UNSYNCHRONIZED output (causes corruption) ===");

    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..8 {
                    // UNSYNCHRONIZED - this causes corruption
                    print!("\x1b]0;Thread {} Title {}\x1b\\", i, j);
                    print!("Thread {} msg {}\n", i, j);
                    let _ = stdout().flush();
                    thread::sleep(Duration::from_millis(5));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    sync_println("Unsynchronized output complete");
}

/// Demonstrate synchronized output that prevents corruption
fn demo_sync_prevention() {
    sync_println("\n=== Demonstrating SYNCHRONIZED output (prevents corruption) ===");

    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..8 {
                    // SYNCHRONIZED - this prevents corruption
                    sync_osc_sequence(&format!("\x1b]0;Thread {} Title {}\x1b\\", i, j));
                    sync_println(&format!("Thread {} msg {}", i, j));
                    thread::sleep(Duration::from_millis(5));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    sync_println("Synchronized output complete");
}

/// Macro for synchronized testing output
macro_rules! test_println {
    ($($arg:tt)*) => {
        sync_println(&format!($($arg)*))
    };
}

/// Example unit test using synchronized output
fn example_unit_test() -> Result<(), Box<dyn std::error::Error>> {
    test_setup()?;

    test_println!("🧪 Running example unit test...");

    // Simulate test that uses terminal output
    let handles: Vec<_> = (0..3)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..5 {
                    test_println!("Test thread {} operation {}", i, j);
                    thread::sleep(Duration::from_millis(10));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    test_println!("✅ Test completed successfully");
    test_teardown()?;
    Ok(())
}

/// Main demo function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Terminal Synchronization for Unit Tests Demo ===");
    println!("This demo shows how to prevent terminal corruption in concurrent tests\n");

    // Initial state check
    println!("1. Checking initial terminal state...");
    if detect_corruption()? {
        println!("⚠️  Terminal already corrupted!");
    } else {
        println!("✅ Terminal state OK");
    }

    // Demo the problem
    demo_unsync_corruption();

    println!("\n2. Checking for corruption after unsynchronized output...");
    if detect_corruption()? {
        println!("❌ CORRUPTION DETECTED after unsynchronized output");
    } else {
        println!("✅ No corruption detected");
    }

    // Demo the solution
    demo_sync_prevention();

    println!("\n3. Checking for corruption after synchronized output...");
    if detect_corruption()? {
        println!("❌ Corruption still present");
    } else {
        println!("✅ No corruption - synchronization worked!");
    }

    // Demo unit test pattern
    println!("\n4. Demonstrating unit test pattern...");
    example_unit_test()?;

    println!("\n=== Implementation Guide ===");
    println!("For your unit tests, you can:");
    println!("1. Add the sync_print/sync_println functions to your test utils");
    println!("2. Use test_setup() and test_teardown() in your test functions");
    println!("3. Replace all print!/println! in tests with sync_print/sync_println");
    println!("4. Use the test_println! macro for convenient synchronized test output");
    println!("5. Set CARGO_TEST=1 environment variable when running tests");

    println!("\n=== Alternative: Single-threaded Tests ===");
    println!("You can also run: cargo test -- --test-threads=1");
    println!("This avoids concurrency entirely but may be slower");

    println!("\n=== Final Check ===");
    if detect_corruption()? {
        println!("❌ Terminal still corrupted - may need terminal restart");
    } else {
        println!("✅ Terminal state clean");
    }

    Ok(())
}
