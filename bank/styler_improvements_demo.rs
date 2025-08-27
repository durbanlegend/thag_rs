/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Demo showcasing new Styler trait improvements
///
/// This demonstrates:
/// 1. Direct paint() method on Role: Role::Error.paint("text")
/// 2. Direct attribute chaining: Role::Info.bold().paint("text")
/// 3. Mixed chaining: Role::Warning.bold().italic().dim().paint("text")
/// 4. Comparison with old verbose syntax
//# Purpose: Demo new Styler trait direct methods and chaining
//# Categories: styling, ergonomics
use thag_styling::{cprtln, ColorInitStrategy, Role, Style, Styleable, Styler, TermAttributes};

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== Styler Improvements Demo ===\n");

    // Section 1: Direct paint() method
    println!("1. Direct paint() method on Role:");
    println!("   {}", Role::Error.paint("Direct error styling"));
    println!("   {}", Role::Success.paint("Direct success styling"));
    println!("   {}", Role::Warning.paint("Direct warning styling"));
    println!("   {}", Role::Info.paint("Direct info styling"));
    println!("   {}", Role::Code.paint("Direct code styling"));

    println!("\n   Compare with old verbose syntax:");
    println!(
        "   {}",
        Style::from(Role::Error).paint("Old verbose error styling")
    );

    // Section 2: Direct attribute chaining
    println!("\n2. Direct attribute chaining from Role:");
    println!("   {}", Role::Normal.bold().paint("Bold text"));
    println!("   {}", Role::Info.italic().paint("Italic text"));
    println!("   {}", Role::Warning.underline().paint("Underlined text"));
    println!("   {}", Role::Success.dim().paint("Dimmed text"));

    println!("\n   Compare with old chaining:");
    println!(
        "   {}",
        Style::from(Role::Normal).bold().paint("Old style bold")
    );

    // Section 3: Multiple attribute chaining
    println!("\n3. Multiple attribute chaining:");
    println!(
        "   {}",
        Role::Error.bold().italic().paint("Bold italic error")
    );
    println!(
        "   {}",
        Role::Info.bold().underline().paint("Bold underlined info")
    );
    println!(
        "   {}",
        Role::Success.italic().dim().paint("Italic dimmed success")
    );
    println!(
        "   {}",
        Role::Warning
            .bold()
            .italic()
            .underline()
            .paint("Triple styling warning")
    );

    // Section 4: Complex combinations
    println!("\n4. Complex styling combinations:");
    println!("   {}", Role::Heading1.bold().paint("Bold Heading 1"));
    println!("   {}", Role::Heading2.italic().paint("Italic Heading 2"));
    println!(
        "   {}",
        Role::Heading3.underline().paint("Underlined Heading 3")
    );
    println!("   {}", Role::Code.dim().paint("Dimmed code block"));
    println!(
        "   {}",
        Role::Emphasis
            .bold()
            .italic()
            .underline()
            .paint("Fully emphasized text")
    );

    // Section 5: Format integration
    println!("\n5. Integration with format! macro:");
    let value = 42;
    let name = "Alice";
    println!(
        "   {}",
        Role::Info.paint(format!("User {} has {} points", name, value))
    );
    println!(
        "   {}",
        Role::Success
            .bold()
            .paint(format!("Operation completed successfully for {}", name))
    );
    println!(
        "   {}",
        Role::Error
            .italic()
            .paint(format!("Error code: {:#04x}", value))
    );

    // Section 6: Side-by-side comparison
    println!("\n6. Side-by-side comparison - Old vs New:");
    println!(
        "   Old: {}",
        Style::from(Role::Warning)
            .bold()
            .italic()
            .paint("Warning message")
    );
    println!(
        "   New: {}",
        Role::Warning.bold().italic().paint("Warning message")
    );

    println!(
        "   Old: {}",
        Style::from(Role::Code)
            .dim()
            .underline()
            .paint("code_snippet()")
    );
    println!(
        "   New: {}",
        Role::Code.dim().underline().paint("code_snippet()")
    );

    // Section 7: Demonstrating return type (Style after chaining)
    println!("\n7. Chaining returns Style (can continue chaining):");
    let intermediate_style = Role::Normal.bold();
    println!(
        "   {}",
        intermediate_style
            .italic()
            .paint("Chained from intermediate")
    );

    // You can also continue chaining
    println!(
        "   {}",
        Role::Debug
            .bold()
            .italic()
            .underline()
            .dim()
            .paint("Maximum styling")
    );

    // Section 8: String extension trait (Styleable)
    println!("\n8. String extension trait (Styleable):");
    println!("   {}", "Direct string styling with role".error());
    println!(
        "   {}",
        "String styling with chained attributes".info().bold()
    );
    println!("   {}", format!("Formatted string: {}", 123).success());
    println!(
        "   {}",
        "Complex styling".warning().bold().italic().underline()
    );

    println!("\n   Compare string extension vs direct role:");
    println!("   String: {}", "message".code());
    println!("   Role:   {}", Role::Code.paint("message"));

    // Section 9: Different string types with Styleable:
    println!("\n9. Different string types with Styleable:");
    let owned_string = String::from("Owned string");
    println!("   {}", owned_string.emphasis());

    let string_ref = "String reference";
    println!("   {}", string_ref.debug().dim());

    let formatted = format!("Formatted: {}", "value");
    println!("   {}", formatted.info());

    println!("\n=== Demo Complete ===");
    println!("\nKey improvements:");
    cprtln!(
        Role::Success,
        "✓ Role::Error.paint('text') instead of Style::from(Role::Error).paint('text')"
    );
    cprtln!(
        Role::Success,
        "✓ Role::Info.bold().paint('text') - direct chaining from Role"
    );
    cprtln!(
        Role::Success,
        "✓ Chaining returns Style, allowing continued attribute chaining"
    );
    cprtln!(
        Role::Success,
        "✓ String extensions: 'text'.error() and 'text'.as_styled(Role::Info.bold())"
    );
    cprtln!(
        Role::Success,
        "✓ Much more concise and readable than previous verbose syntax"
    );
    cprtln!(
        Role::Info,
        "✓ Backward compatible - old Style::from(Role::X) syntax still works"
    );
}
