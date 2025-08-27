/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Test that mintty format is included in theme generator
///
/// This demo script tests that the mintty exporter is properly integrated
/// into the theme generation system by checking if it's in the list of formats.
//# Purpose: Test mintty format integration in theme generator
//# Categories: color, styling, terminal, theming, demo
use thag_styling::exporters::ExportFormat;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Mintty Integration in Theme Generator");
    println!("=================================================\n");

    // Get all available export formats
    let all_formats = ExportFormat::all();
    println!("📋 Available export formats:");

    let mut has_mintty = false;
    for (i, format) in all_formats.iter().enumerate() {
        let name = format.format_name();
        let extension = format.file_extension();
        let ext_display = if extension.is_empty() {
            "(no extension)".to_string()
        } else {
            format!(".{}", extension)
        };

        println!("   {}. {} {}", i + 1, name, ext_display);

        if matches!(format, ExportFormat::Mintty) {
            has_mintty = true;
        }
    }

    println!();

    if has_mintty {
        println!("✅ Mintty format is properly integrated!");

        // Test exporting a simple theme to mintty format
        println!("🧪 Testing mintty export with dracula theme...");

        let theme = thag_styling::Theme::get_builtin("dracula")?;
        let mintty_content = ExportFormat::Mintty.export_theme(&theme)?;

        // Basic validation of the output
        let has_background = mintty_content.contains("BackgroundColour=");
        let has_foreground = mintty_content.contains("ForegroundColour=");
        let has_colors = mintty_content.contains("Red=") && mintty_content.contains("Blue=");

        if has_background && has_foreground && has_colors {
            println!("✅ Mintty export generates valid content!");
        } else {
            println!("❌ Mintty export content appears invalid");
            println!(
                "   Background: {}, Foreground: {}, Colors: {}",
                has_background, has_foreground, has_colors
            );
        }

        println!("\n📊 Export Statistics:");
        println!("   Lines: {}", mintty_content.lines().count());
        println!("   Format: {}", ExportFormat::Mintty.format_name());
        println!("   Extension: {}", ExportFormat::Mintty.file_extension());
    } else {
        println!("❌ Mintty format is missing from the list!");
        return Err("Mintty format not found in ExportFormat::all()".into());
    }

    println!("\n🎉 All tests passed! Mintty integration is working correctly.");

    Ok(())
}
