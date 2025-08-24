/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }
*/

/// Simple test to see if reset replacement is being called
///
/// This creates the exact scenario from the stress test to see where
/// the reset replacement logic is failing.
//# Purpose: Debug reset replacement execution
//# Categories: styling, debugging, testing
use thag_styling::{ColorInitStrategy, Styleable, TermAttributes};

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== Simple Reset Test ===\n");

    // Create the exact scenario: inner styled content + outer styling
    println!("Step 1: Create inner styled content");
    let inner = "Deep4".normal().underline();
    println!("Inner result: {:?}", inner.to_string());
    println!();

    println!("Step 2: Embed in format string");
    let content_with_inner = format!("Deep3 {} end3", inner);
    println!("Content with inner: {:?}", content_with_inner);
    println!();

    println!("Step 3: Apply outer styling");
    let outer_styled = content_with_inner.error().italic();
    println!("Final result: {:?}", outer_styled.to_string());
    println!();

    println!("Expected: The \\x1b[0m after Deep4 should be replaced with:");
    println!("\\x1b[22;23;24m\\x1b[error_color]\\x1b[3m");
    println!();

    println!("If you see DEBUG output above, the replacement is being called.");
    println!("If not, there's a bug in when/how the replacement is invoked.");
}
