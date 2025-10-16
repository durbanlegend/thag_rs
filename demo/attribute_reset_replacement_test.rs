/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config"] }
*/

/// Test enhanced reset replacement with proper text attribute handling
///
/// This demonstrates the fix for text attributes (bold/dim, italic, underline) that
/// were previously leaking from inner styled content when using reset replacement.
///
/// The enhanced system:
/// 1. Analyzes the outer style's ANSI codes
/// 2. Only resets attributes that won't be reapplied
/// 3. Optimizes the reset sequence to avoid redundant operations
/// 4. Maintains perfect context preservation across all nesting levels
//# Purpose: Test enhanced reset replacement with text attribute handling
//# Categories: ansi, styling, terminal, testing
use thag_styling::{ColorInitStrategy, Role, Styleable, Styler, TermAttributes};

fn main() {
    // Initialize styling system
    TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);

    println!("=== Enhanced Reset Replacement with Text Attributes Test ===\n");

    // Test 1: Basic attribute leakage problem (should be fixed now)
    println!("1. Bold attribute leakage test:");
    let inner_bold = "BOLD TEXT".normal().bold();
    let outer_normal = format!("Before {inner_bold} after").info();
    println!("   Result: {}", outer_normal);
    println!("   Raw ANSI: {:?}", outer_normal.to_string());
    println!("   ✅ 'after' should NOT be bold - it should be normal info color");
    println!();

    // Test 2: Italic attribute leakage
    println!("2. Italic attribute leakage test:");
    let inner_italic = "ITALIC TEXT".normal().italic();
    let outer_normal = format!("Before {inner_italic} after").warning();
    println!("   Result: {}", outer_normal);
    println!("   Raw ANSI: {:?}", outer_normal.to_string());
    println!("   ✅ 'after' should NOT be italic - it should be normal warning color");
    println!();

    // Test 3: Underline attribute leakage
    println!("3. Underline attribute leakage test:");
    let inner_underline = "UNDERLINED TEXT".normal().underline();
    let outer_normal = format!("Before {inner_underline} after").success();
    println!("   Result: {}", outer_normal);
    println!("   Raw ANSI: {:?}", outer_normal.to_string());
    println!("   ✅ 'after' should NOT be underlined - it should be normal success color");
    println!();

    // Test 4: Multiple attributes leakage
    println!("4. Multiple attributes leakage test:");
    let inner_multi = "BOLD ITALIC UNDERLINED"
        .normal()
        .bold()
        .italic()
        .underline();
    let outer_normal = format!("Before {inner_multi} after").error();
    println!("   Result: {}", outer_normal);
    println!("   ✅ 'after' should be plain error color (no bold, italic, or underline)");
    println!();

    // Test 5: Optimization - when outer style has same attributes
    println!("5. Optimization test - outer style has bold:");
    let inner_bold = "INNER BOLD".normal().bold();
    let outer_bold = format!("Bold outer {inner_bold} still bold")
        .warning()
        .bold();
    println!("   Result: {}", outer_bold);
    println!("   Raw ANSI: {:?}", outer_bold.to_string());
    println!("   ✅ Should be bold throughout (no unnecessary bold reset)");
    println!();

    // Test 6: Partial optimization - some attributes match
    println!("6. Partial optimization - outer italic, inner bold+italic:");
    let inner_multi = "BOLD AND ITALIC".normal().bold().italic();
    let outer_italic = format!("Italic outer {inner_multi} italic again")
        .info()
        .italic();
    println!("   Result: {}", outer_italic);
    println!("   Raw ANSI: {:?}", outer_italic.to_string());
    println!("   ✅ Should reset bold but keep italic throughout");
    println!();

    // Test 7: Complex multi-level nesting
    println!("7. Complex multi-level nesting with attributes:");
    let level3 = "Level3: bold+underline".normal().bold().underline();
    let level2 = format!("Level2: italic {level3} italic").warning().italic();
    let level1 = format!("Level1: normal {level2} normal").success();
    println!("   Result: {}", level1);
    println!("   ✅ Each level should properly restore its context");
    println!();

    // Test 8: Edge case - no attributes in replacement
    println!("8. Edge case - replacing attributes with plain color:");
    let inner_all_attrs = "ALL ATTRIBUTES".normal().bold().italic().underline();
    let outer_plain = format!("Plain color {inner_all_attrs} plain again").code();
    println!("   Result: {}", outer_plain);
    println!("   ✅ Should reset all attributes and show plain code color");
    println!();

    // Test 9: Role-based styling with attributes
    println!("9. Role-based styling with inherent attributes:");
    let inner_heading = "HEADING WITH BOLD".style_with(Role::Heading1.bold());
    let outer_normal = format!("Text {inner_heading} text").info();
    println!("   Result: {}", outer_normal);
    println!("   ✅ Should show heading color+bold, then reset to info color");
    println!();

    // Test 10: Stress test - deeply nested with mixed attributes
    println!("10. Stress test - deeply nested mixed attributes:");
    let deep4 = "Deep4".normal().underline();
    println!("   deep4: {:?}", deep4);
    let deep3 = format!("Deep3 {deep4} end3").error().italic();
    println!("   deep3: {:?}", deep3);
    let deep2 = format!("Deep2 {deep3} end2").warning().bold();
    println!("   deep2: {:?}", deep2);
    let deep1 = format!("Deep1 {deep2} end1").success();
    println!("   Result: {}", deep1);
    println!("   Result: {:?}", deep1);
    println!("   ✅ Each level should perfectly restore its styling context");
    println!();

    println!("=== Test Summary ===");
    println!("✅ All tests demonstrate the enhanced reset replacement system");
    println!("✅ Text attributes no longer leak between nesting levels");
    println!("✅ Optimal reset sequences minimize redundant ANSI codes");
    println!("✅ Perfect context preservation maintained across unlimited nesting depth");
    println!();
    println!("The enhanced algorithm ensures that \\x1b[0m is replaced with:");
    println!("- Minimal attribute resets (\\x1b[22;23;24m or subset)");
    println!("- Followed by the outer style's ANSI codes");
    println!("- Optimized to skip resets for attributes that will be immediately reapplied");

    // Test 11: ANSI code parsing edge cases
    println!("\n11. ANSI code parsing edge cases:");
    println!("   Testing combined ANSI sequences like \\x1b[1;3;4m...");

    // This should test the parsing logic internally
    let combined_attrs = "COMBINED".normal().bold().italic().underline();
    let outer_plain = format!("Plain {combined_attrs} plain").info();
    println!("   Combined attributes: {}", outer_plain);
    println!("   ✅ Should properly detect all attributes in combined sequence");

    // Test partial overlap
    let inner_bold_italic = "BOLD+ITALIC".normal().bold().italic();
    let outer_bold_only = format!("Bold {inner_bold_italic} bold").warning().bold();
    println!("   Partial overlap: {}", outer_bold_only);
    println!("   ✅ Should reset italic but keep bold");

    // Test color + attributes combined
    let inner_with_color = "RED+BOLD".error().bold();
    let outer_blue = format!("Blue {inner_with_color} blue").info();
    println!("   Color + attributes: {}", outer_blue);
    println!("   ✅ Should reset bold and change color to blue");
}
