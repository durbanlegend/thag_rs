/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }
*/

/// Minimal test to debug the stress test nesting issue
///
/// This isolates the specific problem where "end3" loses its bold attribute
/// in deeply nested styling scenarios.
//# Purpose: Debug deeply nested attribute restoration issue
//# Categories: styling, debugging, nesting
use thag_styling::{ColorInitStrategy, Styleable, TermAttributes};

fn escape_ansi(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c == '\x1b' {
                "\\x1b".to_string()
            } else {
                c.to_string()
            }
        })
        .collect::<String>()
}

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== Debug Stress Test Nesting Issue ===\n");

    // Step-by-step reconstruction of the failing case
    println!("Building up the nested structure step by step:\n");

    // Level 4: Underlined text
    let deep4 = "Deep4".normal().underline();
    println!("Step 1 - deep4 (underline):");
    println!("   Visual: {}", deep4);
    println!("   Raw: {}", escape_ansi(&deep4.to_string()));
    println!();

    // Level 3: Error + italic containing deep4
    let deep3_content = format!("Deep3 {deep4} end3");
    println!("Step 2 - deep3 content before styling:");
    println!("   Visual: {}", deep3_content);
    println!("   Raw: {}", escape_ansi(&deep3_content));
    println!();

    let deep3 = deep3_content.error().italic();
    println!("Step 3 - deep3 after error().italic():");
    println!("   Visual: {}", deep3);
    println!("   Raw: {}", escape_ansi(&deep3.to_string()));
    println!("   ❓ Does 'end3' have correct error+italic styling?");
    println!();

    // Level 2: Warning + bold containing deep3
    let deep2_content = format!("Deep2 {deep3} end2");
    println!("Step 4 - deep2 content before styling:");
    println!("   Visual: {}", deep2_content);
    println!("   Raw: {}", escape_ansi(&deep2_content));
    println!();

    let deep2 = deep2_content.warning().bold();
    println!("Step 5 - deep2 after warning().bold():");
    println!("   Visual: {}", deep2);
    println!("   Raw: {}", escape_ansi(&deep2.to_string()));
    println!("   ❓ Does 'end2' have correct warning+bold styling?");
    println!("   ❓ Does 'end3' still have correct error+italic styling?");
    println!();

    // Level 1: Success (no attributes) containing deep2
    let deep1_content = format!("Deep1 {deep2} end1");
    println!("Step 6 - deep1 content before styling:");
    println!("   Visual: {}", deep1_content);
    println!("   Raw: {}", escape_ansi(&deep1_content));
    println!();

    let deep1 = deep1_content.success();
    println!("Step 7 - deep1 after success():");
    println!("   Visual: {}", deep1);
    println!("   Raw: {}", escape_ansi(&deep1.to_string()));
    println!("   ❓ Does 'end1' have correct success color (no attributes)?");
    println!("   ❓ Does 'end2' still have correct warning+bold styling?");
    println!("   ❓ Does 'end3' still have correct error+italic styling?");
    println!();

    println!("=== Analysis ===");
    println!("The issue appears to be in the reset replacement logic.");
    println!("When multiple levels of nesting occur, the reset sequences");
    println!("may not be correctly preserving all the outer context attributes.");
    println!();
    println!("Expected behavior:");
    println!("- Deep4: underlined");
    println!("- end3: error color + italic (NO bold, NO underline)");
    println!("- end2: warning color + bold (NO italic, NO underline)");
    println!("- end1: success color (NO attributes)");
    println!();

    // Additional focused test
    println!("=== Focused Test: Two-level nesting ===");
    let inner = "INNER".normal().underline();
    let middle = format!("middle {inner} middle").error().italic();
    let outer = format!("outer {middle} outer").warning().bold();

    println!("Two-level result: {}", outer);
    println!("Raw: {}", escape_ansi(&outer.to_string()));
    println!("Expected: outer(warning+bold) middle(error+italic) INNER(normal+underline) middle(error+italic) outer(warning+bold)");
}
