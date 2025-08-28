/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config"] }
*/

/// Debug demo to check theme loading and color values
///
/// This demonstrates:
/// 1. Current theme information
/// 2. Color values for each role
/// 3. RGB values and indices
/// 4. Comparison with expected palette colors
//# Purpose: Debug theme and color loading issues
//# Categories: debugging, styling, colors
use thag_styling::{ColorInitStrategy, Role, Styleable, TermAttributes};

fn main() {
    // Initialize styling system
    TermAttributes::initialize(&ColorInitStrategy::Match);

    println!("=== Color Debug Demo ===\n");

    // Get term attributes to check theme
    let term_attrs = TermAttributes::get_or_init();

    println!("1. Terminal Attributes:");
    println!("   Color Support: {:?}", term_attrs.color_support);
    println!("   Background Luma: {:?}", term_attrs.term_bg_luma);
    println!("   Theme Name: {}", term_attrs.theme.name);
    println!("   Theme Description: {}", term_attrs.theme.description);

    println!("\n2. Theme Palette Colors (RGB values):");
    let theme = &term_attrs.theme;

    // Print each role's color info
    let roles = [
        ("Error", Role::Error),
        ("Warning", Role::Warning),
        ("Success", Role::Success),
        ("Info", Role::Info),
        ("Heading1", Role::Heading1),
        ("Heading2", Role::Heading2),
        ("Heading3", Role::Heading3),
        ("Normal", Role::Normal),
        ("Emphasis", Role::Emphasis),
        ("Code", Role::Code),
        ("Subtle", Role::Subtle),
        ("Hint", Role::Hint),
        ("Debug", Role::Debug),
        ("Link", Role::Link),
        ("Quote", Role::Quote),
        ("Commentary", Role::Commentary),
    ];

    for (name, role) in &roles {
        let style = theme.style_for(*role);
        print!("   {:>10}: ", name);

        if let Some(color_info) = &style.foreground {
            let rgb = match &color_info.value {
                thag_styling::ColorValue::TrueColor { rgb } => {
                    format!("RGB({:3},{:3},{:3})", rgb[0], rgb[1], rgb[2])
                }
                thag_styling::ColorValue::Color256 { color256 } => {
                    format!("256-color: {}", color256)
                }
                thag_styling::ColorValue::Basic { index, .. } => format!("Basic: {}", index),
            };
            println!(
                "{} Index={:3} ANSI={}",
                rgb, color_info.index, color_info.ansi
            );
        } else {
            println!("No foreground color");
        }
    }

    println!("\n3. Visual color test:");
    println!("   {}", "Error message".error());
    println!("   {}", "Warning message".warning());
    println!("   {}", "Success message".success());
    println!("   {}", "Info message".info());
    println!("   {}", "Heading1 text".heading1());
    println!("   {}", "Heading2 text".heading2());
    println!("   {}", "Heading3 text".heading3());
    println!("   {}", "Normal text".normal());
    println!("   {}", "Emphasis text".emphasis());
    println!("   {}", "Code text".code());
    println!("   {}", "Subtle text".subtle());
    println!("   {}", "Hint text".hint());
    println!("   {}", "Debug text".debug());
    println!("   {}", "Trace text".trace());

    println!("\n4. Environment check:");
    if let Ok(term) = std::env::var("TERM") {
        println!("   TERM: {}", term);
    }
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        println!("   COLORTERM: {}", colorterm);
    }
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        println!("   TERM_PROGRAM: {}", term_program);
    }

    println!("\n5. Background RGB values:");
    println!("   Theme BG RGB: {:?}", theme.bg_rgbs);

    println!("\n6. Specific palette colors:");
    let palette = &theme.palette;
    println!(
        "   Error RGB:    {:?}",
        extract_rgb_from_style(&palette.error)
    );
    println!(
        "   Heading1 RGB: {:?}",
        extract_rgb_from_style(&palette.heading1)
    );
    println!(
        "   Warning RGB:  {:?}",
        extract_rgb_from_style(&palette.warning)
    );
    println!(
        "   Success RGB:  {:?}",
        extract_rgb_from_style(&palette.success)
    );
}

fn extract_rgb_from_style(style: &thag_styling::Style) -> Option<(u8, u8, u8)> {
    if let Some(color_info) = &style.foreground {
        match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            _ => None,
        }
    } else {
        None
    }
}
