//! Ratatui Integration Test
//!
//! This example traces exactly which implementation is being called
//! when using `Style::themed()` with `ratatui `to identify why colors are wrong.
//!
//! Run with:
//! ```bash
//! cargo run -p thag_styling --example ratatui_integration_test --features "config,ratatui_support"
//! ```

#[cfg(feature = "ratatui_support")]
use ratatui::style::{Color, Style as RataStyle};
#[cfg(feature = "ratatui_support")]
use thag_styling::{paint_for_role, styling::index_to_rgb, Role, Style, ThemedStyle};

#[cfg(feature = "ratatui_support")]
fn main() {
    println!("🔍 Ratatui Integration Tracing Test\n");

    let test_roles = [
        ("HD1", Role::Heading1),
        ("Code", Role::Code),
        ("Emphasis", Role::Emphasis),
        ("Error", Role::Error),
        ("Success", Role::Success),
    ];

    for (name, role) in test_roles {
        println!("━━━ Testing {} (Role::{:?}) ━━━", name, role);

        // Step 1: Get thag Style directly
        let thag_style = Style::from(role);
        println!("1. thag Style::from(role):");
        if let Some(color_info) = &thag_style.foreground {
            match &color_info.value {
                thag_styling::ColorValue::TrueColor { rgb } => {
                    println!(
                        "   RGB({}, {}, {}) - Index: {}",
                        rgb[0], rgb[1], rgb[2], color_info.index
                    );
                }
                thag_styling::ColorValue::Color256 { color256 } => {
                    println!("   Color256({color256}) - Index: {}", color_info.index);
                }
                thag_styling::ColorValue::Basic { index } => {
                    println!("   Basic(Index: {index}",);
                }
            }
        } else {
            println!("   NO FOREGROUND COLOR");
        }

        // Step 2: Call ThemedStyle::themed() explicitly
        let themed_rata_style = RataStyle::themed(role);
        println!("2. RataStyle::themed(role):");
        println!("   Result: {:?}", themed_rata_style);

        // Step 3: Call From<&Style> legacy implementation
        let from_rata_style = RataStyle::from(&thag_style);
        println!("3. RataStyle::from(&thag_style) [legacy]:");
        println!("   Result: {:?}", from_rata_style);

        // Step 4: Check if they're the same
        if format!("{:?}", themed_rata_style) == format!("{:?}", from_rata_style) {
            println!("   ✅ Both methods produce the same result");
        } else {
            println!("   ❌ DIFFERENT RESULTS - This explains the color mismatch!");
            println!("      ThemedStyle: {:?}", themed_rata_style);
            println!("      From legacy: {:?}", from_rata_style);
        }

        // Step 5: Show what the actual painted color looks like
        println!("4. Actual painted color:");
        println!(
            "   {}",
            paint_for_role(role, &format!("This is {} text", name))
        );

        println!();
    }

    // Additional diagnostics
    println!("🔧 Additional Diagnostics:");

    // Test if Style::themed() is calling our implementation or not
    println!("\nTesting trait resolution...");
    let hd1_style = RataStyle::themed(Role::Heading1);
    let code_style = RataStyle::themed(Role::Code);

    println!("HD1 ratatui style: {:?}", hd1_style);
    println!("Code ratatui style: {:?}", code_style);

    // Test the color conversion function directly
    let hd1_thag = Style::from(Role::Heading1);
    if let Some(color_info) = &hd1_thag.foreground {
        let direct_conversion = match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => Color::Rgb(rgb[0], rgb[1], rgb[2]),
            thag_styling::ColorValue::Color256 { color256 } => Color::Indexed(*color256),
            thag_styling::ColorValue::Basic { .. } => Color::Indexed(color_info.index),
        };
        println!("Direct color conversion for HD1: {:?}", direct_conversion);

        let legacy_conversion = Color::Indexed(color_info.index);
        let [r, g, b] = index_to_rgb(color_info.index);
        println!("Legacy conversion for HD1: {legacy_conversion:?}, maps to RGB=({r},{g},{b})",);

        if format!("{:?}", direct_conversion) != format!("{:?}", legacy_conversion) {
            println!("⚠️  CONVERSION MISMATCH - Legacy uses index, direct uses RGB!");
        }
    }

    println!("\n💡 If you see 'DIFFERENT RESULTS' or 'CONVERSION MISMATCH',");
    println!("   that explains why colors are wrong in your ratatui app!");
}

#[cfg(not(feature = "ratatui_support"))]
fn main() {}
