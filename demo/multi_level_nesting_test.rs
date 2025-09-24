/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config"] }
*/

/// Test multi-level nesting behavior with different reset handling approaches
///
/// This demonstrates the difference between:
/// 1. Insert approach (like colored): Inserts outer codes after each reset
/// 2. Replace approach: Replaces inner codes with outer codes
/// 3. Manual construction to show what should happen
//# Purpose: Test multi-level nesting reset handling strategies
//# Categories: prototype, styling
use std::fmt;
use thag_styling::{ColorInitStrategy, Role, Style, TermAttributes};

/// Version 1: Insert approach (mimics colored)
#[derive(Clone, Debug)]
pub struct InsertStyledString {
    content: String,
    style: Style,
}

impl InsertStyledString {
    pub fn new(content: String, style: Style) -> Self {
        Self { content, style }
    }

    fn escape_inner_resets_insert(&self) -> String {
        let reset = "\x1b[0m";
        let outer_codes = self.style.to_ansi_codes();

        if !self.content.contains(reset) {
            return self.content.clone();
        }

        let reset_positions: Vec<usize> = self
            .content
            .match_indices(reset)
            .map(|(idx, _)| idx)
            .collect();

        let mut result = self.content.clone();

        // Insert outer codes after each reset (reverse order to maintain indices)
        for &pos in reset_positions.iter().rev() {
            let insert_pos = pos + reset.len();
            result.insert_str(insert_pos, &outer_codes);
        }

        result
    }

    pub fn to_styled(&self) -> String {
        let escaped_content = self.escape_inner_resets_insert();
        self.style.paint(escaped_content)
    }

    pub fn error(content: &str) -> Self {
        Self::new(content.to_string(), Style::from(Role::Error))
    }

    pub fn info(content: &str) -> Self {
        Self::new(content.to_string(), Style::from(Role::Info))
    }

    pub fn success(content: &str) -> Self {
        Self::new(content.to_string(), Style::from(Role::Success))
    }
}

/// Version 2: Replace approach (your suggested improvement)
#[derive(Clone, Debug)]
pub struct ReplaceStyledString {
    content: String,
    style: Style,
}

impl ReplaceStyledString {
    pub fn new(content: String, style: Style) -> Self {
        Self { content, style }
    }

    fn escape_inner_resets_replace(&self) -> String {
        let reset = "\x1b[0m";
        let outer_codes = self.style.to_ansi_codes();

        if !self.content.contains(reset) {
            return self.content.clone();
        }

        // Find patterns like "\x1b[0m\x1b[...m" and replace the codes after reset
        let mut result = self.content.clone();

        // Simple regex-like replacement: find reset followed by ANSI codes
        let mut pos = 0;
        while let Some(reset_pos) = result[pos..].find(reset) {
            let absolute_reset_pos = pos + reset_pos;
            let after_reset_pos = absolute_reset_pos + reset.len();

            // Look for ANSI codes after the reset
            let remaining = &result[after_reset_pos..];
            let mut ansi_end = after_reset_pos;

            // Find the end of consecutive ANSI escape sequences
            let mut chars = remaining.chars();
            while let Some(ch) = chars.next() {
                if ch == '\x1b' {
                    // Skip the ANSI sequence
                    ansi_end += 1; // \x1b
                    if chars.next() == Some('[') {
                        ansi_end += 1; // [
                                       // Skip until 'm'
                        for next_ch in chars.by_ref() {
                            ansi_end += next_ch.len_utf8();
                            if next_ch == 'm' {
                                break;
                            }
                        }
                    }
                } else {
                    break;
                }
            }

            // Replace the ANSI codes after reset with our outer codes
            if ansi_end > after_reset_pos {
                result.replace_range(after_reset_pos..ansi_end, &outer_codes);
                pos = after_reset_pos + outer_codes.len();
            } else {
                // No ANSI codes after reset, just insert ours
                result.insert_str(after_reset_pos, &outer_codes);
                pos = after_reset_pos + outer_codes.len();
            }
        }

        result
    }

    pub fn to_styled(&self) -> String {
        let escaped_content = self.escape_inner_resets_replace();
        self.style.paint(escaped_content)
    }

    pub fn error(content: &str) -> Self {
        Self::new(content.to_string(), Style::from(Role::Error))
    }

