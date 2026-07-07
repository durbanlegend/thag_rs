/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "1, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "1, thag-auto", features = ["config"] }
*/

/// Demonstrates how `thag_styling` handles nested styles using `StyledString`
///
/// Style nesting is notoriously hard in most colour libraries: an inner styled
/// string emits `\x1b[0m` (full reset) when it ends, which kills any outer
/// colour that was active.  `thag_styling` solves this transparently — every
/// `StyledString` replaces inner reset sequences with attribute-specific resets
/// (`\x1b[22;23;24m`) followed by the outer style's ANSI codes, so the outer
/// context is always restored.  This works to arbitrary nesting depth.
//# Purpose: Demo of context-preserving nested styles with StyledString
//# Categories: concepts, prototype, styling
use thag_styling::{ColorInitStrategy, Role, Styleable, StyledString, Styler, TermAttributes};

fn main() {
    TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);

    println!("=== Nested Style Demo ===\n");

    // ------------------------------------------------------------------
    // 1. The classic problem with most colour libraries
    // ------------------------------------------------------------------
    println!("1. The nesting problem with most colour libraries");
    println!("   Inner styles emit \\x1b[0m (full reset), killing any outer colour.\n");

    // Manually simulate what a naive library does:
    let red_open = "\x1b[31m";
    let green_open = "\x1b[32m";
    let reset = "\x1b[0m";
    let broken =
        format!("{red_open}Outer red [  {green_open}inner green{reset}  ] outer red gone{reset}");
    println!("   Naive: {broken}");
    println!("   Raw:   {:?}\n", broken);

    // ------------------------------------------------------------------
    // 2. Single-level nesting with thag_styling
    // ------------------------------------------------------------------
    // All Styleable methods (.error(), .warning(), .code(), …) return a StyledString.
    // StyledString's Display calls to_styled(), which handles inner resets before printing.
    println!("2. Single-level nesting — thag_styling handles it automatically");

    let inner: StyledString = "inner warning".warning();
    let outer: StyledString = format!("Error context [ {} ] back to error", inner).error();
    println!("   {outer}");
    println!("   Raw: {:?}\n", outer.to_styled());

    // ------------------------------------------------------------------
    // 3. Multi-level nesting (three levels deep)
    // ------------------------------------------------------------------
    println!("3. Three levels of nesting");

    let level1: StyledString = "level-1 code".code();
    let level2 = format!("level-2 success [ {} ] still success", level1).success();
    let level3 = format!("level-3 warning [ {} ] still warning", level2).warning();
    println!("   {level3}");
    println!("   Raw: {:?}\n", level3.to_styled());

    // ------------------------------------------------------------------
    // 4. Mixing colour with text attributes (bold, italic, underline)
    // ------------------------------------------------------------------
    println!("4. Colour + text-attribute nesting");

    let underlined = "underlined heading".style_with(Role::Heading1.underline());
    let italic_inner = "italic emphasis".style_with(Role::Emphasis.italic());
    let outer_msg = format!(
        "Error: [ {} ] and [ {} ] — still error",
        underlined, italic_inner
    )
    .error();
    println!("   {outer_msg}");
    println!("   Raw: {:?}\n", outer_msg.to_styled());

    // ------------------------------------------------------------------
    // 5. Chaining attributes on a StyledString
    // ------------------------------------------------------------------
    // StyledString also exposes .bold(), .italic(), .dim(), .underline() for chaining.
    println!("5. Chaining .bold() / .italic() / .underline() on StyledString");

    let chained: StyledString = "bold italic warning".warning().bold().italic();
    println!("   {chained}");
    println!("   Raw: {:?}\n", chained.to_styled());

    // ------------------------------------------------------------------
    // 6. Nesting a StyledString inside another StyledString
    // ------------------------------------------------------------------
    println!("6. StyledString embedded inside another StyledString");

    let inner_styled = "critical".error().bold();
    // Display converts inner_styled → ANSI string; the outer .warning() then
    // wraps it and replaces every \x1b[0m with the warning-style restoration.
    let outer_styled = format!("Warning: {} — please act", inner_styled).warning();
    println!("   {outer_styled}");
    println!("   Raw: {:?}\n", outer_styled.to_styled());

    // ------------------------------------------------------------------
    // Summary
    // ------------------------------------------------------------------
    println!("=== How it works ===\n");
    println!("  StyledString::to_styled() does three things:");
    println!("  1. Calls replace_resets_with_style() on its content.");
    println!("     Every \\x1b[0m is replaced with \\x1b[22;23;24m (attribute-only");
    println!("     resets for bold/italic/underline) followed by the outer colour");
    println!("     codes — so the outer context is seamlessly restored.");
    println!("  2. Prepends the same \\x1b[22;23;24m + style prefix.");
    println!("  3. Appends a final \\x1b[0m.");
    println!();
    println!("  Result: nesting works at any depth with plain format!() calls.");
    println!("  No special macros, no embed arrays, no manual tracking needed.");
}
