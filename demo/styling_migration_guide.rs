/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }
thag_common = { version = "0.2, thag-auto" }
*/

/// Comprehensive migration guide from old embedding systems to StyledString
///
/// This demo shows side-by-side comparisons of:
/// 1. cprtln_with_embeds! → StyledString with println!
/// 2. cvprtln_with_embeds! → StyledString with vprintln!
/// 3. format_with_embeds → format! with StyledString
/// 4. Embedded struct → StyledString directly
///
/// IMPORTANT: This guide is specifically about replacing the EMBEDDING system.
/// The Styled<T> struct (.style().bold()) serves a different purpose and remains:
/// - Styled<T>: General text effects (bold, italic, etc.) - KEEP USING
/// - StyledString: Semantic roles + embedding/nesting - NEW PREFERRED WAY
///
/// The new StyledString approach provides:
/// - Better attribute reset handling (no bleeding)
/// - More natural Rust syntax with method chaining
/// - Unlimited nesting depth without pre-planning
/// - Better performance (no macro overhead)
/// - Cleaner, more maintainable code
//# Purpose: Migration guide from old embedding systems to StyledString
//# Categories: documentation, examples, migration, styling
use thag_common::Verbosity;
use thag_styling::{ColorInitStrategy, Styleable, StyledStringExt, TermAttributes};

