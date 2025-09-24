/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
*/

//! Debug styled! duplication issue
//!
//! Minimal test to isolate the double-printing problem

//# Purpose: Debug styled! macro duplication issue
//# Categories: debug, test, styling

use thag_proc_macros::{ansi_styling_support, styled};

// Enable the ANSI styling support
ansi_styling_support! {}

fn main() {
    println!("=== Debug styled! Issue ===\n");

    // Test basic color (should work correctly)
    println!("Basic Red: {}", styled!("test", fg = Red));

    // Test 256-color (showing duplication)
    println!("256-color: {}", styled!("test", fg = Color256(196)));

    // Test RGB (showing duplication)
    println!("RGB color: {}", styled!("test", fg = Rgb(255, 0, 0)));

    // Test hex (showing duplication)
    println!("Hex color: {}", styled!("test", fg = "#ff0000"));

    println!("\n=== Raw Debug ===");

    // Let's see what the styled objects actually contain
    let basic = styled!("test", fg = Red);
    let color256 = styled!("test", fg = Color256(196));
    let rgb = styled!("test", fg = Rgb(255, 0, 0));

    println!("Basic object: {:?}", format!("{}", basic));
    println!("256-color object: {:?}", format!("{}", color256));
    println!("RGB object: {:?}", format!("{}", rgb));
}
