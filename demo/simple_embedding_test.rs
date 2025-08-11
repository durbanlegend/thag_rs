/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Demo showcasing both macro and method syntax for StyleLike functionality
///
/// This demonstrates:
/// 1. Traditional macro syntax: cprtln!(Role::Code, "message")
/// 2. New method syntax: Role::Code.prtln(format_args!("message"))
/// 3. Embedding with style preservation
//# Purpose: Demo both macro and method approaches for styling
//# Categories: styling, techniques
use thag_styling::{
    cprtln, cprtln_with_embeds, cvprtln, ColorInitStrategy, Role, Style, StyleLike, TermAttributes,
    Verbosity,
};

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== Macro vs Method Syntax Demo ===\n");

    // Section 1: Compare macro vs method syntax
    println!("1. Traditional macro syntax:");
    cprtln!(Role::Code, "Hello from macro: {}", "world");
    cvprtln!(Role::Debug, Verbosity::Normal, "Debug from macro: {}", 42);

    println!("\n2. New method syntax (verbose):");
    Role::Code.prtln(format_args!("Hello from method: {}", "world"));
    Role::Debug.vprtln(Verbosity::Normal, format_args!("Debug from method: {}", 42));
    Role::Success.prtln(format_args!("Hello from method macro: {}", "world"));
    Role::Warning.vprtln(
        Verbosity::Normal,
        format_args!("Warning from method macro: {}", "careful"),
    );

    println!("\n3. Method syntax with parentheses for disambiguation:");
    (Role::Info).prtln(format_args!("Info with parentheses: {}", "clear"));
    (Role::Error).vprtln(
        Verbosity::Normal,
        format_args!("Error with parentheses: {}", "problem"),
    );

    // Section 2: Embedding functionality
    println!("\n5. Basic embedding test:");
    let code_embed = Role::Code.embed("embedded_code");
    cprtln_with_embeds!(
        Role::Normal,
        "This is normal text with {} inside",
        &[code_embed]
    );

    println!("\n6. Bold outer style with embedded content:");
    let error_embed = Role::Error.embed("ERROR");
    cprtln_with_embeds!(
        Style::from(Role::Info).bold(),
        "This is BOLD info text with embedded {} that should return to BOLD",
        &[error_embed]
    );

    println!("\n7. Multiple embeds in sequence:");
    let embeds = &[
        Role::Success.embed("OK"),
        Role::Warning.embed("WARN"),
        Role::Error.embed("FAIL"),
    ];
    cprtln_with_embeds!(
        Style::from(Role::Info).underline(),
        "Status: {} or {} or {} - all within underlined text",
        embeds
    );
    Style::from(Role::Info).underline().prtln_embed(
        "Same trick using trait method `{}`: Status: {} or {} or {} - all within underlined text",
        &[
            Role::Emphasis.embed("prtln_embed"),
            Role::Success.embed("OK"),
            Role::Warning.embed("WARN"),
            Role::Error.embed("FAIL"),
        ],
    );

    println!("\n8. Style chaining with methods:");
    Style::from(Role::Normal)
        .bold()
        .italic()
        .prtln(format_args!("This is bold italic: {}", "chained"));

    cprtln!(
        Style::from(Role::Warning).underline(),
        "This is underlined warning: {}",
        "be careful"
    );

    println!("\n=== Demo Complete ===");
    println!("\nKey observations:");
    cprtln!(
        Role::Success,
        r#"✓ Macro syntax: cprtln!(Role::Code, "message")"#
    );
    cprtln!(
        Role::Success,
        r#"✓ Method syntax: Role::Code.prtln(format_args!("message"))"#
    );
    cprtln!(
        Role::Success,
        r#"✓ Helper macro: cprtln!(Role::Code, "message")"#
    );
    cprtln!(
        Role::Info,
        "✓ Outer styles are preserved in embedded content"
    );
    (Role::Emphasis).prtln(format_args!(
        "✓ Parentheses work for disambiguation: (Role::X).prtln(...)"
    ));
}