fn main() {
    // Initialize styling system
    TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);

    println!("=== Styling System Migration Guide ===\n");

    // Example 1: Basic embedding
    println!("1. Basic embedding:");
    println!("   OLD: cprtln_with_embeds!(Role::Warning, \"Warning {{}} warning\", &[embed]);");
    println!("   NEW: format!(\"Warning {{}} warning\", \"error\".error()).warning().println();");
    println!();

    println!("   Old approach would have required:");
    println!("   let embed = \"error\".embed_with(Role::Error);");
    println!(
        "   thag_styling::cprtln_with_embeds!(Role::Warning, \"Warning {{}} warning\", &[embed]);"
    );
    println!();

    println!("   New approach:");
    format!("Warning {} warning", "error".error())
        .warning()
        .println();
    println!();

    // Example 2: Multiple embeds
    println!("2. Multiple embeds:");
    println!(
        "   OLD: cprtln_with_embeds!(Role::Info, \"Status: {{}} and {{}}\", &[embed1, embed2]);"
    );
    println!("   NEW: format!(\"Status: {{}} and {{}}\", \"success\".success(), \"warning\".warning()).info().println();");
    println!();

    println!("   Old approach would have required:");
    println!("   let embed1 = \"success\".embed_with(Role::Success);");
    println!("   let embed2 = \"warning\".embed_with(Role::Warning);");
    println!("   thag_styling::cprtln_with_embeds!(Role::Info, \"Status: {{}} and {{}}\", &[embed1, embed2]);");
    println!();

    println!("   New approach:");
    format!(
        "Status: {} and {}",
        "success".success(),
        "warning".warning()
    )
    .info()
    .println();
    println!();

    // Example 3: Verbosity-gated printing
    println!("3. Verbosity-gated printing:");
    println!("   OLD: cvprtln_with_embeds!(Role::Debug, V::Debug, \"Debug: {{}}\", &[embed]);");
    println!("   NEW: format!(\"Debug: {{}}\", \"value\".code()).debug().vprintln(V::Debug);");
    println!();

    println!("   Old approach would have required:");
    println!("   let debug_embed = \"value\".embed_with(Role::Code);");
    println!("   thag_styling::cvprtln_with_embeds!(Role::Debug, Verbosity::Debug, \"Debug: {{}}\", &[debug_embed]);");
    println!();

    println!("   New approach:");
    format!("Debug: {}", "value".code())
        .debug()
        .vprintln(Verbosity::Debug);
    println!();

    // Example 4: Complex multi-level nesting
    println!("4. Complex multi-level nesting:");
    println!("   OLD: Required manual embed array construction");
    println!("   NEW: Natural nested format! calls");
    println!();

    println!("   New approach (unlimited nesting):");
    let deep_result = format!(
        "Level1: Success [{}] [{}] Level1: Success",
        format!(
            "Level2a: Warning [{}] [{}] Level2a: Warning]",
            format!(
                "Level3a: Error italic [{}] Level3a: Error italic",
                "Level 4: Code bold".code().bold()
            )
            .error()
            .italic(),
            "Level3b: Plain Error".error()
        )
        .warning()
        .bold(),
        "Level 2b: Normal".normal()
    )
    .success();
    deep_result.println();
    println!();

    println!("   New approach stepwise:");
    let level4 = "Level 4: Code bold".code().bold();
    let level3a = format!("Level3a: Error italic [{level4}] Level3a: Error italic")
        .error()
        .italic();
    let level3b = "Level3b: Plain Error".error();
    let level2a = format!("Level2a: Warning [{level3a}] [{level3b}] Level2a: Warning]",)
        .warning()
        .bold();
    let level2b = "Level 2b: Normal".normal();
    format!("Level1: Success [{level2a}] [{level2b}] Level1: Success")
        .success()
        .println();
    println!();

    // Example 5: Attribute handling comparison
    println!("5. Text attribute handling:");
    println!("   The new system prevents attribute bleeding between levels");
    println!();

    println!("   StyledString approach (no bleeding):");
    let result = format!(
        "Normal {} {} normal",
        "bold text".normal().bold(),
        "italic text".normal().italic()
    )
    .info();
    result.println();
    println!();

    // Example 6: Method chaining
    println!("6. Method chaining:");
    println!("   NEW: Fluent interface with chaining");
    println!();

    "Simple error".error().bold().println();
    "Warning with style"
        .warning()
        .italic()
        .underline()
        .println();
    println!();

    // Example 7: Migration helpers (deprecated but available)
    println!("7. Migration helpers (deprecated but available temporarily):");
    println!("   styled_println and styled_vprintln functions");
    println!();

    println!("   Example migration helpers (now removed):");
    println!("   styled_println(Role::Success, &format!(\"Migrated: {{}}\", \"success\".code()));");
    println!(
        "   styled_vprintln(Role::Debug, V::Debug, &format!(\"Debug: {{}}\", \"info\".info()));"
    );
    println!();

    println!("   Direct modern equivalent:");
    format!("Migrated: {}", "success".code())
        .success()
        .println();
    format!("Debug: {}", "info".info())
        .debug()
        .vprintln(Verbosity::Debug);
    println!();

    // Example 8: Performance comparison
    println!("8. Performance benefits:");
    println!("   - No macro expansion overhead");
    println!("   - No temporary embed array allocations");
    println!("   - Direct string formatting");
    println!("   - Fewer function calls");
    println!();

    // Example 9: Code clarity comparison
    println!("9. Code clarity:");
    println!();

    println!("   OLD (verbose, manual):");
    println!("   let embed1 = \"error\".embed_with(Role::Error);");
    println!("   let embed2 = \"warning\".embed_with(Role::Warning);");
    println!("   cprtln_with_embeds!(Role::Info, \"Status: {{}} and {{}}\", &[embed1, embed2]);");
    println!();

    println!("   NEW (concise, natural):");
    println!("   format!(\"Status: {{}} and {{}}\", \"error\".error(), \"warning\".warning()).info().println();");
    println!();

    // Example 10: Error cases and edge handling
    println!("10. Robust error handling:");
    println!("    The new system handles edge cases better:");
    println!();

    // Empty strings
    "".error().println();
    format!("Text {} text", "".warning()).info().println();

    // Special characters
    format!("Unicode: {} {}", "🚀".success(), "emoji".code())
        .normal()
        .println();
    println!();

    println!("=== Migration Summary ===");
    println!("✅ Replace cprtln_with_embeds! with format!().role_method().println()");
    println!("✅ Replace cvprtln_with_embeds! with format!().role_method().vprintln(verbosity)");
    println!("✅ Replace format_with_embeds with direct format! calls");
    println!("✅ Replace Embedded struct with direct StyledString usage");
    println!("✅ Use semantic role methods (.error(), .warning(), etc.)");
    println!("✅ Enjoy perfect attribute reset handling automatically");
    println!();

    println!("📝 NOTE: Keep using Styled<T> for general text effects:");
    println!("   \"text\".style().bold().italic() - STILL RECOMMENDED");
    println!("   This migration is specifically for embedding/nesting scenarios");
    println!();

    println!("🎯 The new approach is more:");
    println!("   • Natural (standard Rust patterns)");
    println!("   • Powerful (unlimited nesting)");
    println!("   • Performant (no macro overhead)");
    println!("   • Reliable (perfect attribute handling)");
    println!("   • Maintainable (cleaner code structure)");
}
