/*[toml]
[dependencies]
crossterm = "0.28"
*/

/// Simple diagnostic script to debug the terminal corruption detection
//# Purpose: Debug why we're getting false positives in corruption detection
//# Categories: debugging, diagnosis, terminal
use crossterm::{
    cursor::position,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{stdout, Write};

/// Simple corruption detection without synchronization
fn simple_detect_corruption() -> Result<(u16, u16), Box<dyn std::error::Error>> {
    println!("Starting simple corruption detection...");

    enable_raw_mode()?;

    // Clear any pending output
    stdout().flush()?;

    // Move to a known position first
    print!("\x1b[10;1H"); // Move to row 10, column 1
    stdout().flush()?;

    // Print test text
    print!("SIMPLE_TEST");
    stdout().flush()?;

    // Get position after text
    let pos_after_text = position()?;
    println!("Position after text: {:?}", pos_after_text);

    // Send newline
    print!("\n");
    stdout().flush()?;

    // Get position after newline
    let pos_after_newline = position()?;
    println!("Position after newline: {:?}", pos_after_newline);

    disable_raw_mode()?;

    Ok(pos_after_newline)
}

/// Test what happens with normal println
fn test_normal_println() {
    println!("=== Testing normal println behavior ===");
    println!("Line 1");
    println!("Line 2");
    println!("Line 3");
    println!("These should all align at column 0");
}

/// Test manual cursor positioning
fn test_manual_positioning() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing manual cursor positioning ===");

    enable_raw_mode()?;

    // Move to column 20
    print!("\x1b[20G");
    stdout().flush()?;

    let pos1 = position()?;
    println!("Position at column 20: {:?}", pos1);

    // Print text
    print!("TEXT_AT_COL_20");
    stdout().flush()?;

    let pos2 = position()?;
    println!("Position after text: {:?}", pos2);

    // Send newline
    print!("\n");
    stdout().flush()?;

    let pos3 = position()?;
    println!("Position after newline: {:?}", pos3);

    disable_raw_mode()?;

    // The key question: is pos3.0 == 0?
    if pos3.0 == 0 {
        println!("✅ Newline correctly returned to column 0");
    } else {
        println!(
            "❌ Newline failed to return to column 0 (at column {})",
            pos3.0
        );
    }

    Ok(())
}

/// Test raw mode behavior
fn test_raw_mode_effects() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing raw mode effects ===");

    println!("Before raw mode:");
    println!("Line A");
    println!("Line B");

    enable_raw_mode()?;

    print!("In raw mode - this is line C\n");
    stdout().flush()?;

    print!("In raw mode - this is line D\n");
    stdout().flush()?;

    disable_raw_mode()?;

    println!("After raw mode:");
    println!("Line E");
    println!("Line F");

    Ok(())
}

/// Test the specific sequence that our detection function uses
fn test_detection_sequence() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing our detection sequence ===");

    // This is exactly what our detection function does
    enable_raw_mode()?;

    stdout().flush()?;

    print!("TEST_CORRUPTION_DETECTION");
    stdout().flush()?;

    let pos_before_newline = position()?;
    println!("Position before newline: {:?}", pos_before_newline);

    print!("\n");
    stdout().flush()?;

    let pos_after_newline = position()?;
    println!("Position after newline: {:?}", pos_after_newline);

    disable_raw_mode()?;

    // Analysis
    println!("Analysis:");
    println!("- Text was at column {}", pos_before_newline.0);
    println!("- After newline, cursor at column {}", pos_after_newline.0);

    if pos_after_newline.0 == 0 {
        println!("✅ ONLCR working correctly");
    } else {
        println!("❌ ONLCR not working - cursor should be at column 0");

        // But let's check if it's really broken
        println!("Let's verify with a visual test:");
        print!("Does this text align properly? ");
        println!("It should be at column 0.");
    }

    Ok(())
}

/// Visual alignment test
fn visual_alignment_test() {
    println!("\n=== Visual Alignment Test ===");
    println!("If terminal corruption is real, these lines won't align:");
    println!("Line 1 - should start at column 0");
    println!("Line 2 - should start at column 0");
    println!("Line 3 - should start at column 0");
    println!("Line 4 - should start at column 0");

    // Force some output that might cause issues
    print!("Mixed output: ");
    print!("no newline here");
    println!(" - but this should still align");

    println!("Final line - visual check complete");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Debug Terminal Detection ===");
    println!("This script will help us understand why we're detecting corruption");
    println!("when the terminal appears to be working fine.\n");

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
    println!("\n=== Simple Detection Test ===");
    let final_pos = simple_detect_corruption()?;
    println!("Final cursor position: {:?}", final_pos);

    println!("\n=== Conclusion ===");
    println!("Look at the output above:");
    println!("1. Do all the 'Line X' entries align at column 0?");
    println!("2. Does the cursor position after newline show column 0?");
    println!("3. If lines align but cursor position != 0, it's a detection bug");
    println!("4. If lines don't align, we have real corruption");

    Ok(())
}
