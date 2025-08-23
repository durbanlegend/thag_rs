/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Demo showcasing improved Styleable trait with individual role methods
///
/// This demonstrates:
/// 1. Consolidated style_with() method that works with any Styler
/// 2. Individual role methods: .error(), .success(), .info(), etc.
/// 3. Comparison with Role.paint() approach
/// 4. Using &self instead of self (non-consuming)
//# Purpose: Demo improved Styleable trait with role methods
//# Categories: styling, ergonomics
use thag_styling::{cprtln, ColorInitStrategy, Role, Styleable, Styler, TermAttributes};

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== Styleable Improvements Demo ===\n");

    // Section 1: Consolidated style_with() method
    println!("1. Consolidated style_with() method:");
    println!("   {}", "Error message".style_with(Role::Error));
    println!("   {}", "Success message".style_with(Role::Success));
    println!("   {}", "Warning message".style_with(Role::Warning));
    println!("   {}", "Info message".style_with(Role::Info));

    println!("\n   With chained attributes:");
    println!("   {}", "Bold error".style_with(Role::Error.bold()));
    println!("   {}", "Italic info".style_with(Role::Info.italic()));
    println!("   {}", "Underlined warning".style_with(Role::Warning.underline()));

    // Section 2: Individual role methods
    println!("\n2. Individual role methods (like colored's color methods):");
    println!("   {}", "Error message".error());
    println!("   {}", "Success message".success());
    println!("   {}", "Warning message".warning());
    println!("   {}", "Info message".info());
    println!("   {}", "Code snippet".code());
    println!("   {}", "Emphasized text".emphasis());
    println!("   {}", "Normal text".normal());
    println!("   {}", "Subtle text".subtle());
    println!("   {}", "Hint text".hint());
    println!("   {}", "Debug info".debug());
    println!("   {}", "Trace data".trace());

    // Section 3: Heading methods
    println!("\n3. Heading methods:");
    println!("   {}", "Main Heading".heading1());
    println!("   {}", "Sub Heading".heading2());
    println!("   {}", "Minor Heading".heading3());

    // Section 4: Comparison with Role.paint()
    println!("\n4. Comparison - String method vs Role method:");
    println!("   String: {}", "message".error());
    println!("   Role:   {}", Role::Error.paint("message"));

    println!("   String: {}", "code".code());
    println!("   Role:   {}", Role::Code.paint("code"));

    // Section 5: Non-consuming (&self) - reuse strings
    println!("\n5. Non-consuming methods (can reuse strings):");
    let message = "Important notification";
    println!("   As error:   {}", message.error());
    println!("   As warning: {}", message.warning());
    println!("   As success: {}", message.success());
    println!("   Original:   {}", message); // Still available!

    // Section 6: Working with different string types
    println!("\n6. Different string types:");
    let owned = String::from("Owned string");
    println!("   Owned:     {}", owned.info());

    let borrowed = "Borrowed string";
    println!("   Borrowed:  {}", borrowed.success());

    let formatted = format!("Formatted: {}", 42);
    println!("   Formatted: {}", formatted.warning());

    // Section 7: Complex styling with style_with
    println!("\n7. Complex styling with style_with:");
    println!("   {}", "Bold italic error".style_with(Role::Error.bold().italic()));
    println!("   {}", "Underlined dim info".style_with(Role::Info.underline().dim()));
    println!("   {}", "All attributes".style_with(Role::Warning.bold().italic().underline().dim()));

    // Section 8: Practical examples
    println!("\n8. Practical examples:");
    let user = "Alice";
    let count = 42;
    println!("   {}", format!("User {} logged in", user).success());
    println!("   {}", format!("Found {} items", count).info());
    println!("   {}", format!("Failed to connect to server").error());
    println!("   {}", format!("cargo build --release").code());

    // Section 9: Comparison with old verbose approach
    println!("\n9. Comparison with old verbose approach:");
    println!("   Old verbose: {}", Role::Error.paint("Error message"));
    println!("   New concise: {}", "Error message".error());

    println!("   Old with formatting: {}", Role::Success.paint(format!("Done: {}", 100)));
    println!("   New with formatting: {}", format!("Done: {}", 100).success());

    println!("\n=== Demo Complete ===");
    println!("\nKey improvements:");
    cprtln!(Role::Success, "✓ Single style_with() method works with any Styler (Role or Style)");
    cprtln!(Role::Success, "✓ Individual role methods: .error(), .success(), .info(), etc.");
    cprtln!(Role::Success, "✓ Non-consuming (&self) - can reuse strings without moving them");
    cprtln!(Role::Success, "✓ Very concise: 'text'.error() vs Role::Error.paint('text')");
    cprtln!(Role::Success, "✓ Familiar API similar to colored crate but with thag_styling power");
    cprtln!(Role::Info, "✓ Works with all string types: &str, String, format!() results");
}
