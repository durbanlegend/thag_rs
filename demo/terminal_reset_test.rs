/// Demo script to test terminal reset logic for OSC sequence corruption
//# Purpose: Test terminal reset when OSC sequences cause line discipline corruption
//# Categories: debugging, terminal, xterm
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;

/// Soft terminal reset - attempts to restore normal terminal behavior
fn reset_terminal_line_discipline() {
    print!("\x1b[!p"); // Soft terminal reset (RIS - Reset to Initial State)
    print!("\x1b[?7h"); // Enable auto-wrap mode
    print!("\x1b[?25h"); // Show cursor
    let _ = stdout().flush();
}

/// Hard terminal reset - more aggressive reset
fn hard_reset_terminal() {
    print!("\x1bc"); // Full reset (ESC c)
    print!("\x1b[0m"); // Reset all attributes
    print!("\x1b[?7h"); // Enable auto-wrap mode
    print!("\x1b[H"); // Move cursor to home
    let _ = stdout().flush();
}

/// Simulate the problematic OSC sequence output that causes corruption
fn simulate_osc_corruption() {
    println!("Simulating OSC sequence corruption...");

    // Simulate multiple threads sending OSC sequences
    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..10 {
                    // OSC sequence for setting terminal title (could be interrupted)
                    print!("\x1b]0;Thread {} Message {}\x1b\\", i, j);
                    // Some regular output
                    print!("Thread {} output {}", i, j);
                    // Force a newline that might not return to column 0
                    print!("\n");
                    let _ = stdout().flush();
                    thread::sleep(Duration::from_millis(10));
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    println!("OSC corruption simulation complete");
}

/// Test if terminal state is corrupted by checking cursor position
fn test_terminal_state() -> bool {
    println!("Testing terminal state...");

    // Send a test line
    print!("TEST LINE");
    let _ = stdout().flush();

    // Send newline
    print!("\n");
    let _ = stdout().flush();

    // Query cursor position
    print!("\x1b[6n");
    let _ = stdout().flush();

    // In a real implementation, you'd read the response
    // For now, we'll just assume corruption if we can't read properly

    // Send another test line
    print!("SECOND TEST LINE");
    let _ = stdout().flush();
    print!("\n");
    let _ = stdout().flush();

    // Visual check - if lines don't align properly, state is corrupted
    true // Placeholder - in real implementation would check cursor position response
}

/// Demonstrate detection of terminal corruption
fn demonstrate_detection() {
    println!("=== Terminal State Detection Demo ===");

    // Test 1: Normal state
    println!("Test 1: Normal terminal state");
    println!("Line 1");
    println!("Line 2");
    println!("Line 3");
    println!("Lines should align at column 0");

    println!("\nTest 2: After OSC corruption simulation");
    simulate_osc_corruption();

    println!("Check if these lines align properly:");
    println!("Line A");
    println!("Line B");
    println!("Line C");

    // If lines don't align at column 0, the terminal state is corrupted
    println!("\nDo the lines above align at column 0? If not, corruption detected!");
}

/// Main demo function
fn main() {
    println!("=== Terminal Reset Test Demo ===\n");

    // Demonstrate the problem
    demonstrate_detection();

    println!("\n=== Testing Soft Reset ===");
    reset_terminal_line_discipline();
    println!("Soft reset applied. Testing alignment:");
    println!("Line 1 after soft reset");
    println!("Line 2 after soft reset");
    println!("Line 3 after soft reset");

    println!("\n=== Testing Hard Reset ===");
    hard_reset_terminal();
    println!("Hard reset applied. Testing alignment:");
    println!("Line 1 after hard reset");
    println!("Line 2 after hard reset");
    println!("Line 3 after hard reset");

    println!("\n=== Manual Test Instructions ===");
    println!("1. Run this script and observe line alignment");
    println!("2. If lines don't align at column 0 after 'OSC corruption simulation',");
    println!("   then the reset functions should fix the alignment");
    println!("3. Compare before/after reset to see if the fix works");
    println!("4. If corruption persists, try the hard reset option");

    println!("\n=== Testing Complete ===");
    println!("Note: This is a visual test. Look for misaligned text output.");
}
