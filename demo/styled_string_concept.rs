/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config"] }
*/

/// Concept demo for StyledString that preserves outer styling context
///
/// This demonstrates a potential StyledString type that could work like
/// colored's ColoredString, automatically restoring outer styling after
/// inner reset sequences.
//# Purpose: Concept for context-preserving styled strings
//# Categories: styling, concepts, prototypes
use std::fmt;
use thag_styling::{ColorInitStrategy, Role, Style, Styleable, Styler, TermAttributes};

/// A styled string that preserves styling context like colored's ColoredString
#[derive(Clone, Debug)]
pub struct StyledString {
    content: String,
    style: Style,
}

impl StyledString {
    /// Create a new StyledString with the given style
    pub fn new(content: String, style: Style) -> Self {
        Self { content, style }
    }

    /// Check if this string contains ANSI reset sequences
    fn has_inner_resets(&self) -> bool {
        self.content.contains("\x1b[0m")
    }

    /// Escape inner reset sequences by restoring outer style after each one
    /// This mimics `colored`'s `escape_inner_reset_sequences` method
    fn escape_inner_resets(&self) -> String {
        if !self.has_inner_resets() {
            return self.content.clone();
        }

        let reset = "\x1b[0m";
        let outer_style_codes = self.style.to_ansi_codes();

        // Find all reset sequence positions
        let reset_positions: Vec<usize> = self
            .content
            .match_indices(reset)
            .map(|(idx, _)| idx)
            .collect();

        if reset_positions.is_empty() {
            return self.content.clone();
        }

        let mut result = self.content.clone();
        result.reserve(reset_positions.len() * outer_style_codes.len());

        // Insert outer style after each reset sequence (in reverse order to maintain indices)
        for &pos in reset_positions.iter().rev() {
            let insert_pos = pos + reset.len();
            result.insert_str(insert_pos, &outer_style_codes);
        }

        result
    }

    /// Apply this string's styling and return the ANSI-escaped result
    pub fn to_styled(&self) -> String {
        let escaped_content = self.escape_inner_resets();
        self.style.paint(escaped_content)
    }

    /// Chain additional styling (returns a new StyledString)
    pub fn bold(self) -> Self {
        Self {
            content: self.content,
            style: self.style.bold(),
        }
    }

    /// Chain additional styling
    pub fn italic(self) -> Self {
        Self {
            content: self.content,
            style: self.style.italic(),
        }
    }
}

impl fmt::Display for StyledString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_styled())
    }
}

/// Extended Styleable trait that returns StyledString instead of plain String
trait StyledStringExt {
    fn styled_error(&self) -> StyledString;
    fn styled_warning(&self) -> StyledString;
    fn styled_success(&self) -> StyledString;
    fn styled_info(&self) -> StyledString;
    fn styled_with(&self, styler: impl Styler) -> StyledString;
}

impl StyledStringExt for str {
    fn styled_error(&self) -> StyledString {
        StyledString::new(self.to_string(), Style::from(Role::Error))
    }

    fn styled_warning(&self) -> StyledString {
        StyledString::new(self.to_string(), Style::from(Role::Warning))
    }

    fn styled_success(&self) -> StyledString {
        StyledString::new(self.to_string(), Style::from(Role::Success))
    }

    fn styled_info(&self) -> StyledString {
        StyledString::new(self.to_string(), Style::from(Role::Info))
    }

    fn styled_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.to_string(), styler.to_style())
    }
}

impl StyledStringExt for String {
    fn styled_error(&self) -> StyledString {
        StyledString::new(self.clone(), Style::from(Role::Error))
    }

    fn styled_warning(&self) -> StyledString {
        StyledString::new(self.clone(), Style::from(Role::Warning))
    }

    fn styled_success(&self) -> StyledString {
        StyledString::new(self.clone(), Style::from(Role::Success))
    }

    fn styled_info(&self) -> StyledString {
        StyledString::new(self.clone(), Style::from(Role::Info))
    }

    fn styled_with(&self, styler: impl Styler) -> StyledString {
        StyledString::new(self.clone(), styler.to_style())
    }
}

// Extension to Style to generate ANSI codes
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

    println!("=== StyledString Concept Demo ===\n");

    println!("1. Simple usage (like current approach):");
    let simple = "Simple error message".styled_error();
    println!("   {}", simple);

    println!("\n2. The nesting problem with current approach:");
    let inner1 = "Heading1 text".style_with(Role::Heading1.underline());
    let inner2 = "Heading2 text".style_with(Role::Heading2.italic());
    let broken_embed = format!("Error {} error {} error", inner1, inner2).error();
    println!("broken_embed=   {}", broken_embed);
    println!("broken_embed=   {:?}", broken_embed);
    let broken_result = format!("Warning {} warning", broken_embed).warning();
    println!("broken_result=   {}", broken_result);
    println!("broken_result=   {:?}", broken_result);
    println!("   ❌ Problem: Warning color likely lost after inner resets");

    println!("\n3. StyledString with inner reset handling:");
    let smart_inner1 = "Heading1 text".styled_with(Role::Heading1.underline());
    let smart_inner2 = "Heading2 text".styled_with(Role::Heading2.italic());
    let smart_embed = format!("Error {} error {} error", smart_inner1, smart_inner2).styled_error();
    println!("smart_embed=   {}", smart_embed);
    println!("smart_embed=   {:?}", smart_embed);
    let smart_result = format!("Warning {} warning", smart_embed).styled_warning();
    println!("smart_result=   {}", smart_result);
    println!("smart_result=   {:?}", smart_result);
    println!("   ✅ Should work: Warning color restored after each inner reset");

    println!("\n4. Chaining with StyledString:");
    let chained = "Bold italic warning".styled_warning().bold().italic();
    println!("   {}", chained);

    println!("\n5. Debug: Show raw ANSI codes:");
    let debug_inner = "red text".styled_error();
    let debug_outer = format!("blue {} blue", debug_inner).styled_info();
    println!("   Raw output: {:?}", debug_outer.to_styled());

    println!("\n=== Key Advantages of StyledString ===");
    println!("✅ Automatic context preservation like colored");
    println!("✅ Chainable styling methods");
    println!("✅ Works with format! macro seamlessly");
    println!("✅ No need for explicit embedding syntax");
    println!("✅ Drop-in replacement for current string methods");

    println!("\n=== Comparison ===");
    println!("Current approach:");
    println!("  cprtln_with_embeds!(Role::Warning, \"Warning {{}} warning\", &[embed]);");
    println!();
    println!("StyledString approach:");
    println!("  println!(\"{{}}\", format!(\"Warning {{}} warning\", embed).styled_warning());");
    println!("  // Automatic context preservation, no special macros needed!");
}
