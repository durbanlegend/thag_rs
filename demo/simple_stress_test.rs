/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config"] }
*/

/// Simple stress test with raw ANSI inspection
///
/// This recreates the stress test scenario and shows the raw ANSI codes
/// to verify if the reset replacement is working correctly.
//# Purpose: Debug stress test with raw ANSI inspection
//# Categories: styling, debugging, testing
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

    println!("=== Simple Stress Test with Raw ANSI ===\n");

    // Recreate the exact stress test scenario
    let deep4 = "Deep4".normal().underline();
    let deep3 = format!("Deep3 {} end3", deep4).error().italic();
    let deep2 = format!("Deep2 {} end2", deep3).warning().bold();
    let deep1 = format!("Deep1 {} end1", deep2).success();

    println!("Final visual result:");
    println!("{}", deep1);
    println!();

    println!("Raw ANSI codes:");
    println!("{}", escape_ansi(&deep1.to_string()));
    println!();

    println!("Expected styling:");
    println!("- Deep1: success color (no attributes)");
    println!("- Deep2: warning color + bold");
    println!("- Deep3: error color + italic (NO bold)");
    println!("- Deep4: normal color + underline");
    println!("- end3: error color + italic (NO bold)");
    println!("- end2: warning color + bold");
    println!("- end1: success color (no attributes)");
    println!();

    println!("Analysis:");
    println!("If Deep3 shows bold, it means attributes are bleeding forward.");
    println!("If end3 shows bold, it means reset replacement isn't working.");
    println!("The raw ANSI codes above will show exactly what's happening.");
}
