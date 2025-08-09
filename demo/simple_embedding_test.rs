//! Simple test to verify StyleLike embedding functionality works correctly
//!
//! This is a minimal test to show that outer styles are preserved when using
//! embedded styled content, addressing the ANSI reset issue.
//# Purpose: Test StyleLike embedding with style preservation
//# Categories: styling, embedding, test

/*
[toml]
[dependencies]
*/

use thag_rs::styling::{Role, Style, StyleLike, TermAttributes};
use thag_rs::{sprtln, sprtln_with_embeds};

fn main() {
    // Initialize styling system
    TermAttributes::initialize(None);

    println!("=== Simple Embedding Test ===\n");

    // Test 1: Basic embedding
    println!("1. Basic embedding test:");
    let code_embed = Role::Code.embed("embedded_code");
    sprtln_with_embeds!(
        Role::Normal,
        "This is normal text with {} inside",
        &[code_embed]
    );

    // Test 2: Bold outer style with embedded content
    println!("\n2. Bold outer style test:");
    let error_embed = Role::Error.embed("ERROR");
    sprtln_with_embeds!(
        Style::from(Role::Info).bold(),
        "This is BOLD info text with embedded {} that should return to BOLD",
        &[error_embed]
    );

    // Test 3: Multiple embeds in sequence
    println!("\n3. Multiple embeds test:");
    let embeds = vec![
        Role::Success.embed("OK"),
        Role::Warning.embed("WARN"),
        Role::Error.embed("FAIL"),
    ];
    sprtln_with_embeds!(
        Style::from(Role::Info).underline(),
        "Status: {} or {} or {} - all within underlined text",
        &embeds
    );

    // Test 4: Show that regular sprtln still works
    println!("\n4. Regular sprtln for comparison:");
    sprtln!(Role::Code, "This is regular code styling");
    sprtln!(
        Style::from(Role::Warning).italic(),
        "This is italic warning"
    );

    println!("\n=== Test Complete ===");
    println!("If outer styles are preserved, you should see:");
    println!("- Bold text continues after embedded ERROR");
    println!("- Underlined text continues between and after embedded statuses");
}
