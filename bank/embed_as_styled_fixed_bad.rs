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

    println!("\n3. CORRECT APPROACH - nested embedding:");
    cprtln_with_embeds!(
        Role::Warning,
        "Warning {} warning",
        &[Role::Error.embed(&format!(
            "Error {} error {} error",
            Role::Heading1.underline().paint("Heading1 and underlined!"),
            Role::Heading2.italic().paint("Heading2 and italic!")
        ))]
    );

    println!("\n4. EVEN BETTER - proper multi-level embedding:");
    // This would be the ideal approach but requires nested embedding support
    cprtln_with_embeds!(
        Role::Warning,
        "Warning Error {} error {} error warning",
        &[
            "Heading1 and underlined!".embed_with(Role::Heading1.underline()),
            "Heading2 and italic!".embed_with(Role::Heading2.italic())
        ]
    );

    println!("\n✅ Problem solved! Outer styling is now properly preserved.");
}
