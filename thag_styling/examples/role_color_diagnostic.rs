//! Role Color Mapping Diagnostic
//!
//! This example helps diagnose role-to-color mapping issues by showing
//! exactly what colors are assigned to each role in the current theme.
//!
//! Run with:
//! ```bash
//! cargo run --example role_color_diagnostic --features "color_detect,ratatui_support"
//! ```
#[cfg(feature = "ratatui_support")]
fn main() {
    use thag_styling::{paint_for_role, Role, Style, TermAttributes};

    println!("🎨 Role Color Mapping Diagnostic\n");

    // Get current theme info
    let term_attrs = TermAttributes::get_or_init();
    println!("Current Theme: {}", term_attrs.theme.name);
    println!("Background: {:?}", term_attrs.term_bg_luma);
    println!("Color Support: {:?}", term_attrs.color_support);
    println!();

    // Test all the roles with their aliases
    let test_roles = [
        ("Heading1 (HD1)", Role::Heading1),
        ("Heading2 (HD2)", Role::Heading2),
        ("Heading3 (HD3)", Role::Heading3),
        ("Error (ERR)", Role::Error),
        ("Warning (WARN)", Role::Warning),
        ("Success (SUCC)", Role::Success),
        ("Info", Role::Info),
        ("Code", Role::Code),
        ("Emphasis (EMPH)", Role::Emphasis),
        ("Normal (NORM)", Role::Normal),
        ("Subtle", Role::Subtle),
        ("Debug (DBUG)", Role::Debug),
        ("Hint", Role::Hint),
        ("Link", Role::Link),
        ("Quote", Role::Quote),
        ("Commentary", Role::Commentary),
    ];

    println!("Role Color Mappings:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for (name, role) in test_roles {
        // Get the thag Style
        let thag_style = Style::from(role);

        // Get the ratatui Style
        // let rata_style = RataStyle::themed(role);

        // Print role info
        print!("{:<20} │ ", name);

        if let Some(color_info) = &thag_style.foreground {
            match &color_info.value {
                thag_styling::ColorValue::TrueColor { rgb } => {
                    print!("RGB({:3},{:3},{:3}) ", rgb[0], rgb[1], rgb[2]);

                    // Color classification
                    let (r, g, b) = (rgb[0] as f32, rgb[1] as f32, rgb[2] as f32);
                    let brightness = (r * 0.299 + g * 0.587 + b * 0.114) / 255.0;

                    let color_name = if r > g && r > b && r > 150.0 {
                        "Red-ish"
                    } else if g > r && g > b && g > 150.0 {
                        "Green-ish"
                    } else if b > r && b > g && b > 150.0 {
                        "Blue-ish"
                    } else if r > 120.0 && g > 120.0 && b < 80.0 {
                        "Yellow-ish"
                    } else if r > 100.0 && g > 60.0 && b < 60.0 {
                        "Brown-ish"
                    } else if brightness > 0.7 {
                        "Light"
                    } else if brightness < 0.3 {
                        "Dark"
                    } else {
                        "Medium"
                    };

                    print!("{:<10} │ ", color_name);
                }
                thag_styling::ColorValue::Color256 { color256 } => {
                    print!("Color256({:3})      Medium     │ ", color256);
                }
                thag_styling::ColorValue::Basic { index } => {
                    print!("Basic( Index: {index}",);
                }
            }
        } else {
            print!("No Color            None       │ ");
        }

        // Show the actual colored text
        print!("{}", paint_for_role(role, "Sample Text"));

        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Test the specific issue mentioned
    println!("\n🔍 Specific Issue Analysis:");
    println!("Expected vs Actual colors:");

    println!("\nRole::HD1 (Heading1) - Expected: Blue-ish");
    let hd1_style = Style::from(Role::Heading1);
    if let Some(color_info) = &hd1_style.foreground {
        if let thag_styling::ColorValue::TrueColor { rgb } = &color_info.value {
            println!("  Actual: RGB({}, {}, {})", rgb[0], rgb[1], rgb[2]);
            if rgb[2] > rgb[0] && rgb[2] > rgb[1] {
                println!("  ✅ This IS blue-ish");
            } else if rgb[0] > rgb[2] && rgb[1] > rgb[2] {
                println!("  ⚠️  This is BROWN-ish, not blue!");
            } else {
                println!("  ❓ This is some other color");
            }
        }
    }

    println!("\nRole::Code - Expected: Brown/Gold-ish");
    let code_style = Style::from(Role::Code);
    if let Some(color_info) = &code_style.foreground {
        if let thag_styling::ColorValue::TrueColor { rgb } = &color_info.value {
            println!("  Actual: RGB({}, {}, {})", rgb[0], rgb[1], rgb[2]);
            if rgb[0] > rgb[2] && rgb[1] > rgb[2] && rgb[0] > 100 {
                println!("  ✅ This IS brown/gold-ish");
            } else if rgb[2] > rgb[0] && rgb[2] > rgb[1] {
                println!("  ⚠️  This is BLUE-ish, not brown!");
            } else {
                println!("  ❓ This is some other color");
            }
        }
    }

    println!("\nRole::Emphasis - Expected: ???");
    let emph_style = Style::from(Role::Emphasis);
    if let Some(color_info) = &emph_style.foreground {
        if let thag_styling::ColorValue::TrueColor { rgb } = &color_info.value {
            println!("  Actual: RGB({}, {}, {})", rgb[0], rgb[1], rgb[2]);
        }
    }

    // Check if there's a pattern mixup
    println!("\n🔄 Checking for Role Mixup Pattern:");
    let color_info = &Style::from(Role::Heading1).foreground;
    let heading_rgb = if let Some(color_info) = color_info {
        if let thag_styling::ColorValue::TrueColor { rgb } = &color_info.value {
            Some(rgb)
        } else {
            None
        }
    } else {
        None
    };

    let color_info = &Style::from(Role::Code).foreground;
    let code_rgb = if let Some(color_info) = color_info {
        if let thag_styling::ColorValue::TrueColor { rgb } = &color_info.value {
            Some(rgb)
        } else {
            None
        }
    } else {
        None
    };

    if let (Some(h_rgb), Some(c_rgb)) = (heading_rgb, code_rgb) {
        if h_rgb[0] > h_rgb[2] && c_rgb[2] > c_rgb[0] {
            println!("  ⚠️  POSSIBLE MIXUP: HD1 has brown color, Code has blue color");
            println!("      This suggests the theme has swapped these role assignments");
        }
    }

    // Check for missing foreground colors that would cause fallback
    println!("\n🔍 Checking for Missing Foreground Colors:");
    println!("(Missing colors fall back to hardcoded u8::from(&role) mapping)");

    let mut missing_count = 0;
    for (name, role) in test_roles {
        let thag_style = Style::from(role);
        if thag_style.foreground.is_none() {
            println!("  ⚠️  {} has NO foreground color - will use fallback", name);
            missing_count += 1;
        }
    }

    if missing_count == 0 {
        println!("  ✅ All roles have foreground colors defined");
    } else {
        println!("  ❌ {} roles missing foreground colors!", missing_count);
        println!("     This causes fallback to hardcoded ANSI color numbers");
        println!("     which explains the wrong colors in ratatui!");
    }

    println!("\n💡 If you see mismatched colors, the theme file may have incorrect role-to-color mappings.");
    println!(
        "   Check: themes/built_in/{}.toml",
        term_attrs.theme.name.to_lowercase().replace(' ', "_")
    );
}

#[cfg(not(feature = "ratatui_support"))]
fn main() {}
