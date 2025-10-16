/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Quick test to verify filename handling for different formats
//# Purpose: Test filename handling
//# Categories: file_handling, testing
use thag_styling::{export_theme_to_file, ExportFormat, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Filename Handling");
    println!("============================\n");

    // Load the theme
    let theme = Theme::get_builtin("thag-vibrant-dark")?;

    // Test each format individually with explicit filenames
    let test_dir = "test_filenames";
    std::fs::create_dir_all(test_dir)?;

    for format in ExportFormat::all() {
        let base_name = "test_theme";
        let filename = match format {
            ExportFormat::Alacritty => {
                format!("{}_alacritty.{}", base_name, format.file_extension())
            }
            ExportFormat::WezTerm => format!("{}_wezterm.{}", base_name, format.file_extension()),
            ExportFormat::WindowsTerminal => {
                format!("{}_windows_terminal.{}", base_name, format.file_extension())
            }
            _ => format!("{}.{}", base_name, format.file_extension()),
        };

        let file_path = std::path::Path::new(test_dir).join(&filename);

        println!(
            "📄 Exporting {} to {}",
            format.format_name(),
            file_path.display()
        );

        export_theme_to_file(&theme, *format, &file_path)?;

        if file_path.exists() {
            let size = std::fs::metadata(&file_path)?.len();
            println!("   ✅ Created {} ({} bytes)", filename, size);
        } else {
            println!("   ❌ Failed to create {}", filename);
        }
    }

    println!("\n📁 Files created in {} directory:", test_dir);
    for entry in std::fs::read_dir(test_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("   📄 {}", path.file_name().unwrap().to_string_lossy());
        }
    }

    Ok(())
}
