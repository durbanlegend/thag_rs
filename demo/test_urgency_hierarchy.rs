/*[toml]
[dependencies]
# thag_rs = { version = "0.2, thag-auto", features = ["config", "simplelog"] }
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }
*/

//! Urgency Hierarchy Demonstration
//!
//! This script demonstrates the new urgency-based ANSI color hierarchy where
//! bright colors are used for the most critical/urgent messages, following
//! established ANSI safety color standards and terminal application conventions.
//!
//# Purpose: Demonstrate the urgency-based color hierarchy in terminal output
//# Categories: styling, testing, development

use thag_styling::{styling::Role, Styleable, Styler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚨 Urgency-Based Color Hierarchy Demonstration");
    println!("==============================================");
    println!();

    // Demonstrate the urgency hierarchy
    println!("📊 URGENCY HIERARCHY (Most to Least Critical):");
    println!("───────────────────────────────────────────────");
    println!();

    // CRITICAL LEVEL (Bright Colors - Maximum Visibility)
    println!("🔥 CRITICAL LEVEL (Bright Colors):");
    println!(
        "  {} {}",
        "❌".error(),
        "CRITICAL ERROR: System failure detected!".error()
    );
    println!(
        "  {} {}",
        "⚠️".warning(),
        "HIGH WARNING: Potential data loss risk!".warning()
    );
    println!();

    // IMPORTANT LEVEL (Regular Colors - Semantic but Calmer)
    println!("📢 IMPORTANT LEVEL (Regular Colors):");
    println!(
        "  {} {}",
        "💪".emphasis(),
        "EMPHASIS: This needs your attention".emphasis()
    );
    println!(
        "  {} {}",
        "📝".commentary(),
        "COMMENTARY: Additional context provided".commentary()
    );
    println!(
        "  {} {}",
        "✅".success(),
        "SUCCESS: Operation completed successfully".success()
    );
    println!(
        "  {} {}",
        "ℹ️".info(),
        "INFO: General information available".info()
    );
    println!();

    // STRUCTURAL LEVEL (Organized Content)
    println!("📋 STRUCTURAL LEVEL (Content Organization):");
    println!(
        "  {} {}",
        "🎯".heading1(),
        "PRIMARY HEADING: Main Section".heading1()
    );
    println!(
        "  {} {}",
        "📌".heading2(),
        "Secondary Heading: Subsection".heading2()
    );
    println!("  {} {}", "💻".code(), "code_snippet()".code());
    println!(
        "  {} {}",
        "🔗".link(),
        "https://example.com/important-link".link()
    );
    println!(
        "  {} {}",
        "💬".quote(),
        "\"Quoted text or citation\"".quote()
    );
    println!();

    // BACKGROUND LEVEL (Supporting Information)
    println!("🔍 BACKGROUND LEVEL (Supporting Information):");
    println!(
        "  {} {}",
        "📝".normal(),
        "Normal text: Standard content".normal()
    );
    println!("  {} {}", "💡".hint(), "Hint: Helpful suggestion".hint());
    println!(
        "  {} {}",
        "🐛".debug(),
        "Debug: Development information".debug()
    );
    println!(
        "  {} {}",
        "👻".subtle(),
        "Subtle: De-emphasized content".subtle()
    );
    println!();

    println!("🎨 ANSI Color Mapping (0-15):");
    println!("─────────────────────────────");

    // Show the actual ANSI mapping
    let mappings = [
        ("0: Background", "Theme background color", "⚫"),
        ("1: Red", "Role::Emphasis (important emphasis)", "🔴"),
        ("2: Green", "Role::Success (positive outcomes)", "🟢"),
        ("3: Yellow", "Role::Commentary (highlighted notes)", "🟡"),
        ("4: Blue", "Role::Info (calm information)", "🔵"),
        ("5: Magenta", "Role::Heading1 (primary headings)", "🟣"),
        ("6: Cyan", "Role::Code (technical content)", "🔵"),
        ("7: White", "Role::Normal (standard text)", "⚪"),
        ("8: Bright Black", "Role::Subtle (dimmed text)", "⚫"),
        ("9: Bright Red", "Role::Error (maximum urgency)", "🔴"),
        ("10: Bright Green", "Role::Debug (development info)", "🟢"),
        ("11: Bright Yellow", "Role::Warning (high visibility)", "🟡"),
        ("12: Bright Blue", "Role::Link (web convention)", "🔵"),
        ("13: Bright Magenta", "Role::Heading2 (secondary)", "🟣"),
        ("14: Bright Cyan", "Role::Hint (helpful suggestions)", "🔵"),
        ("15: Bright White", "Role::Quote (prominent quotes)", "⚪"),
    ];

    for (ansi, role, emoji) in &mappings {
        println!("  {} {:20} → {}", emoji, ansi, role);
    }

    println!();
    println!("🎯 KEY DESIGN PRINCIPLES:");
    println!("────────────────────────");
    println!(
        "  {} Bright colors = Maximum urgency/visibility",
        "💡".hint()
    );
    println!("  {} Regular colors = Semantic but calmer", "💡".hint());
    println!("  {} Follows ANSI safety color psychology", "💡".hint());
    println!(
        "  {} Aligns with terminal application conventions",
        "💡".hint()
    );
    println!(
        "  {} Red = Danger/Urgent, Blue = Information, etc.",
        "💡".hint()
    );

    println!();
    println!("🚀 This hierarchy makes thag_styling intuitive and standards-compliant!");

    Ok(())
}
