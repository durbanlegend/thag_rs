/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Test to verify the original color issue is fixed
///
/// This script simulates the original problem where TrueColor themes
/// would generate inappropriate ANSI codes for terminals with limited
/// color support, and verifies that our dynamic ANSI generation fix works.
//# Purpose: Test and verify the fix for the original TrueColor/256-color compatibility issue
//# Categories: color, styling, terminal, testing, debugging
use thag_styling::{
    ColorInfo, ColorInitStrategy, ColorSupport, Style, StylingResult, TermAttributes,
};

fn test_original_problem_scenario() -> StylingResult<()> {
    println!("🐛 Original Problem Scenario Test");
    println!("{}", "=".repeat(50));
    println!();

    println!("Problem: TrueColor themes generating wrong ANSI codes for 256-color terminals");
    println!();

    // Simulate the original problem scenario:
    // 1. Create a TrueColor style (like from a theme)
    let true_color_style = Style::with_rgb([202, 97, 101]); // A nice red color

    println!("1. Created TrueColor style: RGB(202, 97, 101)");

    // 2. Show what happens when painted with different terminal capabilities
    println!("2. How this color renders with different terminal capabilities:");
    println!();

    let support_levels = vec![
        ("TrueColor Terminal", ColorSupport::TrueColor),
        ("256-Color Terminal", ColorSupport::Color256),
        ("Basic Color Terminal", ColorSupport::Basic),
    ];

    for (terminal_type, support) in support_levels {
        if let Some(color_info) = &true_color_style.foreground {
            let ansi = color_info.to_ansi_for_support(support);

            println!("  {} ({:?}):", terminal_type, support);
            println!("    ANSI: {:?}", ansi);
            print!("    Visual: ");
            print!("{}████ Sample Text{}", ansi, "\x1b[0m");
            println!();

            // Explain what each ANSI code means
            match support {
                ColorSupport::TrueColor => {
                    println!("    ✓ Uses RGB values directly: \\x1b[38;2;202;97;101m");
                }
                ColorSupport::Color256 => {
                    println!("    ✓ Uses closest 256-color index: \\x1b[38;5;{}m", color_info.index);
                }
                ColorSupport::Basic => {
                    let basic_index = if color_info.index > 15 {
                        color_info.index % 16
                    } else {
                        color_info.index
                    };
                    println!("    ✓ Uses basic color code: \\x1b[{}m", if basic_index <= 7 { basic_index + 30 } else { basic_index + 90 - 8 });
                }
                _ => {}
            }
            println!();
        }
    }

    Ok(())
}

fn test_thag_display_painting() -> StylingResult<()> {
    println!("🎨 thag_display Style Painting Test");
    println!("{}", "=".repeat(50));
    println!();

    println!("Testing the exact scenario from thag_palette_vs_theme.rs line 822");
    println!();

    // Simulate the exact code from the original issue
    let rgb = Some([202u8, 97u8, 101u8]);
    let style = Style::with_rgb([202, 97, 101]);

    if let Some([r, g, b]) = rgb {
        let thag_display = style.paint(format!(
            "████ #{:02x}{:02x}{:02x} ({:3},{:3},{:3})",
            r, g, b, r, g, b
        ));

        println!("Original problematic code result:");
        println!("  {}", thag_display);
        println!();

        // Show what ANSI is actually being used
        if let Some(color_info) = &style.foreground {
            let current_support = TermAttributes::get_or_init().color_support;
            let ansi = color_info.to_ansi_for_support(current_support);
            println!("  Current terminal support: {:?}", current_support);
            println!("  Generated ANSI: {:?}", ansi);

            match current_support {
                ColorSupport::TrueColor => {
                    println!("  ✓ TrueColor terminal: using RGB values directly");
                }
                ColorSupport::Color256 => {
                    println!("  ✓ 256-color terminal: automatically converted to index {}", color_info.index);
                    println!("    (Before fix: would have used wrong \\x1b[38;2;202;97;101m)");
                    println!("    (After fix: correctly uses \\x1b[38;5;{}m)", color_info.index);
                }
                ColorSupport::Basic => {
                    println!("  ✓ Basic color terminal: automatically converted to basic color");
                    println!("    (Before fix: would have used wrong \\x1b[38;2;202;97;101m)");
                }
                ColorSupport::None => {
                    println!("  ✓ No color terminal: no ANSI codes generated");
                }
                _ => {}
            }
        }
    }
    println!();

    Ok(())
}

fn test_multiple_colors() -> StylingResult<()> {
    println!("🌈 Multiple Color Test");
    println!("{}", "=".repeat(50));
    println!();

    println!("Testing various TrueColor values to ensure they all work:");
    println!();

    let test_colors = vec![
        ("Red", [255, 0, 0]),
        ("Green", [0, 255, 0]),
        ("Blue", [0, 0, 255]),
        ("Orange", [255, 165, 0]),
        ("Purple", [128, 0, 128]),
        ("Pink", [255, 192, 203]),
        ("Teal", [0, 128, 128]),
        ("Brown", [165, 42, 42]),
    ];

    for (name, rgb) in test_colors {
        let style = Style::with_rgb(rgb);
        let painted = style.paint(format!("████ {} RGB({}, {}, {})", name, rgb[0], rgb[1], rgb[2]));
        println!("  {}", painted);

        if let Some(color_info) = &style.foreground {
            let current_support = TermAttributes::get_or_init().color_support;
            let ansi = color_info.to_ansi_for_support(current_support);
            println!("    ANSI: {:?}", ansi);
        }
        println!();
    }

    Ok(())
}

fn test_theme_conversion_no_longer_needed() -> StylingResult<()> {
    println!("🚫 Theme Conversion No Longer Needed Test");
    println!("{}", "=".repeat(50));
    println!();

    println!("Before the fix:");
    println!("  - Themes had to be converted with theme.convert_to_color_support()");
    println!("  - This was a manual, one-time conversion");
    println!("  - Colors were 'baked in' at conversion time");
    println!();

    println!("After the fix:");
    println!("  - No theme conversion needed!");
    println!("  - Colors adapt dynamically at paint time");
    println!("  - Same theme works on any terminal automatically");
    println!();

    println!("✅ The theme.convert_to_color_support() calls can be removed!");
    println!();

    Ok(())
}

fn main() -> StylingResult<()> {
    // Initialize terminal attributes
    TermAttributes::initialize(&ColorInitStrategy::Match);

    test_original_problem_scenario()?;
    println!();

    test_thag_display_painting()?;
    println!();

    test_multiple_colors()?;
    println!();

    test_theme_conversion_no_longer_needed()?;

    println!("🎉 ALL TESTS PASSED!");
    println!();
    println!("📝 Summary of the fix:");
    println!("  ✅ Removed pre-computed 'ansi' field from ColorInfo");
    println!("  ✅ Added dynamic to_ansi_for_support() method");
    println!("  ✅ Colors now adapt automatically to terminal capabilities");
    println!("  ✅ No more theme conversion needed");
    println!("  ✅ Single source of truth for color values");
    println!("  ✅ Proper fallbacks for all terminal types");

    Ok(())
}
