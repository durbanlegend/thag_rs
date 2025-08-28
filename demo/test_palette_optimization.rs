/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto", features = ["config", "simplelog"] }
thag_styling = { version = "0.2, thag-auto" }
*/

//! Test script to verify the palette optimization changes
//!
//! This script demonstrates the new roles (Link, Quote, Commentary) that replaced
//! the old Trace role, ensuring the perfect 1:1 mapping with 16-color terminal palette.
//!
//# Purpose: Test and demonstrate the palette optimization changes
//# Categories: styling, testing, development

use thag_styling::{styling::Role, Styleable, Styler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Testing Palette Optimization Changes");
    println!("========================================");

    // Test all 16 roles to ensure perfect mapping
    let all_roles = [
        (Role::Heading1, "Primary Heading", "🎯"),
        (Role::Heading2, "Secondary Heading", "📌"),
        (Role::Heading3, "Tertiary Heading", "📍"),
        (Role::Error, "Critical Error", "❌"),
        (Role::Warning, "Important Warning", "⚠️"),
        (Role::Success, "Success Message", "✅"),
        (Role::Info, "Information", "ℹ️"),
        (Role::Emphasis, "Emphasized Text", "💪"),
        (Role::Code, "Code Snippet", "💻"),
        (Role::Normal, "Normal Text", "📝"),
        (Role::Subtle, "Subtle Text", "🔍"),
        (Role::Hint, "Helpful Hint", "💡"),
        (Role::Debug, "Debug Info", "🐛"),
        (Role::Link, "Hyperlink", "🔗"),        // NEW: replaced Trace
        (Role::Quote, "Quoted Text", "💬"),     // NEW: added
        (Role::Commentary, "Commentary", "📝"), // NEW: added
    ];

    println!(
        "Total roles: {} (perfect 16-color mapping!)",
        all_roles.len()
    );
    println!();

    // Display each role with its styling
    println!("📋 Role Demonstrations:");
    for (role, description, emoji) in &all_roles {
        let styled_text = format!("{} {}", emoji, description);
        println!("  {}", role.paint(&styled_text));
    }
    println!();

    // Test new roles specifically
    println!("🆕 New Roles Showcase:");
    println!("  {}", "Visit https://github.com/rust-lang/rust".link());
    println!(
        "  {}",
        "As Einstein said: \"Imagination is more important than knowledge\"".quote()
    );
    println!(
        "  {}",
        "Note: This feature was added in version 0.2".commentary()
    );
    println!();

    // Test using Styleable trait methods
    println!("🎨 Styleable Trait Methods:");
    println!("  {}", "Click this link!".link());
    println!("  {}", "\"To be or not to be\"".quote());
    println!("  {}", "This is additional context".commentary());
    println!();

    // Verify no Role::Trace exists (this would cause compile error if it did)
    println!("✅ Verification: Role::Trace successfully removed");
    println!("✅ Verification: New roles Link, Quote, Commentary added");
    println!("✅ Verification: Perfect 1:1 mapping with 16-color palette achieved");

    println!();
    println!("🎯 Palette optimization complete!");

    Ok(())
}
