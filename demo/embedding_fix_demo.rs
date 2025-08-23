/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Demo comparing embedding behavior: broken vs fixed approaches
///
/// This demonstrates:
/// 1. Problem: Plain string styling loses outer context
/// 2. Solution: Using embed methods that preserve outer styling
/// 3. Comparison with cprtln_with_embeds! approach
/// 4. Recreating the colored.rs embed behavior with thag_styling
//# Purpose: Demo embedding fix for string extensions
//# Categories: styling, embedding, ergonomics
use thag_styling::{
    cprtln, cprtln_with_embeds, ColorInitStrategy, Role, Styleable, Styler, TermAttributes,
};

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== Embedding Fix Demo ===\n");

    // Section 1: The problem - plain strings lose outer styling
    println!("1. THE PROBLEM: Plain string methods lose outer styling");
    println!("   (Notice how the outer 'Warning' styling is lost after the embedded text)");

    let broken_embed1 = "Heading1 text".heading1();
    let broken_embed2 = "Heading2 text".heading2();
    let broken_combined = format!("Error {broken_embed1} error {broken_embed2} error").error();

    println!("   Broken: Warning {} warning", broken_combined);
    // The warning color is lost after the embedded content!

    // Section 2: The solution - embedding-aware methods
    println!("\n2. THE SOLUTION: Embedding-aware methods preserve outer styling");

    cprtln_with_embeds!(
        Role::Warning,
        "Warning {} warning - outer styling preserved!",
        &["Error content error content error".embed_error()]
    );

    // Section 3: Better approach - direct embedding in the outer call
    println!("\n3. BETTER: Direct embedding in the outer call");

    cprtln_with_embeds!(
        Role::Warning,
        "Warning with {} and {} - all styling preserved!",
        &[
            "Heading1 and underlined!".embed_with(Role::Heading1.underline()),
            "Heading2 and italic!".embed_with(Role::Heading2.italic())
        ]
    );

    // Section 4: Recreating the colored.rs example
    println!("\n4. Recreating colored.rs embed example with thag_styling");
    println!("   Original colored version:");
    println!("   let embed = format!(\"Magenta {{}} magenta {{}} magenta\", cstring1, cstring2).magenta();");
    println!("   println!(\"Normal {{}} normal\", embed);");
    println!();
    println!("   Our thag_styling equivalent:");

    let bold_red = "Bold and Red!".embed_with(Role::Error.bold());
    let italic_blue = "Italic and Blue!".embed_with(Role::Info.italic());

    cprtln_with_embeds!(
        Role::Normal,
        "Normal {} normal - outer styling preserved throughout!",
        &["Magenta content magenta content magenta".embed_with(Role::Heading1)]
    );

    // More accurate recreation
    // Better approach: build the embedded content properly
    cprtln_with_embeds!(
        Role::Normal,
        "Normal {} normal",
        &[Role::Heading1.embed("Magenta styled magenta styled magenta")]
    );

    // Section 5: Individual embed methods
    println!("\n5. Individual embed methods for common roles");

    let embeds = vec![
        "Critical error".embed_error(),
        "Success message".embed_success(),
        "Warning text".embed_warning(),
        "Info details".embed_info(),
        "code_snippet()".embed_code(),
        "Important!".embed_emphasis(),
    ];

    cprtln_with_embeds!(Role::Normal, "Status: {}, {}, {}, {}, run {}, {}", &embeds);

    // Section 6: Comparison table
    println!("\n6. Method comparison:");
    println!("   ┌─────────────────────────────────────────────────────────────────┐");
    println!("   │ Method                    │ Returns  │ Embeddable │ Preserves    │");
    println!("   │                           │          │            │ Outer Style? │");
    println!("   ├─────────────────────────────────────────────────────────────────┤");
    println!("   │ \"text\".error()            │ String   │ No         │ No ❌        │");
    println!("   │ \"text\".embed_error()      │ Embedded │ Yes        │ Yes ✅       │");
    println!("   │ Role::Error.embed(\"text\") │ Embedded │ Yes        │ Yes ✅       │");
    println!("   └─────────────────────────────────────────────────────────────────┘");

    // Section 7: Performance note
    println!("\n7. When to use each approach:");
    cprtln!(
        Role::Info,
        "   • Use .error(), .success(), etc. for simple standalone styling"
    );
    cprtln!(
        Role::Info,
        "   • Use .embed_error(), .embed_success(), etc. when embedding in other styled text"
    );
    cprtln!(
        Role::Success,
        "   • Embedding methods preserve outer styling context perfectly!"
    );

    println!("\n=== Demo Complete ===");
    println!("\nKey takeaways:");
    cprtln!(
        Role::Success,
        "✓ Plain string methods (.error()) are great for standalone use"
    );
    cprtln!(
        Role::Success,
        "✓ Embed methods (.embed_error()) preserve outer styling context"
    );
    cprtln!(
        Role::Success,
        "✓ Use cprtln_with_embeds! with embed methods for complex nested styling"
    );
    cprtln!(
        Role::Warning,
        "⚠ Plain string methods don't work well when embedded in other styled text"
    );
    cprtln!(
        Role::Info,
        "ℹ This gives us the best of both worlds: simplicity AND powerful embedding"
    );
}
