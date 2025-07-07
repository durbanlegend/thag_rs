/*[toml]
[dependencies]
crossterm = "0.28"
*/

/// Demo script to detect terminal state corruption from OSC sequence interference
//# Purpose: Detect when terminal line discipline is corrupted by concurrent OSC sequences
//# Categories: terminal, debugging, OSC, detection
use crossterm::{
    cursor::{position, MoveTo, MoveToColumn},
    event::{poll, read, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;

/// Detect if terminal line discipline is corrupted by testing cursor behavior
fn detect_terminal_corruption() -> Result<bool, Box<dyn std::error::Error>> {
    println!("Detecting terminal state corruption...");

    // Enable raw mode to read cursor position
    enable_raw_mode()?;

    // Test sequence:
    // 1. Move to known position
    // 2. Print text without newline
    // 3. Send newline
    // 4. Check if cursor is at column 0

    execute!(stdout(), MoveTo(0, 10))?;
    print!("TEST_LINE_FOR_DETECTION");
    stdout().flush()?;

    // Send newline - should move to column 0 of next line
    print!("\n");
    stdout().flush()?;

    // Get cursor position
    let pos = position()?;

    disable_raw_mode()?;

    // If cursor is not at column 0, line discipline is corrupted
    let is_corrupted = pos.0 != 0;

    if is_corrupted {
        println!(
            "CORRUPTION DETECTED: Cursor at column {} instead of 0",
            pos.0
        );
    } else {
        println!("Terminal state OK: Cursor properly at column 0");
    }

    Ok(is_corrupted)
}

/// Test the ONLCR (Output NL to CR-LF) flag behavior
fn test_onlcr_behavior() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting ONLCR (newline to carriage return + line feed) behavior:");

    enable_raw_mode()?;

    // Move to a known position
    execute!(stdout(), MoveTo(10, 15))?;
    print!("Starting at column 10");
    stdout().flush()?;

    // Send just LF (no CR) - should move to column 0 if ONLCR is working
    print!("\nAfter LF: ");
    stdout().flush()?;

    let pos = position()?;
    println!("cursor at column {}", pos.0);

    disable_raw_mode()?;

    if pos.0 == 10 {
        println!("❌ ONLCR NOT WORKING: LF did not reset to column 0");
        return Ok(());
    }

    println!("✅ ONLCR WORKING: LF properly reset to column 0");
    Ok(())
}

/// Simulate concurrent OSC sequences that cause corruption
fn simulate_concurrent_osc_corruption() {
    println!("\nSimulating concurrent OSC sequences...");

    let handles: Vec<_> = (0..8)
        .map(|thread_id| {
            thread::spawn(move || {
                for i in 0..20 {
                    // Simulate various OSC sequences
                    match i % 4 {
                        0 => {
                            // Set terminal title
                            print!("\x1b]0;Thread {} - Title {}\x1b\\", thread_id, i);
                        }
                        1 => {
                            // Set icon name
                            print!("\x1b]1;Icon {}-{}\x1b\\", thread_id, i);
                        }
                        2 => {
                            // Set window title
                            print!("\x1b]2;Window {}-{}\x1b\\", thread_id, i);
                        }
                        3 => {
                            // Custom OSC sequence
                            print!(
                                "\x1b]8;;http://example.com/{}\x1b\\Link{}\x1b]8;;\x1b\\",
                                thread_id, i
                            );
                        }
                        _ => unreachable!(),
                    }

                    // Add some regular output
                    print!("Thread {} output {}", thread_id, i);
                    print!("\n");
                    stdout().flush().unwrap();

                    // Small delay to increase chance of interference
                    thread::sleep(Duration::from_millis(1));
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Concurrent OSC simulation complete");
}

/// Interactive test to manually verify corruption
fn interactive_corruption_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Interactive Corruption Test ===");
    println!("Watch the output carefully. Press any key to continue...");

    enable_raw_mode()?;

    // Wait for keypress
    loop {
        if poll(Duration::from_millis(100))? {
            match read()? {
                Event::Key(event) if event.code == KeyCode::Char('q') => break,
                Event::Key(_) => break,
                _ => {}
            }
        }
    }

    disable_raw_mode()?;

    println!("Starting corruption test...");

    // Before corruption
    println!("BEFORE corruption - these should align:");
    println!("Line 1");
    println!("Line 2");
    println!("Line 3");

    // Simulate corruption
    simulate_concurrent_osc_corruption();

    // After corruption
    println!("\nAFTER corruption - do these still align?");
    println!("Line A");
    println!("Line B");
    println!("Line C");

    // Test detection
    let is_corrupted = detect_terminal_corruption()?;

    if is_corrupted {
        println!("\n❌ CORRUPTION CONFIRMED by detection function");
    } else {
        println!("\n✅ No corruption detected");
    }

    Ok(())
}

/// Reset terminal line discipline using various methods
fn reset_terminal_discipline() {
    println!("\n=== Testing Terminal Reset Methods ===");

    // Method 1: Soft reset
    println!("Applying soft reset (RIS)...");
    print!("\x1b[!p"); // Soft terminal reset
    print!("\x1b[?7h"); // Auto-wrap mode
    print!("\x1b[?25h"); // Show cursor
    stdout().flush().unwrap();

    println!("Test after soft reset:");
    println!("Line 1");
    println!("Line 2");

    // Method 2: Line discipline specific reset
    println!("\nApplying line discipline reset...");
    print!("\x1b[20h"); // Set newline mode (LNM)
    stdout().flush().unwrap();

    println!("Test after LNM reset:");
    println!("Line A");
    println!("Line B");

    // Method 3: Full reset
    println!("\nApplying full reset...");
    print!("\x1bc"); // Full reset
    print!("\x1b[0m"); // Reset all attributes
    stdout().flush().unwrap();

    println!("Test after full reset:");
    println!("Final line 1");
    println!("Final line 2");
}

/// Main demo function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Terminal State Detection Demo ===");
    println!("This demo tests for terminal corruption caused by concurrent OSC sequences");

    // Initial state test
    println!("\n1. Testing initial terminal state...");
    test_onlcr_behavior()?;

    // Corruption detection
    println!("\n2. Testing corruption detection...");
    let initial_corruption = detect_terminal_corruption()?;

    if initial_corruption {
        println!("⚠️  Terminal already corrupted before test!");
    }

    // Interactive test
    interactive_corruption_test()?;

    // Reset test
    reset_terminal_discipline();

    // Final detection
    println!("\n3. Final corruption check...");
    let final_corruption = detect_terminal_corruption()?;

    if final_corruption {
        println!("❌ Terminal still corrupted after reset");
    } else {
        println!("✅ Terminal state restored");
    }

    println!("\n=== Analysis ===");
    println!("For unit testing, consider:");
    println!("1. Synchronizing terminal output with a mutex");
    println!("2. Running tests with --test-threads=1 to avoid concurrency");
    println!("3. Using this detection function to reset state between tests");
    println!("4. Avoiding OSC sequences in test output");

    Ok(())
}
