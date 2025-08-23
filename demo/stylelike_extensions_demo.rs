/*[toml]
[dependencies]
# thag_common = { version = "0.2, thag-auto" }
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }
*/
/// Demo script showcasing Styler trait extensions for ergonomic printing and embedding
///
/// This script demonstrates:
/// 1. cprtln! - ergonomic styled printing macro
/// 2. cvprtln! - verbosity-gated styled printing macro
/// 3. cprtln_with_embeds! - styled printing with embedded styled content
/// 4. cvprtln_with_embeds! - verbosity-gated embedded styled printing
///
/// The Styler trait allows both Role and Style to be used interchangeably,
/// and the embedding feature preserves outer styles when nesting different styles.
//# Purpose: Demo Styler extensions for ergonomic styling and embedding
//# Categories: styling, ergonomics, embedding
use thag_styling::styling::{Color, Embedded, Role, Style, Styler, TermAttributes};
use thag_styling::{
    cprtln, cprtln_with_embeds, cvprtln, cvprtln_with_embeds, ColorInitStrategy, Verbosity,
};

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== Styler Extensions Demo ===\n");

    // Section 1: Basic cprtln! usage
    println!("1. Basic cprtln! usage:");
    cprtln!(Role::Heading1, "Primary Heading");
    cprtln!(Role::Heading2, "Secondary Heading");
    cprtln!(Role::Code, "let code_snippet = \"Hello, World!\";");
    cprtln!(Role::Error, "Error: Something went wrong!");
    cprtln!(Role::Success, "Success: Operation completed");
    cprtln!(Role::Warning, "Warning: Please be careful");
    cprtln!(Role::Info, "Info: Just so you know");

    // With format arguments
    let user = "Alice";
    let count = 42;
    cprtln!(Role::Normal, "User {} has {} items", user, count);

    println!();

    // Section 2: cprtln! with Style modifications
    println!("2. cprtln! with Style modifications:");
    cprtln!(Style::from(Role::Normal).bold(), "Bold normal text");
    cprtln!(Style::from(Role::Info).italic(), "Italic info text");
    cprtln!(Style::from(Role::Warning).underline(), "Underlined warning");
    cprtln!(Color::yellow().bold().italic(), "Yellow bold italic");

    println!();

    // Section 3: Verbosity-gated printing
    println!("3. Verbosity-gated printing (cvprtln!):");
    cvprtln!(
        Role::Debug,
        Verbosity::Normal,
        "This shows at Normal verbosity"
    );
    cvprtln!(
        Role::Debug,
        Verbosity::Verbose,
        "This shows at Verbose verbosity"
    );
    cvprtln!(
        Role::Trace,
        Verbosity::Debug,
        "This shows at Debug verbosity"
    );

    // This one might not show depending on current verbosity
    cvprtln!(
        Role::Trace,
        Verbosity::Debug,
        "This trace message might be filtered out"
    );

    println!();

    // Section 4: Basic embedding
    println!("4. Basic embedding with cprtln_with_embeds!:");

    let code_embed = Role::Code.embed("println!(\"Hello\")");
    let error_embed = Role::Error.embed("NullPointerException");
    let success_embed = Role::Success.embed("OK");

    cprtln_with_embeds!(
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

    cprtln_with_embeds!(
        Style::from(Role::Heading2).underline(),
        "System {} contains {} function {}",
        &[info_embed, warning_embed, code_embed2]
    );

    // With bold outer style
    let debug_embed = Role::Debug.embed("memory usage: 45%");
    let trace_embed = Role::Trace.embed("GC triggered");

    cprtln_with_embeds!(
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

    cprtln_with_embeds!(
        Color::cyan().bold(),
        "System status: {} services are {}, {} logging active, {} for updates",
        &[red_embed, green_embed, blue_embed, yellow_embed]
    );

    println!();

    // Section 7: Verbosity-gated embedding
    println!("7. Verbosity-gated embedding (cvprtln_with_embeds!):");

    let debug_detail = Role::Debug.embed("connection pool size: 10");
    let trace_detail = Role::Trace.embed("query execution time: 0.5ms");

    cvprtln_with_embeds!(
        Role::Info,
        Verbosity::Normal,
        "Database status with details: {} and {}",
        &[debug_detail, trace_detail]
    );

    cvprtln_with_embeds!(
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
    let custom_embed = Embedded::new(&custom_style, "custom styled text");

    cprtln_with_embeds!(Role::Normal, "This contains {}", &[custom_embed]);

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

    cprtln_with_embeds!(
        Style::from(Role::Heading3).bold().italic(),
        "Rust functions like {} can {} or return {}. Be careful with {} blocks, prefer {}",
        &embeds
    );

    println!();

    // Section 10: Demonstrating the .embed() method on different types
    println!("10. Using .embed() method on different Styler types:");

    let role_embed = Role::Code.embed("role-based");
    let style_embed = Style::from(Role::Error).bold().embed("style-based");
    let color_embed = Color::magenta().embed("color-based");
    let ref_embed = (&Role::Success).embed("reference-based");

    cprtln_with_embeds!(
        Role::Normal,
        "Embedding types: {}, {}, {}, {}",
        &[role_embed, style_embed, color_embed, ref_embed]
    );

    println!("\n=== End of Styler Extensions Demo ===");
    println!("\nKey benefits:");
    cprtln!(
        Role::Success,
        "✓ Ergonomic: cprtln!(Role::Code, \"message\") vs cprtln!(Role::Code, \"message\")"
    );
    cprtln!(
        Role::Success,
        "✓ Embedding: Preserves outer styles when nesting different styled content"
    );
    cprtln!(
        Role::Success,
        "✓ Consistent: Same API works with Role, Style, Color, and references"
    );
    cprtln!(
        Role::Success,
        "✓ Non-breaking: All existing cprtln!/cvprtln! code continues to work"
    );
}
