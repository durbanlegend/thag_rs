/*[toml]
[dependencies]
crossterm = "0.28"
*/

/// Synchronised version - no improvement.
/// Simple diagnostic script to debug the terminal corruption detection
//# Purpose: Debug why we're getting false positives in corruption detection
//# Categories: terminal, debugging, diagnostic
use crossterm::{
    cursor::position,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{stdout, Write};

// Add to your test utilities
use std::sync::Mutex;

static TERMINAL_MUTEX: once_cell::sync::Lazy<Mutex<()>> =
    once_cell::sync::Lazy::new(|| Mutex::new(()));

pub fn safe_print(text: &str) {
    let _guard = TERMINAL_MUTEX.lock().unwrap();
    print!("{}", text);
    let _ = stdout().flush();
}

pub fn safe_println(text: &str) {
    let _guard = TERMINAL_MUTEX.lock().unwrap();
    println!("{}", text);
    let _ = stdout().flush();
}

// Use this in tests instead of safe_println
// safe_println("Test output");

/// Simple corruption detection without synchronization
fn simple_detect_corruption() -> Result<(u16, u16), Box<dyn std::error::Error>> {
    safe_println("Starting simple corruption detection...");

    enable_raw_mode()?;

    // Clear any pending output
    stdout().flush()?;

    // Move to a known position first
    safe_print("\x1b[10;1H"); // Move to row 10, column 1
    stdout().flush()?;

    // Print test text
    safe_print("SIMPLE_TEST");
    stdout().flush()?;

    // Get position after text
    let pos_after_text = position()?;
    safe_println(&format!("Position after text: {:?}", pos_after_text));

    // Send newline
    safe_print("\n");
    stdout().flush()?;

    // Get position after newline
    let pos_after_newline = position()?;
    safe_println(&format!("Position after newline: {pos_after_newline:?}"));

    disable_raw_mode()?;

    Ok(pos_after_newline)
}

/// Test what happens with normal println
fn test_normal_println() {
    safe_println("=== Testing normal println behavior ===");
    safe_println("Line 1");
    safe_println("Line 2");
    safe_println("Line 3");
    safe_println("These should all align at column 0");
}

/// Test manual cursor positioning
fn test_manual_positioning() -> Result<(), Box<dyn std::error::Error>> {
    safe_println("\n=== Testing manual cursor positioning ===");

    enable_raw_mode()?;

    // Move to column 20
    safe_print("\x1b[20G");
    stdout().flush()?;

    let pos1 = position()?;
    safe_println(&format!("Position at column 20: {:?}", pos1));

    // Print text
    safe_print("TEXT_AT_COL_20");
    stdout().flush()?;

    let pos2 = position()?;
    safe_println(&format!("Position after text: {:?}", pos2));

    // Send newline
    safe_print("\n");
    stdout().flush()?;

    let pos3 = position()?;
    safe_println(&format!("Position after newline: {:?}", pos3));

    disable_raw_mode()?;

    // The key question: is pos3.0 == 0?
    if pos3.0 == 0 {
        safe_println("✅ Newline correctly returned to column 0");
    } else {
        safe_println(&format!(
            "❌ Newline failed to return to column 0 (at column {})",
            pos3.0,
        ));
    }

    Ok(())
}

/// Test raw mode behavior
fn test_raw_mode_effects() -> Result<(), Box<dyn std::error::Error>> {
    safe_println("\n=== Testing raw mode effects ===");

    safe_println("Before raw mode:");
    safe_println("Line A");
    safe_println("Line B");

    enable_raw_mode()?;

    safe_print("In raw mode - this is line C\n");
    stdout().flush()?;

    safe_print("In raw mode - this is line D\n");
    stdout().flush()?;

    disable_raw_mode()?;

    safe_println("After raw mode:");
    safe_println("Line E");
    safe_println("Line F");

    Ok(())
}

/// Test the specific sequence that our detection function uses
fn test_detection_sequence() -> Result<(), Box<dyn std::error::Error>> {
    safe_println("\n=== Testing our detection sequence ===");

    // This is exactly what our detection function does
    enable_raw_mode()?;

    stdout().flush()?;

    safe_print("TEST_CORRUPTION_DETECTION");
    stdout().flush()?;

    let pos_before_newline = position()?;
    safe_println(&format!("Position before newline: {pos_before_newline:?}"));

    safe_print("\n");
    stdout().flush()?;

    let pos_after_newline = position()?;
    safe_println(&format!("Position after newline: {pos_after_newline:?}"));

    disable_raw_mode()?;

    // Analysis
    safe_println("Analysis:");
    safe_println(&format!("- Text was at column {}", pos_before_newline.0));
    safe_println(&format!(
        "- After newline, cursor at column {}",
        pos_after_newline.0
    ));

    if pos_after_newline.0 == 0 {
        safe_println("✅ ONLCR working correctly");
    } else {
        safe_println("❌ ONLCR not working - cursor should be at column 0");

        // But let's check if it's really broken
        safe_println("Let's verify with a visual test:");
        safe_print("Does this text align properly? ");
        safe_println("It should be at column 0.");
    }

    Ok(())
}

/// Visual alignment test
fn visual_alignment_test() {
    safe_println("\n=== Visual Alignment Test ===");
    safe_println("If terminal corruption is real, these lines won't align:");
    safe_println("Line 1 - should start at column 0");
    safe_println("Line 2 - should start at column 0");
    safe_println("Line 3 - should start at column 0");
    safe_println("Line 4 - should start at column 0");

    // Force some output that might cause issues
    safe_print("Mixed output: ");
    safe_print("no newline here");
    safe_println(" - but this should still align");

    safe_println("Final line - visual check complete");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    safe_println("=== Debug Terminal Detection ===");
    safe_println("This script will help us understand why we're detecting corruption");
    safe_println("when the terminal appears to be working fine.\n");

    // Test 1: Visual alignment (the real test)
    visual_alignment_test();

    // Test 2: Normal println behavior
    test_normal_println();

    // Test 3: Raw mode effects
    test_raw_mode_effects()?;

    // Test 4: Manual positioning
    test_manual_positioning()?;

    // Test 5: Our detection sequence
    test_detection_sequence()?;

    // Test 6: Simple detection
    safe_println("\n=== Simple Detection Test ===");
    let final_pos = simple_detect_corruption()?;
    safe_println(&format!("Final cursor position: {final_pos:?}"));

    safe_println("\n=== Conclusion ===");
    safe_println("Look at the output above:");
    safe_println("1. Do all the 'Line X' entries align at column 0?");
    safe_println("2. Does the cursor position after newline show column 0?");
    safe_println("3. If lines align but cursor position != 0, it's a detection bug");
    safe_println("4. If lines don't align, we have real corruption");

    Ok(())
}
