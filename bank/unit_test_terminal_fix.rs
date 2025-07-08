/// Practical terminal state management for unit tests
//# Purpose: Provide terminal corruption detection and fixes for unit test environments
//# Categories: terminal, testing, corruption, fix
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

/// Detect if terminal corruption is present
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

/// Comprehensive terminal reset that actually works
fn force_terminal_reset() {
    let _guard = TERMINAL_MUTEX.lock().unwrap();

    // Multiple reset strategies in order of aggressiveness

    // 1. Hard reset (most aggressive)
    print!("\x1bc"); // ESC c - Full reset
    stdout().flush().unwrap();

    // 2. Reset to initial state
    print!("\x1b[!p"); // RIS - Reset to Initial State
    stdout().flush().unwrap();

    // 3. Reset specific modes that affect line discipline
    print!("\x1b[?7h"); // Enable auto-wrap mode
    print!("\x1b[?25h"); // Show cursor
    print!("\x1b[0m"); // Reset all attributes
    print!("\x1b[H"); // Move cursor to home
    stdout().flush().unwrap();

    // 4. Explicitly set line discipline modes
    print!("\x1b[20h"); // Set newline mode (LNM) - forces LF to CR+LF
    stdout().flush().unwrap();

    // 5. Force a known good state
    print!("\r\n"); // Explicit CR+LF
    stdout().flush().unwrap();

    // 6. Terminal-specific resets
    print!("\x1b]0;\x1b\\"); // Clear OSC title
    print!("\x1b]1;\x1b\\"); // Clear OSC icon
    print!("\x1b]2;\x1b\\"); // Clear OSC window title
    stdout().flush().unwrap();

    // 7. Wait for terminal to process
    thread::sleep(Duration::from_millis(50));
}

/// Test and fix terminal state with verification
pub fn fix_terminal_state() -> Result<bool, Box<dyn std::error::Error>> {
    println!("🔧 Checking and fixing terminal state...");

    // Check if already corrupted
    let initially_corrupted = detect_corruption()?;
    if initially_corrupted {
        println!("❌ Terminal corruption detected - applying fixes...");

        // Apply comprehensive reset
        force_terminal_reset();

        // Wait and test again
        thread::sleep(Duration::from_millis(100));

        let still_corrupted = detect_corruption()?;
        if still_corrupted {
            println!("⚠️  Terminal corruption persists after reset");
            println!("💡 Try: export TERM_RESET=1 && printf '\\033[!p\\033[?7h' in your shell");
            return Ok(false);
        } else {
            println!("✅ Terminal corruption fixed!");
            return Ok(true);
        }
    } else {
        println!("✅ Terminal state is clean");
        return Ok(true);
    }
}

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

/// Test environment detection
fn is_test_environment() -> bool {
    std::env::var("CARGO_TEST").is_ok()
        || std::env::var("TEST_ENV").is_ok()
        || std::env::args().any(|arg| arg.contains("test"))
}

/// Unit test setup function
pub fn test_setup() -> Result<(), Box<dyn std::error::Error>> {
    if is_test_environment() {
        sync_println("🧪 Test environment detected");

        // Always try to fix terminal state at test start
        if !fix_terminal_state()? {
            return Err("Could not fix terminal corruption".into());
        }
    }
    Ok(())
}

/// Unit test teardown function
pub fn test_teardown() -> Result<(), Box<dyn std::error::Error>> {
    if is_test_environment() {
        if detect_corruption()? {
            sync_println("❌ Test caused terminal corruption");
            fix_terminal_state()?;
        } else {
            sync_println("✅ Terminal state OK after test");
        }
    }
    Ok(())
}

/// Macro for safe test output
macro_rules! test_println {
    ($($arg:tt)*) => {
        sync_println(&format!($($arg)*))
    };
}

/// Simulate problematic concurrent output
fn simulate_test_corruption() {
    println!("\n=== Simulating concurrent test output (causes corruption) ===");

    let handles: Vec<_> = (0..8)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..10 {
                    // Simulate test output with OSC sequences
                    print!("\x1b]0;Test {} - Step {}\x1b\\", i, j);
                    println!("Test {} assertion {}", i, j);
                    thread::sleep(Duration::from_millis(2));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Demonstrate safe test pattern
fn demonstrate_safe_test_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Demonstrating safe test pattern ===");

    // Setup
    test_setup()?;

    // Simulate test with multiple threads
    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..5 {
                    test_println!("✅ Safe test {} operation {}", i, j);
                    thread::sleep(Duration::from_millis(10));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Teardown
    test_teardown()?;

    Ok(())
}

/// Main demo function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Unit Test Terminal Fix Demo ===");
    println!("This demonstrates detection and fixing of terminal corruption in tests\n");

    // Initial check and fix
    println!("1. Initial terminal state check:");
    fix_terminal_state()?;

    // Demonstrate the problem
    println!("\n2. Simulating problematic test output:");
    simulate_test_corruption();

    // Check for corruption
    println!("\n3. Checking for corruption after simulation:");
    if detect_corruption()? {
        println!("❌ Corruption detected - fixing...");
        fix_terminal_state()?;
    } else {
        println!("✅ No corruption detected");
    }

    // Demonstrate safe pattern
    println!("\n4. Demonstrating safe test pattern:");
    demonstrate_safe_test_pattern()?;

    // Final verification
    println!("\n5. Final verification:");
    if detect_corruption()? {
        println!("❌ Terminal still corrupted after fixes");
        println!("💡 Manual fix: Open new terminal or run:");
        println!("   printf '\\033[!p\\033[?7h\\033[20h\\r\\n'");
    } else {
        println!("✅ Terminal state is clean");
    }

    println!("\n=== Usage in Your Tests ===");
    println!("Add this to your test functions:");
    println!("```rust");
    println!("#[test]");
    println!("fn my_test() {{");
    println!("    test_setup().unwrap();");
    println!("    // Your test code here - use test_println! for output");
    println!("    test_println!(\"Test step 1\");");
    println!("    test_teardown().unwrap();");
    println!("}}");
    println!("```");

    println!("\n=== Alternative Solutions ===");
    println!("1. Run tests single-threaded: cargo test -- --test-threads=1");
    println!("2. Set environment variable: export CARGO_TEST=1");
    println!("3. Add terminal reset to your shell profile");

    Ok(())
}
