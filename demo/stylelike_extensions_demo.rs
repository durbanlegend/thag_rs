//! Demo script showcasing StyleLike trait extensions for ergonomic printing and embedding
//!
//! This script demonstrates:
//! 1. sprtln! - ergonomic styled printing macro
//! 2. svprtln! - verbosity-gated styled printing macro
//! 3. sprtln_with_embeds! - styled printing with embedded styled content
//! 4. svprtln_with_embeds! - verbosity-gated embedded styled printing
//!
//! The StyleLike trait allows both Role and Style to be used interchangeably,
//! and the embedding feature preserves outer styles when nesting different styles.
//# Purpose: Demo StyleLike extensions for ergonomic styling and embedding
//# Categories: styling, ergonomics, embedding

/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
thag_common = { version = "0.2, thag-auto" }
*/
use thag_common::Verbosity;
use thag_styling::styling::{Color, Embedded, Role, Style, StyleLike, TermAttributes};
use thag_styling::{sprtln, sprtln_with_embeds, svprtln, svprtln_with_embeds, ColorInitStrategy};

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== StyleLike Extensions Demo ===\n");

    // Section 1: Basic sprtln! usage
    println!("1. Basic sprtln! usage:");
    sprtln!(Role::Heading1, "Primary Heading");
    sprtln!(Role::Heading2, "Secondary Heading");
    sprtln!(Role::Code, "let code_snippet = \"Hello, World!\";");
    sprtln!(Role::Error, "Error: Something went wrong!");
    sprtln!(Role::Success, "Success: Operation completed");
    sprtln!(Role::Warning, "Warning: Please be careful");
    sprtln!(Role::Info, "Info: Just so you know");

    // With format arguments
    let user = "Alice";
    let count = 42;
    sprtln!(Role::Normal, "User {} has {} items", user, count);

    println!();

    // Section 2: sprtln! with Style modifications
    println!("2. sprtln! with Style modifications:");
    sprtln!(Style::from(Role::Normal).bold(), "Bold normal text");
    sprtln!(Style::from(Role::Info).italic(), "Italic info text");
    sprtln!(Style::from(Role::Warning).underline(), "Underlined warning");
    sprtln!(Color::yellow().bold().italic(), "Yellow bold italic");

    println!();

    // Section 3: Verbosity-gated printing
    println!("3. Verbosity-gated printing (svprtln!):");
    svprtln!(
        Role::Debug,
        Verbosity::Normal,
        "This shows at Normal verbosity"
    );
    svprtln!(
        Role::Debug,
        Verbosity::Verbose,
        "This shows at Verbose verbosity"
    );
    svprtln!(
        Role::Trace,
        Verbosity::Debug,
        "This shows at Debug verbosity"
    );

    // This one might not show depending on current verbosity
    svprtln!(
        Role::Trace,
        Verbosity::Debug,
        "This trace message might be filtered out"
    );

    println!();

    // Section 4: Basic embedding
    println!("4. Basic embedding with sprtln_with_embeds!:");

    let code_embed = Role::Code.embed("println!(\"Hello\")");
    let error_embed = Role::Error.embed("NullPointerException");
    let success_embed = Role::Success.embed("OK");

    sprtln_with_embeds!(
        Role::Normal,
        "The code {} can either throw {} or return {}",
        &[code_embed, error_embed, success_embed]
    );

    println!();

    // Section 5: Complex embedding with different outer styles
    println!("5. Complex embedding with different outer styles:");

    let info_embed = Role::Info.embed("version 2.1.0");
    let warning_embed = Role::Warning.embed("deprecated");
    let code_embed2 = Role::Code.embed("update_system()");

    sprtln_with_embeds!(
        Style::from(Role::Heading2).underline(),
        "System {} contains {} function {}",
        &[info_embed, warning_embed, code_embed2]
    );

    // With bold outer style
    let debug_embed = Role::Debug.embed("memory usage: 45%");
    let trace_embed = Role::Trace.embed("GC triggered");

    sprtln_with_embeds!(
        Style::from(Role::Info).bold(),
        "Performance monitoring: {} and {}",
        &[debug_embed, trace_embed]
    );

    println!();

    // Section 6: Nested different color schemes
    println!("6. Nested different color schemes:");

    let red_embed = Color::red().embed("CRITICAL");
    let green_embed = Color::green().embed("HEALTHY");
    let blue_embed = Color::blue().embed("INFO");
    let yellow_embed = Color::yellow().embed("CAUTION");

    sprtln_with_embeds!(
        Color::cyan().bold(),
        "System status: {} services are {}, {} logging active, {} for updates",
        &[red_embed, green_embed, blue_embed, yellow_embed]
    );

    println!();

    // Section 7: Verbosity-gated embedding
    println!("7. Verbosity-gated embedding (svprtln_with_embeds!):");

    let debug_detail = Role::Debug.embed("connection pool size: 10");
    let trace_detail = Role::Trace.embed("query execution time: 0.5ms");

    svprtln_with_embeds!(
        Role::Info,
        Verbosity::Normal,
        "Database status with details: {} and {}",
        &[debug_detail, trace_detail]
    );

    svprtln_with_embeds!(
        Role::Debug,
        Verbosity::Verbose,
        "This verbose message contains embedded {}: {}",
        &[
            Role::Warning.embed("potential issue"),
            Role::Code.embed("check_memory()")
        ]
    );

    println!();

    // Section 8: Using Embedded::new directly
    println!("8. Using Embedded::new directly:");

    let custom_style = Style::from(Role::Emphasis).italic().underline();
    let custom_embed = Embedded::new(custom_style, "custom styled text");

    sprtln_with_embeds!(Role::Normal, "This contains {}", &[custom_embed]);

    println!();

    // Section 9: Multiple embeddings in sequence
    println!("9. Multiple embeddings showing style preservation:");

    let embeds = vec![
        Role::Code.embed("fn main()"),
        Role::Error.embed("panic!()"),
        Role::Success.embed("Ok(())"),
        Role::Warning.embed("unsafe"),
        Role::Info.embed("Result<T, E>"),
    ];

    sprtln_with_embeds!(
        Style::from(Role::Heading3).bold().italic(),
        "Rust functions like {} can {} or return {}. Be careful with {} blocks, prefer {}",
        &embeds
    );

    println!();

    // Section 10: Demonstrating the .embed() method on different types
    println!("10. Using .embed() method on different StyleLike types:");

    let role_embed = Role::Code.embed("role-based");
    let style_embed = Style::from(Role::Error).bold().embed("style-based");
    let color_embed = Color::magenta().embed("color-based");
    let ref_embed = (&Role::Success).embed("reference-based");

    sprtln_with_embeds!(
        Role::Normal,
        "Embedding types: {}, {}, {}, {}",
        &[role_embed, style_embed, color_embed, ref_embed]
    );

    println!("\n=== End of StyleLike Extensions Demo ===");
    println!("\nKey benefits:");
    sprtln!(
        Role::Success,
        "✓ Ergonomic: sprtln!(Role::Code, \"message\") vs cprtln!(Role::Code, \"message\")"
    );
    sprtln!(
        Role::Success,
        "✓ Embedding: Preserves outer styles when nesting different styled content"
    );
    sprtln!(
        Role::Success,
        "✓ Consistent: Same API works with Role, Style, Color, and references"
    );
    sprtln!(
        Role::Success,
        "✓ Non-breaking: All existing cprtln!/cvprtln! code continues to work"
    );
}
