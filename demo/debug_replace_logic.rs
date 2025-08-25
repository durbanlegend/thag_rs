/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config"] }
*/

/// Debug the replace logic to find why colors are wrong and resets aren't removed
///
/// This demonstrates step-by-step what the replace logic is doing wrong
/// and provides a corrected implementation
//# Purpose: Debug and fix replace logic for multi-level nesting
//# Categories: styling, debugging, prototypes
use std::fmt;
use thag_styling::{ColorInitStrategy, Role, Style, Styler, TermAttributes};

#[derive(Clone, Debug)]
pub struct DebugStyledString {
    content: String,
    style: Style,
    debug_name: String,
}

impl DebugStyledString {
    pub fn new(content: String, style: Style, debug_name: String) -> Self {
        Self {
            content,
            style,
            debug_name,
        }
    }

    // Fixed replace implementation
    fn escape_inner_resets_fixed(&self) -> String {
        println!("  [{}] Processing: {:?}", self.debug_name, self.content);

        let reset = "\x1b[0m";
        let outer_codes = self.style.to_ansi_codes();
        println!("  [{}] Outer codes: {:?}", self.debug_name, outer_codes);

        if !self.content.contains(reset) {
            println!("  [{}] No resets found, returning as-is", self.debug_name);
            return self.content.clone();
        }

        let mut result = self.content.clone();
        let mut search_pos = 0;

        while let Some(reset_pos) = result[search_pos..].find(reset) {
            let absolute_reset_pos = search_pos + reset_pos;
            println!(
                "  [{}] Found reset at position {}",
                self.debug_name, absolute_reset_pos
            );

            let after_reset_pos = absolute_reset_pos + reset.len();

            // Look for ANSI escape sequences immediately following the reset
            let mut end_of_ansi = after_reset_pos;
            let remaining_bytes = result.as_bytes();

            while end_of_ansi < remaining_bytes.len() {
                // Check if we're at the start of an ANSI sequence (\x1b[)
                if end_of_ansi + 1 < remaining_bytes.len()
                    && remaining_bytes[end_of_ansi] == 0x1b
                    && remaining_bytes[end_of_ansi + 1] == b'['
                {
                    // Skip the \x1b[
                    end_of_ansi += 2;

                    // Find the end of this ANSI sequence (ends with 'm')
                    while end_of_ansi < remaining_bytes.len() {
                        if remaining_bytes[end_of_ansi] == b'm' {
                            end_of_ansi += 1; // Include the 'm'
                            break;
                        }
                        end_of_ansi += 1;
                    }
                } else {
                    // Not an ANSI sequence, stop looking
                    break;
                }
            }

            if end_of_ansi > after_reset_pos {
                // Found ANSI codes after reset - replace them
                let old_codes = &result[after_reset_pos..end_of_ansi];
                println!(
                    "  [{}] Replacing codes {:?} with {:?}",
                    self.debug_name, old_codes, outer_codes
                );

                result.replace_range(after_reset_pos..end_of_ansi, &outer_codes);
                search_pos = after_reset_pos + outer_codes.len();
            } else {
                // No ANSI codes after reset - insert ours
                println!(
                    "  [{}] No codes after reset, inserting {:?}",
                    self.debug_name, outer_codes
                );

                result.insert_str(after_reset_pos, &outer_codes);
                search_pos = after_reset_pos + outer_codes.len();
            }

            println!("  [{}] Result so far: {:?}", self.debug_name, result);
        }

        println!("  [{}] Final escaped: {:?}", self.debug_name, result);
        result
    }

    pub fn to_styled(&self) -> String {
        let escaped_content = self.escape_inner_resets_fixed();
        let final_result = self.style.paint(escaped_content);
        println!("  [{}] After paint(): {:?}", self.debug_name, final_result);
        final_result
    }

    pub fn error(content: &str) -> Self {
        Self::new(
            content.to_string(),
            Style::from(Role::Error),
            "ERROR".to_string(),
        )
    }

    pub fn info(content: &str) -> Self {
        Self::new(
            content.to_string(),
            Style::from(Role::Info),
            "INFO".to_string(),
        )
    }

    pub fn success(content: &str) -> Self {
        Self::new(
            content.to_string(),
            Style::from(Role::Success),
            "SUCCESS".to_string(),
        )
    }
}

impl fmt::Display for DebugStyledString {
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
            codes.push_str(color_info.ansi);
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
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== Debug Replace Logic ===\n");

    println!("STEP 1: Create inner (RED)");
    let inner = DebugStyledString::error("RED");
    println!("Inner to_styled():");
    let _inner_styled = inner.to_styled();
    println!("Inner result: {}", inner);

    println!("\nSTEP 2: Create middle (blue {{}} blue)");
    let middle_content = format!("blue {} blue", inner);
    println!("Middle content before processing: {:?}", middle_content);
    let middle = DebugStyledString::info(&middle_content);
    println!("Middle to_styled():");
    let _middle_styled = middle.to_styled();
    println!("Middle result: {}", middle);

    println!("\nSTEP 3: Create outer (green {{}} green)");
    let outer_content = format!("green {} green", middle);
    println!("Outer content before processing: {:?}", outer_content);
    let outer = DebugStyledString::success(&outer_content);
    println!("Outer to_styled():");
    let _outer_styled = outer.to_styled();
    println!("Outer result: {}", outer);

    println!("\n=== ANALYSIS ===");
    println!("Expected visual: 'green blue RED blue green'");
    println!("- 'green' should be Success color (first and last)");
    println!("- 'blue' should be Info color");
    println!("- 'RED' should be Error color");

    println!("\n=== MANUAL TEST ===");
    // Let's manually construct what we think it should be:
    let error_codes = Style::from(Role::Error).to_ansi_codes();
    let info_codes = Style::from(Role::Info).to_ansi_codes();
    let success_codes = Style::from(Role::Success).to_ansi_codes();

    let manual = format!(
        "{}green {}blue {}RED\x1b[0m{} blue\x1b[0m{} green\x1b[0m",
        success_codes, info_codes, error_codes, info_codes, success_codes
    );
    println!("Manual construction: {}", manual);
    println!("Manual raw: {:?}", manual);

    println!("\n=== COLOR CODE REFERENCE ===");
    println!("Error codes:   {:?}", error_codes);
    println!("Info codes:    {:?}", info_codes);
    println!("Success codes: {:?}", success_codes);
}
