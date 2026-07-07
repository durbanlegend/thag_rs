//! Test dynamic theme changing to verify that `ColorInfo` ansi value can be updated
//!
//! This example demonstrates that we can now change themes dynamically without
//! being stuck with static ANSI color codes from `Box::leak`.
//!
//! Run with: `cargo run -p thag_styling --example test_dynamic_theme`

use thag_styling::{ColorInfo, ColorInitStrategy, Role, Style, TermAttributes};

fn main() {
    println!("🔄 Testing Dynamic Theme Changes\n");

    // Run the test twice under different strategies. Because TermAttributes uses a
    // write-once OnceLock singleton, we build a fresh owned instance for each strategy
    // and push it onto the thread-local context stack via `with_context`. Code inside
    // the closure (and everything it calls) sees that instance via
    // `TermAttributes::current()`.
    for strategy in [ColorInitStrategy::Default, ColorInitStrategy::Match] {
        let attrs = TermAttributes::build_from_strategy(&strategy);
        attrs.with_context(|| run_test(&strategy));
    }

    println!("\n✅ Dynamic theme change test complete!");
    println!("   Colors are now stored as owned Strings instead of static references,");
    println!("   allowing for proper theme switching and dynamic color generation.");
    println!("   Each ColorInfo instance has its own independent ANSI string that can");
    println!("   be updated or replaced without affecting other instances.");
}

fn run_test(strategy: &ColorInitStrategy) {
    // Use current() so we pick up the context pushed by with_context above.
    let attrs = TermAttributes::current();
    let theme_name = attrs.theme.name.clone();
    println!("1. Strategy {:?} → theme: {theme_name}", strategy);

    // Create a style for the Success role (also uses current() internally via for_role / From<Role>)
    let success_style = Style::from(Role::Success);
    println!(
        "   Initial Success color ANSI: {:?}",
        success_style
            .foreground
            .as_ref()
            .map(|c| c.to_ansi_for_support(attrs.color_support))
    );

    // Paint some text with the initial theme
    let painted1 = success_style.paint("Success with initial theme");
    println!("   Painted: {}", painted1);

    println!();

    let mut color_info: ColorInfo;

    // Test that we can create multiple different colors dynamically
    println!("2. Testing multiple dynamic RGB colors:");
    let colors = [
        ([255, 0, 0], "Red"),
        ([0, 255, 0], "Green"),
        ([0, 0, 255], "Blue"),
        ([255, 255, 0], "Yellow"),
        ([255, 0, 255], "Magenta"),
    ];

    let color_support = attrs.color_support;
    println!("color_support={color_support}");

    for (rgb, name) in &colors {
        // Create ColorInfo with RGB values
        color_info = ColorInfo::rgb(rgb[0], rgb[1], rgb[2]);
        let style = Style::fg(color_info);

        let painted = style.paint(format!("{name} color remapped by theme {theme_name}"));
        println!("{painted}");

        // Verify the ANSI code is properly stored as owned String
        if let Some(color_info) = &style.foreground {
            println!(
                "      ANSI: {:?}",
                color_info.to_ansi_for_support(color_support)
            );
        }
    }

    println!();

    // Test 256-color mode
    println!("3. Testing 256-color palette:");
    for i in [196, 46, 21, 226, 201] {
        // Some nice colors from 256-color palette
        color_info = ColorInfo::color256(i);
        let style = Style::fg(color_info);

        let painted = style.paint(format!("Color {i} remapped by theme {theme_name}"));
        println!("   {painted}");

        if let Some(color_info) = &style.foreground {
            println!(
                "      ANSI: {:?}",
                color_info.to_ansi_for_support(color_support)
            );
        }
    }

    println!();

    // Test that multiple instances can have different colors (proving no static leak)
    println!("4. Testing independence of color instances:");
    let style1 = Style::fg(ColorInfo::rgb(255, 100, 100));
    // Light red
    let style2 = Style::fg(ColorInfo::rgb(100, 255, 100));
    // Light green
    let style3 = Style::fg(ColorInfo::rgb(100, 100, 255));
    // Light blue

    println!("   Style 3: {}", style3.paint("Remapped Light Blue"));
    println!("   Style 1: {}", style1.paint("Remapped Light Red"));
    println!("   Style 2: {}", style2.paint("Remapped Light Green"));

    // Verify each has its own independent ANSI string
    if let (Some(c1), Some(c2), Some(c3)) =
        (&style1.foreground, &style2.foreground, &style3.foreground)
    {
        println!("   ANSI codes are independent:");
        println!("     Style 1: {:?}", c1.to_ansi_for_support(color_support));
        println!("     Style 2: {:?}", c2.to_ansi_for_support(color_support));
        println!("     Style 3: {:?}", c3.to_ansi_for_support(color_support));
    }
    println!();
}
