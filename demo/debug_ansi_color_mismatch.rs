/*[toml]
[dependencies]
thag_styling = { version = "1, thag-auto" }
*/

/// Debug ANSI color generation mismatch
///
/// This script investigates why RGB values don't match the displayed colors
/// by examining the ANSI codes being generated for specific RGB values.
//# Purpose: Diagnose ANSI color generation mismatch in dynamic color system
//# Categories: color, styling, debugging, terminal
use thag_styling::{ColorInfo, ColorInitStrategy, ColorSupport, Style, TermAttributes};

fn rgb_to_hex(rgb: (u8, u8, u8)) -> String {
    let (r, g, b) = rgb;
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn test_specific_colors() {
    println!("🔍 Testing Specific Color Values from Your Output");
    println!("{}", "=".repeat(60));
    println!();

    let test_colors = vec![
        ("Background", [248, 248, 248]), // Should be very light gray
        ("Heading1", [107, 37, 65]),     // Should be dark purple/burgundy
        ("Heading2", [177, 61, 108]),    // Should be pink/magenta
        ("Heading3", [134, 189, 189]),   // Should be duck-egg blue/cyan
        ("Error", [61, 16, 16]),         // Should be dark red
        ("Warning", [107, 69, 50]),      // Should be brown
        ("Success", [24, 59, 59]),       // Should be dark teal
        ("Info", [33, 83, 83]),          // Should be teal
    ];

    let term_attrs = TermAttributes::get_or_init();
    println!(
        "Current terminal color support: {:?}",
        term_attrs.color_support
    );
    println!();

    for (name, rgb) in test_colors {
        let [r, g, b] = rgb;
        let color_info = ColorInfo::rgb(r, g, b);
        let style = Style::with_rgb(rgb);

        // Show what ANSI codes are generated for different support levels
        println!("🎨 {name}: RGB({r}, {g}, {b})");
        println!("   Expected color: {} (hex)", rgb_to_hex((r, g, b)));

        // Show ANSI for all support levels
        let support_levels = [
            ColorSupport::TrueColor,
            ColorSupport::Color256,
            ColorSupport::Basic,
        ];

        for support in support_levels {
            let ansi = color_info.to_ansi_for_support(support);
            print!("   {:12} ANSI: {:25} → ", format!("{:?}:", support), ansi);

            // Create a temporary style with this specific ANSI to see what it renders
            if support == term_attrs.color_support {
                print!("{ansi}████ THIS IS WHAT YOU SEE\x1b[0m");
            } else {
                print!("{ansi}████ preview\x1b[0m");
            }
            println!();
        }

        // Show what the paint method actually produces
        let painted = style.paint("████ ACTUAL PAINTED RESULT");
        println!("   Style.paint(): {painted}");
        println!("   Color index: {}", color_info.index);
        println!();
    }
}

fn test_color_index_mapping() {
    println!("🔢 Testing Color Index Mapping");
    println!("{}", "=".repeat(60));
    println!();

    // Test some specific RGB values and see what color indices they map to
    let test_rgbs = vec![
        [134, 189, 189], // The duck-egg blue that shows as pink
        [177, 61, 108],  // The pink that might be showing wrong
        [107, 37, 65],   // The dark purple
    ];

    for [r, g, b] in test_rgbs {
        let color_info = ColorInfo::rgb(r, g, b);
        println!("RGB({}, {}, {}) → index: {}", r, g, b, color_info.index);

        // Show what this index looks like when rendered directly as 256-color
        let index_ansi = format!("\x1b[38;5;{}m", color_info.index);
        println!(
            "   Index {} as 256-color: {index_ansi}████ Direct index color\x1b[0m",
            color_info.index
        );

        // Show what TrueColor version looks like
        let rgb_ansi = format!("\x1b[38;2;{r};{g};{b}m");
        println!("   RGB as TrueColor:      {rgb_ansi}████ TrueColor version\x1b[0m");
        println!();
    }
}

fn test_find_closest_color_function() {
    println!("🎯 Testing find_closest_color Function");
    println!("{}", "=".repeat(60));
    println!();

    // Test the find_closest_color function directly if we can access it
    let duck_egg = (134, 189, 189);
    let pink = (177, 61, 108);

    println!("This will help us understand if the color index mapping is correct:");
    println!("Duck-egg RGB{duck_egg:?} should map to a cyan-ish index");
    println!("Pink RGB{pink:?} should map to a magenta-ish index");
    println!();

    // Create ColorInfo for these and see the indices
    let duck_egg_info = ColorInfo::rgb(duck_egg.0, duck_egg.1, duck_egg.2);
    let pink_info = ColorInfo::rgb(pink.0, pink.1, pink.2);

    println!(
        "Duck-egg → index {}: \x1b[38;5;{}m████\x1b[0m",
        duck_egg_info.index, duck_egg_info.index,
    );
    println!(
        "Pink → index {}: \x1b[38;5;{}m████\x1b[0m",
        pink_info.index, pink_info.index,
    );
}

fn test_current_terminal_detection() {
    println!("🖥️  Current Terminal Detection");
    println!("{}", "=".repeat(60));
    println!();

    let term_attrs = TermAttributes::get_or_init();
    println!("Detected color support: {:?}", term_attrs.color_support);
    println!("Theme name: {}", term_attrs.theme.name);
    println!("How initialized: {:?}", term_attrs.how_initialized);

    if let Some(bg_rgb) = term_attrs.term_bg_rgb {
        println!("Terminal background RGB: {bg_rgb:?}");
    }
    println!("Terminal background luma: {:?}", term_attrs.term_bg_luma);
    println!();

    // Show a simple test to confirm color support detection
    println!("Testing simple colors:");
    let red = Style::with_rgb([255, 0, 0]);
    let green = Style::with_rgb([0, 255, 0]);
    let blue = Style::with_rgb([0, 0, 255]);

    println!("Red:   {}", red.paint("████ Should be red"));
    println!("Green: {}", green.paint("████ Should be green"));
    println!("Blue:  {}", blue.paint("████ Should be blue"));
}

fn main() {
    // Initialize terminal attributes
    TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);

    println!("🐛 ANSI Color Generation Diagnostic");
    println!("{}", "=".repeat(80));
    println!();
    println!("Investigating why RGB values don't match displayed colors...");
    println!();

    test_current_terminal_detection();
    println!();

    test_specific_colors();
    println!();

    test_color_index_mapping();
    println!();

    test_find_closest_color_function();

    println!("🔍 Analysis:");
    println!("- If TrueColor support shows correct colors but current terminal doesn't,");
    println!("  the issue is in color downgrading (find_closest_color function)");
    println!("- If indices are swapped/wrong, there may be an issue with color mapping");
    println!("- If ANSI codes look right but colors are wrong, it's a terminal issue");
}
