/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config"] }
*/
use thag_styling::styling::{format_with_embeds, Embedded};
use thag_styling::{
    cprtln, cprtln_with_embeds, ColorInitStrategy, Role, Style, Styleable, StyledString,
    StyledStringExt, Styler, TermAttributes,
};

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== Truly Fixed Embedding Demo ===\n");

    println!("1. BROKEN (original approach) - outer styling lost:");
    let cstring1 = "Heading1 and underlined!".style_with(Role::Heading1.underline());
    let cstring2 = "Heading2 and italic!".style_with(Role::Heading2.italic());
    let embed = format!("Error {cstring1} error {cstring2} error").error();

    cprtln!(Role::Warning, "Warning {embed} warning");
    println!("   ❌ Problem: Warning styling lost after embedded content - now fixed");

    // println!("\n2. ATTEMPT 1 - using embed methods (still wrong):");
    // let embed1 = "Heading1 and underlined!".embed_with(Role::Heading1.underline());
    // let embed2 = "Heading2 and italic!".embed_with(Role::Heading2.italic());

    // cprtln_with_embeds!(
    //     Role::Warning,
    //     "Warning Error {} error {} error warning",
    //     &[embed1, embed2]

    println!("\n2. ATTEMPT 1 - using embed methods (now fixed but with redundant application of Error style):");
    let embed1 = "Heading1 and underlined!".embed_with(Role::Heading1.underline());
    let embed2 = "Heading2 and italic!".embed_with(Role::Heading2.italic());

    // let error_text = format_with_embeds(Role::Error, "Error {} error {} error", &[embed1, embed2]);
    let error_text = format_with_embeds(
        &Style::from(Role::Error),
        "Error {} error {} error",
        &[embed1, embed2],
    );
    // let error_embed = error_text.embed_with(Style::new());
    // let error_embed = Embedded::new(&Style::new(), &error_text); // Use plain style
    let error_embed = error_text.embed_with(Role::Error);
    cprtln_with_embeds!(Role::Warning, "Warning {} warning", &[error_embed]);

    println!("\n3. CORRECT APPROACH - multiple separate embeds (now superseded by StyledString):");
    // The key insight: we need to break this into multiple separate embeds
    // Each styled piece needs to be its own embed in the outer warning context
    cprtln_with_embeds!(
        Role::Warning,
        "Warning {} {} {} {} {} warning",
        &[
            "Error".embed_error(),
            "Heading1 and underlined!".embed_with(Role::Heading1.underline()),
            "error".embed_error(),
            "Heading2 and italic!".embed_with(Role::Heading2.italic()),
            "error".embed_error()
        ]
    );
    println!("   ✅ Success: Each piece maintains its color, Warning preserved throughout");

    println!("\n4. ALTERNATIVE - using Role.embed directly (now superseded by StyledString):");
    cprtln_with_embeds!(
        Role::Warning,
        "Warning {} {} {} {} {} warning",
        &[
            Role::Error.embed("Error"),
            Role::Heading1.underline().embed("Heading1 and underlined!"),
            Role::Error.embed("error"),
            Role::Heading2.italic().embed("Heading2 and italic!"),
            Role::Error.embed("error")
        ]
    );
    println!("   ✅ Success: Same result using Role.embed() syntax");

    println!("\n5. MOST READABLE - descriptive variable names (now superseded by StyledString):");
    let error_start = "Error".embed_error();
    let heading1_styled = "Heading1 and underlined!".embed_with(Role::Heading1.underline());
    let error_middle1 = "error".embed_error();
    let heading2_styled = "Heading2 and italic!".embed_with(Role::Heading2.italic());
    let error_end = "error".embed_error();

    cprtln_with_embeds!(
        Role::Warning,
        "Warning {} {} {} {} {} warning",
        &[
            error_start,
            heading1_styled,
            error_middle1,
            heading2_styled,
            error_end
        ]
    );
    println!("   ✅ Success: Most readable EMBED approach with descriptive names");

    println!("\n6. StyledString approach:");
    let styled_error = format!(
        "Error {} error {} error",
        "Heading1 and underlined!".heading1().underline(),
        "Heading2 and italic!".heading2().italic()
    )
    .error();
    format!("Warning {} warning", styled_error)
        .warning()
        .println();
    println!("   ✅ Success: Most ergonomic approach with nested embedding");

    println!("\n=== Key Insights ===");
    println!("❌ DOESN'T WORK: Trying to embed a multi-colored string as a single embed");
    println!("✅ DOES WORK: Breaking multi-colored content into separate embeds");
    println!("✅ Each embed preserves its own color");
    println!("✅ Outer Warning color is preserved between embeds");
    println!();
    println!("🔑 The fundamental principle: One embed = One color");
    println!("   If you need multiple colors, use multiple embeds");
}
