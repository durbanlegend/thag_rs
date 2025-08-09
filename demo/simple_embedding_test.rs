/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto" }
thag_styling = { version = "0.2, thag-auto" }
*/

/// Demo showcasing both macro and method syntax for StyleLike functionality
///
/// This demonstrates:
/// 1. Traditional macro syntax: cprtln!(Role::Code, "message")
/// 2. New method syntax: Role::Code.prtln(format_args!("message"))
/// 3. Convenient helper macros: prtln_method!(Role::Code, "message")
/// 4. Embedding with style preservation
//# Purpose: Demo both macro and method approaches for styling
//# Categories: styling, embedding, methods, ergonomics
use thag_styling::{cprtln, cprtln_with_embeds, cvprtln, prtln_method, vprtln_method, Verbosity};
use thag_styling::{ColorInitStrategy, Role, Style, StyleLike, TermAttributes};

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

    println!("\n3. Convenient method macros:");
    prtln_method!(Role::Success, "Hello from method macro: {}", "world");
    vprtln_method!(
        Role::Warning,
        Verbosity::Normal,
        "Warning from method macro: {}",
        "careful"
    );

    println!("\n4. Method syntax with parentheses for disambiguation:");
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
    Style::from(Role::Info).underline().prtln_with_embeds(
        "Same trick using trait method `{}`: Status: {} or {} or {} - all within underlined text",
        &[
            Role::Emphasis.embed("prtln_with_embeds"),
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

    prtln_method!(
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
    prtln_method!(
        Role::Success,
        r#"✓ Method syntax: Role::Code.prtln(format_args!("message"))"#
    );
    prtln_method!(
        Role::Success,
        r#"✓ Helper macro: prtln_method!(Role::Code, "message")"#
    );
    cprtln!(
        Role::Info,
        "✓ Outer styles are preserved in embedded content"
    );
    (Role::Emphasis).prtln(format_args!(
        "✓ Parentheses work for disambiguation: (Role::X).prtln(...)"
    ));
}