    pub fn info(content: &str) -> Self {
        Self::new(content.to_string(), Style::from(Role::Info))
    }

    pub fn success(content: &str) -> Self {
        Self::new(content.to_string(), Style::from(Role::Success))
    }
}

impl fmt::Display for InsertStyledString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_styled())
    }
}

impl fmt::Display for ReplaceStyledString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_styled())
    }
}

// Helper trait for Style
trait StyleAnsiExt {
    fn to_ansi_codes(&self) -> String;
}

impl StyleAnsiExt for Style {
    fn to_ansi_codes(&self) -> String {
        let mut codes = String::new();

        if let Some(color_info) = &self.foreground {
            let ansi = color_info.to_ansi_for_support(TermAttributes::get_or_init().color_support);
            codes.push_str(&ansi);
        }

        if self.bold {
            codes.push_str("\x1b[1m");
        }
        if self.italic {
            codes.push_str("\x1b[3m");
        }
        if self.dim {
            codes.push_str("\x1b[2m");
        }
        if self.underline {
            codes.push_str("\x1b[4m");
        }

        codes
    }
}

fn main() {
    // Initialize styling system
    TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);

    println!("=== Multi-Level Nesting Test ===\n");

    // Create a 3-level nesting scenario
    println!("1. INSERT APPROACH (like colored):");
    let inner_insert = InsertStyledString::error("RED");
    let middle_insert = InsertStyledString::info(&format!("blue {} blue", inner_insert));
    let outer_insert = InsertStyledString::success(&format!("green {} green", middle_insert));
    println!("   Result: {}", outer_insert);
    println!("   Raw: {:?}", outer_insert.to_styled());

    println!("\n2. REPLACE APPROACH (your suggestion):");
    let inner_replace = ReplaceStyledString::error("RED");
    let middle_replace = ReplaceStyledString::info(&format!("blue {} blue", inner_replace));
    let outer_replace = ReplaceStyledString::success(&format!("green {} green", middle_replace));
    println!("   Result: {}", outer_replace);
    println!("   Raw: {:?}", outer_replace.to_styled());

    println!("\n3. MANUAL CONSTRUCTION (what we expect):");
    // What we manually expect the result to be
    let manual_result = format!(
        "{}green {}blue {}RED{} blue{} green{}",
        Style::from(Role::Success).to_ansi_codes(), // Start green
        Style::from(Role::Info).to_ansi_codes(),    // Switch to blue
        Style::from(Role::Error).to_ansi_codes(),   // Switch to red
        "\x1b[0m",                                  // Reset after RED
        Style::from(Role::Info).to_ansi_codes(),    // Back to blue (middle level)
        "\x1b[0m",                                  // Reset after middle
                                                    // Final "green" should be in default color or we need outer green restored
    );
    println!("   Manual: {}", manual_result);

    println!("\n4. ANALYSIS:");
    println!("   - INSERT approach: Each level adds its codes after ALL resets");
    println!("   - REPLACE approach: Each level replaces inner codes with its own");
    println!("   - The question: Which produces the visually correct result?");

    println!("\n5. STEP-BY-STEP TRACE:");

    println!("   Step 1 - Inner string:");
    let step1 = InsertStyledString::error("RED");
    println!("     {:?}", step1.to_styled());

    println!("   Step 2 - Middle wraps inner (INSERT):");
    let step2_insert = InsertStyledString::info(&format!("blue {} blue", step1));
    println!("     {:?}", step2_insert.to_styled());

    println!("   Step 2 - Middle wraps inner (REPLACE):");
    let step1_replace = ReplaceStyledString::error("RED");
    let step2_replace = ReplaceStyledString::info(&format!("blue {} blue", step1_replace));
    println!("     {:?}", step2_replace.to_styled());

    println!("   Step 3 - Outer wraps middle (INSERT):");
    println!("     {:?}", outer_insert.to_styled());

    println!("   Step 3 - Outer wraps middle (REPLACE):");
    println!("     {:?}", outer_replace.to_styled());

    println!("\n6. EXPECTED VISUAL RESULT:");
    println!("   'green blue RED blue green'");
    println!("   Where:");
    println!("   - 'green' = Success color (first and last)");
    println!("   - 'blue' = Info color (middle level)");
    println!("   - 'RED' = Error color (innermost)");
}
