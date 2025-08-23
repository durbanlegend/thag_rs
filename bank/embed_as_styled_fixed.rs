/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["full"] }
*/
use thag_styling::{
    cprtln, cprtln_with_embeds, ColorInitStrategy, Role, Styleable, Styler, TermAttributes,
};

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== Fixed Embedding Demo ===\n");

    println!("1. BROKEN (original approach) - outer styling lost:");
    let cstring1 = "Heading1 and underlined!".style_with(Role::Heading1.underline());
    let cstring2 = "Heading2 and italic!".style_with(Role::Heading2.italic());
    let embed = format!("Error {cstring1} error {cstring2} error").error();

    cprtln!(Role::Warning, "Warning {embed} warning");
    // Notice: Warning styling is lost after the embed!

    println!("\n2. FIXED (embedding-aware) - outer styling preserved:");
    let embed1 = "Heading1 and underlined!".embed_with(Role::Heading1.underline());
    let embed2 = "Heading2 and italic!".embed_with(Role::Heading2.italic());

    cprtln_with_embeds!(
        Role::Warning,
        "Warning Error {} error {} error warning",
        &[embed1, embed2]
    );
    // Notice: Warning styling is preserved throughout!

    println!("\n3. ALTERNATIVE (using individual embed methods):");
    cprtln_with_embeds!(
        Role::Warning,
        "Warning {} warning",
        &[format!(
            "Error {} error {} error",
            "Heading1 and underlined!", "Heading2 and italic!"
        )
        .embed_error()]
    );

    println!("\n✅ Problem solved! Outer styling is now properly preserved.");
}
