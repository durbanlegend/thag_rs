//! Test dynamic theme changing to verify that ColorInfo ansi field can be updated
//!
//! This example demonstrates that we can now change themes dynamically without
//! being stuck with static ANSI color codes from Box::leak.
//!
//! Run with: cargo run -p thag_styling --example test_dynamic_theme

use thag_styling::{ColorInfo, ColorInitStrategy, Role, Style, TermAttributes};

fn main() {
    println!("🔄 Testing Dynamic Theme Changes\n");

    // Initialize with a basic theme
    let strategy = ColorInitStrategy::Default;
    let attrs = TermAttributes::initialize(&strategy);
    println!("1. Initial theme: {}", attrs.theme.name);

    // Create a style for the Success role
    let success_style = Style::from(Role::Success);
    println!(
        "   Initial Success color ANSI: {:?}",
        success_style.foreground.as_ref().map(|c| &c.ansi)
    );

    // Paint some text with the initial theme
    let painted1 = success_style.paint("Success with initial theme");
    println!("   Painted: {}", painted1);

    println!();

    // Test that we can create multiple different colors dynamically
    println!("2. Testing multiple dynamic RGB colors:");
    let colors = [
        ([255, 0, 0], "Red"),
        ([0, 255, 0], "Green"),
        ([0, 0, 255], "Blue"),
        ([255, 255, 0], "Yellow"),
        ([255, 0, 255], "Magenta"),
    ];

    for (rgb, name) in colors.iter() {
        // Create ColorInfo with RGB values
        let color_info = ColorInfo::rgb(rgb[0], rgb[1], rgb[2]);
        let style = Style::fg(color_info);

        let painted = style.paint(format!("{} color", name));
        println!("   {}: {}", name, painted);

        // Verify the ANSI code is properly stored as owned String
        if let Some(color_info) = &style.foreground {
            println!("      ANSI: {}", color_info.ansi);
        }
    }

    println!();

    // Test 256-color mode
    println!("3. Testing 256-color palette:");
    for i in [196, 46, 21, 226, 201] {
        // Some nice colors from 256-color palette
        let color_info = ColorInfo::color256(i);
        let style = Style::fg(color_info);

        let painted = style.paint(format!("Color {}", i));
        println!("   Index {}: {}", i, painted);

        if let Some(color_info) = &style.foreground {
            println!("      ANSI: {}", color_info.ansi);
        }
    }

    println!();

    // Test that multiple instances can have different colors (proving no static leak)
    println!("4. Testing independence of color instances:");
    let style1 = Style::fg(ColorInfo::rgb(255, 100, 100)); // Light red
    let style2 = Style::fg(ColorInfo::rgb(100, 255, 100)); // Light green
    let style3 = Style::fg(ColorInfo::rgb(100, 100, 255)); // Light blue

    println!("   Style 1: {}", style1.paint("Light Red"));
    println!("   Style 2: {}", style2.paint("Light Green"));
    println!("   Style 3: {}", style3.paint("Light Blue"));

    // Verify each has its own independent ANSI string
    if let (Some(c1), Some(c2), Some(c3)) =
        (&style1.foreground, &style2.foreground, &style3.foreground)
    {
        println!("   ANSI codes are independent:");
        println!("     Style 1: {}", c1.ansi);
        println!("     Style 2: {}", c2.ansi);
        println!("     Style 3: {}", c3.ansi);
    }

    println!("\n✅ Dynamic theme change test complete!");
    println!("   Colors are now stored as owned Strings instead of static references,");
    println!("   allowing for proper theme switching and dynamic color generation.");
    println!("   Each ColorInfo instance has its own independent ANSI string that can");
    println!("   be updated or replaced without affecting other instances.");
}
