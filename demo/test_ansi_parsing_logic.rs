/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config"] }
*/

/// Simple test for ANSI parsing logic
///
/// This tests the has_ansi_code function directly to verify it correctly
/// distinguishes between color codes and text attributes.
//# Purpose: Test ANSI parsing logic for text attributes
//# Categories: styling, testing, parsing
use thag_styling::{ColorInitStrategy, TermAttributes};

// We need to access the StyledString implementation directly
// Since the methods are private, we'll create a public wrapper for testing
struct TestStyledString {
    content: String,
    style_codes: String,
}

impl TestStyledString {
    fn new(style_codes: &str) -> Self {
        Self {
            content: String::new(),
            style_codes: style_codes.to_string(),
        }
    }

    // Copy of the private has_ansi_code method for testing
    fn has_ansi_code(ansi_string: &str, codes: &[u8]) -> bool {
        // Find all ANSI escape sequences
        let mut pos = 0;
        while let Some(esc_start) = ansi_string[pos..].find("\x1b[") {
            let abs_start = pos + esc_start + 2; // Skip "\x1b["
            if let Some(m_pos) = ansi_string[abs_start..].find('m') {
                let codes_str = &ansi_string[abs_start..abs_start + m_pos];

                if Self::contains_text_attributes(codes_str, codes) {
                    return true;
                }

                pos = abs_start + m_pos + 1;
            } else {
                break;
            }
        }
        false
    }

    // Copy of the private contains_text_attributes method for testing
    fn contains_text_attributes(codes_str: &str, target_codes: &[u8]) -> bool {
        let parts: Vec<&str> = codes_str.split(';').collect();
        let mut i = 0;

        while i < parts.len() {
            if let Ok(code) = parts[i].parse::<u8>() {
                match code {
                    // Handle color codes that consume additional parameters
                    38 | 48 => {
                        // Foreground (38) or background (48) color
                        if i + 1 < parts.len() {
                            if let Ok(color_type) = parts[i + 1].parse::<u8>() {
                                match color_type {
                                    2 => {
                                        // RGB color: 38;2;R;G;B or 48;2;R;G;B
                                        i += 5; // Skip 38/48, 2, R, G, B
                                    }
                                    5 => {
                                        // 256-color: 38;5;N or 48;5;N
                                        i += 3; // Skip 38/48, 5, N
                                    }
                                    _ => i += 1,
                                }
                            } else {
                                i += 1;
                            }
                        } else {
                            i += 1;
                        }
                    }
                    // Text attributes we're looking for
                    _ => {
                        if target_codes.contains(&code) {
                            return true;
                        }
                        i += 1;
                    }
                }
            } else {
                i += 1;
            }
        }

        false
    }

    fn test_has_bold(&self) -> bool {
        Self::has_ansi_code(&self.style_codes, &[1, 2])
    }

    fn test_has_italic(&self) -> bool {
        Self::has_ansi_code(&self.style_codes, &[3])
    }

    fn test_has_underline(&self) -> bool {
        Self::has_ansi_code(&self.style_codes, &[4])
    }
}

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== ANSI Parsing Logic Test ===\n");

    // Test 1: RGB color should NOT be detected as text attributes
    println!("1. RGB Color Tests:");
    let rgb_color = TestStyledString::new("\x1b[38;2;202;97;101m");
    println!("   RGB color code: {:?}", rgb_color.style_codes);
    println!(
        "   Has bold: {} (should be false)",
        rgb_color.test_has_bold()
    );
    println!(
        "   Has italic: {} (should be false)",
        rgb_color.test_has_italic()
    );
    println!(
        "   Has underline: {} (should be false)",
        rgb_color.test_has_underline()
    );
    println!();

    // Test 2: Simple text attributes should be detected
    println!("2. Simple Text Attribute Tests:");
    let bold_only = TestStyledString::new("\x1b[1m");
    println!("   Bold code: {:?}", bold_only.style_codes);
    println!(
        "   Has bold: {} (should be true)",
        bold_only.test_has_bold()
    );
    println!(
        "   Has italic: {} (should be false)",
        bold_only.test_has_italic()
    );
    println!(
        "   Has underline: {} (should be false)",
        bold_only.test_has_underline()
    );
    println!();

    let italic_only = TestStyledString::new("\x1b[3m");
    println!("   Italic code: {:?}", italic_only.style_codes);
    println!(
        "   Has bold: {} (should be false)",
        italic_only.test_has_bold()
    );
    println!(
        "   Has italic: {} (should be true)",
        italic_only.test_has_italic()
    );
    println!(
        "   Has underline: {} (should be false)",
        italic_only.test_has_underline()
    );
    println!();

    // Test 3: Combined attributes
    println!("3. Combined Attribute Tests:");
    let combined = TestStyledString::new("\x1b[1;3;4m");
    println!("   Combined code: {:?}", combined.style_codes);
    println!("   Has bold: {} (should be true)", combined.test_has_bold());
    println!(
        "   Has italic: {} (should be true)",
        combined.test_has_italic()
    );
    println!(
        "   Has underline: {} (should be true)",
        combined.test_has_underline()
    );
    println!();

    // Test 4: RGB color + attributes
    println!("4. RGB Color + Attribute Tests:");
    let rgb_plus_bold = TestStyledString::new("\x1b[38;2;255;0;0m\x1b[1m");
    println!("   RGB + Bold codes: {:?}", rgb_plus_bold.style_codes);
    println!(
        "   Has bold: {} (should be true)",
        rgb_plus_bold.test_has_bold()
    );
    println!(
        "   Has italic: {} (should be false)",
        rgb_plus_bold.test_has_italic()
    );
    println!(
        "   Has underline: {} (should be false)",
        rgb_plus_bold.test_has_underline()
    );
    println!();

    // Test 5: Edge case - mixed in one sequence
    println!("5. Mixed Sequence Tests:");
    let mixed = TestStyledString::new("\x1b[38;2;100;200;50;1;3m");
    println!("   Mixed sequence: {:?}", mixed.style_codes);
    println!("   Has bold: {} (should be true)", mixed.test_has_bold());
    println!(
        "   Has italic: {} (should be true)",
        mixed.test_has_italic()
    );
    println!(
        "   Has underline: {} (should be false)",
        mixed.test_has_underline()
    );
    println!();

    // Test 6: The problematic case from debug output
    println!("6. Problematic Case:");
    let problematic = TestStyledString::new("\x1b[38;2;0;188;188m");
    println!("   Cyan RGB: {:?}", problematic.style_codes);
    println!(
        "   Has bold: {} (should be false - this was the bug!)",
        problematic.test_has_bold()
    );
    println!(
        "   Has italic: {} (should be false)",
        problematic.test_has_italic()
    );
    println!(
        "   Has underline: {} (should be false)",
        problematic.test_has_underline()
    );
    println!();

    println!("=== Test Summary ===");
    println!("✅ RGB color codes should not be mistaken for text attributes");
    println!("✅ Text attributes (1-4) should be correctly detected");
    println!("✅ Combined sequences should work properly");
    println!("✅ The parsing logic should handle complex ANSI sequences");
}
